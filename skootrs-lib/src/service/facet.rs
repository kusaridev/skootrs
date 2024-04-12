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

// TODO: The content should be templatized or at least kept in separate files as opposed to just
// being thrown in giant strings inline with the code.

// TODO: Most of the generators for files need to be parameterized better.

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unused_self)]

use std::str::FromStr;

use askama::Template;
use chrono::Datelike;

use tracing::info;

use crate::service::source::SourceService;
use skootrs_model::{
    security_insights::insights10::{
        SecurityInsightsVersion100YamlSchema,
        SecurityInsightsVersion100YamlSchemaContributionPolicy,
        SecurityInsightsVersion100YamlSchemaDependencies,
        SecurityInsightsVersion100YamlSchemaDependenciesSbomItem,
        SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation,
        SecurityInsightsVersion100YamlSchemaHeader,
        SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion,
        SecurityInsightsVersion100YamlSchemaProjectLifecycle,
        SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus,
        SecurityInsightsVersion100YamlSchemaVulnerabilityReporting,
    },
    skootrs::{
        facet::{
            APIBundleFacet, APIBundleFacetParams, APIContent, CommonFacetCreateParams,
            FacetCreateParams, FacetSetCreateParams, InitializedFacet, SourceBundleFacet,
            SourceBundleFacetCreateParams, SourceFile, SourceFileContent, SourceFileFacet,
            SourceFileFacetParams, SupportedFacetType,
        },
        InitializedEcosystem, InitializedGithubRepo, InitializedRepo, SkootError,
    },
};

use super::source::LocalSourceService;

/// The `LocalFacetService` struct represents a service for creating and managing facets on the local machine.
#[derive(Debug)]
pub struct LocalFacetService {}

/// The `RootFacetService` trait provides an interface for initializing and managing a project's facets.
/// This includes things like initializing and managing source files, source bundles, and API bundles.
/// It is the root service for all facets and handles which other services to delegate to.
pub trait RootFacetService {
    fn initialize(
        &self,
        params: FacetCreateParams,
    ) -> impl std::future::Future<Output = Result<InitializedFacet, SkootError>> + Send;
    fn initialize_all(
        &self,
        params: FacetSetCreateParams,
    ) -> impl std::future::Future<Output = Result<Vec<InitializedFacet>, SkootError>> + Send;
}

/// (DEPRECATED) The `SourceFileFacetService` trait provides an interface for initializing and managing a project's source
/// file facets. This includes things like initializing and managing READMEs, licenses, and security policy
/// files.
///
pub trait SourceFileFacetService {
    /// Initializes a source file facet.
    ///
    /// # Errors
    ///
    /// Returns an error if the source file facet can't be initialized.
    fn initialize(&self, params: SourceFileFacetParams) -> Result<SourceFileFacet, SkootError>;
}

