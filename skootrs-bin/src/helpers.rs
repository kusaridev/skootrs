use inquire::Text;
use octocrab::Page;
use serde::Serialize;
use skootrs_lib::service::{project::ProjectService, source::LocalSourceService};
use skootrs_model::skootrs::{
    facet::InitializedFacet, Config, EcosystemInitializeParams, FacetGetParams, FacetMapKey,
    GithubRepoParams, GithubUser, GoParams, InitializedProject, MavenParams, ProjectArchiveParams,
    ProjectCreateParams, ProjectGetParams, ProjectOutput, ProjectOutputGetParams,
    ProjectOutputReference, ProjectOutputType, ProjectOutputsListParams, ProjectReleaseParam,
    RepoCreateParams, SkootError, SourceInitializeParams, SupportedEcosystems,
};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    str::FromStr,
};
use strum::VariantNames;
use tracing::debug;

use skootrs_statestore::{
    GitProjectStateStore, InMemoryProjectReferenceCache, ProjectReferenceCache, ProjectStateStore,
};

/// Helper trait that lets me inline writing the result of a Skootrs function to a writer.
pub trait HandleResponseOutput<T> {
    #[must_use]
    fn handle_response_output<W: Write>(self, output_handler: W) -> Self;
}

impl<T> HandleResponseOutput<T> for Result<T, SkootError>
where
    T: Serialize,
{
    /// Handles a response that implements `Serialize`.
    /// This is useful for functions that return a response that needs to be printed out, logged, etc. to the user.
    ///
    /// # Errors
    ///
    /// Returns an error if the response can't be serialized to JSON or if the output can't be written to the output handler.
    /// Also returns an error if the function that returns the response returns an error.
    fn handle_response_output<W: Write>(self, mut output_handler: W) -> Self {
        match self {
            Ok(result) => {
                let serialized_result = serde_json::to_string_pretty(&result)?;
                writeln!(output_handler, "{serialized_result}")?;
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }
}

pub struct Project;

impl Project {
    /// Returns `Ok(())` if the project creation is successful, otherwise returns an error.
    ///
    /// Creates a new skootrs project by prompting the user for repository details and language selection.
    /// The project can be created for either Go or Maven ecosystems right now.
    /// The project is created in Github, cloned down, and then initialized along with any other security supporting
    /// tasks. If the `project_params` is not provided, the user will be prompted for the project details.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is not authenticated with Github, or if the project can't be created
    /// for any other reason.
    pub async fn create<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        project_params: Option<ProjectCreateParams>,
    ) -> Result<InitializedProject, SkootError> {
        let project_params = match project_params {
            Some(p) => p,
            None => Project::prompt_create(config).await?,
        };

        let project = project_service.initialize(project_params).await?;
        let git_state_store = GitProjectStateStore {
            source: project.source.clone(),
            source_service: LocalSourceService {},
        };

        let mut local_cache = InMemoryProjectReferenceCache::load_or_create("./skootcache")?;
        git_state_store.create(project.clone()).await?;
        local_cache.set(project.repo.full_url()).await?;
        Ok(project)
    }

    async fn prompt_create(config: &Config) -> Result<ProjectCreateParams, SkootError> {
        let name = Text::new("The name of the repository").prompt()?;
        let description = Text::new("The description of the repository").prompt()?;
        let user = octocrab::instance().current().user().await?.login;
        let Page { items, .. } = octocrab::instance()
            .current()
            .list_org_memberships_for_authenticated_user()
            .send()
            .await?;
        let organization = inquire::Select::new(
            "Select an organization",
            items
                .iter()
                .map(|i| i.organization.login.as_str())
                .chain(vec![user.as_str()])
                .collect(),
        )
        .prompt()?;
        let language =
            inquire::Select::new("Select a language", SupportedEcosystems::VARIANTS.to_vec());

        let gh_org = match organization {
            x if x == user => GithubUser::User(x.to_string()),
            x => GithubUser::Organization(x.to_string()),
        };

        let language_prompt = language.prompt()?;
        let ecosystem_params = match SupportedEcosystems::from_str(language_prompt)? {
            SupportedEcosystems::Go => EcosystemInitializeParams::Go(GoParams {
                name: name.clone(),
                host: format!("github.com/{organization}"),
            }),
            // TODO: Unclear if this is the right way to handle Maven group and artifact.
            SupportedEcosystems::Maven => EcosystemInitializeParams::Maven(MavenParams {
                group_id: format!("com.{organization}.{name}"),
                artifact_id: name.clone(),
            }),
        };

        let repo_params = RepoCreateParams::Github(GithubRepoParams {
            name: name.clone(),
            description,
            organization: gh_org,
        });

        Ok(ProjectCreateParams {
            name: name.clone(),
            repo_params,
            ecosystem_params,
            source_params: SourceInitializeParams {
                parent_path: config.local_project_path.clone(),
            },
        })
    }

    /// Fetches the contents of an `InitializedProject` along with an interactive prompt.
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be fetched for some reason.
    pub async fn get<'a, T: ProjectService + ?Sized>(
        config: &Config,
        _project_service: &'a T,
        project_get_params: Option<ProjectGetParams>,
    ) -> Result<InitializedProject, SkootError> {
        let mut cache = InMemoryProjectReferenceCache::load_or_create("./skootcache")?;
        let project_get_params = match project_get_params {
            Some(p) => p,
            None => Project::prompt_get(config).await?,
        };
        let project = cache.get(project_get_params.project_url.clone()).await?;
        Ok(project)
    }

    async fn prompt_get(config: &Config) -> Result<ProjectGetParams, SkootError> {
        let projects = Project::list(config).await?;
        let selected_project =
            inquire::Select::new("Select a project", projects.iter().collect()).prompt()?;
        Ok(ProjectGetParams {
            project_url: selected_project.clone(),
        })
    }

    /// Returns the list of projects that are stored in the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache can't be loaded or if the list of projects can't be fetched.
    pub async fn list(_config: &Config) -> Result<HashSet<String>, SkootError> {
        let cache = InMemoryProjectReferenceCache::load_or_create("./skootcache")?;
        let projects: HashSet<String> = cache.list().await?;
        Ok(projects)
    }

    /// Archives a project by archiving the repository and removing it from the local cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the project can't be archived or deleted from the cache.
    pub async fn archive<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        project_archive_params: Option<ProjectArchiveParams>,
    ) -> Result<(), SkootError> {
        let project_archive_params = match project_archive_params {
            Some(p) => p,
            None => ProjectArchiveParams {
                initialized_project: Project::get(config, project_service, None).await?,
            },
        };
        let url = project_archive_params.initialized_project.repo.full_url();
        project_service.archive(project_archive_params).await?;
        let mut local_cache = InMemoryProjectReferenceCache::load_or_create("./skootcache")?;
        local_cache.delete(url).await?;
        local_cache.save()?;
        Ok(())
    }
}

