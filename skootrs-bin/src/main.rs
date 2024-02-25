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

//! Tool for creating and managing secure-by-default projects.
//!
//! This crate is for the binary that acts as the CLI which interacts 
//! with the other crates in the Skootrs project.
//!
//! The CLI is built using the `clap` crate, and the commands are
//! using noun-verb syntax. So the commands are structured like:
//! `skootrs <noun> <verb>`. For example, `skootrs project create`.
//!
//! The CLI if not given any arguments to a command will default to
//! giving an interactive prompt to the user to fill in the required
//! information.

pub mod helpers;

use clap::{Parser, Subcommand};
use skootrs_lib::service::ecosystem::LocalEcosystemService;
use skootrs_lib::service::facet::LocalFacetService;
use skootrs_lib::service::project::LocalProjectService;
use skootrs_lib::service::repo::LocalRepoService;
use skootrs_lib::service::source::LocalSourceService;
use skootrs_model::skootrs::SkootError;
use clio::Input;

use helpers::{dump, get_facet, get_output};
use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::error;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use serde::de::DeserializeOwned;

/// Skootrs is a CLI tool for creating and managing secure-by-default projects.
/// The commands are  using noun-verb syntax. So the commands are structured like:
/// `skootrs <noun> <verb>`. For example, `skootrs project create`.
///
/// The CLI if not given any arguments to a command will default to
/// giving an interactive prompt to the user to fill in the required
/// information.
#[derive(Parser)]
#[command(name = "skootrs")]
#[command(bin_name = "skootrs")]
enum SkootrsCli {
    /// Project commands.
    #[command(name = "project")]
    Project {
        #[clap(subcommand)]
        project: ProjectCommands,
    },

    /// Facet commands.
    #[command(name = "facet")]
    Facet {
        #[clap(subcommand)]
        facet: FacetCommands,
    },

    /// Output commands.
    #[command(name = "output")]
    Output {
        #[clap(subcommand)]
        output: OutputCommands,
    },

    /// Daemon commands.
    #[command(name = "daemon")]
    Daemon {
        #[clap(subcommand)]
        daemon: DaemonCommands,
    },
}

/// This is the enum for what nouns the `project` command can take.
#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// Create a new project.
    #[command(name = "create")]
    Create {
        /// This is an optional input parameter that can be used to pass in a file, pipe, url, or stdin.
        /// This is expected to be YAML or JSON. If it is not provided, the CLI will prompt the user for the input.
        #[clap(value_parser)]
        input: Option<Input>,
    },
    /// Get the metadata for a particular project.
    #[command(name = "get")]
    Get,
    /// List all the projects known to the local Skootrs
    #[command(name = "list")]
    List,
}

/// This is the enum for what nouns the `facet` command can take.
#[derive(Subcommand, Debug)]
enum FacetCommands {
    /// Get the data for a facet of a particular project.
    #[command(name = "get")]
    Get,
    /// List all the facets that belong to a particular project.
    #[command(name = "list")]
    List
}

/// This is the enum for what nouns the `output` command can take.
#[derive(Subcommand, Debug)]
enum OutputCommands {
    /// Get the data for a release output of a particular project.
    #[command(name = "get")]
    Get,
    /// List all the release outputs that belong to a particular project.
    #[command(name = "list")]
    List
}

/// This is the enum for what nouns the `daemon` command can take.
#[derive(Subcommand, Debug)]
enum DaemonCommands {
    /// Start the REST server.
    #[command(name = "start")]
    Start,
}

/*enum SkootrsCli {
    /// Create a new project.
    #[command(name = "create")]
    Create,
    /// Start the REST server.
    #[command(name = "daemon")]
    Daemon,
    /// Dump the current state of the projects database.
    #[command(name = "dump")]
    Dump,
    /// Get the data for a facet of a particular project.
    #[command(name = "get-facet")]
    GetFacet,

    #[command(name = "get")]
    Get {
        #[clap(subcommand)]
        resource: GetCommands,
    },
}

/// This is the enum for what nouns the `get` command can take.
#[derive(Subcommand, Debug)]
enum GetCommands {
    Output
}*/

