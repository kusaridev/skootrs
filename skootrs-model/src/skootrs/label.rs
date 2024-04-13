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

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};
use utoipa::ToSchema;

/// A label for various elements of a project like facets and outputs.
/// This is used to provide mechanism for mapping stuff like controls to elements
/// of the project. This makes it easier to audit the project against some set of Security
/// requirements.
#[derive(Serialize, Deserialize, Clone, Debug, EnumString, VariantNames, Display)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Label {
    /// S2C2F Requirement SCA-1
    /// Scan OSS for known vulnerabilities
    S2C2FSCA1,
    /// S2C2F Requirement UPD-2
    /// Enable automated OSS updates
    S2C2FUPD2,
    /// S2C2F Requirement AUD-1
    /// Verify the provenance of your OSS
    S2C2FAUD1,
    /// S2C2F Requirement AUD-3
    /// Validate SBOMs of OSS that you consume into your build
    S2C2FAUD4,

    /// SLSA Build Level 1
    SLSABuildLevel1,
    /// SLSA Build Level 2
    SLSABuildLevel2,
    /// SLSA Build Level 3
    SLSABuildLevel3,

    /// Custom label allow extensibility for end users to define their own labels
    Custom(String),
}

/// A trait for getting the label from a project element.
pub trait Labeled {
    /// Get the labels for the project element.
    fn labels(&self) -> Vec<Label>;
}
