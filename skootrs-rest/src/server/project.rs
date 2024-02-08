use actix_web::{Responder, web::{ServiceConfig, Data, Json, self}, HttpResponse};
use serde::{Serialize, Deserialize};
use skootrs_statestore::SurrealProjectStateStore;
use utoipa::ToSchema;

use skootrs_model::skootrs::ProjectParams;
use skootrs_lib::service::{ecosystem::LocalEcosystemService, facet::LocalFacetService, project::{LocalProjectService, ProjectService}, repo::LocalRepoService, source::LocalSourceService};

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    /// When Project is not found by search term.
    NotFound(String),
    /// When a Project was unable to be initialized.
    InitializationError(String),
    /// When todo endpoint was called without correct credentials
    Unauthorized(String),
}

pub(super) fn configure(store: Data<SurrealProjectStateStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(web::resource("/projects")
                .route(web::post().to(create_project))
                .route(web::get().to(list_projects))
            );
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
    path = "/projects",
    request_body = ProjectParams,
    responses( 
        (status = 201, description = "Project created successfully", body = InitializedProject),
        (status = 409, description = "Project unable to be created", body = ErrorResponse, example = json!(ErrorResponse::InitializationError("Unable to create repo".into())))
    )
)]
pub(super) async fn create_project(params: Json<ProjectParams>, project_store: Data<SurrealProjectStateStore>) -> Result<impl Responder, actix_web::Error> {
    // TODO: This should be initialized elsewhere
    let project_service = LocalProjectService {
        repo_service: LocalRepoService {},
        ecosystem_service: LocalEcosystemService {},
        source_service: LocalSourceService {},
        facet_service: LocalFacetService {},
    };

    let initialized_project = project_service.initialize(params.into_inner()).await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    project_store.create(initialized_project.clone()).await.map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    Ok(HttpResponse::Ok().json(initialized_project))
}

/// Get all projects
#[utoipa::path(
    get,
    path = "/projects",
    responses(
        (status = 200, description = "List all projects", body = [InitializedProject]),
        (status = 500, description = "Internal server error", body = ErrorResponse, example = json!(ErrorResponse::InitializationError("Unable to list repos".into()))),
    )
)]
pub(super) async fn list_projects(project_store: Data<SurrealProjectStateStore>) -> Result<impl Responder, actix_web::Error> {
    let projects = project_store.select_all().await.map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    Ok(HttpResponse::Ok().json(projects))
}
