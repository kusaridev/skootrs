#![allow(clippy::module_name_repetitions)]

use std::process::Command;

use tracing::info;

use skootrs_model::skootrs::{
    EcosystemParams, GoParams, InitializedEcosystem, InitializedGo, InitializedMaven,
    InitializedSource, MavenParams, SkootError,
};

/// The `EcosystemService` trait provides an interface for initializing and managing a project's ecosystem.
/// An ecosystem is the language or packaging ecosystem that a project is built in, such as Maven or Go.
pub trait EcosystemService {
    fn initialize(
        &self,
        params: EcosystemParams,
        source: InitializedSource,
    ) -> Result<InitializedEcosystem, SkootError>;
}

/// The `LocalEcosystemService` struct provides an implementation of the `EcosystemService` trait for initializing 
/// and managing a project's ecosystem on the local machine.
#[derive(Debug)]
pub struct LocalEcosystemService {}

impl EcosystemService for LocalEcosystemService {
    fn initialize(
        &self,
        params: EcosystemParams,
        source: InitializedSource,
    ) -> Result<InitializedEcosystem, SkootError> {
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


/// The `LocalMavenEcosystemHandler` struct represents a handler for initializing and managing a Maven 
/// project on the local machine.
struct LocalMavenEcosystemHandler {}

impl LocalMavenEcosystemHandler {
    /// Returns `Ok(())` if the Maven project initialization is successful,
    /// otherwise returns an error.
    fn initialize(path: &str, params: &MavenParams) -> Result<(), SkootError> {
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

/// The `LocalGoEcosystemHandler` struct represents a handler for initializing and managing a Go
/// project on the local machine.
struct LocalGoEcosystemHandler {}

impl LocalGoEcosystemHandler {
    /// Returns an error if the initialization of a Go module at the specified
    /// path fails.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the Go module should be initialized.
    fn initialize(path: &str, params: &GoParams) -> Result<(), SkootError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_local_maven_ecosystem_handler_initialize_success() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let params = MavenParams {
            group_id: "com.example".to_string(),
            artifact_id: "my-project".to_string(),
        };

        let result = LocalMavenEcosystemHandler::initialize(path, &params);

        assert!(result.is_ok());
    }

    #[test]
    fn test_local_maven_ecosystem_handler_initialize_failure() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let params = MavenParams {
            // Invalid group ID
            group_id: "".to_string(),
            artifact_id: "my-project".to_string(),
        };

        let result = LocalMavenEcosystemHandler::initialize(path, &params);

        assert!(result.is_err());
    }

    #[test]
    fn test_local_go_ecosystem_handler_initialize_success() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let params = GoParams {
            name: "my-project".to_string(),
            host: "github.com".to_string(),
        };

        let result = LocalGoEcosystemHandler::initialize(path, &params);

        assert!(result.is_ok());
    }

    #[test]
    fn test_local_go_ecosystem_handler_initialize_failure() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let params = GoParams {
            // Invalid project name
            name: "".to_string(),
            host: "github.com".to_string(),
        };

        let result = LocalGoEcosystemHandler::initialize(path, &params);

        assert!(result.is_err());
    }
}
