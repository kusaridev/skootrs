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
use skootrs_model::skootrs::{Output, OutputDescriptor, OutputGetParams, OutputType, SkootError};

pub trait OutputService {
    /// Gets an output for a Skootrs project. This is usually something related to a release like
    /// an SBOM or SLSA attestation.
    ///
    /// # Errors
    ///
    /// Returns an error if the output doesn't exist or can't be fetched for some reason.
    fn get(
        &self,
        params: OutputGetParams,
    ) -> impl std::future::Future<Output = Result<Output, SkootError>> + Send;
}

pub struct LocalOutputService;

impl OutputService for LocalOutputService {
    async fn get(&self, params: OutputGetParams) -> Result<Output, SkootError> {}
}

pub struct GithubReleaseHandler;
impl GithubReleaseHandler {
    pub async fn get_release(
        owner: &str,
        repo: &str,
        release: Option<&str>,
    ) -> Result<Release, SkootError> {
        let release = {
            if let Some(release) = release {
                octocrab::instance()
                    .repos(owner, repo)
                    .releases()
                    .get_by_tag(release)
                    .await?
            } else {
                octocrab::instance()
                    .repos(owner, repo)
                    .releases()
                    .get_latest()
                    .await?
            }
        };
        Ok(release)
    }

    pub async fn list_outputs(
        owner: &str,
        repo: &str,
        release: Option<&str>,
    ) -> Result<Vec<OutputDescriptor>, SkootError> {
        //let release = release.unwrap_or("latest");
        let release = GithubReleaseHandler::get_release(owner, repo, release).await?;
        let descriptors = {
            let assets = release.assets;
            assets
                .into_iter()
                .map(|a| {
                    let output_type = GithubReleaseHandler::asset_to_output_type(&a);
                    OutputDescriptor {
                        name: a.name,
                        url: a.browser_download_url,
                        output_type,
                    }
                })
                .collect()
        };

        Ok(descriptors)
    }

    pub async fn get_output(
        owner: &str,
        repo: &str,
        release: Option<&str>,
        output_name: &str,
    ) -> Result<Output, SkootError> {
        let release = GithubReleaseHandler::get_release(owner, repo, release).await?;
        let asset = release
            .assets
            .into_iter()
            .find(|a| a.name == output_name)
            .ok_or("Output not found")?;

        let output_content = reqwest::get(asset.browser_download_url)
            .await?
            .text()
            .await?;

        let output_type = GithubReleaseHandler::asset_to_output_type(&asset);
        match output_type {
            OutputType::SBOM => {
                let sbom = skootrs_model::sbom::SBOM::from_str(&output_content)?;
                Ok(Output::SBOM(sbom))
            }
            OutputType::Custom(_) => Ok(Output::Custom(output_content)),
        }

        Ok(())
    }
    pub fn asset_to_output_type(asset: &Asset) -> OutputType {
        let output_type: OutputType = match asset.url {
            // Follows: https://github.com/ossf/sbom-everywhere/blob/main/reference/sbom_naming.md
            _ if asset.name.contains(".spdx.") => OutputType::SBOM,
            _ if asset.name.contains(".cdx.") => OutputType::SBOM,
            _ => OutputType::Custom("Unknown".to_string()),
        };
        output_type
    }
}