pub struct Facet;

impl Facet {
    /// Returns the contents of a facet. This includes things like source files or API bundles.
    ///
    /// # Errors
    ///
    /// Returns an error if the facet content or project can't be fetched for some reason.
    pub async fn get<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        facet_get_params: Option<FacetGetParams>,
    ) -> Result<InitializedFacet, SkootError> {
        let facet_get_params = if let Some(p) = facet_get_params {
            p
        } else {
            // let project = Project::get(config, project_service, None).await?;
            let project_get_params = Project::prompt_get(config).await?;
            let facet_map_keys = project_service
                .list_facets(project_get_params.clone())
                .await?;
            let fmk = Facet::prompt_get(config, facet_map_keys.into_iter().collect())?;
            FacetGetParams {
                facet_map_key: fmk,
                project_get_params,
            }
        };

        let facet_with_content = project_service
            .get_facet_with_content(facet_get_params)
            .await?;

        debug!("{:?}", facet_with_content);

        Ok(facet_with_content)
    }

    fn prompt_get(
        _config: &Config,
        facet_map_keys: Vec<FacetMapKey>,
    ) -> Result<FacetMapKey, SkootError> {
        let facet_type = inquire::Select::new("Select a facet", facet_map_keys).prompt()?;

        Ok(facet_type)
    }

    /// Returns the list of facets for a project. This includes things like source files or API bundles.
    ///
    /// # Errors
    ///
    /// Returns an error if the project or list of facets can't be fetched for some reason.
    pub async fn list<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        project_get_params: Option<ProjectGetParams>,
    ) -> Result<Vec<FacetMapKey>, SkootError> {
        let project_get_params = match project_get_params {
            Some(p) => p,
            None => Project::prompt_get(config).await?,
        };
        let facet_map_keys = project_service.list_facets(project_get_params).await?;
        Ok(facet_map_keys)
    }
}

