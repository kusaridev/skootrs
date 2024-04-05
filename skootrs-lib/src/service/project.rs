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

use std::collections::HashMap;

use crate::service::facet::{FacetSetParamsGenerator, RootFacetService};

use skootrs_model::skootrs::{
    facet::{CommonFacetCreateParams, InitializedFacet, SourceFile},
    FacetGetParams, FacetMapKey, InitializedProject, InitializedSource, ProjectCreateParams,
    ProjectGetParams, SkootError,
};

use super::{
    ecosystem::EcosystemService, output::OutputService, repo::RepoService, source::SourceService,
};
use tracing::{debug, error, info};

/// The `ProjectService` trait provides an interface for initializing and managing a Skootrs project.
pub trait ProjectService {
    /// Initializes a Skootrs project.
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be initialized for any reason.
    fn initialize(
        &self,
        params: ProjectCreateParams,
    ) -> impl std::future::Future<Output = Result<InitializedProject, SkootError>> + Send;

    /// Gets an initialized project.
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be found or fetched.
    fn get(
        &self,
        params: ProjectGetParams,
    ) -> impl std::future::Future<Output = Result<InitializedProject, SkootError>> + Send;

    /// Gets a facet along with its content from an initialized project.
    ///
    /// # Errors
    ///
    /// Returns an error if the facet can't be found or fetched.
    fn get_facet_with_content(
        &self,
        params: FacetGetParams,
    ) -> impl std::future::Future<Output = Result<InitializedFacet, SkootError>> + Send;

    /// Lists the facets of an initialized project.
    ///
    /// # Errors
    ///
    /// Returns an error if the list of facets can't be fetched.
    fn list_facets(
        &self,
        params: ProjectGetParams,
    ) -> impl std::future::Future<Output = Result<Vec<FacetMapKey>, SkootError>> + Send;
}

/// The `LocalProjectService` struct provides an implementation of the `ProjectService` trait for initializing
/// and managing a Skootrs project on the local machine.
#[derive(Debug)]
pub struct LocalProjectService<
    RS: RepoService,
    ES: EcosystemService,
    SS: SourceService,
    FS: RootFacetService,
    OS: OutputService,
> {
    pub repo_service: RS,
    pub ecosystem_service: ES,
    pub source_service: SS,
    pub facet_service: FS,
    pub output_service: OS,
}

