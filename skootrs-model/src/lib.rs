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

//! This is where all the models for the Skootrs project are defined. The models are just the data
//! representing the abstractions around a project like its repository, source code, and facets.
//! 
//! The models here need to be (de)serializable, i.e. implementing `serde::Serialize` and `serde::Deserialize`
//! so that they can be easily used in RPC calls and other places where data needs to be sent over the wire.
//! For example the REST API that is in the `skootrs-rest` crate. Currently for the sake of simplicity we don'T
//! use much in the way of generics and trait objects because of issues with (de)serialization.
//! 
//! All the models besides those that fall under `/skootrs` are considered external. In most cases they are
//! code generated. The models in `/skootrs` are the core models for the Skootrs project and defined for the
//! purpose of the project.
pub mod security_insights;
pub mod cd_events;
pub mod skootrs;