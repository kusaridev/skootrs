use std::error::Error;
use actix_web::{Responder, web::{ServiceConfig, Data, Json, self}, HttpResponse};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tracing::info;
use utoipa::ToSchema;

use crate::{config::{DefaultConfigBundle, DefaultReadmeInput, ConfigInput, DefaultSecurityInsightsInput}, model::skootrs::{SourceParams, MavenParams, GoParams, InitializedGithubRepo}};

//use crate::repo::{{InitializedRepo, UninitializedRepo}, ecosystem::{Ecosystem, go::GoParams, maven::MavenParams}, source::Source, config::{ConfigBundle, DefaultConfigBundle, ConfigInput, DefaultReadmeInput, DefaultSecurityInsightsInput}};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct APISupportedInitializedProject {
    repo: APISupportedRepo,
    ecosystem: APISupportedEcosystem,
    source: SourceParams
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct APISupportedCreateProjectParams {
    name: String,
    repo_params: APISupportedRepoParams,
    ecosystem_params: APISupportedEcosystemParams,
}

impl APISupportedCreateProjectParams {
    async fn create(&self) -> Result<APISupportedInitializedProject, Box<dyn Error>> {
        let initialized_repo = match &self.repo_params {
            APISupportedRepoParams::Github(g) => g.create().await?,
        };

        // TODO: Make this parameterized
        let source = initialized_repo.clone_repo("/tmp".into())?;
        match &self.ecosystem_params {
            APISupportedEcosystemParams::Go(g) => g.initialize("/tmp".into()),
            APISupportedEcosystemParams::Maven(m) => m.initialize("/tmp".into()),
        }?;
        //self.create_documentation(&source)?;
        self.configure(&source, initialized_repo.full_url())?;
        source.commit_and_push_changes(format!(
            "Added documentation and security insights for {}",
            self.name
        ))?;

        Ok(APISupportedInitializedProject {
            repo: APISupportedRepo::Github(initialized_repo),
            ecosystem: match &self.ecosystem_params {
                APISupportedEcosystemParams::Go(g) => APISupportedEcosystem::Go(g.clone()),
                APISupportedEcosystemParams::Maven(m) => APISupportedEcosystem::Maven(m.clone()),
            },
            source,
        })
    }

    // TODO: Fix this
    fn configure(&self, source: &SourceParams, url: String) -> Result<(), Box<dyn Error>> {
        let config_bundle = DefaultConfigBundle{};
        let readme_bundle = config_bundle.readme_bundle(
            ConfigInput::DefaultReadmeStruct(DefaultReadmeInput{ name: self.name.clone() }))?;
        match readme_bundle {
            crate::config::Config::SourceFileConfig(sfc) => {
                source.write_file(sfc.path, sfc.name, sfc.content)?;
            },
        }
        info!("Created README.md for {}", self.name);
        let security_insights_bundle = config_bundle.security_insights_bundle(
            ConfigInput::DefaultSecurityInsightsStruct(DefaultSecurityInsightsInput{ url }))?;
        match security_insights_bundle {
            crate::config::Config::SourceFileConfig(sfc) => {
                source.write_file(sfc.path, sfc.name, sfc.content)?;
            },
        }
        info!("Created SECURITY_INSIGHTS.yaml for {}", self.name);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum APISupportedRepo {
    Github(InitializedGithubRepo)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum APISupportedEcosystem {
    Go(GoParams),
    Maven(MavenParams)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum APISupportedRepoParams {
    Github(UninitializedGithubRepo)
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum APISupportedEcosystemParams {
    Go(GoParams),
    Maven(MavenParams)
}

#[derive(Default)]
pub(super) struct ProjectStore {
    projects: Mutex<Vec<APISupportedInitializedProject>>
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    /// When Project is not found by search term.
    NotFound(String),
    /// When a Project was unable to be initialized.
    InitializationError(String),
    /// When todo endpoint was called without correct credentials
    Unauthorized(String),
}

pub(super) fn configure(store: Data<ProjectStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(web::resource("/project").route(web::post().to(create_project)))
            .service(web::resource("/project").route(web::get().to(get_projects)));
    }
}

/// Create a new project
/// 
/// Example: 
/// {
/// "ecosystem_params": {
///    "Go": {
///      "host": "github.com/mlieberman85",
///      "name": "test-new-api-2"
///    }
///  },
///  "name": "test-new-api-2",
///  "repo_params": {
///    "Github": { "name": "test-new-api-2", "description": "asdf", "organization": { "User": "mlieberman85" } }
///  }
/// }"
///
#[utoipa::path(
    post,
    path = "/project",
    request_body = APISupportedCreateProjectParams,
    responses( 
        (status = 201, description = "Project created successfully", body = APISupportedInitializedProject),
        (status = 409, description = "Project unable to be created", body = ErrorResponse, example = json!(ErrorResponse::InitializationError("Unable to create repo".into())))
    )
)]
pub(super) async fn create_project(project: Json<APISupportedCreateProjectParams>, project_store: Data<ProjectStore>) -> impl Responder {
    // TODO: Clean this up
    let mut projects = project_store.projects.lock().await;

    let initialized_project = project.create().await.unwrap();
    projects.push(initialized_project.clone());

    HttpResponse::Ok().json(initialized_project)
}

/// Get all projects
#[utoipa::path(
    get,
    path = "/project",
    responses(
        (status = 200, description = "List current todo items", body = [APISupportedInitializedProject])
    )
)]
pub(super) async fn get_projects(project_store: Data<ProjectStore>) -> impl Responder {
    let projects = project_store.projects.lock().await;

    HttpResponse::Ok().json(projects.clone())
}

