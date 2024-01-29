use actix_web::{Responder, web::{ServiceConfig, Data, Json, self}, HttpResponse};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use skootrs_lib::{model::skootrs::{InitializedProject, ProjectParams}, service::{project::{LocalProjectService, ProjectService}, repo::LocalRepoService, ecosystem::LocalEcosystemService, source::LocalSourceService, facet::LocalFacetService}};

#[derive(Default)]
pub(super) struct ProjectStore {
    projects: Mutex<Vec<InitializedProject>>
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
            .service(web::resource("/projects").route(web::get().to(get_projects)));
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
    request_body = ProjectParams,
    responses( 
        (status = 201, description = "Project created successfully", body = InitializedProject),
        (status = 409, description = "Project unable to be created", body = ErrorResponse, example = json!(ErrorResponse::InitializationError("Unable to create repo".into())))
    )
)]
pub(super) async fn create_project(params: Json<ProjectParams>, project_store: Data<ProjectStore>) -> Result<impl Responder, actix_web::Error> {
    // TODO: This should be initialized elsewhere
    let project_service = LocalProjectService {
        repo_service: LocalRepoService {},
        ecosystem_service: LocalEcosystemService {},
        source_service: LocalSourceService {},
        facet_service: LocalFacetService {},
    };

    let initialized_project = project_service.initialize(params.into_inner()).await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    project_store.projects.lock().await.push(initialized_project.clone());
    Ok(HttpResponse::Ok().json(initialized_project))
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

