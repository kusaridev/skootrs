#![allow(clippy::module_name_repetitions)]

use std::{error::Error, process::Command};

use tracing::info;

use crate::model::skootrs::{
    EcosystemParams, GoParams, InitializedEcosystem, InitializedGo, InitializedMaven,
    InitializedSource, MavenParams,
};

pub trait EcosystemService {
    fn initialize(
        &self,
        params: EcosystemParams,
        source: InitializedSource,
    ) -> Result<InitializedEcosystem, Box<dyn Error>>;
}

#[derive(Debug)]
pub struct LocalEcosystemService {}

impl EcosystemService for LocalEcosystemService {
    fn initialize(
        &self,
        params: EcosystemParams,
        source: InitializedSource,
    ) -> Result<InitializedEcosystem, Box<dyn Error>> {
        match params {
            EcosystemParams::Maven(m) => {
                LocalMavenEcosystemHandler::initialize(&source.path, &m)?;
                Ok(InitializedEcosystem::Maven(InitializedMaven {
                    group_id: m.group_id,
                    artifact_id: m.artifact_id,
                }))
            }
            EcosystemParams::Go(g) => {
                LocalGoEcosystemHandler::initialize(&source.path, &g)?;
                Ok(InitializedEcosystem::Go(InitializedGo {
                    name: g.name,
                    host: g.host,
                }))
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
    fn initialize(path: &str, params: &MavenParams) -> Result<(), Box<dyn Error>> {
        let output = Command::new("mvn")
            .arg("archetype:generate")
            .arg(format!("-DgroupId={}", params.group_id))
            .arg(format!("-DartifactId={}", params.artifact_id))
            .arg("-DarchetypeArtifactId=maven-archetype-quickstart")
            .arg("-DinteractiveMode=false")
            .current_dir(path)
            .output()?;
        if output.status.success() {
            info!("Initialized maven project for {}", params.artifact_id);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to run mvn generate",
            )))

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
    fn initialize(path: &str, params: &GoParams) -> Result<(), Box<dyn Error>> {
        let output = Command::new("go")
            .arg("mod")
            .arg("init")
            .arg(params.module())
            .current_dir(path)
            .output()?;
        if output.status.success() {
            info!("Initialized go module for {}", params.name);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to run go mod init: {}",
                    String::from_utf8(output.stderr)?
                ),
            )))
        }
    }
}