/// The `SourceBundleFacetService` trait provides an interface for initializing and managing a project's source
/// bundle facets. This includes things like initializing and managing set of files.
///
/// This replaces the `SourceFileFacetService` trait since it's more generic and can handle more than just
/// single files.
pub trait SourceBundleFacetService {
    /// Initializes a source bundle facet.
    ///
    /// # Errors
    ///
    /// Returns an error if the source bundle facet can't be initialized.
    fn initialize(
        &self,
        params: SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleFacet, SkootError>;
}

impl SourceBundleFacetService for LocalFacetService {
    /// Initializes a source bundle facet.
    ///
    /// # Errors
    ///
    /// Returns an error if the source bundle facet can't be initialized.
    fn initialize(
        &self,
        params: SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleFacet, SkootError> {
        let source_service = LocalSourceService {};
        let default_source_bundle_content_handler = DefaultSourceBundleContentHandler {};
        // TODO: Update this to be more generic on the repo service
        let language_specific_source_bundle_content_handler = match params.common.ecosystem {
            InitializedEcosystem::Go(_) => GoGithubSourceBundleContentHandler {},
            InitializedEcosystem::Maven(_) => todo!(),
        };

        let source_bundle_content = match params.facet_type {
            SupportedFacetType::Readme
            | SupportedFacetType::License
            | SupportedFacetType::SecurityPolicy
            | SupportedFacetType::Scorecard
            | SupportedFacetType::SecurityInsights => {
                default_source_bundle_content_handler.generate_content(&params)?
            }
            SupportedFacetType::Gitignore
            | SupportedFacetType::SLSABuild
            | SupportedFacetType::DependencyUpdateTool => {
                language_specific_source_bundle_content_handler.generate_content(&params)?
            }
            SupportedFacetType::SBOMGenerator => todo!(),
            SupportedFacetType::StaticCodeAnalysis => todo!(),
            SupportedFacetType::BranchProtection => todo!(),
            SupportedFacetType::CodeReview => todo!(),
            SupportedFacetType::Fuzzing => {
                language_specific_source_bundle_content_handler.generate_content(&params)?
            }
            SupportedFacetType::PublishPackages => todo!(),
            SupportedFacetType::PinnedDependencies => todo!(),
            SupportedFacetType::SAST => {
                default_source_bundle_content_handler.generate_content(&params)?
            }
            SupportedFacetType::VulnerabilityScanner => todo!(),
            SupportedFacetType::GUACForwardingConfig => todo!(),
            SupportedFacetType::Allstar => todo!(),
            SupportedFacetType::DefaultSourceCode => {
                language_specific_source_bundle_content_handler.generate_content(&params)?
            }
            SupportedFacetType::VulnerabilityReporting => {
                unimplemented!("VulnerabilityReporting is not implemented for source bundles")
            }
            SupportedFacetType::Other => todo!(),
        };

        for source_file_content in &source_bundle_content.source_files_content {
            info!(
                "Starting to write file {} to {}",
                source_file_content.name, source_file_content.path
            );
            source_service.write_file(
                params.common.source.clone(),
                source_file_content.path.clone(),
                source_file_content.name.clone(),
                source_file_content.content.clone(),
            )?;
        }

        let source_files: Vec<SourceFile> = source_bundle_content
            .source_files_content
            .iter()
            .map(|source_file_content| {
                Ok::<SourceFile, SkootError>(SourceFile {
                    name: source_file_content.name.clone(),
                    path: source_file_content.path.clone(),
                    hash: source_service.hash_file(
                        &params.common.source,
                        source_file_content.path.clone(),
                        source_file_content.name.clone(),
                    )?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let source_bundle_facet = SourceBundleFacet {
            source_files: Some(source_files),
            facet_type: params.facet_type,
            source_files_content: None,
        };

        Ok(source_bundle_facet)
    }
}

/// The `APIBundleFacetService` trait provides an interface for initializing and managing a project's API
/// bundle facets. This includes things like initializing and managing API calls to services like Github.
///
/// These API calls are used to enable features like branch protection, vulnerability reporting, etc.
pub trait APIBundleFacetService {
    fn initialize(
        &self,
        params: APIBundleFacetParams,
    ) -> impl std::future::Future<Output = Result<APIBundleFacet, SkootError>> + Send;
}

impl APIBundleFacetService for LocalFacetService {
    async fn initialize(&self, params: APIBundleFacetParams) -> Result<APIBundleFacet, SkootError> {
        // TODO: This should support more than just Github
        match params.facet_type {
            SupportedFacetType::CodeReview
            | SupportedFacetType::BranchProtection
            | SupportedFacetType::VulnerabilityReporting => {
                let github_api_bundle_handler = GithubAPIBundleHandler {};
                let api_bundle_facet = github_api_bundle_handler.generate(&params).await?;
                Ok(api_bundle_facet)
            }
            _ => todo!("Not implemented yet"),
        }
    }
}

/// The `SourceBundleContent` struct represents the content of a set of source files.
pub struct SourceBundleContent {
    pub source_files_content: Vec<SourceFileContent>,
    pub facet_type: SupportedFacetType,
}

impl RootFacetService for LocalFacetService {
    async fn initialize(&self, params: FacetCreateParams) -> Result<InitializedFacet, SkootError> {
        match params {
            FacetCreateParams::SourceFile(_params) => {
                todo!("This has been removed in favor of SourceBundle")
                /*let source_file_facet = SourceFileFacetService::initialize(self, params)?;
                Ok(InitializedFacet::SourceFile(source_file_facet))*/
            }
            FacetCreateParams::SourceBundle(params) => {
                let source_bundle_facet = SourceBundleFacetService::initialize(self, params)?;
                Ok(InitializedFacet::SourceBundle(source_bundle_facet))
            }
            FacetCreateParams::APIBundle(params) => {
                let api_bundle_facet = APIBundleFacetService::initialize(self, params).await?;
                Ok(InitializedFacet::APIBundle(api_bundle_facet))
            }
        }
    }

    async fn initialize_all(
        &self,
        params: FacetSetCreateParams,
    ) -> Result<Vec<InitializedFacet>, SkootError> {
        let futures = params
            .facets_params
            .iter()
            .map(move |params| RootFacetService::initialize(self, params.clone()));

        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }
}

/// The `APIBundleHandler` trait provides an interface for generating an `APIBundleFacet`.
/// This includes calling APIs to services like Github to enable features like branch protection,
/// vulnerability reporting, etc.
trait APIBundleHandler {
    async fn generate(&self, params: &APIBundleFacetParams) -> Result<APIBundleFacet, SkootError>;
}

/// The `GithubAPIBundleHandler` struct represents a handler for generating an `APIBundleFacet` related to
/// API calls made to Github.
struct GithubAPIBundleHandler {}

impl APIBundleHandler for GithubAPIBundleHandler {
    async fn generate(&self, params: &APIBundleFacetParams) -> Result<APIBundleFacet, SkootError> {
        let InitializedRepo::Github(repo) = &params.common.repo;
        match params.facet_type {
            SupportedFacetType::BranchProtection => self.generate_branch_protection(repo).await,
            SupportedFacetType::VulnerabilityReporting => {
                self.generate_vulnerability_reporting(repo).await
            }
            _ => todo!("Not implemented yet"),
        }
    }
}

impl GithubAPIBundleHandler {
    async fn generate_branch_protection(
        &self,
        repo: &InitializedGithubRepo,
    ) -> Result<APIBundleFacet, SkootError> {
        let enforce_branch_protection_endpoint = format!(
            "/repos/{owner}/{repo}/branches/{branch}/protection",
            owner = repo.organization.get_name(),
            repo = repo.name,
            branch = "main",
        );
        info!(
            "Enabling branch protection for {}",
            enforce_branch_protection_endpoint
        );
        // TODO: This should be a struct that serializes to json instead of just json directly
        let enforce_branch_protection_body = serde_json::json!({
            "enforce_admins": true,
            "required_pull_request_reviews": null,
            "required_status_checks": null,
            "restrictions": null,
            "required_linear_history": true,
            "allow_force_pushes": false,
            "allow_deletions": null,
        });

        // FIXME: I don't quite know why in some cases octocrab loses my auth and I have to re-authenticate
        let o: octocrab::Octocrab = octocrab::Octocrab::builder()
            .personal_token(
                std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var must be populated"),
            )
            .build()?;
        octocrab::initialise(o);
        let response: serde_json::Value = octocrab::instance()
            .put(
                &enforce_branch_protection_endpoint,
                Some(&enforce_branch_protection_body),
            )
            .await?;

        let apis = vec![APIContent {
            name: "Enforce Branch Protection".to_string(),
            url: enforce_branch_protection_endpoint,
            response: serde_json::to_string_pretty(&response)?,
        }];

        Ok(APIBundleFacet {
            facet_type: SupportedFacetType::BranchProtection,
            apis,
        })
    }

    async fn generate_vulnerability_reporting(
        &self,
        repo: &InitializedGithubRepo,
    ) -> Result<APIBundleFacet, SkootError> {
        let vulnerability_reporting_endpoint = format!(
            "/repos/{owner}/{repo}/private-vulnerability-reporting",
            owner = repo.organization.get_name(),
            repo = repo.name,
        );
        info!(
            "Enabling vulnerability reporting for {}",
            &vulnerability_reporting_endpoint
        );
        // Note: This call just returns a status with no JSON output also the normal .put I think expects json
        // output and will fail.
        octocrab::instance()
            ._put(&vulnerability_reporting_endpoint, None::<&()>)
            .await?;
        let apis = vec![APIContent {
            name: "Enabling vulnerability reporting".to_string(),
            url: vulnerability_reporting_endpoint.clone(),
            response: "Success".to_string(),
        }];
        info!(
            "Vulnerability reporting enabled for {}",
            &vulnerability_reporting_endpoint
        );

        Ok(APIBundleFacet {
            facet_type: SupportedFacetType::VulnerabilityReporting,
            apis,
        })
    }
}

/// The `SourceBundleContentGenerator` trait provides an interface for generating the
/// content (i.e. text) for a set of source files.
trait SourceBundleContentGenerator {
    fn generate_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError>;
}

/// Handles the generation of source files content that are generic to all projects by default,
/// e.g. README.md, LICENSE, etc.
struct DefaultSourceBundleContentHandler {}

impl SourceBundleContentGenerator for DefaultSourceBundleContentHandler {
    fn generate_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        match params.facet_type {
            SupportedFacetType::Readme => self.generate_readme_content(params),
            SupportedFacetType::License => self.generate_license_content(params),
            SupportedFacetType::SecurityPolicy => self.generate_security_policy_content(params),
            SupportedFacetType::Scorecard => self.generate_scorecard_content(params),
            SupportedFacetType::SecurityInsights => self.generate_security_insights_content(params),
            SupportedFacetType::SAST => self.generate_sast_content(params),
            _ => todo!("Not implemented yet"),
        }
    }
}
impl DefaultSourceBundleContentHandler {
    fn generate_readme_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "README.md", escape = "none")]
        struct ReadmeTemplateParams {
            project_name: String,
        }

        let readme_template_params = ReadmeTemplateParams {
            project_name: params.common.project_name.clone(),
        };

        let content = readme_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "README.md".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::Readme,
        })
    }
    // TODO: Support more than Apache 2.0
    fn generate_license_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "LICENSE", escape = "none")]
        struct LicenseTemplateParams {
            project_name: String,
            date: i32,
        }

        let license_template_params = LicenseTemplateParams {
            project_name: params.common.project_name.clone(),
            date: chrono::Utc::now().year(),
        };

        let content = license_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "LICENSE".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::License,
        })
    }
    // TODO: Create actual security policy
    fn generate_security_policy_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        // TODO: Turn this into a real default security policy
        #[derive(Template)]
        #[template(path = "SECURITY.prerelease.md", escape = "none")]
        struct SecurityPolicyTemplateParams {}

        let security_policy_template_params = SecurityPolicyTemplateParams {};
        let content = security_policy_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "SECURITY.md".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::SecurityPolicy,
        })
    }

    fn generate_scorecard_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        // TODO: This should serialize to yaml instead of just a file template
        #[derive(Template)]
        #[template(path = "scorecard.yml", escape = "none")]
        struct ScorecardTemplateParams {}

        let scorecard_template_params = ScorecardTemplateParams {};
        let content = scorecard_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "scorecard.yml".to_string(),
                path: "./.github/workflows".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::Scorecard,
        })
    }

    #[allow(clippy::too_many_lines)]
    fn generate_security_insights_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        let insights = SecurityInsightsVersion100YamlSchema {
            contribution_policy: SecurityInsightsVersion100YamlSchemaContributionPolicy {
                accepts_automated_pull_requests: true,
                accepts_pull_requests: true,
                automated_tools_list: None,
                code_of_conduct: None,
                contributing_policy: None,
            },
            dependencies: Some(SecurityInsightsVersion100YamlSchemaDependencies{
                dependencies_lifecycle: None,
                dependencies_lists: vec![
                    format!("{}/blob/main/go.mod", &params.common.repo.full_url())
                ],
                env_dependencies_policy: None,
                sbom: Some(vec![
                    SecurityInsightsVersion100YamlSchemaDependenciesSbomItem {
                        sbom_creation: Some(
                            SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation::from_str("Created by goreleaser")?),
                        sbom_file: Some(format!("{}/releases/latest/download/main-linux-amd64.spdx.sbom.json", &params.common.repo.full_url())), 
                        sbom_format: Some("SPDX".to_string()),
                        sbom_url: Some("https://spdx.github.io/spdx-spec/v2.3/".to_string()), 
                    },
                    SecurityInsightsVersion100YamlSchemaDependenciesSbomItem {
                        sbom_creation: Some(
                            SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation::from_str("Created by goreleaser")?),
                        sbom_file: Some(format!("{}/releases/latest/download/main-linux-arm.spdx.sbom.json", &params.common.repo.full_url())), 
                        sbom_format: Some("SPDX".to_string()),
                        sbom_url: Some("https://spdx.github.io/spdx-spec/v2.3/".to_string()), 
                    },
                    SecurityInsightsVersion100YamlSchemaDependenciesSbomItem {
                        sbom_creation: Some(
                            SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation::from_str("Created by goreleaser")?),
                        sbom_file: Some(format!("{}/releases/latest/download/main-linux-arm64.spdx.sbom.json", &params.common.repo.full_url())), 
                        sbom_format: Some("SPDX".to_string()),
                        sbom_url: Some("https://spdx.github.io/spdx-spec/v2.3/".to_string()), 
                    },
                    SecurityInsightsVersion100YamlSchemaDependenciesSbomItem {
                        sbom_creation: Some(
                            SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation::from_str("Created by goreleaser")?),
                        sbom_file: Some(format!("{}/releases/latest/download/main-windows-amd64.exe.spdx.sbom.json", &params.common.repo.full_url())), 
                        sbom_format: Some("SPDX".to_string()),
                        sbom_url: Some("https://spdx.github.io/spdx-spec/v2.3/".to_string()), 
                    },
                    SecurityInsightsVersion100YamlSchemaDependenciesSbomItem {
                        sbom_creation: Some(
                            SecurityInsightsVersion100YamlSchemaDependenciesSbomItemSbomCreation::from_str("Created by goreleaser")?),
                        sbom_file: Some(format!("{}/releases/latest/download/main.spdx.sbom.json", &params.common.repo.full_url())), 
                        sbom_format: Some("SPDX".to_string()),
                        sbom_url: Some("https://spdx.github.io/spdx-spec/v2.3/".to_string()), 
                    },
                ]),
                third_party_packages: Some(true),
            }),
            distribution_points: Vec::new(),
            documentation: None,
            header: SecurityInsightsVersion100YamlSchemaHeader {
                changelog: None,
                commit_hash: None,
                expiration_date: chrono::Utc::now() + chrono::Duration::days(365),
                last_reviewed: Some(chrono::Utc::now()),
                last_updated: Some(chrono::Utc::now()),
                license: Some(format!(
                    "{}/blob/main/LICENSE",
                    &params.common.repo.full_url()
                )),
                project_release: None,
                project_url: params.common.repo.full_url(),
                schema_version: SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion::_100,
            },
            project_lifecycle: SecurityInsightsVersion100YamlSchemaProjectLifecycle {
                bug_fixes_only: false,
                core_maintainers: None,
                release_cycle: None,
                release_process: None,
                roadmap: None,
                status: SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus::Active,
            },
            // TODO: Since security insights doesn't support SLSA, scorecard, etc. explicitly we might want to add it
            // to security_artifacts.
            security_artifacts: None,
            security_assessments: None,
            security_contacts: Vec::new(),
            security_testing: Vec::new(),
            vulnerability_reporting: SecurityInsightsVersion100YamlSchemaVulnerabilityReporting {
                accepts_vulnerability_reports: true,
                bug_bounty_available: None,
                bug_bounty_url: None,
                comment: None,
                email_contact: None,
                in_scope: None,
                out_scope: None,
                pgp_key: None,
                security_policy: Some(format!("{}/blob/main/SECURITY.md", &params.common.repo.full_url())),
            },
        };

        let content = serde_yaml::to_string(&insights)?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "SECURITY-INSIGHTS.yml".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::SecurityInsights,
        })
    }

    fn generate_sast_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "codeql.yml", escape = "none")]
        struct SASTTemplateParams {}

        let sast_template_params = SASTTemplateParams {};
        let content = sast_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "codeql.yml".to_string(),
                path: "./.github/workflows".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::SAST,
        })
    }
}

