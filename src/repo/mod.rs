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

pub mod github;

use std::error::Error;

use crate::source::Source;

/// UnitializedRepo represents a source repo that hasn't been created in its host yet.
/// e.g. a Github repo that you plan to create.
pub trait UninitializedRepo {
    type Repo: InitializedRepo;

    /// Returns an `InitializedRepo` representing a created and initialized repo if it is
    /// successfully created, otherwise it returns an error.
    fn create(
        &self,
    ) -> impl std::future::Future<Output = Result<Self::Repo, Box<dyn Error>>> + Send
    where
        Self: Sized;

    /// Returns an `InitializedRepo` representing a created and initialized repo based on a 
    /// template that itself is an existing `InitializedRepo`, otherwise it return an error.
    fn create_from_template(
        &self,
        repo: Box<dyn InitializedRepo>,
    ) -> impl std::future::Future<Output = Result<Box<dyn InitializedRepo>, Box<dyn Error>>> + Send
    where
        Self: Sized;

    /// Returns a `String` representing the Host URL of the VCS.
    /// e.g. https://github.com
    fn host_url(&self) -> String;

    /// Returns a `String` representing the eventual full URL of the repo. 
    fn full_url(&self) -> String;
}

/// InitializedRepo represents a repo that has been created and initialized in some version control system.
/// e.g. a Repo living inside of Github
pub trait InitializedRepo: Send {
    /// Returns a `Source` representing a working copy of the source code repo.
    /// e.g. A local clone of a git repo.
    fn clone_repo(&self, path: String) -> Result<Source, Box<dyn Error>>;

    /// Returns a `String` representing the Host URL of the VCS.
    /// e.g. https://github.com
    fn host_url(&self) -> String;

    /// Returns a `String` representing the full URL of the repo. 
    fn full_url(&self) -> String;
}