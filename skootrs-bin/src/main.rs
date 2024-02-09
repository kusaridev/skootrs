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

pub mod helpers;

use clap::Parser;
use skootrs_model::skootrs::SkootError;

use helpers::{create, dump, get_facet};
use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::error;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// This is the enum for what commands the CLI can take.
#[derive(Parser)]
#[command(name = "skootrs")]
#[command(bin_name = "skootrs")]
enum SkootrsCli {
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
}

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
        .expect("Failed to install `tracing` subscriber.")
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
    match cli {
        SkootrsCli::Create => {
            if let Err(ref error) = create().await {
                error!(error = error.as_ref(), "Failed to create project");
            }
        }
        SkootrsCli::Daemon => {
            tokio::task::spawn_blocking(|| {
                skootrs_rest::server::rest::run_server().expect("Failed to start REST Server");
            })
            .await
            .expect("REST Server Task Panicked");
        }
        SkootrsCli::Dump => {
            if let Err(ref error) = dump().await {
                error!(error = error.as_ref(), "Failed to get info");
            }
        }
        SkootrsCli::GetFacet => {
            if let Err(ref error) = get_facet().await {
                error!(error = error.as_ref(), "Failed to get facet");
            }
        }
    };
    Ok(())
}
