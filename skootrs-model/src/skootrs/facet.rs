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

use std::fmt;

use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::{InitializedSource, InitializedRepo, InitializedEcosystem};

/// Represents a facet that has been initialized. This is an enum of
/// the various supported facets like API based, and Source file bundle
/// based.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedFacet {
    SourceFile(SourceFileFacet),
    SourceBundle(SourceBundleFacet),
    APIBundle(APIBundleFacet),
}

/// Represents the parameters for creating a facet. This should mirror the
/// `InitializedFacet` enum.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum FacetParams {
    SourceFile(SourceFileFacetParams),
    SourceBundle(SourceBundleFacetParams),
    APIBundle(APIBundleFacetParams),
}

/// This is required to create an ordering of what facets get applied.
/// There could be issues like a security feature being enabled before
/// some other feature, which could lead to it being blocked.
/// for example, enabling branch protection before pushing the initial
/// boilerplate code.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FacetSetParams {
    pub facets_params: Vec<FacetParams>,
}

/// Represents the common parameters that are shared across all facets.
/// This is mostly the context of the project, like the project name,
/// source, repo, and ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CommonFacetParams {
    pub project_name: String,
    pub source: InitializedSource,
    pub repo: InitializedRepo,
    pub ecosystem: InitializedEcosystem,
}

/// (DEPRECATED) Represents a source file facet which is a facet that 
/// is based on a single source file.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileFacet {
    pub name: String,
    pub path: String,
    pub facet_type: SupportedFacetType,
}

/// (DEPRECATED) Represents the parameters for creating a source file facet.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileFacetParams {
    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

/// Represents the content of a source file.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileContent {
    pub name: String,
    pub path: String,
    // TODO: Since the content can change out of band of Skootrs
    // should we even store the content in the database?
    pub content: String,
    // TODO: Add a hash of the content to verify it hasn't changed
}

/// Represents a source bundle facet which is a facet that is based
/// on a bundle of source files. This can be a single file like a
/// README, or a collection of related files like several yaml files
/// for a set of actions in a github workflow.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacet {
    pub source_files: Vec<SourceFileContent>,
    pub facet_type: SupportedFacetType,
}


/// Represents the parameters for creating a source bundle facet.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacetParams {
    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

/// Represents the content of an API call. This just includes the
/// name of the API call, the URL to the API call and the response.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIContent {
    pub name: String,
    pub url: String,
    // TODO: The response can be multiple things like a response code,
    // a JSON response, or something else. Should we store this as a
    // more comples type.
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
    pub apis: Vec<APIContent>,
    pub facet_type: SupportedFacetType,
}

/// Represents the parameters for creating an API bundle facet.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIBundleFacetParams {
    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

/// Represents the supported facet types. This is an enum of the
/// various supported facets like README, SECURITY.md, as well as
/// API calls like enabling branch protection on GitHub.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SupportedFacetType {
    Readme,
    SecurityInsights,
    SLSABuild,
    SBOMGenerator,
    License,
    StaticCodeAnalysis,
    Gitignore,
    BranchProtection,
    CodeReview,
    DependencyUpdateTool,
    Fuzzing,
    PublishPackages,
    PinnedDependencies,
    SAST,
    SecurityPolicy,
    VulnerabilityScanner,
    GUACForwardingConfig,
    Allstar,
    Scorecard,
    DefaultSourceCode,
    VulnerabilityReporting,
}

impl fmt::Display for SupportedFacetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}