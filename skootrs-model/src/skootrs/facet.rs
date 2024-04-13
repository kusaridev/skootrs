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

//! The `facet` module provides the data model for a project's facets,
//! with are various elements of a project, usually for security purposes.
//! This includes things like README, SECURITY.md, as well as API calls
//! like enabling branch protection on GitHub.

#![allow(clippy::module_name_repetitions)]

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use strum::VariantNames;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::{
    label::{Label, Labeled},
    InitializedEcosystem, InitializedRepo, InitializedSource,
};
use strum::EnumString;

/// Represents a facet that has been initialized. This is an enum of
/// the various supported facets like API based, and Source file bundle
/// based.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedFacet {
    /// A facet that is based on a bundle of source files.
    SourceBundle(SourceBundleFacet),
    /// A facet that is based on one or more API calls.
    APIBundle(APIBundleFacet),
}

impl InitializedFacet {
    /// Helper function to get the facet type of the inner facet.
    #[must_use]
    pub fn facet_type(&self) -> SupportedFacetType {
        match self {
            Self::SourceBundle(facet) => facet.facet_type.clone(),
            Self::APIBundle(facet) => facet.facet_type.clone(),
        }
    }

    /// Helper function to get the labels of the inner facet.
    #[must_use]
    pub fn labels(&self) -> Vec<Label> {
        match self {
            Self::SourceBundle(s) => s.labels(),
            Self::APIBundle(a) => a.labels(),
        }
    }
}

/// Represents the parameters for creating a facet. This should mirror the
/// `InitializedFacet` enum.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum FacetCreateParams {
    /// Params for creating a `SourceBundleFacet`.
    SourceBundle(SourceBundleFacetCreateParams),
    /// Params for creating a `APIBundleFacet`.
    APIBundle(APIBundleFacetParams),
}

/// This is required to create an ordering of what facets get applied.
/// There could be issues like a security feature being enabled before
/// some other feature, which could lead to it being blocked.
/// for example, enabling branch protection before pushing the initial
/// boilerplate code.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FacetSetCreateParams {
    /// The parameters for each `InitializedFacet` that should be created.
    pub facets_params: Vec<FacetCreateParams>,
}

/// Represents the common parameters that are shared across all facets.
/// This is mostly the context of the project, like the project name,
/// source, repo, and ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CommonFacetCreateParams {
    /// The name of the project the facet is being created for.
    pub project_name: String,
    /// The source of the project the facet is being created for.
    pub source: InitializedSource,
    /// The repo of the project the facet is being created for.
    pub repo: InitializedRepo,
    /// The ecosystem of the project the facet is being created for.
    pub ecosystem: InitializedEcosystem,
}

/// Represents the content of a source file.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileContent {
    /// The name of the source file.
    pub name: String,
    /// The path of the source file.
    pub path: String,
    // TODO: Since the content can change out of band of Skootrs
    // should we even store the content in the database?
    /// The `String` content of the source file as a normal UTF8 string.
    pub content: String,
}

/// Represents a source file.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(try_from = "String", into = "String")]
pub struct SourceFile {
    /// The name of the source file.
    pub name: String,
    /// The path of the source file.
    pub path: String,
    /// The hash of the source file.
    pub hash: String,
}

impl TryFrom<String> for SourceFile {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid source file string: {value}"));
        }
        Ok(Self {
            name: parts[0].to_string(),
            path: parts[1].to_string(),
            hash: parts[2].to_string(),
        })
    }
}

impl From<SourceFile> for String {
    fn from(value: SourceFile) -> Self {
        format!("{}:{}:{}", value.name, value.path, value.hash)
    }
}

/// Represents a source bundle facet which is a facet that is based
/// on a bundle of source files. This can be a single file like a
/// README, or a collection of related files like several yaml files
/// for a set of actions in a github workflow.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacet {
    /// The source files that make up the facet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_files: Option<Vec<SourceFile>>,
    /// The type of facet this is.
    pub facet_type: SupportedFacetType,
    /// The content of the source files that make up the facet. This is a map of the source file to the content of the source file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_files_content: Option<HashMap<SourceFile, String>>,
    /// The labels for the facet.
    pub labels: Vec<Label>,
}

