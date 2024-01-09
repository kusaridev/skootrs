
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

use std::error::Error;

use crate::model::skootrs::{SourceParams, InitializedSource, InitializedRepo};

use super::repo::{LocalRepoService, RepoService};

pub trait SourceService {
    fn initialize(&self, params: SourceParams, initialized_repo: InitializedRepo) -> Result<InitializedSource, Box<dyn Error>>;
}

pub struct LocalSourceService {}

impl SourceService for LocalSourceService {
    fn initialize(&self, params: SourceParams, initialized_repo: InitializedRepo) -> Result<InitializedSource, Box<dyn Error>> {
        let repo_service = LocalRepoService {};
        Ok(repo_service.clone_local(initialized_repo, params.path))?
    }
}