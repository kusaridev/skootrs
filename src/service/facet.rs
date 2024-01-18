
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

use tracing::info;

use crate::model::{skootrs::facet::{FacetParams, SourceFileFacetParams, Facet, SourceFileFacet, SupportedFacetType}, security_insights::insights10::{SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy, SecurityInsightsVersion100YamlSchemaHeader, SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion, SecurityInsightsVersion100YamlSchemaProjectLifecycle, SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus, SecurityInsightsVersion100YamlSchemaVulnerabilityReporting}};
use crate::service::source::SourceService;

use super::source::LocalSourceService;

#[derive(Debug)]
pub struct LocalFacetService {}

pub trait RootFaceService {
    fn initialize(&self, params: FacetParams) -> Result<Facet, Box<dyn Error>>;
}

pub trait SourceFileFacetService {
    fn initialize(&self, params: SourceFileFacetParams) -> Result<SourceFileFacet, Box<dyn Error>>;
}

impl RootFaceService for LocalFacetService {
    fn initialize(&self, params: FacetParams) -> Result<Facet, Box<dyn Error>> {
        match params {
            FacetParams::SourceFile(params) => {
                let source_file_facet = SourceFileFacetService::initialize(self, params)?;
                Ok(Facet::SourceFile(source_file_facet))
            }
        }
    }
}

impl SourceFileFacetService for LocalFacetService {
    fn initialize(&self, params: SourceFileFacetParams) -> Result<SourceFileFacet, Box<dyn Error>> {
        let source_service = LocalSourceService {};
        let content_handler = SourceFileContentHandler {};
        let content = content_handler.generate_content(&params)?;
        source_service.write_file(params.common.source, &params.path, params.name.clone(), content)?;
        info!("Successfully created source file facet: {}", params.name);
        Ok(SourceFileFacet {
            name: params.name,
            path: params.path,
        })
    }
}

struct SourceFileContentHandler {}

impl SourceFileContentHandler {
    fn generate_content(&self, params: &SourceFileFacetParams) -> Result<String, Box<dyn Error>> {
        match params.facet_type {
            SupportedFacetType::Readme => self.generate_readme_content(params),
            SupportedFacetType::SecurityInsights => self.generate_security_insights_content(params),
            SupportedFacetType::SLSABuild => todo!("Not supported yet"),
            SupportedFacetType::SBOMGenerator => todo!("Not supported yet"),
        }
    }

    fn generate_readme_content(&self, params: &SourceFileFacetParams) -> Result<String, Box<dyn Error>> {
        let content = format!(
            r#"# {}
This is the README for the {} project."#,
            params.common.project_name, params.common.project_name
        );
        Ok(content)
    }

    fn generate_security_insights_content(&self, params: &SourceFileFacetParams) -> Result<String, Box<dyn Error>> {
        let insights = SecurityInsightsVersion100YamlSchema {
            contribution_policy: SecurityInsightsVersion100YamlSchemaContributionPolicy {
                accepts_automated_pull_requests: true,
                accepts_pull_requests: true,
                automated_tools_list: None,
                code_of_conduct: None,
                contributing_policy: None,
            },
            dependencies: None,
            distribution_points: Vec::new(),
            documentation: None,
            header: SecurityInsightsVersion100YamlSchemaHeader {
                changelog: None,
                commit_hash: None,
                expiration_date: chrono::Utc::now() + chrono::Duration::days(365),
                last_reviewed: None,
                last_updated: None,
                license: Some(format!("{}/blob/main/LICENSE", &params.common.repo.full_url())),
                project_release: None,
                project_url: params.common.repo.full_url(),
                schema_version: SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion::_100,
            },
            project_lifecycle: SecurityInsightsVersion100YamlSchemaProjectLifecycle {
                bug_fixes_only: false,
                core_maintainers: None,
                release_cycle: None,
                release_process: None,
                roadmap: None,
                status: SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus::Active,
            },
            security_artifacts: None,
            security_assessments: None,
            security_contacts: Vec::new(),
            security_testing: Vec::new(),
            vulnerability_reporting: SecurityInsightsVersion100YamlSchemaVulnerabilityReporting {
                accepts_vulnerability_reports: false,
                bug_bounty_available: None,
                bug_bounty_url: None,
                comment: None,
                email_contact: None,
                in_scope: None,
                out_scope: None,
                pgp_key: None,
                security_policy: None,
            },
        };
    
        Ok(serde_yaml::to_string(&insights)?)
    }
}