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

use std::error::Error;

use chrono::Datelike;
use tracing::info;

use crate::model::{
    security_insights::insights10::{
        SecurityInsightsVersion100YamlSchema,
        SecurityInsightsVersion100YamlSchemaContributionPolicy,
        SecurityInsightsVersion100YamlSchemaHeader,
        SecurityInsightsVersion100YamlSchemaHeaderSchemaVersion,
        SecurityInsightsVersion100YamlSchemaProjectLifecycle,
        SecurityInsightsVersion100YamlSchemaProjectLifecycleStatus,
        SecurityInsightsVersion100YamlSchemaVulnerabilityReporting,
    },
    skootrs::{
        facet::{
            CommonFacetParams, FacetSetParams, FacetParams, InitializedFacet, SourceBundleFacet, SourceBundleFacetParams, SourceFileContent, SourceFileFacet, SourceFileFacetParams, SupportedFacetType
        },
        InitializedEcosystem,
    },
};
use crate::service::source::SourceService;

use super::source::LocalSourceService;

#[derive(Debug)]
pub struct LocalFacetService {}

pub trait RootFacetService {
    fn initialize(&self, params: FacetParams) -> Result<InitializedFacet, Box<dyn Error>>;
    fn initialize_all(
        &self,
        params: FacetSetParams,
    ) -> Result<Vec<InitializedFacet>, Box<dyn Error>> {
        params
            .facets_params
            .iter()
            .map(|params| self.initialize(params.clone()))
            .collect::<Result<Vec<InitializedFacet>, Box<dyn Error>>>()
    }
}

pub trait SourceFileFacetService {
    fn initialize(&self, params: SourceFileFacetParams) -> Result<SourceFileFacet, Box<dyn Error>>;
}

pub trait SourceBundleFacetService {
    fn initialize(
        &self,
        params: SourceBundleFacetParams,
    ) -> Result<SourceBundleFacet, Box<dyn Error>>;
}

impl SourceBundleFacetService for LocalFacetService {
    fn initialize(
        &self,
        params: SourceBundleFacetParams,
    ) -> Result<SourceBundleFacet, Box<dyn Error>> {
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
            },
            SupportedFacetType::Gitignore
            | SupportedFacetType::SLSABuild
            | SupportedFacetType::DependencyUpdateTool => {
                language_specific_source_bundle_content_handler.generate_content(&params)?
            },
            SupportedFacetType::SBOMGenerator => todo!(),
            SupportedFacetType::StaticCodeAnalysis => todo!(),
            SupportedFacetType::BranchProtection => todo!(),
            SupportedFacetType::CodeReview => todo!(),
            SupportedFacetType::Fuzzing => todo!(),
            SupportedFacetType::PublishPackages => todo!(),
            SupportedFacetType::PinnedDependencies => todo!(),
            SupportedFacetType::SAST => todo!(),
            SupportedFacetType::VulnerabilityScanner => todo!(),
            SupportedFacetType::GUACForwardingConfig => todo!(),
            SupportedFacetType::Allstar => todo!(),
        };

        for source_file_content in source_bundle_content.source_files_content.iter() {
            info!("Writing file {} to {}", source_file_content.name, source_file_content.path);
            source_service.write_file(
                params.common.source.clone(),
                source_file_content.path.clone(),
                source_file_content.name.clone(),
                source_file_content.content.clone(),
            )?;
        }

        let source_bundle_facet = SourceBundleFacet {
            source_files: source_bundle_content.source_files_content,
            facet_type: params.facet_type,
        };

        Ok(source_bundle_facet)
    }
}

pub struct SourceBundleContent {
    pub source_files_content: Vec<SourceFileContent>,
    pub facet_type: SupportedFacetType,
}

impl RootFacetService for LocalFacetService {
    fn initialize(&self, params: FacetParams) -> Result<InitializedFacet, Box<dyn Error>> {
        match params {
            FacetParams::SourceFile(_params) => {
                todo!("This has been removed in favor of SourceBundle")
                /*let source_file_facet = SourceFileFacetService::initialize(self, params)?;
                Ok(InitializedFacet::SourceFile(source_file_facet))*/
            }
            FacetParams::SourceBundle(params) => {
                let source_bundle_facet = SourceBundleFacetService::initialize(self, params)?;
                Ok(InitializedFacet::SourceBundle(source_bundle_facet))
            }
        }
    }
}

