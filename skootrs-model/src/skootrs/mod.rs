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

pub mod facet;

use std::error::Error;

use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use self::facet::InitializedFacet;

pub type SkootError = Box<dyn Error + Send + Sync>;

/// The general structure of the models here is the struct names take the form:
/// `<Thing>Params` reflecting the parameters for something to be created or initilized, like the parameters
/// to create a repo or project.
/// 
/// `Initialized<Thing>` models the data and state for a created or initialized thing, like a repo created inside of Github.
/// This module is purely focused on the data for skootrs, and not for performing any of the operations. In order to make
/// it easy for (de)serialization, the structs and impls only contain the logic for the data, and not for the operations,
/// which falls under service.
// TODO: These categories of structs should be moved to their own modules.
/// Consts for the supported ecosystems, repos, etc. for convenient use by things like the CLI.
pub const SUPPORTED_ECOSYSTEMS: [&str; 2] = [
    "Go",
    "Maven"
];

// TODO: These should be their own structs, but they're currently not any different from the params structs.

/// Represents a project that has been initialized. This is the data and state of a project that has been 
/// created.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedProject {
    pub repo: InitializedRepo,
    pub ecosystem: InitializedEcosystem,
    pub source: InitializedSource,
    pub facets: Vec<InitializedFacet>,
}

/// Represents the parameters for creating a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectParams {
    pub name: String,
    pub repo_params: RepoParams,
    pub ecosystem_params: EcosystemParams,
    pub source_params: SourceParams,
}

/// Represents an initialized repository along with its host.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedRepo {
    Github(InitializedGithubRepo)
}

impl InitializedRepo {
    /// Returns the host URL of the repo.
    #[must_use] pub fn host_url(&self) -> String {
        match self {
            Self::Github(x) => x.host_url(),
        }
    }

    /// Returns the full URL to the repo.
    #[must_use] pub fn full_url(&self) -> String {
        match self {
            Self::Github(x) => x.full_url(),
        }
    }
}

/// Represents an initialized Github repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedGithubRepo {
    pub name: String,
    pub organization: GithubUser,
}

impl InitializedGithubRepo {
    /// Returns the host URL of github.
    #[must_use] pub fn host_url(&self) -> String {
        "https://github.com".into()
    }

    /// Returns the full URL to the github repo.
    #[must_use] pub fn full_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_url(),
            self.organization.get_name(),
            self.name
        )
    }
}

/// Represents an initialized ecosystem. The enum is used to represent the different types of ecosystems
/// that are supported by Skootrs currently.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedEcosystem {
    Go(InitializedGo),
    Maven(InitializedMaven)
}

/// Represents the parameters for creating a repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RepoParams {
    Github(GithubRepoParams)
}

/// Represents the parameters for initializing an ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum EcosystemParams {
    Go(GoParams),
    Maven(MavenParams)
}

/// Represents a Github user which is really just whether or not a repo belongs to  a user or organization.
/// This is used to create a repo in the Github API. The Github API has different calls for creating a repo
/// that belongs to the current authorized user or an organization the user has access to.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum GithubUser {
    User(String),
    Organization(String),
}

impl GithubUser {
    /// Returns the name of the user or organization.
    #[must_use] pub fn get_name(&self) -> String {
        match self {
            Self::User(x) |
            Self::Organization(x) => x.to_string(),
        }
    }
}

/// Represents the parameters for creating a Github repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GithubRepoParams {
    pub name: String,
    pub description: String,
    pub organization: GithubUser,
}

impl GithubRepoParams {
    #[must_use] pub fn host_url(&self) -> String {
        "https://github.com".into()
    }

    #[must_use] pub fn full_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_url(),
            self.organization.get_name(),
            self.name
        )
    }
}

/// Represents the parameters for initializing a source code repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceParams {
    pub parent_path: String,
}

impl SourceParams {
    /// Returns the full path to the source code repository with the given name.
    #[must_use] pub fn path(&self, name: &str) -> String {
        format!("{}/{}", self.parent_path, name)
    }
}

/// Struct representing a working copy of source code.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InitializedSource {
    pub path: String
}

/// Represents the Maven ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MavenParams {
    /// The group ID of the Maven project.
    pub group_id: String,
    /// The artifact ID of the Maven project.
    pub artifact_id: String,
}

/// Represents the Go ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GoParams {
    /// The name of the Go module.
    pub name: String,
    /// The host of the Go module.
    pub host: String,
}

/// Represents an initialized go module.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedGo {
    /// The name of the Go module.
    pub name: String,
    /// The host of the Go module.
    pub host: String,
}

impl InitializedGo {
    /// Returns the module name in the format "{host}/{name}".
    #[must_use] pub fn module(&self) -> String {
        format!("{}/{}", self.host, self.name)
    }
}

/// Represents an initialized Maven project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedMaven {
    /// The group ID of the Maven project.
    pub group_id: String,
    /// The artifact ID of the Maven project.
    pub artifact_id: String,
}

impl GoParams {
    /// Returns the module name in the format "{host}/{name}".
    #[must_use] pub fn module(&self) -> String {
        format!("{}/{}", self.host, self.name)
    }
}
