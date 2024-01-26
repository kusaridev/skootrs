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



use clap::Parser;
use skootrs::{create, dump, get_facet, model::skootrs::SkootError};
use tracing::error;


#[derive(Parser)]
#[command(name = "skootrs")]
#[command(bin_name = "skootrs")]
enum SkootrsCli{
    #[command(name = "create")]
    Create,
    #[command(name = "daemon")]
    Daemon,
    #[command(name = "dump")]
    Dump,
    #[command(name = "get-facet")]
    GetFacet,
}

#[tokio::main]
async fn main() -> std::result::Result<(), SkootError> {
    //tracing_subscriber::fmt::init();
    let subscriber = tracing_subscriber::fmt()
    .with_file(true)
    .with_line_number(true)
    .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    let cli = SkootrsCli::parse();
    let o: octocrab::Octocrab = octocrab::Octocrab::builder()
    .personal_token(
        std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN env var must be populated")
    )
    .build()?
    ;
    octocrab::initialise(o);
    match cli {
        SkootrsCli::Create => {
            if let Err(ref error) = create().await {
                error!(error = error.as_ref(), "Failed to create project");
            }
        },
        SkootrsCli::Daemon => {
            tokio::task::spawn_blocking(|| {
                skootrs::server::rest::run_server().expect("Failed to start REST Server");
            }).await.expect("REST Server Task Panicked");
        },
        SkootrsCli::Dump => {
            if let Err(ref error) = dump().await {
                error!(error = error.as_ref(), "Failed to get info");
            }
        },
        SkootrsCli::GetFacet => {
            if let Err(ref error) = get_facet().await {
                error!(error = error.as_ref(), "Failed to get facet");
            }
        },
    };
    Ok(())
}