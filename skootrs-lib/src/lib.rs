//
// Copyright 2023 The Skootrs Authors.
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

//! This is the main entry point for core functionality in the Skootrs project.
//! 
//! It contains the main traits and implementations for the core functionality of the project.
//! This includes the creation of a project, managing of the project's repository, and the management
//! of the project's source code.
//! 
//! This crate also contains the concept of a facet, which is an abstraction for some piece of a project
//! that is managed by Skootrs to provide a secure-by-default project. This includes things like sets of files
//! in the source code or calls to the repository API. For example the SECURITY.md file or the API call that
//! enables branch protection would be facets.
#![feature(array_try_map)]

pub mod service;
