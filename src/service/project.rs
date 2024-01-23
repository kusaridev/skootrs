
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

use std::error::Error;




use crate::{model::skootrs::{facet::CommonFacetParams, InitializedProject, InitializedSource, ProjectParams}, service::facet::{FacetSetParamsGenerator, RootFacetService}};

use super::{repo::{LocalRepoService, RepoService}, ecosystem::{LocalEcosystemService, EcosystemService}, source::{LocalSourceService, SourceService}, facet::LocalFacetService};
use tracing::debug;

pub trait ProjectService {
    fn initialize(&self, params: ProjectParams) -> impl std::future::Future<Output = Result<InitializedProject, Box<dyn Error>>> + Send;
}

#[derive(Debug)]
pub struct LocalProjectService {
    pub repo_service: LocalRepoService,
    pub ecosystem_service: LocalEcosystemService,
    pub source_service: LocalSourceService,
    pub facet_service: LocalFacetService,
}

impl ProjectService for LocalProjectService {
    async fn initialize(&self, params: ProjectParams) -> Result<InitializedProject, Box<dyn Error>> {
        debug!("Starting repo initialization");
        let initialized_repo = self.repo_service.initialize(params.repo_params.clone()).await?;
        debug!("Starting source initialization");
        let initialized_source: InitializedSource = self.source_service.initialize(params.source_params.clone(), initialized_repo.clone())?;
        debug!("Starting ecosystem initialization");
        let initialized_ecosystem = self.ecosystem_service.initialize(params.ecosystem_params.clone(), initialized_source.clone())?;
        debug!("Starting facet initialization");
        // TODO: This is ugly and this should probably be configured somewhere better, preferably outside of code.
        let facet_set_params_generator = FacetSetParamsGenerator {};
        let common_params = CommonFacetParams {
            project_name: params.name.clone(),
            source: initialized_source.clone(),
            repo: initialized_repo.clone(),
            ecosystem: initialized_ecosystem.clone(),
        };
        let facet_set_params = facet_set_params_generator.generate_default(common_params)?;
        let initialized_facets = self.facet_service.initialize_all(facet_set_params)?;
        self.source_service.commit_and_push_changes(initialized_source.clone(), "Initialized project".to_string())?;

        debug!("Completed project initialization");

        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: initialized_ecosystem,
            source: initialized_source,
            facets: initialized_facets.into(),
        })
    }
}