trait SourceBundleContentGenerator {
    fn generate_content(
        &self,
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>>;
}

/// Handles the generation of source files content that are generic to all projects by default,
/// e.g. README.md, LICENSE, etc.
struct DefaultSourceBundleContentHandler {}
impl SourceBundleContentGenerator for DefaultSourceBundleContentHandler {
    fn generate_content(
        &self,
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        match params.facet_type {
            SupportedFacetType::Readme => self.generate_readme_content(params),
            SupportedFacetType::License => self.generate_license_content(params),
            SupportedFacetType::SecurityPolicy => self.generate_security_policy_content(params),
            SupportedFacetType::Scorecard => self.generate_scorecard_content(params),
            SupportedFacetType::SecurityInsights => self.generate_security_insights_content(params),
            _ => todo!("Not implemented yet"),
        }
    }
}
impl DefaultSourceBundleContentHandler {
    fn generate_readme_content(
        &self,
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = format!(
            r#"# {}
This is the README for the {} project."#,
            params.common.project_name, params.common.project_name
        );

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
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = format!(
            r#"
            Apache License
            Version 2.0, January 2004
         http://www.apache.org/licenses/
    
    TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION
    
    1. Definitions.
    
    "License" shall mean the terms and conditions for use, reproduction,
    and distribution as defined by Sections 1 through 9 of this document.
    
    "Licensor" shall mean the copyright owner or entity authorized by
    the copyright owner that is granting the License.
    
    "Legal Entity" shall mean the union of the acting entity and all
    other entities that control, are controlled by, or are under common
    control with that entity. For the purposes of this definition,
    "control" means (i) the power, direct or indirect, to cause the
    direction or management of such entity, whether by contract or
    otherwise, or (ii) ownership of fifty percent (50%) or more of the
    outstanding shares, or (iii) beneficial ownership of such entity.
    
    "You" (or "Your") shall mean an individual or Legal Entity
    exercising permissions granted by this License.
    
    "Source" form shall mean the preferred form for making modifications,
    including but not limited to software source code, documentation
    source, and configuration files.
    
    "Object" form shall mean any form resulting from mechanical
    transformation or translation of a Source form, including but
    not limited to compiled object code, generated documentation,
    and conversions to other media types.
    
    "Work" shall mean the work of authorship, whether in Source or
    Object form, made available under the License, as indicated by a
    copyright notice that is included in or attached to the work
    (an example is provided in the Appendix below).
    
    "Derivative Works" shall mean any work, whether in Source or Object
    form, that is based on (or derived from) the Work and for which the
    editorial revisions, annotations, elaborations, or other modifications
    represent, as a whole, an original work of authorship. For the purposes
    of this License, Derivative Works shall not include works that remain
    separable from, or merely link (or bind by name) to the interfaces of,
    the Work and Derivative Works thereof.
    
    "Contribution" shall mean any work of authorship, including
    the original version of the Work and any modifications or additions
    to that Work or Derivative Works thereof, that is intentionally
    submitted to Licensor for inclusion in the Work by the copyright owner
    or by an individual or Legal Entity authorized to submit on behalf of
    the copyright owner. For the purposes of this definition, "submitted"
    means any form of electronic, verbal, or written communication sent
    to the Licensor or its representatives, including but not limited to
    communication on electronic mailing lists, source code control systems,
    and issue tracking systems that are managed by, or on behalf of, the
    Licensor for the purpose of discussing and improving the Work, but
    excluding communication that is conspicuously marked or otherwise
    designated in writing by the copyright owner as "Not a Contribution."
    
    "Contributor" shall mean Licensor and any individual or Legal Entity
    on behalf of whom a Contribution has been received by Licensor and
    subsequently incorporated within the Work.
    
    2. Grant of Copyright License. Subject to the terms and conditions of
    this License, each Contributor hereby grants to You a perpetual,
    worldwide, non-exclusive, no-charge, royalty-free, irrevocable
    copyright license to reproduce, prepare Derivative Works of,
    publicly display, publicly perform, sublicense, and distribute the
    Work and such Derivative Works in Source or Object form.
    
    3. Grant of Patent License. Subject to the terms and conditions of
    this License, each Contributor hereby grants to You a perpetual,
    worldwide, non-exclusive, no-charge, royalty-free, irrevocable
    (except as stated in this section) patent license to make, have made,
    use, offer to sell, sell, import, and otherwise transfer the Work,
    where such license applies only to those patent claims licensable
    by such Contributor that are necessarily infringed by their
    Contribution(s) alone or by combination of their Contribution(s)
    with the Work to which such Contribution(s) was submitted. If You
    institute patent litigation against any entity (including a
    cross-claim or counterclaim in a lawsuit) alleging that the Work
    or a Contribution incorporated within the Work constitutes direct
    or contributory patent infringement, then any patent licenses
    granted to You under this License for that Work shall terminate
    as of the date such litigation is filed.
    
    4. Redistribution. You may reproduce and distribute copies of the
    Work or Derivative Works thereof in any medium, with or without
    modifications, and in Source or Object form, provided that You
    meet the following conditions:
    
    (a) You must give any other recipients of the Work or
    Derivative Works a copy of this License; and
    
    (b) You must cause any modified files to carry prominent notices
    stating that You changed the files; and
    
    (c) You must retain, in the Source form of any Derivative Works
    that You distribute, all copyright, patent, trademark, and
    attribution notices from the Source form of the Work,
    excluding those notices that do not pertain to any part of
    the Derivative Works; and
    
    (d) If the Work includes a "NOTICE" text file as part of its
    distribution, then any Derivative Works that You distribute must
    include a readable copy of the attribution notices contained
    within such NOTICE file, excluding those notices that do not
    pertain to any part of the Derivative Works, in at least one
    of the following places: within a NOTICE text file distributed
    as part of the Derivative Works; within the Source form or
    documentation, if provided along with the Derivative Works; or,
    within a display generated by the Derivative Works, if and
    wherever such third-party notices normally appear. The contents
    of the NOTICE file are for informational purposes only and
    do not modify the License. You may add Your own attribution
    notices within Derivative Works that You distribute, alongside
    or as an addendum to the NOTICE text from the Work, provided
    that such additional attribution notices cannot be construed
    as modifying the License.
    
    You may add Your own copyright statement to Your modifications and
    may provide additional or different license terms and conditions
    for use, reproduction, or distribution of Your modifications, or
    for any such Derivative Works as a whole, provided Your use,
    reproduction, and distribution of the Work otherwise complies with
    the conditions stated in this License.
    
    5. Submission of Contributions. Unless You explicitly state otherwise,
    any Contribution intentionally submitted for inclusion in the Work
    by You to the Licensor shall be under the terms and conditions of
    this License, without any additional terms or conditions.
    Notwithstanding the above, nothing herein shall supersede or modify
    the terms of any separate license agreement you may have executed
    with Licensor regarding such Contributions.
    
    6. Trademarks. This License does not grant permission to use the trade
    names, trademarks, service marks, or product names of the Licensor,
    except as required for reasonable and customary use in describing the
    origin of the Work and reproducing the content of the NOTICE file.
    
    7. Disclaimer of Warranty. Unless required by applicable law or
    agreed to in writing, Licensor provides the Work (and each
    Contributor provides its Contributions) on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
    implied, including, without limitation, any warranties or conditions
    of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
    PARTICULAR PURPOSE. You are solely responsible for determining the
    appropriateness of using or redistributing the Work and assume any
    risks associated with Your exercise of permissions under this License.
    
    8. Limitation of Liability. In no event and under no legal theory,
    whether in tort (including negligence), contract, or otherwise,
    unless required by applicable law (such as deliberate and grossly
    negligent acts) or agreed to in writing, shall any Contributor be
    liable to You for damages, including any direct, indirect, special,
    incidental, or consequential damages of any character arising as a
    result of this License or out of the use or inability to use the
    Work (including but not limited to damages for loss of goodwill,
    work stoppage, computer failure or malfunction, or any and all
    other commercial damages or losses), even if such Contributor
    has been advised of the possibility of such damages.
    
    9. Accepting Warranty or Additional Liability. While redistributing
    the Work or Derivative Works thereof, You may choose to offer,
    and charge a fee for, acceptance of support, warranty, indemnity,
    or other liability obligations and/or rights consistent with this
    License. However, in accepting such obligations, You may act only
    on Your own behalf and on Your sole responsibility, not on behalf
    of any other Contributor, and only if You agree to indemnify,
    defend, and hold each Contributor harmless for any liability
    incurred by, or claims asserted against, such Contributor by reason
    of your accepting any such warranty or additional liability.
    
    END OF TERMS AND CONDITIONS
    
    APPENDIX: How to apply the Apache License to your work.
    
    To apply the Apache License to your work, attach the following
    boilerplate notice, with the fields enclosed by brackets "[]"
    replaced with your own identifying information. (Don't include
    the brackets!)  The text should be enclosed in the appropriate
    comment syntax for the file format. We also recommend that a
    file or class name and description of purpose be included on the
    same "printed page" as the copyright notice for easier
    identification within third-party archives.
    
    Copyright {} {}
    
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
    
    http://www.apache.org/licenses/LICENSE-2.0
    
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
            "#,
            chrono::Utc::now().year(),
            params.common.project_name
        );

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
        _params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = format!(
            r#"
# Reporting Security Issues

This project is pre-release and does not have a security policy yet. Please check back later.
            "#
        );

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
        _params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = r#"
        # This workflow uses actions that are not certified by GitHub. They are provided
        # by a third-party and are governed by separate terms of service, privacy
        # policy, and support documentation.
        
        name: Scorecard supply-chain security
        on:
          # For Branch-Protection check. Only the default branch is supported. See
          # https://github.com/ossf/scorecard/blob/main/docs/checks.md#branch-protection
          branch_protection_rule:
          # To guarantee Maintained check is occasionally updated. See
          # https://github.com/ossf/scorecard/blob/main/docs/checks.md#maintained
          schedule:
            - cron: '17 18 * * 4'
          push:
            branches: [ "main" ]
        
        # Declare default permissions as read only.
        permissions: read-all
        
        jobs:
          analysis:
            name: Scorecard analysis
            runs-on: ubuntu-latest
            permissions:
              # Needed to upload the results to code-scanning dashboard.
              security-events: write
              # Needed to publish results and get a badge (see publish_results below).
              id-token: write
              # Uncomment the permissions below if installing in a private repository.
              # contents: read
              # actions: read
        
            steps:
              - name: "Checkout code"
                uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11 # v4.1.1
                with:
                  persist-credentials: false
        
              - name: "Run analysis"
                uses: ossf/scorecard-action@0864cf19026789058feabb7e87baa5f140aac736 # v2.3.1
                with:
                  results_file: results.sarif
                  results_format: sarif
                  # (Optional) "write" PAT token. Uncomment the `repo_token` line below if:
                  # - you want to enable the Branch-Protection check on a *public* repository, or
                  # - you are installing Scorecard on a *private* repository
                  # To create the PAT, follow the steps in https://github.com/ossf/scorecard-action#authentication-with-pat.
                  # repo_token: ${{ secrets.SCORECARD_TOKEN }}
        
                  # Public repositories:
                  #   - Publish results to OpenSSF REST API for easy access by consumers
                  #   - Allows the repository to include the Scorecard badge.
                  #   - See https://github.com/ossf/scorecard-action#publishing-results.
                  # For private repositories:
                  #   - `publish_results` will always be set to `false`, regardless
                  #     of the value entered here.
                  publish_results: true
        
              # Upload the results as artifacts (optional). Commenting out will disable uploads of run results in SARIF
              # format to the repository Actions tab.
              - name: "Upload artifact"
                uses: actions/upload-artifact@1eb3cb2b3e0f29609092a73eb033bb759a334595 # v4.1.0
                with:
                  name: SARIF file
                  path: results.sarif
                  retention-days: 5
        
              # Upload the results to GitHub's code scanning dashboard.
              - name: "Upload to code-scanning"
                uses: github/codeql-action/upload-sarif@cdcdbb579706841c47f7063dda365e292e5cad7a # v2.13.4
                with:
                  sarif_file: results.sarif"#.to_string();

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "scorecard.yml".to_string(),
                path: "./.github/workflows".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::Scorecard,
        })
    }

    fn generate_security_insights_content(
        &self,
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let insights = SecurityInsightsVersion100YamlSchema {
            contribution_policy: SecurityInsightsVersion100YamlSchemaContributionPolicy {
                accepts_automated_pull_requests: true,
                accepts_pull_requests: true,
                automated_tools_list: None,
                code_of_conduct: None,
                contributing_policy: None,
            },
            dependencies: None,
            distribution_points: Vec::new(),
            documentation: None,
            header: SecurityInsightsVersion100YamlSchemaHeader {
                changelog: None,
                commit_hash: None,
                expiration_date: chrono::Utc::now() + chrono::Duration::days(365),
                last_reviewed: None,
                last_updated: None,
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
            security_artifacts: None,
            security_assessments: None,
            security_contacts: Vec::new(),
            security_testing: Vec::new(),
            vulnerability_reporting: SecurityInsightsVersion100YamlSchemaVulnerabilityReporting {
                accepts_vulnerability_reports: false,
                bug_bounty_available: None,
                bug_bounty_url: None,
                comment: None,
                email_contact: None,
                in_scope: None,
                out_scope: None,
                pgp_key: None,
                security_policy: None,
            },
        };

        let content = serde_yaml::to_string(&insights)?;

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "SECURITY_INSIGHTS.yml".to_string(),
                path: "./".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::SecurityInsights,
        })
    }
}

