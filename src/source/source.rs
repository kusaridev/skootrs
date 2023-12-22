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

use std::{process::Command, error::Error, path::Path, fs};

use tracing::{info, debug};

/// Struct representing a working copy of source code.
pub struct Source {
    pub path: String
}

impl Source {
    /// Returns `Ok(())` if changes are committed and pushed back to the remote  if successful,
    /// otherwise returns an error.
    pub fn commit_and_push_changes(&self, message: String) -> Result<(), Box<dyn Error>> {
        let _output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&self.path)
        .output()?;

    let _output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .current_dir(&self.path)
        .output()?;
    info!("Committed changes for {}", self.path);

    let _output = Command::new("git")
        .arg("push")
        .current_dir(&self.path)
        .output()?;
    info!("Pushed changes for {}", self.path);
    Ok(())
    }

    /// Returns `Ok(())` if a file is successfully written to some path within the source directory. Otherwise,
    /// it returns an error.
    pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, name: String, contents: C) -> Result<(), Box<dyn Error>> {
        let full_path = Path::new(&self.path).join(&path);
        // Ensure path exists
        fs::create_dir_all(&full_path)?;
        let complete_path = full_path.join(name);
        fs::write(complete_path, contents)?;
        debug!("{:?} file written", &full_path);
        Ok(())
    }
}