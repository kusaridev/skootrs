use base64::prelude::*;
use inquire::Text;
use octocrab::Page;
use skootrs_lib::service::{
    ecosystem::LocalEcosystemService,
    facet::LocalFacetService,
    project::{LocalProjectService, ProjectService},
    repo::LocalRepoService,
    source::{LocalSourceService, SourceService},
};
use skootrs_model::{
    security_insights::insights10::SecurityInsightsVersion100YamlSchema,
    skootrs::{
        EcosystemParams, GithubRepoParams, GithubUser, GoParams, InitializedProject, MavenParams,
        ProjectParams, RepoParams, SkootError, SkootrsConfig, SourceParams, SUPPORTED_ECOSYSTEMS,
    },
};
use std::collections::HashMap;

use skootrs_model::skootrs::facet::InitializedFacet;
use skootrs_statestore::SurrealProjectStateStore;

pub struct Project;

impl Project {
    /// Returns `Ok(())` if the project creation is successful, otherwise returns an error.
    ///
    /// Creates a new skootrs project by prompting the user for repository details and language selection.
    /// The project can be created for either Go or Maven ecosystems right now.
    /// The project is created in Github, cloned down, and then initialized along with any other security supporting
    /// tasks. If the project_params is not provided, the user will be prompted for the project details.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is not authenticated with Github, or if the project can't be created
    /// for any other reason.
    pub async fn create<T: ProjectService>(
        config: &SkootrsConfig,
        project_service: T,
        project_params: Option<ProjectParams>,
    ) -> Result<(), SkootError> {
        let project_params = match project_params {
            Some(p) => p,
            None => Project::prompt_project(config).await?,
        };

        project_service.initialize(project_params).await?;
        Ok(())
    }

    async fn prompt_project(config: &SkootrsConfig) -> Result<ProjectParams, SkootError> {
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
        let language = inquire::Select::new("Select a language", SUPPORTED_ECOSYSTEMS.to_vec());

        let gh_org = match organization {
            x if x == user => GithubUser::User(x.to_string()),
            x => GithubUser::Organization(x.to_string()),
        };

        let repo_params = match language.prompt()? {
            "Go" => RepoParams::Github(GithubRepoParams {
                name: name.clone(),
                description,
                organization: gh_org,
            }),
            "Maven" => RepoParams::Github(GithubRepoParams {
                name: name.clone(),
                description,
                organization: gh_org,
            }),
            _ => {
                unreachable!("Unsupported language")
            }
        };

        Ok(ProjectParams {
            name: name.clone(),
            repo_params,
            ecosystem_params: EcosystemParams::Go(GoParams {
                name: name.clone(),
                host: format!("github.com/{organization}"),
            }),
            source_params: SourceParams {
                parent_path: config.local_project_path.clone(),
            },
        })
    }
}

/// Returns `Ok(())` if the project creation is successful, otherwise returns an error.
///
/// Creates a new skootrs project by prompting the user for repository details and language selection.
/// The project can be created for either Go or Maven ecosystems right now.
/// The project is created in Github, cloned down, and then initialized along with any other security supporting
/// tasks.
///
/// # Errors
///
/// Returns an error if the user is not authenticated with Github, or if the project can't be created
/// for any other reason.
pub async fn create() -> std::result::Result<(), SkootError> {
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

    let language = inquire::Select::new("Select a language", SUPPORTED_ECOSYSTEMS.to_vec());

    let gh_org = match organization {
        x if x == user => GithubUser::User(x.to_string()),
        x => GithubUser::Organization(x.to_string()),
    };

    let initialized_project: InitializedProject = match language.prompt()? {
        "Go" => {
            // TODO: support more than just github
            let go_params = GoParams {
                name: name.clone(),
                host: format!("github.com/{organization}"),
            };
            let project_params = ProjectParams {
                name: name.clone(),
                repo_params: RepoParams::Github(GithubRepoParams {
                    name,
                    description,
                    organization: gh_org,
                }),
                ecosystem_params: EcosystemParams::Go(go_params),
                source_params: SourceParams {
                    parent_path: "/tmp".to_string(), // FIXME: This should be configurable
                },
            };
            let local_project_service = LocalProjectService {
                repo_service: LocalRepoService {},
                ecosystem_service: LocalEcosystemService {},
                source_service: LocalSourceService {},
                facet_service: LocalFacetService {},
            };

            local_project_service.initialize(project_params).await?
        }

        "Maven" => {
            let maven_params = MavenParams {
                group_id: format!("com.{organization}.{name}"),
                artifact_id: name.clone(),
            };

            let project_params = ProjectParams {
                name: name.clone(),
                repo_params: RepoParams::Github(GithubRepoParams {
                    name,
                    description,
                    organization: gh_org,
                }),
                ecosystem_params: EcosystemParams::Maven(maven_params),
                source_params: SourceParams {
                    parent_path: "/tmp".to_string(), // FIXME: This should be configurable
                },
            };
            let local_project_service = LocalProjectService {
                repo_service: LocalRepoService {},
                ecosystem_service: LocalEcosystemService {},
                source_service: LocalSourceService {},
                facet_service: LocalFacetService {},
            };

            local_project_service.initialize(project_params).await?
        }

        _ => {
            unreachable!("Unsupported language")
        }
    };

    let state_store = SurrealProjectStateStore::new().await?;
    state_store.create(initialized_project).await?;

    Ok(())
}

