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

//! This is the crate where the statestore where the management of `Skootrs` project state is defined.
//! The statestore currently supports an in memory `SurrealDB` instance that writes to a file.

use surrealdb::{engine::local::{Db, RocksDb}, Surreal};

use skootrs_model::skootrs::{InitializedProject, SkootError};

/// The `SurrealDB` state store for Skootrs projects.
#[derive(Debug)]
pub struct SurrealProjectStateStore {
    pub db: Surreal<Db>
}

/// The functionality for the `SurrealDB` state store for Skootrs projects.
impl SurrealProjectStateStore {
    /// Create a new `SurrealDB` state store for Skootrs projects if `state.db` does not exist, otherwise open it.
    ///
    /// # Errors
    ///
    /// Returns an error if the state store can't be created or opened.
    pub async fn new() -> Result<Self, SkootError> {
        let db = Surreal::new::<RocksDb>("state.db").await?;
        db.use_ns("kusaridev").use_db("skootrs").await?;
        Ok(Self {
            db
        })
    }

    /// Store a new project in the state store. 
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be stored in the state store.
    pub async fn create(&self, project: InitializedProject) -> Result<Option<InitializedProject>, SkootError> {
        let created = self.db
            .create(("project", project.repo.full_url()))
            .content(project)
            .await?;
        Ok(created)
    }

    /// Fetch a project from the state store.
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be fetched from the state store.
    pub async fn select(&self, repo_url: String) -> Result<Option<InitializedProject>, SkootError> {
        let record = self.db
            .select(("project", repo_url))
            .await?;
        Ok(record)
    }

    /// Fetch all projects from the state store.
    ///
    /// # Errors
    ///
    /// Returns an error if all the projects can't be dumped from the state store.
    pub async fn select_all(&self) -> Result<Vec<InitializedProject>, SkootError> {
        let records = self.db
            .select("project")
            .await?;
        Ok(records)
    }
}
