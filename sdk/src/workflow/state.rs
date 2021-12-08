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

//! Representation of a single state of a process within a workflow which describes the possible
//! state transitions the SubWorkflow can make, the constraints that need to be met to make said
//! transitions, and a list of permissions that are required by the acting entity to initiate a
//! transition.

/// Defines the current state of an item within a workflow. A `WorkflowState` contains a list of
/// constraints for items within this state, permission aliases to allow for operations to be made
/// within this state, and a list of transitions that can be made from this state.
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
    /// Determines if an entity may execute a transition to a given state, considering the
    /// permissions of the submitter and the `permission_aliases` defined within this workflow
    /// state.
    ///
    /// # Arguments
    ///
    /// * `new_state` - Name of the workflow state an item is attempting to be transitioned to
    /// * `pike_permissions` - List of Grid Pike permissions assigned to the submitter of the
    /// request
    pub fn can_transition(&self, new_state: String, pike_permissions: &[String]) -> bool {
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

    /// List the workflow permissions stored under the specified permission aliases
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

    /// Retrieve all aliases defined within this state that contain the specified workflow
    /// permission
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

    /// Returns true if this state contains the specified constraint
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

    /// Add a constraint to this workflow state. A constraint is interpreted by the smart contract
    /// as a guidelines that must be met before a transition to this workflow state is able to be
    /// made.
    pub fn add_constraint(mut self, constraint: &str) -> Self {
        self.constraints.push(constraint.to_string());
        self
    }

    /// Add the name of a workflow state that may be transitioned to from this state
    pub fn add_transition(mut self, transition: &str) -> Self {
        self.transitions.push(transition.to_string());
        self
    }

    /// Add a `PermissionAlias` to allow certain entities to perform certain actions within this
    /// workflow state
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

/// An alias for multiple permissions
#[derive(Clone, Default)]
pub struct PermissionAlias {
    name: String,
    /// Permissions assigned to this alias
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

    /// Assign a permission to this alias
    pub fn add_permission(&mut self, permission: &str) {
        self.permissions.push(permission.to_string());
    }

    /// Add a workflow state this alias is able to transition objects to
    pub fn add_transition(&mut self, transition: &str) {
        self.transitions.push(transition.to_string());
    }

    /// Return the name of this alias
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the permissions assigned to this alias
    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    /// Return the state transitions available to this alias
    pub fn transitions(&self) -> &[String] {
        &self.transitions
    }
}