/// Handles the generation of source files content specific to Go projects hosted on Github.
/// e.g. Github actions running goreleaser
struct GoGithubSourceBundleContentHandler {}
impl SourceBundleContentGenerator for GoGithubSourceBundleContentHandler {
    fn generate_content(
        &self,
        params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        match params.facet_type {
            SupportedFacetType::Gitignore => self.generate_gitignore_content(params),
            SupportedFacetType::SLSABuild => self.generate_slsa_build_content(params),
            SupportedFacetType::DependencyUpdateTool => self.generate_dependency_update_tool_content(params),
            _ => todo!("Not implemented yet"),
        }
    }
}
impl GoGithubSourceBundleContentHandler {
    fn generate_gitignore_content(
        &self,
        _params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        // This is taken from Github's defaults: https://github.com/github/gitignore/blob/main/Go.gitignore
        let content = r#"
# If you prefer the allow list template instead of the deny list, see community template:
# https://github.com/github/gitignore/blob/main/community/Golang/Go.AllowList.gitignore
#
# Binaries for programs and plugins
*.exe
*.exe~
*.dll
*.so
*.dylib

# Test binary, built with `go test -c`
*.test

# Output of the go coverage tool, specifically when used with LiteIDE
*.out

# Dependency directories (remove the comment below to include it)
# vendor/

# Go workspace file
go.work
        "#
        .to_string();
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
        _params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = r#"

        #
        # Copyright 2022 The GUAC Authors.
        #
        # Licensed under the Apache License, Version 2.0 (the "License");
        # you may not use this file except in compliance with the License.
        # You may obtain a copy of the License at
        #
        #     http://www.apache.org/licenses/LICENSE-2.0
        #
        # Unless required by applicable law or agreed to in writing, software
        # distributed under the License is distributed on an "AS IS" BASIS,
        # WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
        # See the License for the specific language governing permissions and
        # limitations under the License.
        name: release
        
        on:
          workflow_dispatch: # testing only, trigger manually to test it works
          push:
            branches:
              - main
            tags:
              - 'v*'
        
        permissions:
          actions: read   # for detecting the Github Actions environment.
          contents: write # To upload assets to release.
          packages: write # To publish container images to GHCR
          id-token: write # needed for signing the images with GitHub OIDC Token
        
        jobs:
          goreleaser:
            runs-on: ubuntu-latest
            outputs:
              hashes: ${{ steps.hash.outputs.hashes }}
              image: ${{ steps.hash.outputs.image }}
              digest: ${{ steps.hash.outputs.digest }}
            steps:
              - name: Checkout
                uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11 # v4.1.1
                with:
                  fetch-depth: 0
              - name: Login to GitHub Container Registry
                uses: docker/login-action@343f7c4344506bcbf9b4de18042ae17996df046d # v3.0.0
                with:
                  registry: ghcr.io
                  username: ${{ github.actor }}
                  password: ${{ secrets.GITHUB_TOKEN }}
              - name: Set up Go
                uses: actions/setup-go@0c52d547c9bc32b1aa3301fd7a9cb496313a4491 # v5.0.0
                with:
                  go-version: '1.21'
              - name: Install cosign
                uses:  sigstore/cosign-installer@9614fae9e5c5eddabb09f90a270fcb487c9f7149 # main
              - name: Install syft
                uses: anchore/sbom-action/download-syft@c7f031d9249a826a082ea14c79d3b686a51d485a # v0.15.3
        
              - name: Run GoReleaser Snapshot
                if: ${{ !startsWith(github.ref, 'refs/tags/') }}
                id: run-goreleaser-snapshot
                uses: goreleaser/goreleaser-action@7ec5c2b0c6cdda6e8bbb49444bc797dd33d74dd8 # v5.0.0
                with:
                  distribution: goreleaser
                  version: latest
                  args: release --clean --snapshot --skip-sign
                env:
                  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                  GORELEASER_CURRENT_TAG: v0.0.0-snapshot-tag
                  DOCKER_CONTEXT: default
              - name: Run GoReleaser Release
                if: startsWith(github.ref, 'refs/tags/')
                id: run-goreleaser-release
                uses: goreleaser/goreleaser-action@7ec5c2b0c6cdda6e8bbb49444bc797dd33d74dd8 # v5.0.0
                with:
                  distribution: goreleaser
                  version: latest
                  # use .goreleaser-nightly.yaml for nightly build; otherwise use the default
                  args: ${{ contains( github.ref, 'nightly' ) && 'release --clean -f .goreleaser-nightly.yaml' || 'release --clean' }} 
                env:
                  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                  DOCKER_CONTEXT: default
        
              - name: Generate hashes and extract image digest
                id: hash
                if: startsWith(github.ref, 'refs/tags/')
                env:
                  ARTIFACTS: "${{ steps.run-goreleaser-release.outputs.artifacts }}"
                run: |
                  set -euo pipefail
          
                  hashes=$(echo $ARTIFACTS | jq --raw-output '.[] | {name, "digest": (.extra.Digest // .extra.Checksum)} | select(.digest) | {digest} + {name} | join("  ") | sub("^sha256:";"")' | base64 -w0)
                  if test "$hashes" = ""; then # goreleaser < v1.13.0
                    checksum_file=$(echo "$ARTIFACTS" | jq -r '.[] | select (.type=="Checksum") | .path')
                    hashes=$(cat $checksum_file | base64 -w0)
                  fi
                  echo "hashes=$hashes" >> $GITHUB_OUTPUT
        
                  image=$(echo $ARTIFACTS | jq --raw-output '.[] | select( .type =="Docker Manifest" ).name | split(":")[0]')
                  echo "image=$image" >> $GITHUB_OUTPUT
                  digest=$(echo $ARTIFACTS | jq --raw-output '.[] | select( .type =="Docker Manifest" ).extra.Digest')
                  echo "digest=$digest" >> $GITHUB_OUTPUT
        
          sbom-container:
            # generate sbom for container as goreleaser can't - https://goreleaser.com/customization/sbom/#limitations
            name: generate sbom for container
            runs-on: ubuntu-latest
            needs: [goreleaser]
            if: startsWith(github.ref, 'refs/tags/')
            steps:
              - name: Checkout code
                uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11 # tag=v3
              - name: Login to GitHub Container Registry
                uses: docker/login-action@343f7c4344506bcbf9b4de18042ae17996df046d # v3.0.0
                with:
                  registry: ghcr.io
                  username: ${{ github.actor }}
                  password: ${{ secrets.GITHUB_TOKEN }}
              - name: Run Trivy in fs mode to generate SBOM
                uses: aquasecurity/trivy-action@d43c1f16c00cfd3978dde6c07f4bbcf9eb6993ca # master
                with:
                  scan-type: 'fs'
                  format: 'spdx-json'
                  output: 'spdx.sbom.json'
              - name: Install cosign
                uses: sigstore/cosign-installer@9614fae9e5c5eddabb09f90a270fcb487c9f7149 # main
              - name: Sign image and sbom
                run: |
                  #!/usr/bin/env bash
                  set -euo pipefail
                  cosign attach sbom --sbom spdx.sbom.json ${IMAGE_URI_DIGEST}
                  cosign sign -a git_sha=$GITHUB_SHA --attachment sbom ${IMAGE_URI_DIGEST} --yes
                shell: bash
                env:
                  IMAGE_URI_DIGEST: ${{ needs.goreleaser.outputs.image }}@${{ needs.goreleaser.outputs.digest }}
        
          provenance-bins:
            name: generate provenance for binaries
            needs: [goreleaser]
            if: startsWith(github.ref, 'refs/tags/')
            uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v1.9.0 # must use semver here
            with:
              base64-subjects: "${{ needs.goreleaser.outputs.hashes }}"
              upload-assets: true
        
          provenance-container:
            name: generate provenance for container
            needs: [goreleaser]
            if: startsWith(github.ref, 'refs/tags/')
            uses: slsa-framework/slsa-github-generator/.github/workflows/generator_container_slsa3.yml@v1.9.0 # must use semver here
            with:
              image: ${{ needs.goreleaser.outputs.image }}
              digest: ${{ needs.goreleaser.outputs.digest }}
              registry-username: ${{ github.actor }}
            secrets:
              registry-password: ${{ secrets.GITHUB_TOKEN }}
        
          compose-tarball:
            runs-on: ubuntu-latest
            name: generate compose tarball
            needs: [goreleaser]
            if: startsWith(github.ref, 'refs/tags/')
            steps:
              - name: Checkout code
                uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11 # tag=v3
              - name: Create and publish compose tarball
                env:
                  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                run: |
                  #!/usr/bin/env bash
                  set -euo pipefail
                  mkdir guac-compose
                  cp .env guac-compose/
                  cp docker-compose.yml guac-compose/
                  cp -r container_files guac-compose/
                  sed -i s/local-organic-guac/ghcr.io\\/${{ github.repository_owner }}\\/guac:${{ github.ref_name }}/ guac-compose/.env
                  tar -zcvf guac-compose.tar.gz guac-compose/
                  rm -rf guac-compose/
                  gh release upload ${{ github.ref_name }} guac-compose.tar.gz
                  rm guac-compose.tar.gz
                shell: bash 
"#.to_string();

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "releases.yml".to_string(),
                path: ".github/workflows/".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::SLSABuild,
        })
    }

    fn generate_dependency_update_tool_content(
        &self,
        _params: &SourceBundleFacetParams,
    ) -> Result<SourceBundleContent, Box<dyn Error>> {
        let content = r#"
        version: 2
        updates:
          # Maintain Golang dependencies.
          - package-ecosystem: gomod
            directory: "/"
            schedule:
              interval: weekly
        
          # Maintain dependencies for GitHub Actions.
          - package-ecosystem: "github-actions"
            directory: "/"
            schedule:
              interval: "weekly"
"#
        .to_string();

        Ok(SourceBundleContent {
            source_files_content: vec![SourceFileContent {
                name: "dependabot.yml".to_string(),
                path: ".github/".to_string(),
                content,
            }],
            facet_type: SupportedFacetType::DependencyUpdateTool,
        })
    }
}

