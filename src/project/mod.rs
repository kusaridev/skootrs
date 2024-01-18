//
// Copyright 2023 The Skootrs Authors.
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

use std::{error::Error, result::Result};

use tracing::info;

use crate::{
    ecosystem::Ecosystem,
    repo::{InitializedRepo, UninitializedRepo},
    source::Source, config::{ConfigBundle, ConfigInput::{DefaultSecurityInsightsStruct, self}, DefaultSecurityInsightsInput, DefaultReadmeInput},
};

pub struct InitializeOptions {
    pub local_path: String,
}

pub struct UninitializedProject<T: UninitializedRepo, E: Ecosystem> {
    // TODO: This should take in InitializeOptions
    pub repo: T,
    pub ecosystem: E,
    pub name: String,
    pub config_bundle: Box<dyn ConfigBundle>
}

pub struct InitializedProject<T: InitializedRepo, E: Ecosystem> {
    pub repo: T,
    pub ecosystem: E,
    pub source: Source,
}

/// Returns a `Result` containing the initialized project if successful, or a `Box<dyn Error>`
/// if an error occurs.
///  
/// This method creates an initialized repository, clones the repository to the local path,
/// initializes the ecosystem, creates documentation and security insights, and commits and
/// pushes the changes to the repository.
///
/// # Arguments
///
/// * `options` - The options for initializing the project.
impl<T: UninitializedRepo, E: Ecosystem> UninitializedProject<T, E> {
    pub async fn initialize(
        &self,
        options: InitializeOptions,
    ) -> Result<InitializedProject<T::Repo, E>, Box<dyn Error>> {
        let initialized_repo = self.repo.create().await?;
        let source = initialized_repo.clone_repo(options.local_path.clone())?;
        self.ecosystem.initialize(options.local_path.clone())?;
        //self.create_documentation(&source)?;
        self.configure(&source)?;
        source.commit_and_push_changes(format!(
            "Added documentation and security insights for {}",
            self.name
        ))?;

        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: self.ecosystem.clone(),
            source,
        })
    }

    fn configure(&self, source: &Source) -> Result<(), Box<dyn Error>> {
        let readme_bundle = self.config_bundle.readme_bundle(
            ConfigInput::DefaultReadmeStruct(DefaultReadmeInput{ name: self.name.clone() }))?;
        match readme_bundle {
            crate::config::Config::SourceFileConfig(sfc) => {
                source.write_file(sfc.path, sfc.name, sfc.content)?;
            },
        }
        info!("Created README.md for {}", self.name);
        let url = self.repo.full_url();
        let security_insights_bundle = self.config_bundle.security_insights_bundle(
            DefaultSecurityInsightsStruct(DefaultSecurityInsightsInput{ url }))?;
        match security_insights_bundle {
            crate::config::Config::SourceFileConfig(sfc) => {
                source.write_file(sfc.path, sfc.name, sfc.content)?;
            },
        }
        info!("Created SECURITY_INSIGHTS.yaml for {}", self.name);
        Ok(())
    }
}
