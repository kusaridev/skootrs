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

use std::{collections::HashMap, error::Error};

use thiserror::Error;

use crate::model::security_insights::insights10::{SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy, SecurityInsightsVersion100YamlSchemaHeader, SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion, SecurityInsightsVersion100YamlSchemaProjectLifecycle, SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus, SecurityInsightsVersion100YamlSchemaVulnerabilityReporting};

/// Enum that represents various configuration, including security configuration
/// for a project. This will include things like files in the source repository,
/// API calls to manage build systems, etc.
pub enum Config {
    SourceFileConfig(SourceFileBundle),
}

/// Struct representing everything needed for a source file.
pub struct SourceFileBundle {
    pub name: String,
    pub path: String,
    pub content: String,
}

/// Enum that represents common config options to select between at runtime for
/// inputs to the config.
/// TODO: There might be a better way of doing this.
pub enum ConfigInput {
    VecString(Vec<String>),
    MapStringString(HashMap<String, String>),
    DefaultReadmeStruct(DefaultReadmeInput),
    DefaultSecurityInsightsStruct(DefaultSecurityInsightsInput),
    DefaultSBOMStruct(DefaultSBOMInput),
    DefaultSLSAStruct(DefaultSLSAInput),
}

pub struct DefaultReadmeInput {
    pub name: String,
}

pub struct DefaultSecurityInsightsInput {
    pub url: String,
}

pub struct DefaultSBOMInput {}

pub struct DefaultSLSAInput {}

/// An empty struct that can be used to implement the ConfigBundle trait.
pub struct DefaultConfigBundle {}

pub trait ConfigBundle {
    fn readme_bundle(&self, config_input: ConfigInput) -> Result<Config, Box<dyn Error>>;
    fn security_insights_bundle(
        &self,
        config_input: ConfigInput,
    ) -> Result<Config, Box<dyn Error>>;
    fn sbom_bundle(&self, config_input: ConfigInput) -> Result<Config, Box<dyn Error>>;
    fn slsa_bundle(&self, config_input: ConfigInput) -> Result<Config, Box<dyn Error>>;
}

impl ConfigBundle for DefaultConfigBundle {
    fn readme_bundle(&self, config_input: ConfigInput) -> Result<Config, Box<dyn Error>> {
        match config_input {
            ConfigInput::DefaultReadmeStruct(input) => {
                let content = format!(
                    r#"# {}
This is the README for the {} project."#,
                    input.name, input.name
                );

                Ok(Config::SourceFileConfig(SourceFileBundle {
                    name: "README.md".to_string(),
                    path: "./".to_string(),
                    content,
                }))
            }
            _ => Err(Box::new(ConfigError::UnsupportedConfigInput)),
        }
    }

    fn security_insights_bundle(&self, 
        config_input: ConfigInput,
    ) -> Result<Config, Box<dyn Error>> {
        match config_input {
            ConfigInput::DefaultSecurityInsightsStruct(input) => {
                let content = create_security_insights_content(&input.url)?;
                Ok(Config::SourceFileConfig(SourceFileBundle {
                    name: "SECURITY_INSIGHTS.yml".to_string(),
                    path: "./".to_string(),
                    content,
                }))
            },

            _ => Err(Box::new(ConfigError::UnsupportedConfigInput))
        }
    }

    fn sbom_bundle(&self, _config_input: ConfigInput) -> Result<Config, Box<dyn Error>> {
        // TODO: Create action here
        todo!()
    }

    fn slsa_bundle(&self, _config_input: ConfigInput) -> Result<Config, Box<dyn Error>> {
        // TODO: Create action here
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Unsupported config input for this enum variant")]
    UnsupportedConfigInput,
}


fn create_security_insights_content(url: &str) -> std::result::Result<String, Box<dyn Error>> {
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
            license: Some(format!("{}/blob/main/LICENSE", &url)),
            project_release: None,
            project_url: url.to_string(),
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

    let yaml_string = serde_yaml::to_string(&insights)?;
    Ok(yaml_string)
}