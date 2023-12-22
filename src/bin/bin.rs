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

use std::error::Error;

use clap::Parser;
use skootrs::{create, new_create};



#[derive(Parser)]
#[command(name = "skootrs")]
#[command(bin_name = "skootrs")]
enum SkootrsCli{
    #[command(name = "create")]
    Create,
    #[command(name = "create2")]
    Create2,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let cli = SkootrsCli::parse();
    let o: octocrab::Octocrab = octocrab::Octocrab::builder()
    .personal_token(std::env::var("GITHUB_TOKEN")?)
    .build()?
    ;
    octocrab::initialise(o);
    match cli {
        SkootrsCli::Create => {
            create().await?;
        }
        SkootrsCli::Create2 => {
            new_create().await?;
        }
    };
    Ok(())
}