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

use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

/// The general structure of the models here is the struct names take the form:
/// <Thing>Params reflecting the parameters for something to be created or initilized, like the parameters
/// to create a repo or project.
/// 
/// Initialized<Thing> models the data and state for a created or initialized thing, like a repo created inside of Github.
/// This module is purely focused on the data for skootrs, and not for performing any of the operations. In order to make
/// it easy for (de)serialization, the structs and impls only contain the logic for the data, and not for the operations,
/// which falls under service.

// TODO: These should be their own structs, but they're currently not any different from the params structs.
pub type InitializedGo = GoParams;
pub type InitializedMaven = MavenParams;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct InitializedProject {
    pub repo: InitializedRepo,
    pub ecosystem: InitializedEcosystem,
    pub source: InitializedSource
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ProjectParams {
    pub name: String,
    pub repo_params: RepoParams,
    pub ecosystem_params: EcosystemParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum InitializedRepo {
    Github(InitializedGithubRepo)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct InitializedGithubRepo {
    pub name: String,
    pub organization: GithubUser,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum InitializedEcosystem {
    Go(InitializedGo),
    Maven(InitializedMaven)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum RepoParams {
    Github(GithubRepoParams)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum EcosystemParams {
    Go(GoParams),
    Maven(MavenParams)
}

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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct GithubRepoParams {
    pub name: String,
    pub description: String,
    pub organization: GithubUser,
}

impl GithubRepoParams {
    pub fn host_url(&self) -> String {
        "https://github.com".into()
    }

    pub fn full_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_url(),
            self.organization.get_name(),
            self.name
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct SourceParams {
    pub path: String,
    pub repo: InitializedRepo
}

/// Struct representing a working copy of source code.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InitializedSource {
    pub path: String
}

/// Represents the Maven ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct MavenParams {
    /// The group ID of the Maven project.
    pub group_id: String,
    /// The artifact ID of the Maven project.
    pub artifact_id: String,
}

/// Represents the Go ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct GoParams {
    /// The name of the Go module.
    pub name: String,
    /// The host of the Go module.
    pub host: String,
}

impl GoParams {
    /// Returns the module name in the format "{host}/{name}".
    pub fn module(&self) -> String {
        format!("{}/{}", self.host, self.name)
    }
}
