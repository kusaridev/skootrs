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

use std::{error::Error, process::Command, path::Path, fs};

use tracing::{info, debug};

use crate::model::skootrs::{InitializedRepo, InitializedSource, SourceParams};

use super::repo::{LocalRepoService, RepoService};

pub trait SourceService {
    fn initialize(
        &self,
        params: SourceParams,
        initialized_repo: InitializedRepo,
    ) -> Result<InitializedSource, Box<dyn Error>>;
    fn commit_and_push_changes(&self, source: InitializedSource, message: String) -> Result<(), Box<dyn Error>>;
    fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        source: InitializedSource,
        path: P,
        name: String,
        contents: C,
    ) -> Result<(), Box<dyn Error>>;
    fn read_file<P: AsRef<Path>>(&self, source: &InitializedSource, path: P, name: String) -> Result<String, Box<dyn Error>>;
}

#[derive(Debug)]
pub struct LocalSourceService {}

impl SourceService for LocalSourceService {
    /// Returns `Ok(())` if changes are committed and pushed back to the remote  if successful,
    /// otherwise returns an error.
    fn initialize(
        &self,
        params: SourceParams,
        initialized_repo: InitializedRepo,
    ) -> Result<InitializedSource, Box<dyn Error>> {
        let repo_service = LocalRepoService {};
        Ok(repo_service.clone_local(initialized_repo, params.parent_path)?)
    }

    fn commit_and_push_changes(&self, source: InitializedSource, message: String) -> Result<(), Box<dyn Error>> {
        let _output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&source.path)
            .output()?;

        let _output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&source.path)
            .output()?;
        info!("Committed changes for {}", source.path);

        let _output = Command::new("git")
            .arg("push")
            .current_dir(&source.path)
            .output()?;
        info!("Pushed changes for {}", source.path);
        Ok(())
    }

    /// Returns `Ok(())` if a file is successfully written to some path within the source directory. Otherwise,
    /// it returns an error.
    fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        source: InitializedSource,
        path: P,
        name: String,
        contents: C,
    ) -> Result<(), Box<dyn Error>> {
        let full_path = Path::new(&source.path).join(&path);
        // Ensure path exists
        info!("Creating path {:?}", &full_path);
        fs::create_dir_all(&full_path)?;
        let complete_path = full_path.join(name);
        fs::write(complete_path, contents)?;
        debug!("{:?} file written", &full_path);
        Ok(())
    }

    fn read_file<P: AsRef<Path>>(&self, source: &InitializedSource, path: P, name: String) -> Result<String, Box<dyn Error>> {
        let full_path = Path::new(&source.path).join(&path).join(&name);
        let contents = fs::read_to_string(full_path)?;
        Ok(contents)
    }
}
