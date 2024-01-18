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

pub mod go;
pub mod maven;

use std::error::Error;

/// Trait representing a packaging/language ecosystem.
/// e.g. Go, Maven
pub trait Ecosystem: Clone {
    /// Returns `Ok(())` if the initialization of a project for a
    /// package/language ecosystem is successful, otherwise returns an error.
    /// 
    ///
    /// # Arguments
    ///
    /// * `path` - A string representing the path to initialize the ecosystem.
    fn initialize(&self, path: String) -> Result<(), Box<dyn Error>>;
}