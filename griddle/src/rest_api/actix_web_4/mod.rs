// Copyright 2018-2022 Cargill Incorporated
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

mod builder;
mod runnable;

pub use builder::GriddleRestApiBuilder;
pub use runnable::RunnableGriddleRestApi;

use actix_web::Resource;

/// A `ResourceProvider` provides a list of resources.
///
/// This trait serves as a `Resource` factory, which allows dynamically building a REST API at
/// runtime.
pub trait GriddleResourceProvider: Send {
    /// Returns a list of Actix `Resource`s.
    fn resources(&self) -> Vec<Resource>;
}
