use std::net::Ipv4Addr;

use actix_web::{App, HttpServer, web::Data};
use tracing_actix_web::TracingLogger;
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, ApiKey, ApiKeyValue}};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{server::project::{ProjectStore, APISupportedInitializedProject, APISupportedCreateProjectParams, ErrorResponse, APISupportedRepo, APISupportedEcosystem, APISupportedRepoParams, APISupportedEcosystemParams}, ecosystem::{maven::MavenParams, go::GoParams}, repo::github::{UninitializedGithubRepo, GithubUser, InitializedGithubRepo}, source::Source};

pub struct SkootrsWebConfig {

}

pub async fn run_server(_config: Option<SkootrsWebConfig>) -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::server::project::create_project
        ),
        components(
            schemas(
                APISupportedInitializedProject,
                APISupportedCreateProjectParams,
                ErrorResponse, 
                APISupportedRepo,
                APISupportedEcosystem,
                APISupportedRepoParams,
                APISupportedEcosystemParams,
                MavenParams,
                GoParams,
                UninitializedGithubRepo,
                GithubUser,
                InitializedGithubRepo,
                Source,
            )
        ),
        tags(
            (name = "skootrs", description = "Skootrs endpoints.")
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("skootrs_apikey"))),
            )
        }
    }

    let store = Data::new(ProjectStore::default());
    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(crate::server::project::configure(store.clone()))
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}