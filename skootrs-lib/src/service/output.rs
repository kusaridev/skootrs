//
// Copyright 2024 The Skootrs Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::module_name_repetitions)]

use octocrab::models::repos::{Asset, Release};
use skootrs_model::skootrs::{
    label::Label, ProjectOutput, ProjectOutputGetParams, ProjectOutputReference, ProjectOutputType,
    ProjectOutputsListParams, SkootError,
};
pub trait OutputService {
    fn list(
        &self,
        params: ProjectOutputsListParams,
    ) -> impl std::future::Future<Output = Result<Vec<ProjectOutputReference>, SkootError>> + Send;

    fn get(
        &self,
        _params: ProjectOutputGetParams,
    ) -> impl std::future::Future<Output = Result<ProjectOutput, SkootError>> + Send;
}

pub struct LocalOutputService;

impl OutputService for LocalOutputService {
    fn list(
        &self,
        params: ProjectOutputsListParams,
    ) -> impl std::future::Future<Output = Result<Vec<ProjectOutputReference>, SkootError>> + Send
    {
        match params.initialized_project.repo {
            skootrs_model::skootrs::InitializedRepo::Github(g) => {
                let github_params = GithubReleaseParams {
                    owner: g.organization.get_name(),
                    repo: g.name,
                    tag: params.release.tag(),
                };
                GithubReleaseHandler::outputs_list(github_params)
            }
        }
    }

    async fn get(&self, params: ProjectOutputGetParams) -> Result<ProjectOutput, SkootError> {
        match params.initialized_project.repo {
            skootrs_model::skootrs::InitializedRepo::Github(g) => {
                let github_params = GithubOutputGetParams {
                    release: GithubReleaseHandler::get_release(GithubReleaseParams {
                        owner: g.organization.get_name(),
                        repo: g.name.clone(),
                        tag: params.release.tag(),
                    })
                    .await?,
                    name: params.project_output,
                };
                GithubReleaseHandler::get_output(github_params).await
            }
        }
    }
}

struct GithubReleaseHandler;
impl GithubReleaseHandler {
    async fn outputs_list(
        params: GithubReleaseParams,
    ) -> Result<Vec<ProjectOutputReference>, SkootError> {
        let release = Self::get_release(params).await?;

        let assets = release.assets;
        let references = assets
            .iter()
            .map(|asset| ProjectOutputReference {
                name: asset.name.clone(),
                output_type: Self::get_type(asset),
                labels: Self::get_labels(asset),
            })
            .collect();

        Ok(references)
    }

    async fn get_release(params: GithubReleaseParams) -> Result<Release, octocrab::Error> {
        match params.tag {
            Some(tag) => {
                octocrab::instance()
                    .repos(params.owner, params.repo)
                    .releases()
                    .get_by_tag(tag.as_str())
                    .await
            }
            None => {
                octocrab::instance()
                    .repos(params.owner, params.repo)
                    .releases()
                    .get_latest()
                    .await
            }
        }
    }

    fn get_type(asset: &Asset) -> ProjectOutputType {
        // TODO: This matching probably isn't GitHub specific and can live somewhere more generalized.
        match asset.url {
            // Follows: https://github.com/ossf/sbom-everywhere/blob/main/reference/sbom_naming.md
            _ if asset.name.contains(".spdx.") => ProjectOutputType::SBOM,
            _ if asset.name.contains(".cdx.") => ProjectOutputType::SBOM,
            _ if asset.name.contains(".intoto.") => ProjectOutputType::InToto,
            // TODO: Add more types
            _ => ProjectOutputType::Unknown("Unknown".to_string()),
        }
    }

    fn get_labels(asset: &Asset) -> Vec<Label> {
        match asset.url {
            _ if asset.name.contains(".spdx.") => vec![Label::S2C2FAUD4],
            _ if asset.name.contains(".cdx.") => vec![Label::S2C2FAUD4],
            _ if asset.name.contains(".intoto.") => vec![Label::SLSABuildLevel3],
            _ => vec![],
        }
    }

    async fn get_output(params: GithubOutputGetParams) -> Result<ProjectOutput, SkootError> {
        let asset = params
            .release
            .assets
            .iter()
            .find(|a| a.name == params.name)
            .ok_or("Asset not found".to_string())?;

        // TODO: Figure out how to support assets in private repos
        let content = reqwest::get(asset.browser_download_url.clone())
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await?;

        Ok(ProjectOutput {
            reference: ProjectOutputReference {
                name: asset.name.clone(),
                output_type: Self::get_type(asset),
                labels: Self::get_labels(asset),
            },
            output: serde_json::to_string_pretty(&content)?,
        })
    }
}

struct GithubReleaseParams {
    owner: String,
    repo: String,
    tag: Option<String>,
}

struct GithubOutputGetParams {
    release: Release,
    name: String,
}
