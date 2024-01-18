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

/*use std::{process::Command, error::Error, str::FromStr};

use serde::{Serialize, Deserialize};
use tracing::info;
use utoipa::ToSchema;

use crate::{source::Source, model::cd_events::repo_created::{RepositoryCreatedEvent, RepositoryCreatedEventContext, RepositoryCreatedEventContextId, RepositoryCreatedEventSubject, RepositoryCreatedEventContextVersion, RepositoryCreatedEventSubjectContent, RepositoryCreatedEventSubjectContentName, RepositoryCreatedEventSubjectContentUrl, RepositoryCreatedEventSubjectId}};

use super::{UninitializedRepo, InitializedRepo};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct UninitializedGithubRepo {
    //pub client: Arc<octocrab::Octocrab>,
    pub name: String,
    pub description: String,
    pub organization: GithubUser,
}

/// Enum representing whether or not a particular permissioned path in Github, e.g. github.com/kusaridev references
/// a user or an organization. This is useful since the way you create and manage repos within Github is different
/// depending on whether it's owned by a User or an Organization.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum GithubUser {
    User(String),
    Organization(String),
}

impl GithubUser {
    pub fn get_name(&self) -> String {
        match self {
            GithubUser::User(x) => x.to_string(),
            GithubUser::Organization(x) => x.to_string(),
        }
    }
}

impl UninitializedRepo for UninitializedGithubRepo {
    type Repo = InitializedGithubRepo;

    /// Returns an initialized Repo if it is successfully created in Github, otherwise it returns
    /// an error.
    async fn create(&self) -> Result<Self::Repo, Box<dyn Error>> {
        let new_repo = NewGithubRepoParams {
            name: self.name.clone(),
            description: self.description.clone(),
            private: false,
            has_issues: true,
            has_projects: true,
            has_wiki: true,
        };

        let _response: serde_json::Value = match self.organization.clone() {
            GithubUser::User(_) => octocrab::instance().post("/user/repos", Some(&new_repo)).await?,
            GithubUser::Organization(name) => {
                octocrab::instance()
                    .post(format!("/orgs/{}/repos", name), Some(&new_repo))
                    .await?
            }
        };

        info!("Github Repo Created: {}", self.name);
        let rce = RepositoryCreatedEvent {
             context: RepositoryCreatedEventContext {
                id: RepositoryCreatedEventContextId::from_str(format!("{}/{}", self.organization.get_name(), self.name.clone()).as_str())?,
                source: "skootrs.github.creator".into(),
                timestamp: Utc::now(),
                type_: crate::model::cd_events::repo_created::RepositoryCreatedEventContextType::DevCdeventsRepositoryCreated011,
                version: RepositoryCreatedEventContextVersion::from_str("0.3.0")?,
            }, 
             custom_data: None,
             custom_data_content_type: None,
             subject: RepositoryCreatedEventSubject {
                content: RepositoryCreatedEventSubjectContent{
                    name: RepositoryCreatedEventSubjectContentName::from_str(self.name.as_str())?,
                    owner: Some(self.organization.get_name()),
                    url: RepositoryCreatedEventSubjectContentUrl::from_str(self.full_url().as_str())?,
                    view_url: Some(self.full_url()),
                },
                id: RepositoryCreatedEventSubjectId::from_str(format!("{}/{}", self.organization.get_name(), self.name.clone()).as_str())?,
                source: Some("skootrs.github.creator".into()),
                type_: crate::model::cd_events::repo_created::RepositoryCreatedEventSubjectType::Repository,
            } 
        };
        info!("{}", serde_json::to_string(&rce)?);

        Ok(InitializedGithubRepo {
            name: self.name.clone(),
            organization: self.organization.clone(),
        })
    }

    async fn create_from_template(
        &self,
        _repo: Box<dyn InitializedRepo>,
    ) -> Result<Box<dyn InitializedRepo>, Box<dyn Error>> {
        todo!()
    }

    fn host_url(&self) -> String {
        "https://github.com".into()
    }

    fn full_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_url(),
            self.organization.get_name(),
            self.name
        )
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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct InitializedGithubRepo {
    pub name: String,
    pub organization: GithubUser,
}

impl InitializedRepo for InitializedGithubRepo {
    fn clone_repo(&self, path: String) -> Result<Source, Box<dyn Error>> {
        let url = self.full_url();
        let output = Command::new("git")
            .arg("clone")
            .arg(&url)
            .current_dir(&path)
            .output()?;
        if !output.status.success() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to clone repository",
            )))
        } else {
            info!("Cloned {} to {}/{}", url, path, self.name);
            Ok(Source { path: format!("{}/{}", path, self.name) })
        }
    }

    fn host_url(&self) -> String {
        "https://github.com".into()
    }

    fn full_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_url(),
            self.organization.get_name(),
            self.name
        )
    }
}*/