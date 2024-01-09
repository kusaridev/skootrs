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

use super::{InitializedSource, InitializedRepo, InitializedEcosystem};

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum Facet {
    SourceFile(SourceFileFacet),
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum FacetParams {
    SourceFile(SourceFileFacetParams),
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct CommonFacetParams {
    pub project_name: String,
    pub source: InitializedSource,
    pub repo: InitializedRepo,
    pub ecosystem: InitializedEcosystem,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct SourceFileFacet {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct SourceFileFacetParams {
    pub name: String,
    pub path: String,

    pub common: CommonFacetParams,
    pub facet_type: SupportedFacetType,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum SupportedFacetType {
    Readme,
    SecurityInsights,
    SLSABuild,
    SBOMGenerator,
}
