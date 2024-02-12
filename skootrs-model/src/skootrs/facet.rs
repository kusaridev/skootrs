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

use std::fmt;

use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::{InitializedSource, InitializedRepo, InitializedEcosystem};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum InitializedFacet {
    SourceFile(SourceFileFacet),
    SourceBundle(SourceBundleFacet),
    APIBundle(APIBundleFacet),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum FacetParams {
    SourceFile(SourceFileFacetParams),
    SourceBundle(SourceBundleFacetParams),
    APIBundle(APIBundleFacetParams),
}

/// This is required to create an ordering of what facets get applied
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FacetSetParams {
    pub facets_params: Vec<FacetParams>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CommonFacetParams {
    pub project_name: String,
    pub source: InitializedSource,
    pub repo: InitializedRepo,
    pub ecosystem: InitializedEcosystem,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileFacet {
    pub name: String,
    pub path: String,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileFacetParams {
    //pub name: String,
    //pub path: String,

    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceFileContent {
    pub name: String,
    pub path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacet {
    pub source_files: Vec<SourceFileContent>,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SourceBundleFacetParams {
    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIContent {
    pub name: String,
    pub url: String,
    pub response: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIBundleFacet {
    pub apis: Vec<APIContent>,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct APIBundleFacetParams {
    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

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