fn init_tracing() {
    let app_name = "skootrs";

    // Start a new Jaeger trace pipeline.
    // Spans are exported in batch - recommended setup for a production application.
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(app_name)
        .install_simple()
        .expect("Failed to install OpenTelemetry tracer.");

    // Filter based on level - trace, debug, info, warn, error
    // Tunable via `RUST_LOG` env variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    // Create a `tracing` layer using the Jaeger tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // Create a `tracing` layer to emit spans as structured logs to stdout
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);
    // Combined them all together in a `tracing` subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");
}

/// TODO: This probably should be configurable in some way.
fn init_project_service() -> LocalProjectService<
    LocalRepoService,
    LocalEcosystemService,
    LocalSourceService,
    LocalFacetService,
> {
    let project_service = LocalProjectService {
        repo_service: LocalRepoService {},
        ecosystem_service: LocalEcosystemService {},
        source_service: LocalSourceService {},
        facet_service: LocalFacetService {},
    };
    project_service
}

fn parse_optional_input<T: DeserializeOwned>(input: Option<Input>) -> Result<Option<T>, SkootError> {
    match input {
        Some(input) => {
            // This should also support JSON since most modern YAML is a superset of JSON.
            // I don't care enough to support the edge cases right now.
            let params: T = serde_yaml::from_reader(input)?;
            Ok(Some(params))
        }
        None => Ok(None)
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), SkootError> {
    init_tracing();
    let cli = SkootrsCli::parse();
    let o: octocrab::Octocrab = octocrab::Octocrab::builder()
        .personal_token(
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var must be populated"),
        )
        .build()?;
    octocrab::initialise(o);

    let project_service = init_project_service();
    // TODO: This should only default when it can't pull a valid config from the environment.
    let config = skootrs_model::skootrs::SkootrsConfig::default();

    match cli {
        SkootrsCli::Project { project } => {
            match project {
                ProjectCommands::Create { input } => {
                    let project_params = parse_optional_input(input)?;
                    if let Err(ref error) = helpers::Project::create(&config, project_service, project_params).await {
                        error!(error = error.as_ref(), "Failed to create project");
                    }
                }
                ProjectCommands::Get => {
                    if let Err(ref error) = dump().await {
                        error!(error = error.as_ref(), "Failed to get project info");
                    }
                }
                ProjectCommands::List => {
                    if let Err(ref error) = dump().await {
                        error!(error = error.as_ref(), "Failed to list projects");
                    }
                }
            }
        }
        SkootrsCli::Facet { facet } => {
            match facet {
                FacetCommands::Get => {
                    if let Err(ref error) = get_facet().await {
                        error!(error = error.as_ref(), "Failed to get facet");
                    }
                }
                FacetCommands::List => {
                    if let Err(ref error) = get_facet().await {
                        error!(error = error.as_ref(), "Failed to list facets for project");
                    }
                }
            }
        }
        SkootrsCli::Output { output } => {
            match output {
                OutputCommands::Get => {
                    if let Err(ref error) = get_output().await {
                        error!(error = error.as_ref(), "Failed to get output");
                    }
                }
                OutputCommands::List => {
                    if let Err(ref error) = get_output().await {
                        error!(error = error.as_ref(), "Failed to list outputs for project");
                    }
                }
            }
        }
        SkootrsCli::Daemon { daemon } => {
            match daemon {
                DaemonCommands::Start => {
                    tokio::task::spawn_blocking(|| {
                        skootrs_rest::server::rest::run_server().expect("Failed to start REST Server");
                    })
                    .await
                    .expect("REST Server Task Panicked");
                }
            }
        }
    }

    Ok(())
}
