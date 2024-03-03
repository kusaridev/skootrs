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

#![allow(clippy::module_name_repetitions)]

use std::{fs, path::Path, process::Command};

use sha2::Digest;
use tracing::{debug, info};

use skootrs_model::skootrs::{
    InitializedRepo, InitializedSource, SkootError, SourceInitializeParams,
};

use super::repo::{LocalRepoService, RepoService};
/// The `SourceService` trait provides an interface for and managing a project's source code.
/// This code is usually something a local git repo. The service differs from the repo service
/// in that it's focused on the files and not the repo itself.
pub trait SourceService {
    /// Initializes a source code directory for a project. This usually involves cloning a repo from
    /// a repo service.
    ///
    /// # Errors
    ///
    /// Returns an error if the source code directory can't be initialized.
    fn initialize(
        &self,
        params: SourceInitializeParams,
        initialized_repo: InitializedRepo,
    ) -> Result<InitializedSource, SkootError>;

    /// Commits changes to the repo and pushed them to the remote.
    ///
    /// # Errors
    ///
    /// Returns an error if the changes can't be committed and pushed to the remote.
    fn commit_and_push_changes(
        &self,
        source: InitializedSource,
        message: String,
    ) -> Result<(), SkootError>;

    /// Writes a file to the source code directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be written to the source code directory.
    fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        source: InitializedSource,
        path: P,
        name: String,
        contents: C,
    ) -> Result<(), SkootError>;

    /// Reads a file from the source code directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be read from the source code directory.
    fn read_file<P: AsRef<Path>>(
        &self,
        source: &InitializedSource,
        path: P,
        name: String,
    ) -> Result<String, SkootError>;

    /// `hash_file` returns the SHA256 hash of a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be opened and hashed.
    fn hash_file<P: AsRef<Path>>(
        &self,
        source: &InitializedSource,
        path: P,
        name: String,
    ) -> Result<String, SkootError>;

    /// Pulls updates from the remote repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the updates can't be pulled from the remote repo.
    fn pull_updates(&self, source: InitializedSource) -> Result<(), SkootError>;
}

/// The `LocalSourceService` struct provides an implementation of the `SourceService` trait for initializing
/// and managing a project's source files from the local machine.
#[derive(Debug)]
pub struct LocalSourceService {}

impl SourceService for LocalSourceService {
    /// Returns `Ok(())` if changes are committed and pushed back to the remote  if successful,
    /// otherwise returns an error.
    fn initialize(
        &self,
        params: SourceInitializeParams,
        initialized_repo: InitializedRepo,
    ) -> Result<InitializedSource, SkootError> {
        let repo_service = LocalRepoService {};
        repo_service.clone_local(initialized_repo, params.parent_path)
    }

    fn commit_and_push_changes(
        &self,
        source: InitializedSource,
        message: String,
    ) -> Result<(), SkootError> {
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
    ) -> Result<(), SkootError> {
        let full_path = Path::new(&source.path).join(&path);
        // Ensure path exists
        info!("Creating path {:?}", &full_path);
        fs::create_dir_all(&full_path)?;
        let complete_path = full_path.join(name);
        fs::write(complete_path, contents)?;
        debug!("{:?} file written", &full_path);
        Ok(())
    }

    fn read_file<P: AsRef<Path>>(
        &self,
        source: &InitializedSource,
        path: P,
        name: String,
    ) -> Result<String, SkootError> {
        let full_path = Path::new(&source.path).join(&path).join(name);
        let contents = fs::read_to_string(full_path)?;
        Ok(contents)
    }

    fn hash_file<P: AsRef<Path>>(
        &self,
        source: &InitializedSource,
        path: P,
        name: String,
    ) -> Result<String, SkootError> {
        let full_path: std::path::PathBuf = Path::new(&source.path).join(&path).join(name);
        debug!("Hashing file {:?}", &full_path);
        let mut contents = fs::File::open(&full_path)?;
        let mut hasher = sha2::Sha256::new();
        std::io::copy(&mut contents, &mut hasher)?;
        let hash = hasher.finalize();
        //let hash = "test".to_string();

        debug!("Hashed file {:?} with hash: {:x}", &full_path, &hash);
        Ok(format!("{hash:x}"))
    }

    fn pull_updates(&self, source: InitializedSource) -> Result<(), SkootError> {
        let _output = Command::new("git")
            .arg("pull")
            .current_dir(&source.path)
            .output()?;
        info!("Pulled updates for {}", source.path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skootrs_model::skootrs::{GithubUser, InitializedGithubRepo};
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn test_initialize() {
        let source_service = LocalSourceService {};
        let temp_dir = TempDir::new("test").unwrap();
        let parent_path = temp_dir.path().to_str().unwrap();
        let params = SourceInitializeParams {
            parent_path: parent_path.to_string(),
        };
        let initialized_repo = InitializedRepo::Github(InitializedGithubRepo {
            name: "skootrs".to_string(),
            organization: GithubUser::Organization("kusaridev".to_string()),
        });
        let result = source_service.initialize(params, initialized_repo);
        assert!(result.is_ok());
        let initialized_source = result.unwrap();
        assert_eq!(
            initialized_source.path,
            format!("{}/{}", parent_path, "skootrs")
        );
    }

    #[test]
    fn test_write_file() {
        let source_service = LocalSourceService {};
        let temp_dir = TempDir::new("test").unwrap();
        let initialized_source = InitializedSource {
            path: temp_dir.path().to_str().unwrap().to_string(),
        };
        let path = "subdirectory";
        let name = "file.txt".to_string();
        let contents = "File contents".as_bytes();
        let result = source_service.write_file(initialized_source, path, name.clone(), contents);
        assert!(result.is_ok());
        let file_path =
            PathBuf::from(format!("{}/{}", temp_dir.path().to_str().unwrap(), path)).join(name);
        assert!(file_path.exists());
        let file_contents = fs::read_to_string(file_path).unwrap();
        assert_eq!(file_contents, "File contents");
    }

    #[test]
    fn test_read_file() {
        let source_service = LocalSourceService {};
        let temp_dir = TempDir::new("test").unwrap();
        let initialized_source = InitializedSource {
            path: temp_dir.path().to_str().unwrap().to_string(),
        };
        let path = "subdirectory";
        let name = "file.txt".to_string();
        let contents = "File contents".as_bytes();
        let result =
            source_service.write_file(initialized_source.clone(), path, name.clone(), contents);
        assert!(result.is_ok());
        let file_path = PathBuf::from(format!("{}/{}", temp_dir.path().to_str().unwrap(), path))
            .join(name.clone());
        assert!(file_path.exists());
        let file_contents = source_service
            .read_file(&initialized_source, path, name)
            .unwrap();
        assert_eq!(file_contents, "File contents");
    }
}
