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

#![allow(clippy::module_name_repetitions)]

use std::error::Error;

use crate::service::facet::{FacetSetParamsGenerator, RootFacetService};
use skootrs_model::skootrs::{
    facet::CommonFacetParams, InitializedProject, InitializedSource, ProjectParams,
};

use super::{
    ecosystem::EcosystemService,
    repo::RepoService,
    source::SourceService,
};
use tracing::debug;

/// The `ProjectService` trait provides an interface for initializing and managing a Skootrs project.
pub trait ProjectService {
    /// Initializes a Skootrs project.
    ///h
    /// # Errors
    ///
    /// Returns an error if the project can't be initialized for any reason.
    fn initialize(
        &self,
        params: ProjectParams,
    ) -> impl std::future::Future<Output = Result<InitializedProject, Box<dyn Error + Send + Sync>>> + Send;
}

/// The `LocalProjectService` struct provides an implementation of the `ProjectService` trait for initializing
/// and managing a Skootrs project on the local machine.
#[derive(Debug)]
pub struct LocalProjectService<RS: RepoService, ES: EcosystemService, SS: SourceService, FS: RootFacetService> {
    pub repo_service: RS,
    pub ecosystem_service: ES,
    pub source_service: SS,
    pub facet_service: FS,
}

impl<RS, ES, SS, FS> ProjectService for LocalProjectService<RS, ES, SS, FS>
where
    RS: RepoService + Send + Sync,
    ES: EcosystemService + Send + Sync,
    SS: SourceService + Send + Sync,
    FS: RootFacetService + Send + Sync,
{
    async fn initialize(
        &self,
        params: ProjectParams,
    ) -> Result<InitializedProject, Box<dyn Error + Send + Sync>> {
        debug!("Starting repo initialization");
        let initialized_repo = self
            .repo_service
            .initialize(params.repo_params.clone())
            .await?;
        debug!("Starting source initialization");
        let initialized_source: InitializedSource = self
            .source_service
            .initialize(params.source_params.clone(), initialized_repo.clone())?;
        debug!("Starting ecosystem initialization");
        let initialized_ecosystem = self
            .ecosystem_service
            .initialize(params.ecosystem_params.clone(), initialized_source.clone())?;
        debug!("Starting facet initialization");
        // TODO: This is ugly and this should probably be configured somewhere better, preferably outside of code.
        let facet_set_params_generator = FacetSetParamsGenerator {};
        let common_params = CommonFacetParams {
            project_name: params.name.clone(),
            source: initialized_source.clone(),
            repo: initialized_repo.clone(),
            ecosystem: initialized_ecosystem.clone(),
        };
        //let facet_set_params = facet_set_params_generator.generate_default(&common_params)?;
        let source_facet_set_params =
            facet_set_params_generator.generate_default_source_bundle_facet_params(&common_params)?;
        let api_facet_set_params =
            facet_set_params_generator.generate_default_api_bundle(&common_params)?;
        let initialized_source_facets = self
            .facet_service
            .initialize_all(source_facet_set_params)
            .await?;
        // TODO: Figure out how to better order commits and pushes
        self.source_service.commit_and_push_changes(
            initialized_source.clone(),
            "Initialized project".to_string(),
        )?;
        let initialized_api_facets = self
            .facet_service
            .initialize_all(api_facet_set_params)
            .await?;
        let initialized_facets = [initialized_source_facets, initialized_api_facets].concat();

        debug!("Completed project initialization");

        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: initialized_ecosystem,
            source: initialized_source,
            facets: initialized_facets,
        })
    }
}

#[cfg(test)]
mod tests {
    use skootrs_model::skootrs::{
        facet::{
            APIBundleFacet, APIContent, FacetParams, FacetSetParams, InitializedFacet,
            SourceBundleFacet, SourceFileContent, SupportedFacetType,
        }, EcosystemParams, GithubRepoParams, GithubUser, GoParams, InitializedEcosystem, InitializedGithubRepo, InitializedGo, InitializedMaven, InitializedRepo, RepoParams, SkootError, SourceParams
    };

    use super::*;
    struct MockRepoService;
    struct MockEcosystemService;
    struct MockSourceService;
    struct MockFacetService;

    impl RepoService for MockRepoService {
        fn initialize(&self, params: RepoParams) -> impl std::future::Future<Output = Result<InitializedRepo, SkootError>> + Send {
            async {
                let inner_params = match params {
                    RepoParams::Github(g) => g,
                };
    
                // Special case for testing error handling
                if inner_params.name == "error" {
                    return Err("Error".into())
                }
    
                let initialized_repo = InitializedRepo::Github(InitializedGithubRepo {
                    name: inner_params.name,
                    organization: inner_params.organization,
                });
    
                Ok(initialized_repo)
            }
        }

        fn clone_local(
            &self,
            initialized_repo: InitializedRepo,
            path: String,
        ) -> Result<InitializedSource, SkootError> {
            let inner_repo = match initialized_repo {
                InitializedRepo::Github(g) => g,
            };

            if inner_repo.name == "error" {
                return Err("Error".into());
            }

            let initialized_source = InitializedSource {
                path: format!("{}/{}", path, inner_repo.name),
            };

            Ok(initialized_source)
        }
    }

