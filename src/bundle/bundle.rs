use std::{error::Error, fs, process::Command, thread, time::Duration};

use crate::models::security_insights::insights10::{
    SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy,
    SecurityInsightsVersion100YamlSchemaHeader,
    SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion,
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

    pub async fn create(&self) -> std::result::Result<(), Box<dyn Error>> {
        self.create_repository().await?;
        self.create_readme().await?;
        self.clone_repository().await?;
        self.initialize_project().await?;
        self.create_security_insights_yaml().await?;
        self.commit_and_push_changes().await?;
        Ok(())
    }

    async fn create_repository(&self) -> std::result::Result<(), Box<dyn Error>> {
        let (organization, repo) = match self.ecosystem_init_config {
            EcosystemInitConfig::Go(_) => ("kusaridev", "skoot-go"),
            EcosystemInitConfig::Maven(_) => ("kusaridev", "skoot-maven"),
        };
        self.client
            .repos(organization, repo)
            .generate(self.name)
            .owner(self.organization)
            .description(self.description)
            .send()
            .await?;
        println!(
            "Created {} repository from template: kusaridev/skoot-go",
            self.name
        );
        // TODO: figure out what the best way to wait for the repository to be created and template generated
        thread::sleep(Duration::from_millis(4000));
        Ok(())
    }

    async fn create_readme(&self) -> std::result::Result<(), Box<dyn Error>> {
        self.client
            .repos(self.organization, self.name)
            .create_file(
                "README.md",
                "Create README",
                format!(
                    r#"# {}
{}
"#,
                    self.name, self.description
                ),
            )
            .send()
            .await?;
        println!("Created README.md for {}", self.name);
        Ok(())
    }

    async fn clone_repository(&self) -> std::result::Result<(), Box<dyn Error>> {
        let url = format!("https://github.com/{}/{}", self.organization, self.name);
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
            .output()
            .expect("Failed to execute mvn archetype:generate command");
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
            .output()
            .expect("Failed to execute go mod init command");
        println!("Initialized go module for {}", self.name);
        Ok(())
    }

    async fn create_security_insights_yaml(&self) -> std::result::Result<(), Box<dyn Error>> {
        let url = format!("https://github.com/{}/{}", self.organization, self.name);
        fs::write(
            format!("/tmp/{}/SECURITY_INSIGHTS.yaml", self.name),
            create_security_insights(url)?,
        )?;
        println!("Created SECURITY_INSIGHTS.yaml for {}", self.name);
        Ok(())
    }

    async fn commit_and_push_changes(&self) -> std::result::Result<(), Box<dyn Error>> {
        let _output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(format!("/tmp/{}", self.name))
            .output()
            .expect("Failed to execute git add command");

        let commit_message = format!("Initialize go module for {}", self.name);
        let _output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&commit_message)
            .current_dir(format!("/tmp/{}", self.name))
            .output()
            .expect("Failed to execute git commit command");
        println!("Committed changes to {}", self.name);

        let _output = Command::new("git")
            .arg("push")
            .current_dir(format!("/tmp/{}", self.name))
            .output()
            .expect("Failed to execute git push command");
        println!("Pushed changes to {}", self.name);
        Ok(())
    }
}

// TODO: Make this do more. This just creates a very simple config
pub fn create_security_insights(url: String) -> std::result::Result<String, Box<dyn Error>> {
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
