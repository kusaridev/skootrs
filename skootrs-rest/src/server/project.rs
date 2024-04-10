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

use actix_web::{Responder, web::{ServiceConfig, Data, Json, self}, HttpResponse};
use serde::{Serialize, Deserialize};
use skootrs_statestore::{InMemoryProjectReferenceCache, ProjectReferenceCache};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use skootrs_model::skootrs::ProjectCreateParams;
use skootrs_lib::service::{ecosystem::LocalEcosystemService, facet::LocalFacetService, output::LocalOutputService, project::{LocalProjectService, ProjectService}, repo::LocalRepoService, source::LocalSourceService};

/// An Error response for the REST API
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    /// When Project is not found by search term.
    NotFound(String),
    /// When a Project was unable to be initialized.
    InitializationError(String),
    /// When todo endpoint was called without correct credentials
    Unauthorized(String),
}

/// Configures the services and routes for the Skootrs REST API
pub(super) fn configure(store: Data<Mutex<InMemoryProjectReferenceCache>>) -> impl FnOnce(&mut ServiceConfig) {
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
pub(super) async fn create_project(params: Json<ProjectCreateParams>, project_store: Data<Mutex<InMemoryProjectReferenceCache>>) -> Result<impl Responder, actix_web::Error> {
    // TODO: This should be initialized elsewhere
    let project_service = LocalProjectService {
        repo_service: LocalRepoService {},
        ecosystem_service: LocalEcosystemService {},
        source_service: LocalSourceService {},
        facet_service: LocalFacetService {},
        output_service: LocalOutputService {},
    };

    let initialized_project = project_service.initialize(params.into_inner()).await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    project_store.lock().await.set(initialized_project.repo.full_url()).await.map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    // TODO: Should this return an internal server error if it can't save the cache?
    project_store.lock().await.save().map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
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
pub(super) async fn list_projects(project_store: Data<InMemoryProjectReferenceCache>) -> Result<impl Responder, actix_web::Error> {
    let projects = project_store.list().await.map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;
    Ok(HttpResponse::Ok().json(projects))
}
