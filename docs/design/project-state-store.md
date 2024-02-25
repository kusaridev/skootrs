# Project State Store Design

## Summary

Skootrs needs a way for knowing what actions it has done in creating and managing a project. This is required in order for Skootrs to be able to keep track of what projects it is managing for CRUD purposes.

## Motivation

Currently, all state is stored in a local database on the host running Skootrs. This has multiple problems:
- If something is changed out of band fro the Skootrs tool it can be difficult to know what has changed.
- It is impossible to run Skootrs on a project from systems that weren't the one that originally created the project without exporting/importing from the database or copying the database to a new host.
- It is impossible for someone to audit if someone else has run Skootrs on a project without access to this database.
- It is impossible for multiple authorized parties to manage the same project through Skootrs

### Goals:
- Provide mechanism to store and retrieve the state of a Skootrs project This would include:
    - High level metadata like name of project
    - List of facets created along with their associated metadata
- Provide mechanism to give an identifier for a project (e.g. repo url) and have Skootrs be able to manage and audit that project.

### Non-Goals:
- Provide secure mechanism for ensuring the Skootrs project state can't be modified by unauthorized actors or outside of the Skootrs tool. Eventually this will need to be considered.

## Proposal

There are two main pieces to the proposal:
- Store the state for a Skootrs project in the repo for the project itself
- Store a local cache of references to projects the local Skootrs knows about

### In-repo State Store Design

The main idea would be store the entirety of the state as a file inside the repo assuming it's not too big and doesn't change often.

#### File

The proposed solution is to just have a single file with the Skootrs project state inside of the repo. This file should just be called: `./skootrs` and kept in the root of the repo.

#### Structure and Format

The format of the state file should just be json with the structure being just the `InitializedProject` struct.

### Changes to be made

#### Data Model

Various `Initialized` structs will need to be updated to include information on:
- Hash of the file(s) (if it's a file based facet)

#### Services
- `SourceBundleFacetService` needs to be updated to also take the hash of the file.
- Some of the other services will need to be updated to support pulling/pushing 


### Local Reference Cache Design

This can just be a simple file called `.skootrscache` with just a list of repo URLs.

## Future

There's a lot of things that can be done to improve on this in the future that could allow for things like:
- Local caching of project state so you don't have to fetch it every time
- Having some human readable mapping of project to project url