/// Handles the generation of source files content specific to Go projects hosted on Github.
/// e.g. Github actions running goreleaser
struct GoGithubSourceBundleContentHandler {}

impl SourceBundleContentGenerator for GoGithubSourceBundleContentHandler {
    fn generate_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        match params.facet_type {
            SupportedFacetType::Gitignore => self.generate_gitignore_content(params),
            // TODO: Rename this to something like SecureBuild.
            // This also does a bunch of other stuff like setting up releases, generating SBOM, etc.
            // So for now just we just use it instead of creating multiple facets.
            // The better option is to probably set up some mapping of properties like SLSA, SBOMGenerating, etc.
            // to a single SecureBuild facet.
            SupportedFacetType::SLSABuild => self.generate_slsa_build_content(params),
            SupportedFacetType::DependencyUpdateTool => {
                self.generate_dependency_update_tool_content(params)
            }
            SupportedFacetType::Fuzzing => self.generate_fuzzing_content(params),
            SupportedFacetType::DefaultSourceCode => {
                self.generate_default_source_code_content(params)
            }
            _ => todo!("Not implemented yet"),
        }
    }
}
impl GoGithubSourceBundleContentHandler {
    fn generate_gitignore_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "go.gitignore", escape = "none")]
        struct GitignoreTemplateParams {}

        let gitignore_template_params = GitignoreTemplateParams {};
        let content = gitignore_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: ".gitignore".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::Gitignore,
        })
    }
    // Note: GoReleaser also does a bunch of other stuff like setting up releases, generating SBOM, etc.
    // So for now just we just use it instead of creating multiple facets.
    // Note: Content mostly taken from https://github.com/guacsec/guac/blob/f1703bd4ca3c0ec0fa55c5a3401d50578fb1680e/.github/workflows/release.yaml
    fn generate_slsa_build_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        // TODO: This should really be a struct that serializes to yaml instead of just a file template
        #[derive(Template)]
        #[template(path = "go.releases.yml", escape = "none")]
        struct ReleaseTemplateParams {}

        #[derive(Template)]
        #[template(path = "Dockerfile.goreleaser", escape = "none")]
        struct DockerfileTemplateParams {
            project_name: String,
        }

        #[derive(Template)]
        #[template(path = "goreleaser.yml", escape = "none")]
        struct GoReleaserTemplateParams {
            project_name: String,
            module_name: String,
        }

        #[allow(clippy::match_wildcard_for_single_variants)]
        let module = match &params.common.ecosystem {
            InitializedEcosystem::Go(go) => go.module(),
            _ => unreachable!("Ecosystem should be Go"),
        };

        let slsa_build_template_params = ReleaseTemplateParams {};
        let dockerfile_template_params = DockerfileTemplateParams {
            project_name: params.common.project_name.clone(),
        };
        let goreleaser_template_params = GoReleaserTemplateParams {
            project_name: params.common.project_name.clone(),
            module_name: module,
        };

        Ok(SourceBundleContent {
            source_files_content: vec![
                SourceFileContent {
                    name: "releases.yml".to_string(),
                    path: ".github/workflows/".to_string(),
                    content: slsa_build_template_params.render()?,
                },
                SourceFileContent {
                    name: "Dockerfile.goreleaser".to_string(),
                    path: "./".to_string(),
                    content: dockerfile_template_params.render()?,
                },
                SourceFileContent {
                    name: ".goreleaser.yml".to_string(),
                    path: "./".to_string(),
                    content: goreleaser_template_params.render()?,
                },
            ],
            facet_type: SupportedFacetType::SLSABuild,
        })
    }

    fn generate_dependency_update_tool_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "dependabot.yml", escape = "none")]
        struct DependabotTemplateParams {
            ecosystem: String,
        }

        let dependabot_template_params = DependabotTemplateParams {
            ecosystem: "gomod".to_string(),
        };
        let content = dependabot_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "dependabot.yml".to_string(),
                path: ".github/".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::DependencyUpdateTool,
        })
    }

    fn generate_fuzzing_content(
        &self,
        params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "cifuzz.yml", escape = "none")]
        struct FuzzingTemplateParams {
            project_name: String,
            language: String,
        }

        let fuzzing_template_params = FuzzingTemplateParams {
            project_name: params.common.project_name.clone(),
            language: "go".to_string(),
        };
        let content = fuzzing_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "cifuzz.yml".to_string(),
                path: ".github/workflows/".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::Fuzzing,
        })
    }

    fn generate_default_source_code_content(
        &self,
        _params: &SourceBundleFacetCreateParams,
    ) -> Result<SourceBundleContent, SkootError> {
        #[derive(Template)]
        #[template(path = "main.go.tmpl", escape = "none")]
        struct DefaultSourceCodeTemplateParams {}

        let default_source_code_template_params = DefaultSourceCodeTemplateParams {};
        let content = default_source_code_template_params.render()?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "main.go".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::DefaultSourceCode,
        })
    }
}

