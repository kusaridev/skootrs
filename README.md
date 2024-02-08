# Skootrs

A CLI tool for creating secure-by-design/default source repos.

**Note**: This is a POC! Don't use this outside of testing.

## Pre-reqs

**Note**: These pre-reqs will change often as the tool develops and matures
- Rust nightly >=1.77 - [Read more](https://www.rust-lang.org/tools/install)
- GitHub token with the following permissions: `admin:org, admin:repo_hook, admin:ssh_signing_key, audit_log, delete_repo, repo, workflow, write:packages`

## Running Skootrs

Skootrs is currently pre-release so you will need to use cargo to compile and run Skootrs yourself.

```shell
$ cargo run
Usage: skootrs <COMMAND>

Commands:
  create     
  daemon     
  dump       
  get-facet  
  help       Print this message or the help of the given subcommand(s)
```

- Create - The main command for creating a Skootrs secure-by-default repo.
- Daemon - The command for running Skootrs as a REST API.
- Dump - The command for dumping the output of Skootrs' state database to stdout. Useful for debugging.
- Get-Facet - The command for dumping the file or API output for a facet for a given project.

To get pretty printing of the logs which are in [bunyan](https://github.com/trentm/node-bunyan) format I recommend piping the skootrs into the bunyan cli. I recommend using [bunyan-rs](https://github.com/LukeMathWalker/bunyan). For example:

```shell
$ cargo run create | bunyan                                                              ~/Projects/skootrs
    Finished dev [unoptimized + debuginfo] target(s) in 0.19s
     Running `target/debug/skootrs-bin create`
> The name of the repository skoot-test-bunyan
> The description of the repository asdf
> Select an organization mlieberman85
> Select a language Go
[2024-02-08T05:34:11.249Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Github Repo Created: skoot-test-bunyan (file=skootrs-lib/src/service/repo.rs,line=81,target=skootrs_lib::service::repo)
[2024-02-08T05:34:11.251Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: {"context":{"id":"mlieberman85/skoot-test-bunyan","source":"skootrs.github.creator","timestamp":"2024-02-08T05:34:11.250869Z","type":"dev.cdevents.repository.created.0.1.1","version":"0.3.0"},"subject":{"content":{"name":"skoot-test-bunyan","owner":"mlieberman85","url":"https://github.com/mlieberman85/skoot-test-bunyan","viewUrl":"https://github.com/mlieberman85/skoot-test-bunyan"},"id":"mlieberman85/skoot-test-bunyan","source":"skootrs.github.creator","type":"repository"}} (file=skootrs-lib/src/service/repo.rs,line=106,target=skootrs_lib::service::repo)
[2024-02-08T05:34:11.675Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Initialized go module for skoot-test-bunyan (file=skootrs-lib/src/service/ecosystem.rs,line=95,target=skootrs_lib::service::ecosystem)
[2024-02-08T05:34:11.675Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Writing file README.md to ./ (file=skootrs-lib/src/service/facet.rs,line=115,target=skootrs_lib::service::facet)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Creating path "/tmp/skoot-test-bunyan/./" (file=skootrs-lib/src/service/source.rs,line=92,target=skootrs_lib::service::source)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Writing file LICENSE to ./ (file=skootrs-lib/src/service/facet.rs,line=115,target=skootrs_lib::service::facet)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Creating path "/tmp/skoot-test-bunyan/./" (file=skootrs-lib/src/service/source.rs,line=92,target=skootrs_lib::service::source)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Writing file .gitignore to ./ (file=skootrs-lib/src/service/facet.rs,line=115,target=skootrs_lib::service::facet)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Creating path "/tmp/skoot-test-bunyan/./" (file=skootrs-lib/src/service/source.rs,line=92,target=skootrs_lib::service::source)
[2024-02-08T05:34:11.676Z]  INFO: skootrs/16973 on Michaels-MBP-2.localdomain: Writing file SECURITY.md to ./ (file=skootrs-lib/src/service/facet.rs,line=115,target=skootrs_lib::service::facet)
```

The initial talk given on Skootrs appears to not have been recorded but here are the locations of slides that include the reason why Skootrs is being built along with some architecture:

- [OpenSSF Day Japan 2023](https://github.com/mlieberman85/talks/blob/91cf3bef51f7d277a744098863389e362920b4c8/2023-12-04-ossfday/presentation.pdf)
- [NYU Guest Talk](https://github.com/mlieberman85/talks/blob/main/2024-01-30-skootrs/presentation.pdf)