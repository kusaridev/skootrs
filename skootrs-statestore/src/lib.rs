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

use surrealdb::{engine::local::{Db, RocksDb}, Surreal};

use skootrs_lib::model::skootrs::{InitializedProject, SkootError};

#[derive(Debug)]
pub struct SurrealProjectStateStore {
    pub db: Surreal<Db>
}

impl SurrealProjectStateStore {
    pub async fn new() -> Result<Self, SkootError> {
        let db = Surreal::new::<RocksDb>("state.db").await?;
        db.use_ns("kusaridev").use_db("skootrs").await?;
        Ok(Self {
            db
        })
    }

    pub async fn create(&self, project: InitializedProject) -> Result<Option<InitializedProject>, SkootError> {
        let created = self.db
            .create(("project", project.repo.full_url()))
            .content(project)
            .await?;
        Ok(created)
    }

    pub async fn select(&self, repo_url: String) -> Result<Option<InitializedProject>, SkootError> {
        let record = self.db
            .select(("project", repo_url))
            .await?;
        Ok(record)
    }

    pub async fn select_all(&self) -> Result<Vec<InitializedProject>, SkootError> {
        let records = self.db
            .select("project")
            .await?;
        Ok(records)
    }
}