/// The `FacetSetParamsGenerator` struct represents a service for generating params for a set of facets.
/// This includes things like generating default params for source bundles and API bundles.
pub struct FacetSetParamsGenerator {}

impl FacetSetParamsGenerator {
    /// Generates the default set of facet params for a project.
    /// This includes things like generating default source bundle and API bundle facet params.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the facet set params can't be generated.
    pub fn generate_default(
        &self,
        common_params: &CommonFacetCreateParams,
    ) -> Result<FacetSetCreateParams, SkootError> {
        let source_bundle_params =
            self.generate_default_source_bundle_facet_params(common_params)?;
        let api_bundle_params = self.generate_default_api_bundle(common_params)?;
        let total_params = FacetSetCreateParams {
            facets_params: [
                source_bundle_params.facets_params,
                api_bundle_params.facets_params,
            ]
            .concat(),
        };

        Ok(total_params)
    }

    /// Generates the default set of API bundle facet params for a project.
    ///
    /// # Errors
    ///
    /// Returns an error if the default set of API bundle facets can't be generated.
    pub fn generate_default_api_bundle(
        &self,
        common_params: &CommonFacetCreateParams,
    ) -> Result<FacetSetCreateParams, SkootError> {
        use SupportedFacetType::{BranchProtection, VulnerabilityReporting};
        let supported_facets = [
            //CodeReview,
            BranchProtection,
            VulnerabilityReporting,
        ];
        let facets_params = supported_facets
            .iter()
            .map(|facet_type| {
                FacetCreateParams::APIBundle(APIBundleFacetParams {
                    common: common_params.clone(),
                    facet_type: facet_type.clone(),
                })
            })
            .collect::<Vec<FacetCreateParams>>();

        Ok(FacetSetCreateParams { facets_params })
    }

