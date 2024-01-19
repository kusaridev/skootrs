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
use model::skootrs::{facet::SourceFileFacet, EcosystemParams, GithubRepoParams, GithubUser, GoParams, InitializedProject, MavenParams, ProjectParams, RepoParams, SourceParams, SUPPORTED_ECOSYSTEMS};
use octocrab::Page;
use service::{project::{LocalProjectService, ProjectService}, repo::LocalRepoService, ecosystem::LocalEcosystemService, source::LocalSourceService, facet::LocalFacetService};
use std::{collections::HashMap, error::Error};

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

    let gh_org = match organization {
        x if x == user => GithubUser::User(x.to_string()),
        x => GithubUser::Organization(x.to_string()),
    };

    let initialized_project: InitializedProject = match language.prompt()? {
        "Go" => {
            // TODO: support more than just github
            let go_params = GoParams { name: name.clone(), host: format!("github.com/{}", organization) };
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

            local_project_service.initialize(project_params).await?
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

            local_project_service.initialize(project_params).await?
        }

        _ => {
            unreachable!("Unsupported language")
        }
    };

    let state_store = statestore::SurrealProjectStateStore::new().await?;
    state_store.create(initialized_project).await?;

    Ok(())
}

pub async fn get_facet() -> std::result::Result<(), Box<dyn Error>> {
    let projects = get_all().await?;
    let repo_to_project: HashMap<String, &InitializedProject> = projects.iter().map(|p| (p.repo.full_url(), p)).collect::<HashMap<_,_>>();
    let selected_project = inquire::Select::new(
        "Select a project",
        repo_to_project.keys().collect::<Vec<_>>(),
    )
    .prompt()?;

    // FIXME: Support more than SouceFileFacet
    let facet_to_project: HashMap<String, &SourceFileFacet> = repo_to_project.get(selected_project)
        .ok_or_else(|| Box::<dyn Error>::from("Failed to get selected project"))?
        .facets.iter()
        .filter_map(|f| match f {
            model::skootrs::facet::Facet::SourceFile(f) => Some((f.name.clone(), f)),
        })
        .collect::<HashMap<_,_>>();

    let selected_facet = inquire::Select::new(
        "Select a facet",
        facet_to_project.keys().collect::<Vec<_>>(),
    )
    .prompt()?;

    let facet = facet_to_project.get(selected_facet)
        .ok_or_else(|| Box::<dyn Error>::from("Failed to get selected facet"))?;

    let facet_path = format!("{}/{}", facet.path, facet.name);

    let content = std::fs::read_to_string(facet_path)?;
    println!("{}", content);

    Ok(())
}

pub async fn dump() -> std::result::Result<(), Box<dyn Error>> {
    let projects = get_all().await?;
    println!("{}", serde_json::to_string_pretty(&projects).unwrap());
    Ok(())
}

async fn get_all() -> std::result::Result<Vec<InitializedProject>, Box<dyn Error>> {
    let state_store = statestore::SurrealProjectStateStore::new().await?;
    let projects = state_store.select_all().await?;
    Ok(projects)
}