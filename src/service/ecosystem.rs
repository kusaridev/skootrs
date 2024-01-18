use std::{error::Error, process::Command};

use tracing::info;

use crate::model::skootrs::{EcosystemParams, InitializedEcosystem, MavenParams};

pub trait EcosystemService {
    fn initialize(&self, params: EcosystemParams) -> Result<InitializedEcosystem, Box<dyn Error>>;
}

pub struct LocalEcosystemService {
    path: String,
}

impl EcosystemService for LocalEcosystemService {
    fn initialize(&self, params: EcosystemParams) -> Result<InitializedEcosystem, Box<dyn Error>> {
        match params {
            EcosystemParams::Maven(m) => {
                let handler = LocalMavenEcosystemHandler {};
                handler.initialize(self.path, m)?;
                Ok(InitializedEcosystem::Maven)
            }
            EcosystemParams::Go(g) => {
                let handler = LocalGoEcosystemHandler {};
                handler.initialize(self.path)?;
                Ok(InitializedEcosystem::Go)
            }
        }
    }
}

struct LocalMavenEcosystemHandler {}

impl LocalMavenEcosystemHandler {
    /// Returns `Ok(())` if the Maven project initialization is successful,
    /// otherwise returns an error.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the Maven project should be initialized.
    fn initialize(&self, path: String, params: MavenParams) -> Result<(), Box<dyn Error>> {
        let output = Command::new("mvn")
            .arg("archetype:generate")
            .arg(format!("-DgroupId={}", params.group_id))
            .arg(format!("-DartifactId={}", params.artifact_id))
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

struct LocalGoEcosystemHandler {}

impl LocalGoEcosystemHandler {
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
                format!(
                    "Failed to run go mod init: {}",
                    String::from_utf8(output.stderr)?
                ),
            )))
        } else {
            info!("Initialized go module for {}", self.name);
            Ok(())
        }
    }
}
