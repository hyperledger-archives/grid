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

use super::super::{yaml_parser::v1, CircuitTemplateError, SplinterServiceBuilder};
use super::Value;

pub(super) struct CreateServices {
    service_type: String,
    service_args: Vec<ServiceArgument>,
    first_service: String,
}

#[derive(Debug)]
struct ServiceArgument {
    key: String,
    value: Value,
}

impl From<v1::CreateServices> for CreateServices {
    fn from(yaml_create_services: v1::CreateServices) -> Self {
        CreateServices {
            service_type: yaml_create_services.service_type().to_string(),
            service_args: yaml_create_services
                .service_args()
                .to_owned()
                .into_iter()
                .map(ServiceArgument::from)
                .collect(),
            first_service: yaml_create_services.first_service().to_string(),
        }
    }
}

impl From<v1::ServiceArgument> for ServiceArgument {
    fn from(yaml_service_argument: v1::ServiceArgument) -> Self {
        ServiceArgument {
            key: yaml_service_argument.key().to_string(),
            value: Value::from(yaml_service_argument.value().clone()),
        }
    }
}
