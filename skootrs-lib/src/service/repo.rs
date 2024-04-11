
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

#![allow(clippy::module_name_repetitions)]

use std::{process::Command, str::FromStr, sync::Arc};

use chrono::Utc;
use octocrab::Octocrab;
use tracing::{info, debug};

use skootrs_model::{cd_events::repo_created::{RepositoryCreatedEvent, RepositoryCreatedEventContext, RepositoryCreatedEventContextId, RepositoryCreatedEventContextVersion, RepositoryCreatedEventSubject, RepositoryCreatedEventSubjectContent, RepositoryCreatedEventSubjectContentName, RepositoryCreatedEventSubjectContentUrl, RepositoryCreatedEventSubjectId}, skootrs::{InitializedRepoGetParams, GithubRepoParams, GithubUser, InitializedGithubRepo, InitializedRepo, InitializedSource, RepoCreateParams, SkootError}};

/// The `RepoService` trait provides an interface for initializing and managing a project's source code
/// repository. This repo is usually something like Github or Gitlab.
pub trait RepoService {
    /// Initializes a project's source code repository. This is usually a remote repo hosted on a service
    /// like Github or Gitlab.
    ///
    /// # Errors
    ///
    /// Returns an error if the source code repository can't be initialized.
    fn initialize(&self, params: RepoCreateParams) -> impl std::future::Future<Output = Result<InitializedRepo, SkootError>> + Send;  

    /// Gets a project's source code repository metadata abstraction.
    ///
    /// # Errors
    ///
    /// Returns an error if the source code repository metadata can't be retrieved.
    fn get(&self, params: InitializedRepoGetParams) -> impl std::future::Future<Output = Result<InitializedRepo, SkootError>> + Send; 

