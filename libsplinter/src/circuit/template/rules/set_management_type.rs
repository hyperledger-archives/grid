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

use super::super::{yaml_parser::v1, CircuitTemplateError, CreateCircuitBuilder};

pub(super) struct CircuitManagement {
    management_type: String,
}

impl CircuitManagement {
    pub fn apply_rule(
        &self,
        builder: CreateCircuitBuilder,
    ) -> Result<CreateCircuitBuilder, CircuitTemplateError> {
        Ok(builder.with_circuit_management_type(&self.management_type))
    }
}

impl From<v1::CircuitManagement> for CircuitManagement {
    fn from(yaml_circuit_management: v1::CircuitManagement) -> Self {
        CircuitManagement {
            management_type: yaml_circuit_management.management_type().to_string(),
        }
    }
}
