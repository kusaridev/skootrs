# Skootrs

A CLI tool for creating secure-by-design/default source repos.

**Note**: This is a POC! Don't use this outside of testing.

## Pre-reqs

**Note**: These pre-reqs will change often as the tool develops and matures
- Rust nightly >=1.77 - [Read more](https://www.rust-lang.org/tools/install)
- GitHub token with the following permissions: `admin:org, admin:repo_hook, admin:ssh_signing_key, audit_log, delete_repo, repo, workflow, write:packages`

## Running Skootrs

Skootrs is currently pre-release so you will need to use cargo to compile and run Skootrs yourself.

```
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

The initial talk given on Skootrs appears to not have been recorded but here are the locations of slides that include the reason why Skootrs is being built along with some architecture:

- [OpenSSF Day Japan 2023](https://github.com/mlieberman85/talks/blob/91cf3bef51f7d277a744098863389e362920b4c8/2023-12-04-ossfday/presentation.pdf)
- [NYU Guest Talk](https://github.com/mlieberman85/talks/blob/main/2024-01-30-skootrs/presentation.pdf)