/// Represents the parameters for creating a source bundle facet.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacetCreateParams {
    /// The common parameters for the facet being created.
    pub common: CommonFacetCreateParams,
    /// The type of facet that is being created.
    pub facet_type: SupportedFacetType,
    /// The labels for the facet.
    pub labels: Vec<Label>,
}

/// Represents the content of an API call. This just includes the
/// name of the API call, the URL to the API call and the response.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIContent {
    /// The name of the API call.
    pub name: String,
    /// The URL of the API call.
    pub url: String,
    // TODO: The response can be multiple things like a response code,
    // a JSON response, or something else. Should we store this as a
    // more comples type.
    /// The response of the API call as a `String`.
    pub response: String,
    // TODO: Include the request as well. This could be useful for
    // audits, as well as for evaluating when the API facet might
    // need to be changed if an API call upstream changes.
}

/// Represents an API bundle facet which is a facet that is based on
/// an api call. This can be a single API call, or a collection of
/// related API calls that represent a single facet. For example, a
/// facet might expect multiple security flags on a GitHub project
/// to be set that can't be included as one API call.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIBundleFacet {
    /// The API calls that make up the facet.
    pub apis: Vec<APIContent>,
    /// The type of facet this is.
    pub facet_type: SupportedFacetType,
    /// The labels for the facet.
    pub labels: Vec<Label>,
}

/// Represents the parameters for creating an API bundle facet.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIBundleFacetParams {
    /// The common parameters for the facet being created.
    pub common: CommonFacetCreateParams,
    /// The type of facet that is being created.
    pub facet_type: SupportedFacetType,
}

impl Labeled for SourceBundleFacet {
    fn labels(&self) -> Vec<Label> {
        self.labels.clone()
    }
}

impl Labeled for APIBundleFacet {
    fn labels(&self) -> Vec<Label> {
        self.labels.clone()
    }
}

/// Represents the supported facet types. This is an enum of the
/// various supported facets like README, SECURITY.md, as well as
/// API calls like enabling branch protection on GitHub.
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, VariantNames, EnumString, Hash, Default,
)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SupportedFacetType {
    /// A facet type for a README file.
    Readme,

    /// A facet type for a SECURITY-INSIGHTS.yml file.
    SecurityInsights,

    /// A facet type that supports building the project via SLSA.
    SLSABuild,

    /// A facet type that supports generation of SBOMs in the project.
    SBOMGenerator,

    /// A facet type for the project's license.
    License,

    /// A facet type that shows static code analysis (SCA) is run on the project.
    StaticCodeAnalysis,

    /// A facet type for the project's gitignore file.
    Gitignore,

    /// A facet type showing that branch protection has been enabled on the project.
    BranchProtection,

    /// A facet type showing that code review is enabled on the project.
    CodeReview,

    /// A facet type showing that a tool for updating dependencies has been enabled on the project.
    DependencyUpdateTool,

    /// A facet type showing that a tool for fuzzing has been enabled on the project.
    Fuzzing,

    /// A facet type showing that the project publishes its packages.
    PublishPackages,

    /// A facet type showing that the project pins its dependencies.
    PinnedDependencies,

    /// A facet type showing that the project runs a Static Application Security Testing (SAST) tool.
    SAST,

    /// A facet type for the project's security policy.
    SecurityPolicy,

    /// A facet type showing that the project runs a vulnerability scanner.
    VulnerabilityScanner,

    /// A facet type showing that the project has configuration for forwarding security metadata to GUAC.
    GUACForwardingConfig,

    /// A facet type showing that the project has `OpenSSF Allstar` running against it.
    Allstar,

    /// A facet type showing that the project is running `OpenSSF Scorecard`.
    Scorecard,

    /// A facet type showing for the project's default source code. This should be something simple to just show that the project can build with
    /// trivial source code and all the other facets enabled.
    DefaultSourceCode,

    /// A facet type showing that the project has a mechanism for reporting vulnerabilities.
    VulnerabilityReporting,

    /// A catch all facet type for other facets that don't fit into the above categories.
    #[default]
    Other,
}

impl fmt::Display for SupportedFacetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
