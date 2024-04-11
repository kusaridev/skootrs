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

use std::{collections::HashMap, error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};
use url::Host;
use utoipa::ToSchema;

use self::facet::{InitializedFacet, SupportedFacetType};

/// A helper type for the error type used throughout Skootrs. This is a `Box<dyn Error + Send + Sync>`.
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
pub const SUPPORTED_ECOSYSTEMS: [&str; 2] = ["Go", "Maven"];

/// The set of supported ecosystems.
#[derive(Serialize, Deserialize, Clone, Debug, EnumString, VariantNames, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SupportedEcosystems {
    /// The Go ecosystem
    #[default]
    Go,
    /// The Maven ecosystem
    Maven,
}

// TODO: These should be their own structs, but they're currently not any different from the params structs.

/// Represents a project that has been initialized. This is the data and state of a project that has been
/// created.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedProject {
    /// The metadata associated with an Skootrs initilialized source repository.
    pub repo: InitializedRepo,
    /// The metadata associated with an Skootrs initilialized ecosystem.
    pub ecosystem: InitializedEcosystem,
    /// The metadata associated with an Skootrs initilialized source location.
    pub source: InitializedSource,
    /// The facets associated with the project.
    pub facets: HashMap<FacetMapKey, InitializedFacet>,
}

/// A helper enum for how a facet can be pulled from a `HashMap`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(try_from = "String", into = "String")]
pub enum FacetMapKey {
    /// A map key based on the name of a facet. Useful for filtering based on the name of a facet.
    Name(String),
    /// A map key based on the type of a facet. Useful for filtering based on the type of facet.
    Type(SupportedFacetType),
}

impl TryFrom<String> for FacetMapKey {
    type Error = SkootError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(": ").collect();
        if parts.len() != 2 {
            return Err("Invalid facet map key".into());
        }
        match parts.first() {
            Some(&"Name") => Ok(Self::Name(parts[1].to_string())),
            Some(&"Type") => Ok(Self::Type(parts[1].parse()?)),
            _ => Err("Invalid facet map key".into()),
        }
    }
}

impl From<FacetMapKey> for String {
    fn from(val: FacetMapKey) -> Self {
        match val {
            FacetMapKey::Name(x) => format!("Name: {x}"),
            FacetMapKey::Type(x) => format!("Type: {x}"),
        }
    }
}

impl FromStr for FacetMapKey {
    type Err = SkootError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(": ").collect();
        if parts.len() != 2 {
            return Err("Invalid facet map key".into());
        }
        match parts.first() {
            Some(&"Name") => Ok(Self::Name(parts[1].to_string())),
            Some(&"Type") => Ok(Self::Type(parts[1].parse()?)),
            _ => Err("Invalid facet map key".into()),
        }
    }
}

// TODO: This seems redundant with From<FacetMapKey> for String.
// I am not sure why this can't be automatically derived
impl fmt::Display for FacetMapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        String::from(self.clone()).fmt(f)?;
        Ok(())
    }
}

/// The parameters for creating a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectCreateParams {
    /// The name of the project to be created.
    pub name: String,
    /// The parameters for creating the repository for the project.
    pub repo_params: RepoCreateParams,
    /// The parameters for initializing the ecosystem for the project.
    pub ecosystem_params: EcosystemInitializeParams,
    /// The parameters for initializing the source code for the project.
    pub source_params: SourceInitializeParams,
}

/// The parameters for getting an existing Skootrs project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectGetParams {
    /// The URL of the Skootrs project to get.
    pub project_url: String,
}

/// The parameters for listing all the outputs for a Skootrs project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectOutputsListParams {
    /// The initialized project to list the outputs for.
    pub initialized_project: InitializedProject,
    /// The release to get the outputs for.
    pub release: ProjectReleaseParam,
}

/// The parameters for getting a release from a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum ProjectReleaseParam {
    /// A release based on a tag.
    Tag(String),
    /// The latest release.
    Latest,
}

impl ProjectReleaseParam {
    /// Returns the tag of the release.
    #[must_use]
    pub fn tag(&self) -> Option<String> {
        match self {
            Self::Tag(x) => Some(x.to_string()),
            Self::Latest => None,
        }
    }
}

/// The paramaters for getting the output of a project, e.g. an SBOM from a release
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectOutputParams {
    /// The URL of the Skootrs project to get the output from.
    pub project_url: String,
    /// The type of output to get from the project.
    pub project_output_type: ProjectOutputType,
    // TODO: Should project_output be a part of the ProjectOutputType enum?
    /// The output to get from the project.
    pub project_output: String,
}

/// The parameters for archiving a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectArchiveParams {
    /// The initialized project to archive.
    pub initialized_project: InitializedProject,
}

/// The set of supported output types
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum ProjectOutputType {
    /// An output type for getting an SBOM from a project.
    SBOM,
    /// An output type for getting a custom output from a project.
    Custom(String),
}

/// The output of a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectOutput {
    /// The reference to the project output.
    pub reference: ProjectOutputReference,
    /// The output to get from the project.
    pub output: String,
}

/// A reference to the output of a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProjectOutputReference {
    /// The type of output to get from the project.
    pub output_type: ProjectOutputType,
    /// The name of the output to get from the project.
    pub name: String,
}

