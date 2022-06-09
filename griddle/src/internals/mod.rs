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
mod error;
mod runnable;
mod running;

pub use builder::GriddleBuilder;
#[cfg(feature = "rest-api")]
pub use builder::GriddleRestApiVariant;
pub use error::GriddleError;
pub use runnable::RunnableGriddle;
#[cfg(feature = "rest-api")]
pub use runnable::RunnableGriddleRestApiVariant;
pub use running::Griddle;
#[cfg(feature = "rest-api")]
pub use running::RunningGriddleRestApiVariant;

#[derive(Clone, Debug, PartialEq)]
pub enum DLTBackend {
    Splinter,
    Sawtooth,
}
