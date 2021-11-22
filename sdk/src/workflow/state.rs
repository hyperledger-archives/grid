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

//! Representation of a single state within a workflow

/// Defines the current state of an item within a workflow. WorkflowState represents a single
/// point within a workflow and defines the logic used by the smart contract to determine if an
/// item may be in this workflow state.
#[derive(Clone)]
pub struct WorkflowState {
    name: String,
    /// Defines specific attributes an item must have to be in this workflow state
    constraints: Vec<String>,
    /// Permission definitions for operating within this workflow state
    permission_aliases: Vec<PermissionAlias>,
    /// Workflow states that may be transitioned to from this workflow state
    transitions: Vec<String>,
}

impl WorkflowState {
    /// Determines if an item may be transitioned to a new workflow state, considering the
    /// permissions of the submitter and the `permission_aliases` defined within this workflow
    /// state.
    ///
    /// # Arguments
    ///
    /// * `new_state` - Name of the workflow state an item is attempting to be transitioned to
    /// * `pike_permissions` - List of Grid Pike permissions assigned to the submitter of the
    /// request
    pub fn can_transition(&self, new_state: String, pike_permissions: Vec<String>) -> bool {
        if self.name == new_state {
            return true;
        }

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

    /// List the workflow permissions available to a permission alias
    ///
    /// # Arguments
    ///
    /// `names` - List of names of the permission aliases to expand
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

    /// Retrieve the aliases that contain the specified workflow permissions
    ///
    /// # Arguments
    ///
    /// `permission` - Permission to search for within the workflow aliases
    pub fn get_aliases_by_permission(&self, permission: &str) -> Vec<String> {
        let mut aliases = Vec::new();

        for alias in &self.permission_aliases {
            if alias.permissions().contains(&permission.to_string()) {
                aliases.push(alias.name().to_string());
            }
        }

        aliases
    }

    /// Check if a workflow state contains the specified constraint
    ///
    /// # Arguments
    ///
    /// `constraint` - Name of the constraint a workflow state may hold
    pub fn has_constraint(&self, constraint: &str) -> bool {
        self.constraints.contains(&constraint.to_string())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Builder used to create a `WorkflowState` object
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

/// An alias that houses multiple permissions
#[derive(Clone, Default)]
pub struct PermissionAlias {
    name: String,
    /// Permissions granted to this alias
    permissions: Vec<String>,
    /// Workflow states this alias is able to transition an object to
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
