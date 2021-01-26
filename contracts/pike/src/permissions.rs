// Copyright 2021 Cargill Incorporated
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

pub enum Permission {
    CanCreateAgents,
    CanUpdateAgents,
    CanDeleteAgents,
    CanUpdateOrganization,
    CanCreateRoles,
    CanUpdateRoles,
    CanDeleteRoles,
}

pub fn permission_to_perm_string(permission: Permission) -> String {
    match permission {
        Permission::CanCreateAgents => String::from("pike::can-create-agents"),
        Permission::CanUpdateAgents => String::from("pike::can-update-agents"),
        Permission::CanDeleteAgents => String::from("pike::can-delete-agents"),
        Permission::CanUpdateOrganization => String::from("pike::can-update-organization"),
        Permission::CanCreateRoles => String::from("pike::can-create-roles"),
        Permission::CanUpdateRoles => String::from("pike::can-update-roles"),
        Permission::CanDeleteRoles => String::from("pike::can-delete-roles"),
    }
}