/// The parameters for getting a facet from a project.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FacetGetParams {
    /// Parameters for first getting the project.
    pub project_get_params: ProjectGetParams,
    /// The key of the facet to get from the project.
    pub facet_map_key: FacetMapKey,
}

/// Represents an initialized repository along with its host.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedRepo {
    /// An initialized Github repository.
    Github(InitializedGithubRepo),
}

impl InitializedRepo {
    /// Returns the host URL of the repo.
    #[must_use]
    pub fn host_url(&self) -> String {
        match self {
            Self::Github(x) => x.host_url(),
        }
    }

    /// Returns the full URL to the repo.
    #[must_use]
    pub fn full_url(&self) -> String {
        match self {
            Self::Github(x) => x.full_url(),
        }
    }
}

impl TryFrom<String> for InitializedRepo {
    type Error = SkootError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts = url::Url::parse(&value)?;
        let path_segments = parts
            .path_segments()
            .map_or(Vec::new(), Iterator::collect::<Vec<_>>);
        if path_segments.len() != 2 {
            return Err(format!("Invalid repo URL: {value}").into());
        }

        let organization = *path_segments
            .first()
            .ok_or(format!("Invalid repo URL: {value}"))?;
        let name = *path_segments
            .get(1)
            .ok_or(format!("Invalid repo URL: {value}"))?;
        match parts.host() {
            Some(Host::Domain("github.com")) => {
                Ok(Self::Github(InitializedGithubRepo {
                    name: name.to_string(),
                    // FIXME: This will have issues if this isn't a user repo and in fact an organization user.
                    organization: GithubUser::User(organization.into()),
                }))
            }
            _ => Err("Unsupported repo host".into()),
        }
    }
}

/// Represents an initialized Github repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct InitializedGithubRepo {
    /// The name of the Github repository.
    pub name: String,
    /// The organization the Github repository belongs to.
    pub organization: GithubUser,
}

impl InitializedGithubRepo {
    /// Returns the host URL of github.
    #[must_use]
    pub fn host_url(&self) -> String {
        "https://github.com".into()
    }

    /// Returns the full URL to the github repo.
    #[must_use]
    pub fn full_url(&self) -> String {
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
    /// An initialized Go ecosystem for `InitializedSource`.
    Go(InitializedGo),
    /// An initialized Maven ecosystem `InitializedSource`.
    Maven(InitializedMaven),
}

/// The parameters for creating a repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RepoCreateParams {
    /// The parameters for creating a Github repository.
    Github(GithubRepoParams),
}

/// The parameters for initializing an ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum EcosystemInitializeParams {
    /// The parameters for initializing a Go ecosystem for `InitializedSource`.
    Go(GoParams),
    /// The parameters for initializing a Maven ecosystem for `InitializedSource`.
    Maven(MavenParams),
}

/// The parameter for getting an initialized repository
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitializedRepoGetParams {
    /// The URL of the repository that Skootrs has previously initialized and you want to get.
    pub repo_url: String,
}

/// Represents a Github user which is really just whether or not a repo belongs to  a user or organization.
/// This is used to create a repo in the Github API. The Github API has different calls for creating a repo
/// that belongs to the current authorized user or an organization the user has access to.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum GithubUser {
    /// A Github user, i.e. not an organization.
    User(String),
    /// A Github organization, i.e. not a user.
    Organization(String),
}

impl GithubUser {
    /// Returns the name of the user or organization.
    #[must_use]
    pub fn get_name(&self) -> String {
        match self {
            Self::User(x) | Self::Organization(x) => x.to_string(),
        }
    }
}

/// Represents the parameters for creating a Github repository.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GithubRepoParams {
    /// The name of the Github repository.
    pub name: String,
    /// The description of the Github repository.
    pub description: String,
    /// The organization the Github repository belongs to.
    pub organization: GithubUser,
}

impl GithubRepoParams {
    /// Helper for returning the github host.
    #[must_use]
    pub fn host_url(&self) -> String {
        "https://github.com".into()
    }

    /// Helper for returning the full URL to the github repo.
    #[must_use]
    pub fn full_url(&self) -> String {
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
pub struct SourceInitializeParams {
    /// The parent path of the source code repository.
    pub parent_path: String,
}

impl SourceInitializeParams {
    /// Returns the full path to the source code repository with the given name.
    #[must_use]
    pub fn path(&self, name: &str) -> String {
        format!("{}/{}", self.parent_path, name)
    }
}

/// Struct representing a working copy of source code.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InitializedSource {
    /// The path to the source code repository.
    pub path: String,
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
    #[must_use]
    pub fn module(&self) -> String {
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
    #[must_use]
    pub fn module(&self) -> String {
        format!("{}/{}", self.host, self.name)
    }
}

/// A set of configuration options for Skootrs.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Config {
    /// The local path to cached projects. This is used by `LocalProjectService` for performing operations locally.
    pub local_project_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_project_path: "/tmp".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn test_initialized_repo_try_from() {
        let repo: InitializedRepo =
            InitializedRepo::try_from("https://github.com/kusaridev/skootrs".to_string()).unwrap();
        assert_eq!(repo.host_url(), "https://github.com");
        assert_eq!(repo.full_url(), "https://github.com/kusaridev/skootrs");
    }
}