    impl EcosystemService for MockEcosystemService {
        fn initialize(
            &self,
            params: EcosystemParams,
            _source: InitializedSource,
        ) -> Result<InitializedEcosystem, SkootError> {
            let initialized_ecosystem = match params {
                EcosystemParams::Go(g) => {
                    if g.host == "error" {
                        return Err("Error".into());
                    }
                    InitializedEcosystem::Go(InitializedGo{
                        name: g.name,
                        host: g.host,
                    })
                }
                EcosystemParams::Maven(m) => {
                    if m.group_id == "error" {
                        return Err("Error".into());
                    }
                    InitializedEcosystem::Maven(InitializedMaven{
                        group_id: m.group_id,
                        artifact_id: m.artifact_id,
                    })
                }
            };

            Ok(initialized_ecosystem)
        }
    }

    impl SourceService for MockSourceService {
        fn initialize(
            &self,
            params: skootrs_model::skootrs::SourceParams,
            initialized_repo: InitializedRepo,
        ) -> Result<InitializedSource, SkootError> {
            if params.parent_path == "error" {
                return Err("Error".into());
            }

            let repo_name = match initialized_repo {
                InitializedRepo::Github(g) => g.name,
            };

            let initialized_source = InitializedSource {
                path: format!("{}/{}", params.parent_path, repo_name),
            };

            Ok(initialized_source)
        }

        fn commit_and_push_changes(
            &self,
            source: InitializedSource,
            message: String,
        ) -> Result<(), SkootError> {
            if message == "error" {
                return Err("Error".into());
            }

            Ok(())
        }

        fn write_file<P: AsRef<std::path::Path>, C: AsRef<[u8]>>(
            &self,
            _source: InitializedSource,
            _path: P,
            name: String,
            _contents: C,
        ) -> Result<(), SkootError> {
            if name == "error" {
                return Err("Error".into());
            }

            Ok(())
        }

        fn read_file<P: AsRef<std::path::Path>>(
            &self,
            _source: &InitializedSource,
            _path: P,
            name: String,
        ) -> Result<String, SkootError> {
            if name == "error" {
                return Err("Error".into());
            }

            Ok("Worked".to_string())
        }
    }

    impl RootFacetService for MockFacetService {
        fn initialize(
            &self,
            params: FacetParams,
        ) -> impl std::future::Future<Output = Result<InitializedFacet, SkootError>> + Send
        {
            async {
                match params {
                    FacetParams::SourceFile(_) => Err("Error".into()),
                    FacetParams::SourceBundle(s) => {
                        if s.common.project_name == "error" {
                            return Err("Error".into());
                        }
                        let source_bundle_facet = SourceBundleFacet {
                            source_files: vec![SourceFileContent {
                                name: "README.md".to_string(),
                                path: "./".to_string(),
                                content: s.common.project_name.clone(),
                            }],
                            facet_type: SupportedFacetType::Readme,
                        };

                        Ok(InitializedFacet::SourceBundle(source_bundle_facet))
                    }
                    FacetParams::APIBundle(a) => {
                        if a.common.project_name == "error" {
                            return Err("Error".into());
                        }
                        let api_bundle_facet = APIBundleFacet {
                            apis: vec![APIContent {
                                name: "test".to_string(),
                                url: "https://foo.bar/test".to_string(),
                                response: "worked".to_string(),
                            }],
                            facet_type: SupportedFacetType::BranchProtection,
                        };

                        Ok(InitializedFacet::APIBundle(api_bundle_facet))
                    }
                }
            }
        }

        fn initialize_all(
            &self,
            params: FacetSetParams,
        ) -> impl std::future::Future<Output = Result<Vec<InitializedFacet>, SkootError>> + Send
        {
            async {
                let mut initialized_facets = Vec::new();
                for facet_params in params.facets_params {
                    let initialized_facet = self.initialize(facet_params).await?;
                    initialized_facets.push(initialized_facet);
                }

                Ok(initialized_facets)
            }
        }
    }

    #[tokio::test]
    async fn test_initialize_project() {
        let project_params = ProjectParams { 
            name: "test".to_string(), 
            repo_params: RepoParams::Github(GithubRepoParams { 
                name: "test".to_string(),
                description: "foobar".to_string(), 
                organization: GithubUser::User("testuser".to_string())
            }), 
            ecosystem_params: EcosystemParams::Go(GoParams { 
                name: "test".to_string(), 
                host: "github.com".to_string() 
            }),
            source_params: SourceParams { 
                parent_path: "test".to_string() 
            }
        };

        let local_project_service = LocalProjectService {
            repo_service: MockRepoService,
            ecosystem_service: MockEcosystemService,
            source_service: MockSourceService,
            facet_service: MockFacetService,
        };

        let result = local_project_service.initialize(project_params).await;

        assert!(result.is_ok());
        let initialized_project = result.unwrap();

        assert!(initialized_project.repo.full_url() == "https://github.com/testuser/test");
        let module = match initialized_project.ecosystem {
            InitializedEcosystem::Go(g) => g,
            _ => panic!("Wrong ecosystem type"),
        };
        assert!(module.name == "test");
        assert!(initialized_project.source.path == "test/test");
        // TODO: This just pulls in the default set of facets which has a length of 12.
        // This should be more configurable.
        assert_eq!(initialized_project.facets.len(), 12);
    }
}
