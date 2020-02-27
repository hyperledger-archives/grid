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

mod error;
mod rules;
mod yaml_parser;

pub use error::CircuitTemplateError;

pub(self) use crate::admin::messages::{CreateCircuitBuilder, SplinterServiceBuilder};

pub struct Builders {
    create_circuit_builder: CreateCircuitBuilder,
    service_builders: Vec<SplinterServiceBuilder>,
}

impl Builders {
    pub fn set_create_circuit_builder(&mut self, builder: CreateCircuitBuilder) {
        self.create_circuit_builder = builder;
    }

    pub fn set_service_builders(&mut self, builders: Vec<SplinterServiceBuilder>) {
        self.service_builders = builders;
    }

    pub fn create_circuit_builder(&self) -> CreateCircuitBuilder {
        self.create_circuit_builder.clone()
    }

    pub fn service_builders(&self) -> Vec<SplinterServiceBuilder> {
        self.service_builders.clone()
    }
}
