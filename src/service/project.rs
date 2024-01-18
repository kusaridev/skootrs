
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

use crate::model::skootrs::{ProjectParams, InitializedProject, InitializedSource};

use super::{repo::LocalRepoService, ecosystem::LocalEcosystemService, source::{LocalSourceService, SourceService}};

trait ProjectService {
    fn initialize(&self, params: ProjectParams) -> Result<InitializedProject, Box<dyn Error>>;
}

struct LocalProjectService {
    repo_service: LocalRepoService,
    ecosystem_service: LocalEcosystemService,
    source_service: LocalSourceService,
}

impl ProjectService for LocalProjectService {
    fn initialize(&self, params: ProjectParams) -> Result<InitializedProject, Box<dyn Error>> {
        let initialized_repo = self.repo_service.initialize(params.repo_params)?;
        let initialized_source: InitializedSource = self.source_service.initialize(params, initialized_repo)?;
        let initialized_ecosystem = self.ecosystem_service.initialize(params.ecosystem_params)?;
        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: initialized_ecosystem,
            source: initialized_source,
        })
    }
}