    // TODO: Come up with a better solution than hard coding the default facets
    /// Generates the default set of source bundle facet params for a project.
    ///
    /// # Errors
    ///
    /// Returns an error if the default set of source bundle facets can't be generated.
    pub fn generate_default_source_bundle_facet_params(
        &self,
        common_params: &CommonFacetCreateParams,
    ) -> Result<FacetSetCreateParams, SkootError> {
        use SupportedFacetType::{
            DefaultSourceCode, DependencyUpdateTool, Gitignore, License, Readme, SLSABuild,
            Scorecard, SecurityInsights, SecurityPolicy, SAST,
        };
        let supported_facets = [
            Readme,
            License,
            Gitignore,
            SecurityPolicy,
            SecurityInsights,
            SLSABuild,
            // SBOMGenerator, // Handled by the SLSABuild facet
            // StaticCodeAnalysis,
            DependencyUpdateTool,
            // TODO: Fuzzing right now requires a bunch of resources that are unavailable to most projects without
            // some sort of manual intervention. This is disabled until some option becomes available.
            // Fuzzing,
            Scorecard,
            // PublishPackages,
            // PinnedDependencies,
            SAST,
            // VulnerabilityScanner,
            // GUACForwardingConfig,
            // These are at the end to allow Skootrs to push initial commits without needing
            // code review or branches.
            // CodeReview, // TODO: Implement this
            //BranchProtection, //TODO: Implement this
            DefaultSourceCode,
        ];
        let facets_params = supported_facets
            .iter()
            .map(|facet_type| {
                FacetCreateParams::SourceBundle(SourceBundleFacetCreateParams {
                    common: common_params.clone(),
                    facet_type: facet_type.clone(),
                })
            })
            .collect::<Vec<FacetCreateParams>>();

        Ok(FacetSetCreateParams { facets_params })
    }
}
