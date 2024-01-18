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

use serde::{Serialize, Deserialize};
use tracing::info;
use utoipa::ToSchema;

use super::Ecosystem;

/// Represents the Maven ecosystem.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct Maven {
    /// The group ID of the Maven project.
    pub group_id: String,
    /// The artifact ID of the Maven project.
    pub artifact_id: String,
}

impl Ecosystem for Maven {
    /// Returns `Ok(())` if the Maven project initialization is successful, 
    /// otherwise returns an error.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the Maven project should be initialized.
    fn initialize(&self, path: String) -> Result<(), Box<dyn Error>> {
        let output = Command::new("mvn")
            .arg("archetype:generate")
            .arg(format!("-DgroupId={}", self.group_id))
            .arg(format!("-DartifactId={}", self.artifact_id))
            .arg("-DarchetypeArtifactId=maven-archetype-quickstart")
            .arg("-DinteractiveMode=false")
            .current_dir(format!("{}/{}", path, self.artifact_id))
            .output()?;
        if !output.status.success() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to run mvn generate",
            )))
        } else {
            info!("Initialized maven project for {}", self.artifact_id);
            Ok(())
        }
    }
}