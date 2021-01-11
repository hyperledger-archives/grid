// Copyright 2018-2021 Cargill Incorporated
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

#[derive(Clone)]
pub struct WorkflowState {
    name: String,
    constraints: Vec<String>,
    permission_aliases: Vec<PermissionAlias>,
    transitions: Vec<String>,
}

impl WorkflowState {
    pub fn can_transition(&self, new_state: String, pike_permissions: Vec<String>) -> bool {
        if !self.transitions.contains(&new_state) {
            return false;
        }

        for perm in pike_permissions {
            for alias in &self.permission_aliases {
                if alias.name() == perm && alias.transitions.contains(&new_state) {
                    return true;
                }
            }
        }

        false
    }

    pub fn expand_permissions(&self, names: &[String]) -> Vec<String> {
        let mut perms = Vec::new();

        for name in names {
            for alias in &self.permission_aliases {
                if alias.name() == name {
                    perms.append(&mut alias.permissions().to_vec());
                }
            }
        }

        perms
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Default)]
pub struct WorkflowStateBuilder {
    name: String,
    constraints: Vec<String>,
    permission_aliases: Vec<PermissionAlias>,
    transitions: Vec<String>,
}

impl WorkflowStateBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    pub fn add_constraint(mut self, constraint: &str) -> Self {
        self.constraints.push(constraint.to_string());
        self
    }

    pub fn add_transition(mut self, transition: &str) -> Self {
        self.transitions.push(transition.to_string());
        self
    }

    pub fn add_permission_alias(mut self, alias: PermissionAlias) -> Self {
        self.permission_aliases.push(alias);
        self
    }

    pub fn build(self) -> WorkflowState {
        WorkflowState {
            name: self.name,
            constraints: self.constraints,
            permission_aliases: self.permission_aliases,
            transitions: self.transitions,
        }
    }
}

#[derive(Clone, Default)]
pub struct PermissionAlias {
    name: String,
    permissions: Vec<String>,
    transitions: Vec<String>,
}

impl PermissionAlias {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            permissions: vec![],
            transitions: vec![],
        }
    }

    pub fn add_permission(&mut self, permission: &str) {
        self.permissions.push(permission.to_string());
    }

    pub fn add_transition(&mut self, transition: &str) {
        self.transitions.push(transition.to_string());
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn transitions(&self) -> &[String] {
        &self.transitions
    }
}
