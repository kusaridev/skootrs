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

use std::{process::Command, error::Error};

use tracing::info;

use super::ecosystem::Ecosystem;

/// Represents the Go ecosystem.
#[derive(Clone)]
pub struct Go {
    /// The name of the Go module.
    pub name: String,
    /// The host of the Go module.
    pub host: String,
}

impl Go {
    /// Returns the module name in the format "{host}/{name}".
    pub fn module(&self) -> String {
        format!("{}/{}", self.host, self.name)
    }
}

impl Ecosystem for Go {
    /// Returns an error if the initialization of a Go module at the specified
    /// path fails.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the Go module should be initialized.
    fn initialize(&self, path: String) -> Result<(), Box<dyn Error>> {
        let output = Command::new("go")
            .arg("mod")
            .arg("init")
            .arg(self.module())
            .current_dir(format!("{}/{}", path, self.name))
            .output()?;
        if !output.status.success() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run go mod init: {}",  String::from_utf8(output.stderr)?),
            )))
        } else {
            info!("Initialized go module for {}", self.name);
            Ok(())
        }
    }
}