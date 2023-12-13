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

pub mod bundle;
pub mod models;

use bundle::bundle::{EcosystemInitConfig, GithubBundle};
use inquire::Text;
use octocrab::Page;
use std::error::Error;
use crate::bundle::bundle::Bundle;

pub async fn create() -> std::result::Result<(), Box<dyn Error>> {
    let instance = octocrab::instance();
    let name = Text::new("The name of the repository").prompt()?;
    let description = Text::new("The description of the repository").prompt()?;
    let user = instance.current().user().await?.login;
    let Page { items, .. } = instance
        .current()
        .list_org_memberships_for_authenticated_user()
        .send()
        .await?;
    let organization = inquire::Select::new(
        "Select an organization",
        items
            .iter()
            .map(|i| i.organization.login.as_str())
            .chain(vec![user.as_str()])
            .collect(),
    )
    .prompt()?;

    let language = inquire::Select::new(
        "Select a language",
        bundle::bundle::SUPPORTED_ECOSYSTEMS.to_vec(),
    );

    let ecosystem_init_config: EcosystemInitConfig = match language.prompt()? {
        "Go" => {
            let go_config = bundle::bundle::GoConfig {
                // TODO: Support more than Github
                module: format!("github.com/{}/{}", organization, name),
            };
            EcosystemInitConfig::Go(go_config)
        }
        "Maven" => {
            // TODO: Make this configurable
            let maven_config = bundle::bundle::MavenConfig {
                group_id: format!("com.{}.{}", organization, name),
                artifact_id: name.clone(),
            };
            EcosystemInitConfig::Maven(maven_config)
        }
        _ => {
            unreachable!("Unsupported language");
        }
    };

    let bundle = GithubBundle::new(
        &instance,
        &name,
        &description,
        &organization,
        ecosystem_init_config,
    );
    bundle.create().await?;

    Ok(println!("Created a new skootrs project"))
}