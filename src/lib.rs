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

pub mod models;

use inquire::Text;
use models::security_insights::insights10::{
    SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy,
    SecurityInsightsVersion100YamlSchemaHeader,
    SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion,
    SecurityInsightsVersion100YamlSchemaProjectLifecycle,
    SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus, SecurityInsightsVersion100YamlSchemaVulnerabilityReporting,
};
use std::{error::Error, process::Command, thread, time::Duration, fs};

pub async fn create() -> std::result::Result<(), Box<dyn Error>> {
    let instance = octocrab::instance();
    let name = Text::new("The name of the repository").prompt()?;
    let description = Text::new("The description of the repository").prompt()?;
    instance
        .repos("kusaridev", "skoot-go")
        .generate(name.as_str())
        .description(&description)
        .send()
        .await?;
    println!("Created {} repository from template: kusaridev/skoot-go", &name);

    thread::sleep(Duration::from_millis(4000));

    instance
        .repos("mlieberman85", name.as_str())
        .create_file(
            "README.md",
            "Create README",
            format!(
                r#"# {}
{}
"#,
                &name, description
            ),
        )
        .send()
        .await?;
    println!("Created README.md for {}", &name);

    let url = format!("https://github.com/{}/{}", "mlieberman85", &name);
    let _output = Command::new("git")
        .arg("clone")
        .arg(&url)
        .arg(format!("/tmp/{}", &name))
        .output()
        .expect("Failed to execute git clone command");
    println!("Cloned {} to /tmp/{}", &url, &name);

    let _output = Command::new("go")
        .arg("mod")
        .arg("init")
        .arg(format!("github.com/mlieberman85/{}", &name))
        .current_dir(format!("/tmp/{}", &name))
        .output()
        .expect("Failed to execute go mod init command");
    println!("Initialized go module for {}", &name);

    fs::write(
        format!("/tmp/{}/SECURITY_INSIGHTS.yaml", &name),
        create_security_insights(url)?,
    )?;
    println!("Created SECURITY_INSIGHTS.yaml for {}", &name);

    let _output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(format!("/tmp/{}", &name))
        .output()
        .expect("Failed to execute git add command");

    let commit_message = format!("Initialize go module for {}", &name);
    let _output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .current_dir(format!("/tmp/{}", &name))
        .output()
        .expect("Failed to execute git commit command");
    println!("Committed changes to {}", &name);

    let _output = Command::new("git")
        .arg("push")
        .current_dir(format!("/tmp/{}", &name))
        .output()
        .expect("Failed to execute git push command");
    println!("Pushed changes to {}", &name);

    Ok(println!("Created a new skootrs project"))
}

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