impl<RS, ES, SS, FS, OS> ProjectService for LocalProjectService<RS, ES, SS, FS, OS>
where
    RS: RepoService + Send + Sync,
    ES: EcosystemService + Send + Sync,
    SS: SourceService + Send + Sync,
    FS: RootFacetService + Send + Sync,
    OS: OutputService + Send + Sync,
{
    async fn initialize(
        &self,
        params: ProjectCreateParams,
    ) -> Result<InitializedProject, SkootError> {
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
        let common_params = CommonFacetCreateParams {
            project_name: params.name.clone(),
            source: initialized_source.clone(),
            repo: initialized_repo.clone(),
            ecosystem: initialized_ecosystem.clone(),
        };
        let source_facet_set_params = facet_set_params_generator
            .generate_default_source_bundle_facet_params(&common_params)?;
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
        // FIXME: Also add facet by name as well
        let initialized_facets = [initialized_source_facets, initialized_api_facets]
            .concat()
            .into_iter()
            .map(|f| (FacetMapKey::Type(f.facet_type()), f))
            .collect::<HashMap<FacetMapKey, InitializedFacet>>();

        info!("Completed project initialization");

        Ok(InitializedProject {
            repo: initialized_repo,
            ecosystem: initialized_ecosystem,
            source: initialized_source,
            facets: initialized_facets,
        })
    }

    async fn get(&self, params: ProjectGetParams) -> Result<InitializedProject, SkootError> {
        let get_repo_params = skootrs_model::skootrs::InitializedRepoGetParams {
            repo_url: params.project_url.clone(),
        };
        debug!("Getting repo: {get_repo_params:?}");
        let repo = self.repo_service.get(get_repo_params).await?;
        // TODO: Skootrs file path should be kept as a global constant somewhere.
        let skootrs_file = self
            .repo_service
            .fetch_file_content(&repo, ".skootrs")
            .await?;
        debug!("Skootrs file: {skootrs_file}");
        let initialized_project: InitializedProject = serde_json::from_str(&skootrs_file)?;
        Ok(initialized_project)
    }

    async fn get_facet_with_content(
        &self,
        params: FacetGetParams,
    ) -> Result<InitializedFacet, SkootError> {
        let initialized_project = self.get(params.project_get_params.clone()).await?;
        let facet = initialized_project
            .facets
            .get(&params.facet_map_key)
            .ok_or(SkootError::from("Facet not found"))?;

        match facet {
            InitializedFacet::SourceBundle(s) => {
                if let Some(source_files) = s.source_files.clone() {
                    let source_files_content_futures = source_files.into_iter().map(|sf| async {
                        let path = std::path::Path::new(&sf.path).join(&sf.name);
                        // FIXME: This stripped path should probably be handled by the fetch facet content function
                        let stripped_path = path.strip_prefix("./").unwrap_or(&path);

                        let content = self
                            .repo_service
                            .fetch_file_content(&initialized_project.repo, stripped_path)
                            .await;
                        match content {
                            Ok(c) => Ok((sf, c)),
                            Err(e) => {
                                error!(
                                    "Error fetching file content for path: {stripped_path:#?}, error: {e}"
                                );
                                Err(e)
                            }
                        }
                    });
                    let source_files_content_results =
                        futures::future::join_all(source_files_content_futures)
                            .await
                            .into_iter()
                            .collect::<Result<Vec<(SourceFile, String)>, SkootError>>()?;
                    let source_files_content_map = source_files_content_results
                        .into_iter()
                        .collect::<HashMap<SourceFile, String>>();
                    Ok(InitializedFacet::SourceBundle(
                        skootrs_model::skootrs::facet::SourceBundleFacet {
                            facet_type: s.facet_type.clone(),
                            source_files: None,
                            source_files_content: Some(source_files_content_map),
                        },
                    ))
                } else {
                    Err(SkootError::from("No source files found"))
                }
            }
            InitializedFacet::APIBundle(a) => Ok(InitializedFacet::APIBundle(a.clone())),
            InitializedFacet::SourceFile(_) => Err(SkootError::from("Facet type not supported")),
        }
    }

    async fn list_facets(&self, params: ProjectGetParams) -> Result<Vec<FacetMapKey>, SkootError> {
        Ok(self.get(params).await?.facets.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use skootrs_model::skootrs::{
        facet::{
            APIBundleFacet, APIContent, FacetCreateParams, FacetSetCreateParams, SourceBundleFacet,
            SupportedFacetType,
        },
        EcosystemInitializeParams, GithubRepoParams, GithubUser, GoParams, InitializedEcosystem,
        InitializedGithubRepo, InitializedGo, InitializedMaven, InitializedRepo, RepoCreateParams,
        SourceInitializeParams,
    };

    use super::*;
    struct MockRepoService;
    struct MockEcosystemService;
    struct MockSourceService;
    struct MockFacetService;

    impl RepoService for MockRepoService {
        async fn initialize(
            &self,
            params: RepoCreateParams,
        ) -> Result<InitializedRepo, SkootError> {
            let RepoCreateParams::Github(inner_params) = params;

            // Special case for testing error handling
            if inner_params.name == "error" {
                return Err("Error".into());
            }

            let initialized_repo = InitializedRepo::Github(InitializedGithubRepo {
                name: inner_params.name,
                organization: inner_params.organization,
            });

            Ok(initialized_repo)
        }

        fn clone_local(
            &self,
            initialized_repo: InitializedRepo,
            path: String,
        ) -> Result<InitializedSource, SkootError> {
            let InitializedRepo::Github(inner_repo) = initialized_repo;

            if inner_repo.name == "error" {
                return Err("Error".into());
            }

            let initialized_source = InitializedSource {
                path: format!("{}/{}", path, inner_repo.name),
            };

            Ok(initialized_source)
        }

        fn clone_local_or_pull(
            &self,
            initialized_repo: InitializedRepo,
            path: String,
        ) -> Result<InitializedSource, SkootError> {
            self.clone_local(initialized_repo, path)
        }

        async fn get(
            &self,
            params: skootrs_model::skootrs::InitializedRepoGetParams,
        ) -> Result<InitializedRepo, SkootError> {
            let repo_url = params.repo_url.clone();
            if repo_url == "error" {
                return Err("Error".into());
            }

            let initialized_repo = InitializedRepo::Github(InitializedGithubRepo {
                name: "test".to_string(),
                organization: GithubUser::User("testuser".to_string()),
            });

            Ok(initialized_repo)
        }

        async fn fetch_file_content<P: AsRef<std::path::Path> + Send>(
            &self,
            _initialized_repo: &InitializedRepo,
            path: P,
        ) -> Result<String, SkootError> {
            if path.as_ref().to_str().unwrap() == "error" {
                return Err("Error".into());
            }

            Ok("Worked".to_string())
        }
    }

    impl EcosystemService for MockEcosystemService {
        fn initialize(
            &self,
            params: EcosystemInitializeParams,
            _source: InitializedSource,
        ) -> Result<InitializedEcosystem, SkootError> {
            let initialized_ecosystem = match params {
                EcosystemInitializeParams::Go(g) => {
                    if g.host == "error" {
                        return Err("Error".into());
                    }
                    InitializedEcosystem::Go(InitializedGo {
                        name: g.name,
                        host: g.host,
                    })
                }
                EcosystemInitializeParams::Maven(m) => {
                    if m.group_id == "error" {
                        return Err("Error".into());
                    }
                    InitializedEcosystem::Maven(InitializedMaven {
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
            params: skootrs_model::skootrs::SourceInitializeParams,
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
            _source: InitializedSource,
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

        fn hash_file<P: AsRef<Path>>(
            &self,
            _source: &InitializedSource,
            path: P,
            _name: String,
        ) -> Result<String, SkootError> {
            if path.as_ref().to_str().unwrap() == "error" {
                return Err("Error".into());
            }

            Ok("fakehash".to_string())
        }

        fn pull_updates(&self, source: InitializedSource) -> Result<(), SkootError> {
            if source.path == "error" {
                return Err("Error".into());
            }

            Ok(())
        }
    }

    impl RootFacetService for MockFacetService {
        async fn initialize(
            &self,
            params: FacetCreateParams,
        ) -> Result<InitializedFacet, SkootError> {
            match params {
                FacetCreateParams::SourceFile(_) => Err("Error".into()),
                FacetCreateParams::SourceBundle(s) => {
                    if s.common.project_name == "error" {
                        return Err("Error".into());
                    }
                    let source_bundle_facet = SourceBundleFacet {
                        source_files: Some(vec![SourceFile {
                            name: "README.md".to_string(),
                            path: "./".to_string(),
                            hash: "fakehash".to_string(),
                        }]),
                        facet_type: SupportedFacetType::Readme,
                        source_files_content: None,
                    };

                    Ok(InitializedFacet::SourceBundle(source_bundle_facet))
                }
                FacetCreateParams::APIBundle(a) => {
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

        async fn initialize_all(
            &self,
            params: FacetSetCreateParams,
        ) -> Result<Vec<InitializedFacet>, SkootError> {
            let mut initialized_facets = Vec::new();
            for facet_params in params.facets_params {
                let initialized_facet = self.initialize(facet_params).await?;
                initialized_facets.push(initialized_facet);
            }

            Ok(initialized_facets)
        }
    }

    #[tokio::test]
    async fn test_initialize_project() {
        let project_params = ProjectCreateParams {
            name: "test".to_string(),
            repo_params: RepoCreateParams::Github(GithubRepoParams {
                name: "test".to_string(),
                description: "foobar".to_string(),
                organization: GithubUser::User("testuser".to_string()),
            }),
            ecosystem_params: EcosystemInitializeParams::Go(GoParams {
                name: "test".to_string(),
                host: "github.com".to_string(),
            }),
            source_params: SourceInitializeParams {
                parent_path: "test".to_string(),
            },
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
        println!("{:#?}", initialized_project.facets);

        // TODO: This will always be equal to 2 because we are initializing two facets in the mock facet service
        // and the `HashMap` for the facets will keep getting the same key. This is probably not a great way
        // of handling that.
        assert_eq!(initialized_project.facets.len(), 2);
    }
}
