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

use std::{error::Error, fs, process::Command};

use crate::model::security_insights::insights10::{
        SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy,
        SecurityInsightsVersion100YamlSchemaHeader, SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion,
        SecurityInsightsVersion100YamlSchemaProjectLifecycle,
        SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus,
        SecurityInsightsVersion100YamlSchemaVulnerabilityReporting,
    };

pub enum EcosystemInitConfig {
    Go(GoConfig),
    Maven(MavenConfig),
}

pub struct GoConfig {
    pub module: String,
}

pub struct MavenConfig {
    pub group_id: String,
    pub artifact_id: String,
}

pub const SUPPORTED_ECOSYSTEMS: [&str; 2] = ["Go", "Maven"];

pub struct GithubBundle<'a> {
    client: &'a octocrab::Octocrab,
    name: &'a str,
    description: &'a str,
    organization: &'a str,
    ecosystem_init_config: EcosystemInitConfig,
}

pub trait Bundle<'a> {
    fn create(&self) -> impl std::future::Future<Output = std::result::Result<(), Box<dyn Error>>> + Send;
}

impl<'a> Bundle<'a> for GithubBundle<'a> {
    async fn create(&self) -> std::result::Result<(), Box<dyn Error>> {
        self.create_repository().await?;
        self.clone_repository().await?;
        self.initialize_project().await?;
        self.create_security_insights_yaml().await?;
        self.commit_and_push_changes().await?;
        self.create_readme().await?;
        Ok(())
    }
}

impl<'a> GithubBundle<'a> {
    pub fn new(
        client: &'a octocrab::Octocrab,
        name: &'a str,
        description: &'a str,
        organization: &'a str,
        ecosystem_init_config: EcosystemInitConfig,
    ) -> GithubBundle<'a> {
        GithubBundle {
            client,
            name,
            description,
            organization,
            ecosystem_init_config,
        }
    }

    async fn create_repository(&self) -> std::result::Result<(), Box<dyn Error>> {
        let (organization, repo) = self.get_repository_info();
        self.client
            .repos(organization, repo)
            .generate(self.name)
            .owner(self.organization)
            .description(self.description)
            .send()
            .await?;
        println!("Created {} repository from template: {}/{}", self.name, organization, repo);
        Ok(())
    }

    async fn create_readme(&self) -> std::result::Result<(), Box<dyn Error>> {
        self.client
            .repos(self.organization, self.name)
            .create_file(
                "README.md",
                "Create README",
                self.generate_readme_content(),
            )
            .send()
            .await?;
        println!("Created README.md for {}", self.name);
        Ok(())
    }

    async fn clone_repository(&self) -> std::result::Result<(), Box<dyn Error>> {
        let url = self.get_repository_url();
        let _output = Command::new("git")
            .arg("clone")
            .arg(&url)
            .arg(format!("/tmp/{}", self.name))
            .output()
            .expect("Failed to execute git clone command");
        println!("Cloned {} to /tmp/{}", url, self.name);
        Ok(())
    }

    async fn initialize_project(&self) -> std::result::Result<(), Box<dyn Error>> {
        match &self.ecosystem_init_config {
            EcosystemInitConfig::Go(go_config) => self.initialize_go_module(go_config).await,
            EcosystemInitConfig::Maven(maven_config) => self.initialize_mvn_project(maven_config).await,
        }
    }

    async fn initialize_mvn_project(
        &self,
        maven_config: &MavenConfig,
    ) -> std::result::Result<(), Box<dyn Error>> {
        let _output = Command::new("mvn")
            .arg("archetype:generate")
            .arg(format!("-DgroupId={}", maven_config.group_id))
            .arg(format!("-DartifactId={}", maven_config.artifact_id))
            .arg("-DarchetypeArtifactId=maven-archetype-quickstart")
            .arg("-DinteractiveMode=false")
            .current_dir(format!("/tmp/{}", self.name))
            .output()?;
        println!("Initialized maven project for {}", self.name);
        Ok(())
    }

    async fn initialize_go_module(
        &self,
        go_config: &GoConfig,
    ) -> std::result::Result<(), Box<dyn Error>> {
        let _output = Command::new("go")
            .arg("mod")
            .arg("init")
            .arg(go_config.module.as_str())
            .current_dir(format!("/tmp/{}", self.name))
            .output()?;
        println!("Initialized go module for {}", self.name);
        Ok(())
    }

    async fn create_security_insights_yaml(&self) -> std::result::Result<(), Box<dyn Error>> {
        let url = self.get_repository_url();
        fs::write(
            format!("/tmp/{}/SECURITY_INSIGHTS.yaml", self.name),
            self.create_security_insights(url)?,
        )?;
        println!("Created SECURITY_INSIGHTS.yaml for {}", self.name);
        Ok(())
    }

    async fn commit_and_push_changes(&self) -> std::result::Result<(), Box<dyn Error>> {
        let _output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(format!("/tmp/{}", self.name))
            .output()?;

        let commit_message = format!("Initialize go module for {}", self.name);
        let _output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&commit_message)
            .current_dir(format!("/tmp/{}", self.name))
            .output()?;
        println!("Committed changes to {}", self.name);

        let _output = Command::new("git")
            .arg("push")
            .current_dir(format!("/tmp/{}", self.name))
            .output()?;
        println!("Pushed changes to {}", self.name);
        Ok(())
    }

    fn get_repository_info(&self) -> (&str, &str) {
        match self.ecosystem_init_config {
            EcosystemInitConfig::Go(_) => ("kusaridev", "skoot-go"),
            EcosystemInitConfig::Maven(_) => ("kusaridev", "skoot-maven"),
        }
    }

    fn get_repository_url(&self) -> String {
        let (organization, repo) = self.get_repository_info();
        format!("https://github.com/{}/{}", organization, repo)
    }

    fn generate_readme_content(&self) -> String {
        format!(
            "# {}\n{}",
            self.name, self.description
        )
    }

    fn create_security_insights(&self, url: String) -> std::result::Result<String, Box<dyn Error>> {
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
                project_url: url,
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

        let yaml = serde_yaml::to_string(&insights)?;
        Ok(yaml)
    }
}
