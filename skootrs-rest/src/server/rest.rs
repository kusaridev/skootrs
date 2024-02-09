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

use std::net::Ipv4Addr;

use actix_web::{App, HttpServer, web::Data};
use skootrs_statestore::SurrealProjectStateStore;
use tracing_actix_web::TracingLogger;
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, ApiKey, ApiKeyValue}};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::server::project::ErrorResponse;
use skootrs_model::{skootrs::{InitializedProject, ProjectParams, InitializedRepo, InitializedGithubRepo, InitializedEcosystem, RepoParams, EcosystemParams, GithubUser, GithubRepoParams, SourceParams, InitializedSource, MavenParams, GoParams, InitializedGo, InitializedMaven, facet::{CommonFacetParams, SourceFileFacet, SourceFileFacetParams, InitializedFacet, FacetParams, SupportedFacetType}}, cd_events::repo_created::{RepositoryCreatedEvent, RepositoryCreatedEventContext, RepositoryCreatedEventContextId, RepositoryCreatedEventContextVersion, RepositoryCreatedEventSubject, RepositoryCreatedEventSubjectContent, RepositoryCreatedEventSubjectContentUrl, RepositoryCreatedEventSubjectId}, security_insights::insights10::{SecurityInsightsVersion100YamlSchema, SecurityInsightsVersion100YamlSchemaContributionPolicy, SecurityInsightsVersion100YamlSchemaContributionPolicyAutomatedToolsListItem, SecurityInsightsVersion100YamlSchemaContributionPolicyAutomatedToolsListItemComment, SecurityInsightsVersion100YamlSchemaDependencies, SecurityInsightsVersion100YamlSchemaDependenciesDependenciesLifecycle, SecurityInsightsVersion100YamlSchemaDependenciesDependenciesLifecycleComment, SecurityInsightsVersion100YamlSchemaDependenciesEnvDependenciesPolicy, SecurityInsightsVersion100YamlSchemaDependenciesEnvDependenciesPolicyComment, SecurityInsightsVersion100YamlSchemaDependenciesSbomItem, SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation, SecurityInsightsVersion100YamlSchemaHeader, SecurityInsightsVersion100YamlSchemaHeaderCommitHash, SecurityInsightsVersion100YamlSchemaProjectLifecycle, SecurityInsightsVersion100YamlSchemaProjectLifecycleReleaseProcess, SecurityInsightsVersion100YamlSchemaSecurityArtifacts, SecurityInsightsVersion100YamlSchemaSecurityArtifactsSelfAssessment, SecurityInsightsVersion100YamlSchemaSecurityArtifactsSelfAssessmentComment, SecurityInsightsVersion100YamlSchemaSecurityArtifactsThreatModel, SecurityInsightsVersion100YamlSchemaSecurityArtifactsThreatModelComment, SecurityInsightsVersion100YamlSchemaSecurityAssessmentsItem, SecurityInsightsVersion100YamlSchemaSecurityAssessmentsItemComment, SecurityInsightsVersion100YamlSchemaSecurityContactsItem, SecurityInsightsVersion100YamlSchemaSecurityContactsItemValue, SecurityInsightsVersion100YamlSchemaSecurityTestingItem, SecurityInsightsVersion100YamlSchemaSecurityTestingItemComment, SecurityInsightsVersion100YamlSchemaSecurityTestingItemIntegration, SecurityInsightsVersion100YamlSchemaVulnerabilityReporting, SecurityInsightsVersion100YamlSchemaVulnerabilityReportingComment, SecurityInsightsVersion100YamlSchemaVulnerabilityReportingPgpKey}};
use skootrs_model::skootrs::facet::{SourceBundleFacet, SourceBundleFacetParams, APIBundleFacet, APIBundleFacetParams, SourceFileContent, APIContent};

