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

#![feature(array_try_map)]

pub mod model;
pub mod server;
pub mod statestore;
pub mod service;

use inquire::Text;
use model::skootrs::{MavenParams, GoParams, GithubRepoParams, ProjectParams, GithubUser, RepoParams, EcosystemParams, SourceParams, SUPPORTED_ECOSYSTEMS};
use octocrab::Page;
use service::{project::{LocalProjectService, ProjectService}, repo::LocalRepoService, ecosystem::LocalEcosystemService, source::LocalSourceService, facet::LocalFacetService};
use std::error::Error;

/// Returns `Ok(())` if the project creation is successful, otherwise returns an error.
/// 
/// Creates a new skootrs project by prompting the user for repository details and language selection.
/// The project can be created for either Go or Maven ecosystems right now.
/// The project is created in Github, cloned down, and then initialized along with any other security supporting
/// tasks.
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
        SUPPORTED_ECOSYSTEMS.to_vec(),
    );

    /*let options = InitializeOptions {
        local_path: "/tmp".to_string(),
    };*/

    let gh_org = match organization {
        x if x == user => GithubUser::User(x.to_string()),
        x => GithubUser::Organization(x.to_string()),
    };

    match language.prompt()? {
        "Go" => {
            // TODO: support more than just github
            let go_params = GoParams { name: name.clone(), host: format!("github.com/{}", organization) };
            /*let project_config = ProjectParams { 
                repo: GithubRepoParams { name: name.clone(), description, organization: gh_org }, 
                ecosystem: go_config, 
                name: name.clone(),
                //config_bundle: Box::new(DefaultConfigBundle{}),
                repo_params: todo!(),
                ecosystem_params: todo!(),
            };*/
            let project_params = ProjectParams {
                name: name.clone(),
                repo_params: RepoParams::Github(
                    GithubRepoParams {
                        name,
                        description,
                        organization: gh_org,
                    }
                ),
                ecosystem_params: EcosystemParams::Go(go_params),
                source_params: SourceParams {
                    parent_path: "/tmp".to_string(), // FIXME: This should be configurable
                },
            };
            let local_project_service = LocalProjectService {
                repo_service: LocalRepoService {},
                ecosystem_service: LocalEcosystemService {},
                source_service: LocalSourceService {},
                facet_service: LocalFacetService {},
            };

            //let _initialized_project = project_params.initialize(options).await?;
            let _initialized_project = local_project_service.initialize(project_params).await?;
        },

        "Maven" => {
            let maven_params = MavenParams { 
                group_id: format!("com.{}.{}", organization, name),
                artifact_id: name.clone()
            };

            let project_params = ProjectParams {
                name: name.clone(),
                repo_params: RepoParams::Github(
                    GithubRepoParams {
                        name,
                        description,
                        organization: gh_org,
                    }
                ),
                ecosystem_params: EcosystemParams::Maven(maven_params),
                source_params: SourceParams {
                    parent_path: "/tmp".to_string(), // FIXME: This should be configurable
                },
            };
            let local_project_service = LocalProjectService {
                repo_service: LocalRepoService {},
                ecosystem_service: LocalEcosystemService {},
                source_service: LocalSourceService {},
                facet_service: LocalFacetService {},
            };

            let _initialized_project = local_project_service.initialize(project_params).await?;
        }

        _ => {
            unreachable!("Unsupported language");
        }
    }

    Ok(())
}