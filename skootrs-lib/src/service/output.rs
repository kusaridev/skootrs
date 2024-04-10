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
    ProjectOutputReference, ProjectOutputType, ProjectOutputsListParams, SkootError,
};

pub trait OutputService {
    fn outputs_list(
        &self,
        params: ProjectOutputsListParams,
    ) -> impl std::future::Future<Output = Result<Vec<ProjectOutputReference>, SkootError>> + Send;
}

pub struct LocalOutputService;

impl OutputService for LocalOutputService {
    fn outputs_list(
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
            // TODO: Add more types
            _ => ProjectOutputType::Custom("Unknown".to_string()),
        }
    }
}

struct GithubReleaseParams {
    owner: String,
    repo: String,
    tag: Option<String>,
}