    /// Clones a project's source code repository to the local machine.
    ///
    /// # Errors
    ///
    /// Returns an error if the source code repository can't be cloned to the local machine.
    fn clone_local(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, SkootError>;

    /// Clones a project's source code repository to the local machine, or pulls it if it already exists.
    ///
    /// # Errors
    /// 
    /// Returns an error if the source code repository can't be cloned or if updates can't be pulled.
    fn clone_local_or_pull(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, SkootError>;

    /// Fectches an arbitrary file from the repository. This is useful for things like fetching a remote
    /// Skootrs state file, or something like a remote SECURITY-INSIGHTS file kept in the repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be fetched from the repository for any reason.
    fn fetch_file_content<P: AsRef<std::path::Path> + Send>(&self, initialized_repo: &InitializedRepo, path: P) -> impl std::future::Future<Output = Result<String, SkootError>> + std::marker::Send;

    fn archive(&self, initialized_repo: InitializedRepo) -> impl std::future::Future<Output = Result<String, SkootError>> + Send;
}

/// The `LocalRepoService` struct provides an implementation of the `RepoService` trait for initializing
/// and managing a project's source code repository from the local machine. This doesn't mean the repo is
/// local, but that the operations like API calls are run from the local machine.
#[derive(Debug)]
pub struct LocalRepoService {}

impl RepoService for LocalRepoService {
    async fn initialize(&self, params: RepoCreateParams) -> Result<InitializedRepo, SkootError> {
        // TODO: The octocrab initialization should be done in a better place and be parameterized
        let o: octocrab::Octocrab = octocrab::Octocrab::builder()
            .personal_token(
                    std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var must be populated"),
            )
            .build()?;
        octocrab::initialise(o);
        match params {
            RepoCreateParams::Github(g) => {
                let github_repo_handler = GithubRepoHandler {
                    client: octocrab::instance(),
                };
                Ok(InitializedRepo::Github(github_repo_handler.create(g).await?))
            },
        }
    }

    fn clone_local(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, SkootError> {
        match initialized_repo {
            InitializedRepo::Github(g) => {
                GithubRepoHandler::clone_local(&g, &path)
            },
        }
    }
    
    fn clone_local_or_pull(&self, initialized_repo: InitializedRepo, path: String) -> Result<InitializedSource, SkootError> {
        // Check if path exists and is a git repo
        let output = Command::new("git")
            .arg("status")
            .current_dir(&path)
            .output()?;

        // If it is, pull updates
        if output.status.success() {
            let _output = Command::new("git")
                .arg("pull")
                .current_dir(&path)
                .output()?;
            Ok(InitializedSource {
                path,
            })
        } else {
            // If it isn't, clone the repo
            self.clone_local(initialized_repo, path)
        }
    }
    
    async fn get(&self, params: InitializedRepoGetParams) -> Result<InitializedRepo, SkootError> {
        let parsed_url = url::Url::parse(&params.repo_url)?;
        match parsed_url.host_str() {
            Some("github.com") => {
                let path = parsed_url.path();
                let parts: Vec<&str> = path.split('/').collect();
                let organization = parts[1];    
                let name = parts[2];
                let exists = octocrab::instance().repos(organization, name).get().await.is_ok();
                if !exists {
                    return Err("Repo does not exist".into());
                }
                Ok(InitializedRepo::Github(InitializedGithubRepo {
                    name: name.to_string(),
                    // FIXME: This will probably break in weird ways since repos from a user and organization are handled
                    // slightly different in the Github API. I am not sure yet what the best way to determine if a repo
                    // belongs to a user or organization is.
                    organization: GithubUser::User(organization.to_string()),
                }))
            },
            Some(_) => Err("Unsupported repo host".into()),
            _ => Err("Invalid repo URL".into()),
        }
    }

    async fn fetch_file_content<P: AsRef<std::path::Path> + Send>(&self, initialized_repo: &InitializedRepo, path: P) -> Result<String, SkootError> {
        match &initialized_repo {
            InitializedRepo::Github(g) => {
                let path_str = path.as_ref().to_str().ok_or_else(|| SkootError::from("Failed to convert path to string"))?;
                let content_items = octocrab::instance().repos(
                    g.organization.get_name(), g.name.clone()
                )
                .get_content()
                .path(path_str)
                // TODO: Should this support multiple branches?
                .r#ref("main")
                .send()
                .await?;

                let content = content_items
                .items
                .first()
                .ok_or_else(|| SkootError::from(format!("Failed to get {} from {}", path_str, initialized_repo.full_url())))?;

                debug!("Content: {content:?}");
                let content_decoded = content.decoded_content().ok_or_else(|| SkootError::from(format!("Failed to decode content from {path_str}")))?;
                debug!("Content Decoded: {content_decoded:?}");
                
                Ok(content_decoded)
            }
        }
    }

    async fn archive(&self, initialized_repo: InitializedRepo) -> Result<String, SkootError> {
        match initialized_repo {
            InitializedRepo::Github(g) => {
                #[derive(serde::Serialize)]
                struct ArchiveParams {
                    archived: bool,
                }
                let owner = g.organization.get_name();
                let repo = g.name.clone();
                let body = ArchiveParams {
                    archived: true,
                };

                info!("Archiving {owner}/{repo}");

                // FIXME: This should work with `Octocrabe::instance()` but for some reason it doesn't pick up the token/session
                let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
                let octocrab = Octocrab::builder().personal_token(token).build()?;
                let archived_response: serde_json::Value = octocrab.patch(format!("/repos/{owner}/{repo}"), Some(&body)).await?;
                info!("Archived: {archived_response}");

                Ok(g.full_url())
            }
        }
    }
}

/// The `GithubRepoHandler` struct represents a handler for initializing and managing Github repos.
#[derive(Debug)]
struct GithubRepoHandler {
    client: Arc<octocrab::Octocrab>,
}

impl GithubRepoHandler {
    async fn create(&self, github_params: GithubRepoParams) -> Result<InitializedGithubRepo, SkootError> {
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
                self.client
                    .post(format!("/orgs/{name}/repos"), Some(&new_repo))
                    .await?
            }
        };

        info!("Github Repo Created: {}", github_params.name);
        let rce = RepositoryCreatedEvent {
             context: RepositoryCreatedEventContext {
                id: RepositoryCreatedEventContextId::from_str(format!("{}/{}", github_params.organization.get_name(), github_params.name.clone()).as_str())?,
                source: "skootrs.github.creator".into(),
                timestamp: Utc::now(),
                type_: skootrs_model::cd_events::repo_created::RepositoryCreatedEventContextType::DevCdeventsRepositoryCreated011,
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
                type_: skootrs_model::cd_events::repo_created::RepositoryCreatedEventSubjectType::Repository,
            } 
        };

        // TODO: Turn this into an event
        info!("{}", serde_json::to_string(&rce)?);

        Ok(InitializedGithubRepo {
            name: github_params.name.clone(),
            organization: github_params.organization.clone(),
        })
    }

    fn clone_local(initialized_github_repo: &InitializedGithubRepo, path: &str) -> Result<InitializedSource, SkootError> {
        debug!("Cloning {}", initialized_github_repo.full_url());
        let clone_url = initialized_github_repo.full_url();
        let _output = Command::new("git")
            .arg("clone")
            .arg(clone_url)
            .current_dir(path)
            .output()?;

        Ok(InitializedSource{
            path: format!("{}/{}", path, initialized_github_repo.name),
        })
    }
}

/// This is needed to easily send over Github new repo parameters to the post.
#[allow(clippy::struct_excessive_bools)] // Clippy doesn't like the Github API
#[derive(serde::Serialize)]
struct NewGithubRepoParams {
    name: String,
    description: String,
    private: bool,
    has_issues: bool,
    has_projects: bool,
    has_wiki: bool,
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;

    // TODO: Mock out, or create test to create a repo/delete a repo

    #[test]
    fn test_clone_local_github_repo() {
        let initialized_github_repo = InitializedGithubRepo {
            name: "skootrs".to_string(),
            organization: GithubUser::Organization("kusaridev".to_string()),
        };

        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let result = GithubRepoHandler::clone_local(&initialized_github_repo, path);
        assert!(result.is_ok());

        let initialized_source = result.unwrap();
        assert_eq!(
            initialized_source.path,
            format!("{}/{}", path, initialized_github_repo.name)
        );
    }
}