/// Run the Skootrs REST API server.
#[actix_web::main]
pub async fn run_server() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::server::project::create_project,
            crate::server::project::list_projects,
        ),
        components(
            schemas(
                // Server only schemas
                ErrorResponse, 

                // Skootrs Model schemas
                InitializedProject,
                ProjectParams,
                InitializedRepo,
                InitializedGithubRepo,
                InitializedEcosystem,
                RepoParams,
                EcosystemParams,
                GithubUser,
                GithubRepoParams,
                SourceParams,
                InitializedSource,
                MavenParams,
                GoParams,
                InitializedGo,
                InitializedMaven,
                // Facet Schemas
                CommonFacetParams,
                SourceFileFacet,
                SourceFileFacetParams,
                InitializedFacet,
                FacetParams,
                SupportedFacetType,
                InitializedProject,
                SourceBundleFacet,
                SourceBundleFacetParams,
                APIBundleFacet,
                APIBundleFacetParams,
                SourceFileContent,
                APIContent,

                // CD Events Schemas
                RepositoryCreatedEvent,
                RepositoryCreatedEventContext,
                RepositoryCreatedEventContextId,
                RepositoryCreatedEventContextVersion,
                RepositoryCreatedEventSubject,
                RepositoryCreatedEventSubjectContent,
                RepositoryCreatedEventContext,
                RepositoryCreatedEventSubjectContentUrl,
                RepositoryCreatedEventSubjectId,

                // Security Insights Schemas
                SecurityInsightsVersion100YamlSchema,
                SecurityInsightsVersion100YamlSchemaContributionPolicy,
                SecurityInsightsVersion100YamlSchemaContributionPolicyAutomatedToolsListItem,
                SecurityInsightsVersion100YamlSchemaContributionPolicyAutomatedToolsListItemComment,
                SecurityInsightsVersion100YamlSchemaDependencies,
                SecurityInsightsVersion100YamlSchemaDependenciesDependenciesLifecycle,
                SecurityInsightsVersion100YamlSchemaDependenciesDependenciesLifecycleComment,
                SecurityInsightsVersion100YamlSchemaDependenciesEnvDependenciesPolicy,
                SecurityInsightsVersion100YamlSchemaDependenciesEnvDependenciesPolicyComment,
                SecurityInsightsVersion100YamlSchemaDependenciesSbomItem,
                SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation,
                SecurityInsightsVersion100YamlSchemaHeader,
                SecurityInsightsVersion100YamlSchemaHeaderCommitHash,
                SecurityInsightsVersion100YamlSchemaProjectLifecycle,
                SecurityInsightsVersion100YamlSchemaProjectLifecycleReleaseProcess,
                SecurityInsightsVersion100YamlSchemaSecurityArtifacts,
                SecurityInsightsVersion100YamlSchemaSecurityArtifactsSelfAssessment,
                SecurityInsightsVersion100YamlSchemaSecurityArtifactsSelfAssessmentComment,
                SecurityInsightsVersion100YamlSchemaSecurityArtifactsThreatModel,
                SecurityInsightsVersion100YamlSchemaSecurityArtifactsThreatModelComment,
                SecurityInsightsVersion100YamlSchemaSecurityAssessmentsItem,
                SecurityInsightsVersion100YamlSchemaSecurityAssessmentsItemComment,
                SecurityInsightsVersion100YamlSchemaSecurityContactsItem,
                SecurityInsightsVersion100YamlSchemaSecurityContactsItemValue,
                SecurityInsightsVersion100YamlSchemaSecurityTestingItem,
                SecurityInsightsVersion100YamlSchemaSecurityTestingItemComment,
                SecurityInsightsVersion100YamlSchemaSecurityTestingItemIntegration,
                SecurityInsightsVersion100YamlSchemaVulnerabilityReporting,
                SecurityInsightsVersion100YamlSchemaVulnerabilityReportingComment,
                SecurityInsightsVersion100YamlSchemaVulnerabilityReportingPgpKey,
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
            let components = openapi.components.as_mut().expect("Components must exist");
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("skootrs_apikey"))),
            );
        }
    }

    let store: Data<SurrealProjectStateStore> = Data::new(SurrealProjectStateStore::new().await.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?);
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