/// Returns `Ok(())` if the able to print out the content of the facet, otherwise returns an error.
///
/// This function prompts the user to select a project and then a facet of that project to fetch from the state store.
/// It then prints out the content of the facet.
///
/// # Errors
///
/// Returns an error if the state store is not able to be accessed or if the selected project or facet
/// is not found.
pub async fn get_facet() -> std::result::Result<(), SkootError> {
    let project = prompt_project().await?;

    let facet_to_project: HashMap<String, InitializedFacet> = project
        .facets
        .iter()
        .map(|f| match f {
            InitializedFacet::SourceFile(f) => (
                f.facet_type.to_string(),
                InitializedFacet::SourceFile(f.clone()),
            ),
            InitializedFacet::SourceBundle(f) => (
                f.facet_type.to_string(),
                InitializedFacet::SourceBundle(f.clone()),
            ),
            InitializedFacet::APIBundle(f) => (
                f.facet_type.to_string(),
                InitializedFacet::APIBundle(f.clone()),
            ),
        })
        .collect::<HashMap<_, _>>();

    let selected_facet = inquire::Select::new(
        "Select a facet",
        facet_to_project.keys().collect::<Vec<_>>(),
    )
    .prompt()?;

    let facet = facet_to_project
        .get(selected_facet)
        .ok_or_else(|| SkootError::from("Failed to get selected facet"))?;

    let facet_content = get_facet_content(facet, &project)?;

    println!("{facet_content}");

    Ok(())
}

/// Returns `Ok(())` if the able to print out a dump of the statestore.
///
/// This function prints out the content of the state store in a pretty printed JSON format.
/// # Errors
///
/// Returns an error if the state store is not able to be accessed.
pub async fn dump() -> std::result::Result<(), SkootError> {
    let projects = get_all().await?;
    println!("{}", serde_json::to_string_pretty(&projects)?);
    Ok(())
}

async fn get_all() -> std::result::Result<Vec<InitializedProject>, SkootError> {
    let state_store = SurrealProjectStateStore::new().await?;
    let projects = state_store.select_all().await?;
    Ok(projects)
}

fn get_facet_content(
    facet: &InitializedFacet,
    project: &InitializedProject,
) -> std::result::Result<String, SkootError> {
    match facet {
        InitializedFacet::SourceFile(f) => {
            let source_service = LocalSourceService {};
            let content = source_service.read_file(&project.source, &f.path, f.name.clone())?;
            Ok(content)
        }
        InitializedFacet::SourceBundle(f) => {
            let source_service = LocalSourceService {};
            let content = f
                .source_files
                .iter()
                .map(|f| source_service.read_file(&project.source, &f.path, f.name.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(content.join("\n"))
        }
        InitializedFacet::APIBundle(f) => {
            // TODO: This can make it unclear which API was used
            let content = f.apis.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>();
            Ok(content.join("\n"))
        }
    }
}

/// This function is for prompting the user to get outputs from a Skootrs project
///
/// # Errors
///
/// This return an error if it can't get an intended output for some reason. This
/// includes issues like unable to get or parse SECURITY-INSIGHTS.yml or unable
/// to get the intended output.
pub async fn get_output() -> std::result::Result<(), SkootError> {
    let project = prompt_project().await?;

    let skootrs_model::skootrs::InitializedRepo::Github(repo) = &project.repo;

    let sec_ins_content_items = octocrab::instance()
        .repos(repo.organization.get_name(), &repo.name)
        .get_content()
        .path("SECURITY-INSIGHTS.yml")
        .r#ref("main")
        .send()
        .await?;

    let sec_ins = sec_ins_content_items
        .items
        .first()
        .ok_or_else(|| SkootError::from("Failed to get security insights"))?;

    let content = sec_ins
        .content
        .as_ref()
        .ok_or_else(|| SkootError::from("Failed to get content of  security insights"))?;
    let content_decoded =
        base64::engine::general_purpose::STANDARD.decode(content.replace('\n', ""))?;
    let content_str = std::str::from_utf8(&content_decoded)?;
    let insights: SecurityInsightsVersion100YamlSchema =
        serde_yaml::from_str::<SecurityInsightsVersion100YamlSchema>(content_str)?;
    let sbom_vec = insights
        .dependencies
        .ok_or_else(|| SkootError::from("Failed to get dependencies value from security insights"))?
        .sbom
        .ok_or_else(|| SkootError::from("Failed to get sbom value from security insights"))?;

    let sbom_url = inquire::Select::new(
        "Select an SBOM",
        sbom_vec.iter().flat_map(|s| &s.sbom_file).collect(),
    )
    .prompt()?;

    let sbom = reqwest::get(sbom_url).await?.text().await?;

    println!("{sbom}");

    Ok(())
}

async fn prompt_project() -> Result<InitializedProject, SkootError> {
    let projects = get_all().await?;
    let repo_to_project: HashMap<String, &InitializedProject> = projects
        .iter()
        .map(|p| (p.repo.full_url(), p))
        .collect::<HashMap<_, _>>();
    let selected_project = inquire::Select::new(
        "Select a project",
        repo_to_project.keys().collect::<Vec<_>>(),
    )
    .prompt()?;

    let project = *repo_to_project
        .get(selected_project)
        .ok_or_else(|| SkootError::from("Failed to get selected project"))?;

    Ok(project.clone())
}
