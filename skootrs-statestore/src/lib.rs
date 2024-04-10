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

use std::collections::HashSet;

use skootrs_lib::service::{
    repo::{LocalRepoService, RepoService},
    source::{LocalSourceService, SourceService},
};

use skootrs_model::skootrs::{InitializedProject, InitializedRepo, InitializedSource, SkootError};

pub trait ProjectStateStore {
    fn create(
        &self,
        project: InitializedProject,
    ) -> impl std::future::Future<Output = Result<(), SkootError>> + Send;
    fn read(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<InitializedProject>, SkootError>> + Send;
    fn update(
        &self,
        project: InitializedProject,
    ) -> impl std::future::Future<Output = Result<(), SkootError>> + Send;
}

pub struct GitProjectStateStore<S: SourceService> {
    // TODO: This should be a git repo type of some sort
    pub source: InitializedSource,
    pub source_service: S,
}

impl ProjectStateStore for GitProjectStateStore<LocalSourceService> {
    async fn create(&self, project: InitializedProject) -> Result<(), SkootError> {
        self.source_service.write_file(
            self.source.clone(),
            "./",
            ".skootrs".to_string(),
            serde_json::to_string(&project)?,
        )?;
        self.source_service.commit_and_push_changes(
            self.source.clone(),
            "Updated skootrs project state".to_string(),
        )?;
        Ok(())
    }

    async fn read(&self) -> Result<Option<InitializedProject>, SkootError> {
        let project = self
            .source_service
            .read_file(&self.source, "./", ".skootrs".to_string())?;
        Ok(Some(serde_json::from_str(&project).unwrap()))
    }

    fn update(
        &self,
        project: InitializedProject,
    ) -> impl std::future::Future<Output = Result<(), SkootError>> + Send {
        self.create(project)
    }
}

pub trait ProjectReferenceCache {
    fn list(&self)
        -> impl std::future::Future<Output = Result<HashSet<String>, SkootError>> + Send;
    fn get(
        &mut self,
        repo_url: String,
    ) -> impl std::future::Future<Output = Result<InitializedProject, SkootError>> + Send;
    fn set(
        &mut self,
        repo_url: String,
    ) -> impl std::future::Future<Output = Result<(), SkootError>> + Send;
}

pub struct InMemoryProjectReferenceCache {
    pub save_path: String,
    pub cache: HashSet<String>,
    pub local_source_service: LocalSourceService,
    pub local_repo_service: LocalRepoService,
    pub clone_path: String,
}

impl ProjectReferenceCache for InMemoryProjectReferenceCache {
    async fn list(&self) -> Result<HashSet<String>, SkootError> {
        Ok(self.cache.clone())
    }

    async fn get(&mut self, repo_url: String) -> Result<InitializedProject, SkootError> {
        let repo = InitializedRepo::try_from(repo_url)?;
        /*let local_clone = self
            .local_repo_service
            .clone_local(repo, self.clone_path.clone())?;
        let project =
            self.local_source_service
                .read_file(&local_clone, "./", ".skootrs".to_string())?;*/
        let project = self
            .local_repo_service
            .fetch_file_content(&repo, ".skootrs")
            .await?;
        let initialized_project: InitializedProject = serde_json::from_str(&project)?;
        Ok(initialized_project)
    }

    async fn set(&mut self, repo_url: String) -> Result<(), SkootError> {
        self.cache.insert(repo_url);
        self.save()?;
        Ok(())
    }
}

impl InMemoryProjectReferenceCache {
    /// Create a new `InMemoryProjectReferenceCache` instance. The `save_path` is the path to the file where the cache will be saved.
    #[must_use]
    pub fn new(save_path: String) -> Self {
        Self {
            save_path,
            cache: HashSet::new(),
            local_source_service: LocalSourceService {},
            local_repo_service: LocalRepoService {},
            clone_path: "/tmp".to_string(),
        }
    }

    /// Load the cache from the file at `save_path` or create a new cache if the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache can't be loaded or created.
    pub fn load_or_create(path: &str) -> Result<Self, SkootError> {
        let mut cache = Self::new(path.to_string());
        Ok(if cache.load().is_ok() {
            cache
        } else {
            cache.save()?;
            cache
        })
    }

    /// Load the cache from the file at `save_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache can't be loaded.
    pub fn load(&mut self) -> Result<(), SkootError> {
        let deserialized_cache: HashSet<String> =
            serde_json::from_str(&std::fs::read_to_string(&self.save_path)?)?;
        self.cache = deserialized_cache;
        Ok(())
    }

    /// Save the cache to the file at `save_path` in the struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache can't be saved.
    pub fn save(&self) -> Result<(), SkootError> {
        let serialized_cache = serde_json::to_string(&self.cache)?;
        std::fs::write(&self.save_path, serialized_cache)?;
        Ok(())
    }
}