pub struct Output;

impl Output {
    /// Returns the content of a project output. This includes things like SBOMs or SLSA attestations.
    ///
    /// # Errors
    ///
    /// Returns an error if the project output can't be fetched from a project release.
    pub async fn get<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        project_output_params: Option<ProjectOutputGetParams>,
    ) -> Result<ProjectOutput, SkootError> {
        let project_output_params = match project_output_params {
            Some(p) => p,
            None => Output::prompt_output_get(config, project_service).await?,
        };

        let output = project_service.output_get(project_output_params).await?;

        Ok(output)
    }

    /// Returns the list of project outputs for a project. This includes things like SBOMs or SLSA attestations.
    ///
    /// # Errors
    ///
    /// Returns an error if the project output list can't be fetched.
    pub async fn list<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
        project_outputs_list_params: Option<ProjectOutputsListParams>,
    ) -> Result<Vec<ProjectOutputReference>, SkootError> {
        let project_outputs_list_params = match project_outputs_list_params {
            Some(p) => p,
            None => ProjectOutputsListParams {
                initialized_project: Project::get(config, project_service, None).await?,
                release: ProjectReleaseParam::Latest,
            },
        };
        let output_list = project_service
            .outputs_list(project_outputs_list_params)
            .await?;
        Ok(output_list)
    }

    async fn prompt_output_get<'a, T: ProjectService + ?Sized>(
        config: &Config,
        project_service: &'a T,
    ) -> Result<ProjectOutputGetParams, SkootError> {
        let selected_project = Project::get(config, project_service, None).await?;
        let project_output_list_params = ProjectOutputsListParams {
            initialized_project: selected_project.clone(),
            // TODO: This should be a prompt.
            release: ProjectReleaseParam::Latest,
        };
        let output_list =
            Output::list(config, project_service, Some(project_output_list_params)).await?;
        let type_output_map: HashMap<String, Vec<String>> = output_list
            .iter()
            .map(|o| (o.output_type.to_string(), o.name.clone()))
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<String>>, (key, value)| {
                    acc.entry(key).or_default().push(value);
                    acc
                },
            );
        let selected_output_type = inquire::Select::new(
            "Select an output type",
            type_output_map.keys().cloned().collect(),
        )
        .prompt()?;
        let select_output_type_enum = ProjectOutputType::from_str(&selected_output_type)?;
        let selected_output = inquire::Select::new(
            "Select an output",
            type_output_map
                .get(&selected_output_type)
                .ok_or_else(|| SkootError::from("Failed to get output type"))?
                .clone(),
        )
        .prompt()?;
        Ok(ProjectOutputGetParams {
            initialized_project: selected_project.clone(),
            project_output_type: select_output_type_enum,
            project_output: selected_output.clone(),
            // TODO: This should be selectable
            release: ProjectReleaseParam::Latest,
        })
    }
}