pub struct FacetSetParamsGenerator {}

impl FacetSetParamsGenerator {
    // TODO: Come up with a better solution than hard coding the default facets
    pub fn generate_default(
        &self,
        common_params: CommonFacetParams,
    ) -> Result<FacetSetParams, Box<dyn Error>> {
        use SupportedFacetType::*;
        let supported_facets = vec![
            Readme,
            License,
            Gitignore,
            SecurityPolicy,
            SecurityInsights,
            SLSABuild,
            // SBOMGenerator, // Handled by the SLSABuild facet
            // StaticCodeAnalysis,
            DependencyUpdateTool,
            //Fuzzing,
            // PublishPackages,
            // PinnedDependencies,
            // SAST,
            // VulnerabilityScanner,
            // GUACForwardingConfig,
            // These are at the end to allow Skootrs to push initial commits without needing
            // code review or branches.
            // CodeReview, // TODO: Implement this
            // BranchProtection, //TODO: Implement this
        ];
        let facets_params = supported_facets
            .iter()
            .map(|facet_type| {
                FacetParams::SourceBundle(SourceBundleFacetParams {
                    common: common_params.clone(),
                    facet_type: facet_type.clone(),
                })
            })
            .collect::<Vec<FacetParams>>();

        Ok(FacetSetParams { facets_params })
    }
}

pub struct SourceFileContentParams {
    pub name: String,
    pub path: String,
    pub content: String,
}
