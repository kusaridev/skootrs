
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

use std::{error::Error, process::Command, str::FromStr};

use chrono::Utc;
use tracing::info;

use crate::model::{skootrs::{RepoParams, InitializedRepo, GithubUser, InitializedGithubRepo, InitializedSource, GithubRepoParams}, cd_events::repo_created::{RepositoryCreatedEvent, RepositoryCreatedEventContext, RepositoryCreatedEventContextId, RepositoryCreatedEventContextVersion, RepositoryCreatedEventSubject, RepositoryCreatedEventSubjectContent, RepositoryCreatedEventSubjectContentName, RepositoryCreatedEventSubjectContentUrl, RepositoryCreatedEventSubjectId}};

pub trait RepoService {
    fn initialize(&self, params: RepoParams) -> Result<InitializedRepo, Box<dyn Error>>;
    fn clone_local(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, Box<dyn Error>>;
}

pub struct LocalRepoService {}

impl RepoService for LocalRepoService {
    fn initialize(&self, params: RepoParams) -> Result<InitializedRepo, Box<dyn Error>> {
        match params {
            RepoParams::Github(g) => {
                let github_repo_handler = GithubRepoHandler {
                    client: octocrab::instance(),
                };
                github_repo_handler.create(g)
            },
        }
    }

    fn clone_local(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, Box<dyn Error>> {
        match initialized_repo {
            InitializedRepo::Github(g) => {
                let github_repo_handler = GithubRepoHandler {
                    client: octocrab::instance(),
                };
                github_repo_handler.clone_local(g, path)
            },
        }
    }
}

struct GithubRepoHandler {
    client: octocrab::Octocrab,
}

impl GithubRepoHandler {
    async fn create(&self, github_params: GithubRepoParams) -> Result<InitializedGithubRepo, Box<dyn Error>> {
        let new_repo = NewGithubRepoParams {
            name: github_params.name.clone(),
            description: github_params.description.clone(),
            private: false,
            has_issues: true,
            has_projects: true,
            has_wiki: true,
        };

        let _response: serde_json::Value = match github_params.organization.clone() {
            GithubUser::User(_) => octocrab::instance().post("/user/repos", Some(&new_repo)).await?,
            GithubUser::Organization(name) => {
                octocrab::instance()
                    .post(format!("/orgs/{}/repos", name), Some(&new_repo))
                    .await?
            }
        };

        info!("Github Repo Created: {}", github_params.name);
        let rce = RepositoryCreatedEvent {
             context: RepositoryCreatedEventContext {
                id: RepositoryCreatedEventContextId::from_str(format!("{}/{}", github_params.organization.get_name(), github_params.name.clone()).as_str())?,
                source: "skootrs.github.creator".into(),
                timestamp: Utc::now(),
                type_: crate::model::cd_events::repo_created::RepositoryCreatedEventContextType::DevCdeventsRepositoryCreated011,
                version: RepositoryCreatedEventContextVersion::from_str("0.3.0")?,
            }, 
             custom_data: None,
             custom_data_content_type: None,
             subject: RepositoryCreatedEventSubject {
                content: RepositoryCreatedEventSubjectContent{
                    name: RepositoryCreatedEventSubjectContentName::from_str(github_params.name.as_str())?,
                    owner: Some(github_params.organization.get_name()),
                    url: RepositoryCreatedEventSubjectContentUrl::from_str(github_params.full_url().as_str())?,
                    view_url: Some(github_params.full_url()),
                },
                id: RepositoryCreatedEventSubjectId::from_str(format!("{}/{}", github_params.organization.get_name(), github_params.name.clone()).as_str())?,
                source: Some("skootrs.github.creator".into()),
                type_: crate::model::cd_events::repo_created::RepositoryCreatedEventSubjectType::Repository,
            } 
        };
        info!("{}", serde_json::to_string(&rce)?);

        Ok(InitializedGithubRepo {
            name: github_params.name.clone(),
            organization: github_params.organization.clone(),
        })
    }

    fn clone_local(&self, initialized_github_repo: InitializedGithubRepo, path: String) -> Result<InitializedSource, Box<dyn Error>> {
        let clone_url = initialized_github_repo.full_url();
        let _output = Command::new("git")
            .arg("clone")
            .arg(clone_url)
            .arg(path)
            .output()?;

        Ok(InitializedSource{
            path,
        })
    }
}

/// This is needed to easily send over Github new repo parameters to the post.
#[derive(serde::Serialize)]
struct NewGithubRepoParams {
    name: String,
    description: String,
    private: bool,
    has_issues: bool,
    has_projects: bool,
    has_wiki: bool,
}