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

//! The goal of Grid Workflow is to make permissioning and state code for smart contracts easier to
//! write and maintain by offering a flexible framework for implementing it. Grid Workflow provides
//! a framework for modeling business workflows as state transitions within smart contracts. The
//! Grid Workflow module encapsulates business process complexity and allows for those rules to
//! become decoupled from the smart contract logic.

mod state;
mod subworkflow;

pub use state::{
    PermissionAlias, StartWorkflowState, StartWorkflowStateBuilder, WorkflowState,
    WorkflowStateBuilder,
};
pub use subworkflow::{SubWorkflow, SubWorkflowBuilder};

/// A single workflow may involve multiple processes; these processes are defined by the list of
/// subworkflows, which are different smaller workflows that make up the overall workflow.
pub struct Workflow {
    subworkflow: Vec<SubWorkflow>,
}

impl Workflow {
    /// Create a workflow by explicitly defining the workflow's processes
    pub fn new(subworkflow: Vec<SubWorkflow>) -> Self {
        Self { subworkflow }
    }

    /// Retrieve a specific process within the overall workflow
    pub fn subworkflow(&self, name: &str) -> Option<SubWorkflow> {
        for sub_wf in &self.subworkflow {
            if sub_wf.name() == name {
                return Some(sub_wf.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Validate a `PermissionAlias` is able to be built successfully.
    fn test_permission_alias() {
        let mut permission = PermissionAlias::new("po::seller");
        permission.add_permission("can-create-po");
        permission.add_permission("can-update-po");
        permission.add_transition("confirm");

        assert_eq!("po::seller", permission.name());
        assert_eq!(
            &["can-create-po".to_string(), "can-update-po".to_string()],
            permission.permissions()
        );
        assert_eq!(&["confirm".to_string()], permission.transitions());
    }

    #[test]
    /// Validate a `WorkflowState` object is able to be built successfully, with specific
    /// permissions.
    fn test_workflow_state() {
        let mut permission = PermissionAlias::new("po::seller");
        permission.add_permission("can-create-po");
        permission.add_permission("can-update-po");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("create")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission)
            .build();

        assert_eq!(
            vec!["can-create-po".to_string(), "can-update-po".to_string()],
            state.expand_permissions(&vec!["po::seller".to_string()]),
        );

        assert_eq!(
            true,
            state.can_transition("confirm".to_string(), &["po::seller".to_string()]),
        );

        assert_eq!(
            false,
            state.can_transition("issued".to_string(), &["po::seller".to_string()]),
        );
    }

    #[test]
    /// Validate a `SubWorkflow` is able to be built successfully, containing permissions
    /// and workflow states.
    fn test_subworkflow() {
        let mut permission = PermissionAlias::new("po::seller");
        permission.add_permission("can-create-po");
        permission.add_permission("can-update-po");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("issued")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission.clone())
            .build();

        let start_state = StartWorkflowStateBuilder::default()
            .add_permission_alias(permission)
            .add_transition("issued")
            .build();

        let subworkflow = SubWorkflowBuilder::new("po")
            .with_start_state(start_state.clone())
            .add_state(state)
            .build();

        assert_eq!("po", subworkflow.name());
        assert!(subworkflow.start_state().is_some());
        assert!(subworkflow.state("issued").is_some());
    }

    #[test]
    /// Validate a `Workflow` object is able to be built successfully.
    fn test_workflow() {
        let mut permission = PermissionAlias::new("po::seller");
        permission.add_permission("can-create-po");
        permission.add_permission("can-update-po");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("issued")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission.clone())
            .build();

        let start_state = StartWorkflowStateBuilder::default()
            .add_permission_alias(permission)
            .add_transition("issued")
            .build();

        let subworkflow = SubWorkflowBuilder::new("po")
            .with_start_state(start_state)
            .add_state(state)
            .build();

        let workflow = Workflow::new(vec![subworkflow]);

        assert!(workflow.subworkflow("po").is_some());
    }
}
