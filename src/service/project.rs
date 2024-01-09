
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




use crate::{model::skootrs::{ProjectParams, InitializedProject, InitializedSource, facet::{SupportedFacetType, FacetParams, SourceFileFacetParams, CommonFacetParams}}, service::facet::RootFaceService};

use super::{repo::{LocalRepoService, RepoService}, ecosystem::{LocalEcosystemService, EcosystemService}, source::{LocalSourceService, SourceService}, facet::LocalFacetService};
use tracing::debug;

// TODO: Where should this go?
static DEFAULT_SOURCE_FACETS: [SupportedFacetType; 2] = [
    SupportedFacetType::Readme,
    SupportedFacetType::SecurityInsights,
];

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
        let initialized_facets = DEFAULT_SOURCE_FACETS.iter().map(|facet_type| {
            let params = params.clone();
            let name = match facet_type {
                SupportedFacetType::Readme => "README.md".to_string(),
                SupportedFacetType::SecurityInsights => "SECURITY_INSIGHTS.yml".to_string(),
                SupportedFacetType::SLSABuild => todo!("Not supported yet"),
                SupportedFacetType::SBOMGenerator => todo!("Not supported yet"),
            };
            let facet_params = FacetParams::SourceFile(SourceFileFacetParams {
                name: name,
                path: params.source_params.path(params.name.clone()),
                common: CommonFacetParams {
                    project_name: params.name.clone(),
                    source: initialized_source.clone(),
                    repo: initialized_repo.clone(),
                    ecosystem: initialized_ecosystem.clone(),
                },
                facet_type: facet_type.clone(),
            });
            self.facet_service.initialize(facet_params)
        }).collect::<Result<Vec<_>, _>>()?;
        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: initialized_ecosystem,
            source: initialized_source,
            facets: initialized_facets.into(),
        })
    }
}