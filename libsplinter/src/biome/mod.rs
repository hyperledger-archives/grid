// Copyright 2018-2020 Cargill Incorporated
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

//! The Biome submodule provides support for user management,
//! user credential management, private key management, and user
//! notifications.
//!
//! User Management: API for CRUD operations around managing users.
//!
//! Credential Management: API to register and authenticate a user using
//! a username and password. Not recommend for use in production.
//!
//! Private Key Management: API to store and retrieve encrypted private keys.
//!
//! User Notifications: API to create and manage user notifications.

#[cfg(feature = "biome-credentials")]
pub mod credentials;

pub mod datastore;

#[cfg(feature = "biome-key-management")]
pub mod key_management;

#[cfg(feature = "biome-notifications")]
pub mod notifications;

#[cfg(all(feature = "rest-api", feature = "biome-rest-api"))]
pub mod rest_api;
mod user;
