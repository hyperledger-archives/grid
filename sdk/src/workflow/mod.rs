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

mod state;
mod subworkflow;

pub use state::{PermissionAlias, WorkflowState, WorkflowStateBuilder};
pub use subworkflow::{SubWorkflow, SubWorkflowBuilder};

pub struct Workflow {
    subworkflow: Vec<SubWorkflow>,
}

impl Workflow {
    pub fn new(subworkflow: Vec<SubWorkflow>) -> Self {
        Self { subworkflow }
    }

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
    fn test_permission_alias() {
        let mut permission = PermissionAlias::new("po.seller");
        permission.add_permission("po.create");
        permission.add_permission("po.update");
        permission.add_transition("confirm");

        assert_eq!("po.seller", permission.name());
        assert_eq!(
            &["po.create".to_string(), "po.update".to_string()],
            permission.permissions()
        );
        assert_eq!(&["confirm".to_string()], permission.transitions());
    }

    #[test]
    fn test_workflow_state() {
        let mut permission = PermissionAlias::new("po.seller");
        permission.add_permission("po.create");
        permission.add_permission("po.update");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("issued")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission)
            .build();

        assert_eq!(
            vec!["po.create".to_string(), "po.update".to_string()],
            state.expand_permissions(&vec!["po.seller".to_string()]),
        );

        assert_eq!(
            true,
            state.can_transition("confirm".to_string(), vec!["po.seller".to_string()]),
        );

        assert_eq!(
            false,
            state.can_transition("issued".to_string(), vec!["po.seller".to_string()]),
        );
    }

    #[test]
    fn test_subworkflow() {
        let mut permission = PermissionAlias::new("po.seller");
        permission.add_permission("po.create");
        permission.add_permission("po.update");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("issued")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission)
            .build();

        let subworkflow = SubWorkflowBuilder::new("po")
            .add_state(state)
            .add_starting_state("issued")
            .add_starting_state("proposed")
            .build();

        assert_eq!("po", subworkflow.name());
        assert_eq!(
            &["issued".to_string(), "proposed".to_string()],
            subworkflow.starting_states()
        );
        assert!(subworkflow.state("issued").is_some());
    }

    #[test]
    fn test_workflow() {
        let mut permission = PermissionAlias::new("po.seller");
        permission.add_permission("po.create");
        permission.add_permission("po.update");
        permission.add_transition("confirm");

        let state = WorkflowStateBuilder::new("issued")
            .add_constraint("active=None")
            .add_transition("issued")
            .add_transition("confirm")
            .add_permission_alias(permission)
            .build();

        let subworkflow = SubWorkflowBuilder::new("po")
            .add_state(state)
            .add_starting_state("issued")
            .add_starting_state("proposed")
            .build();

        let workflow = Workflow::new(vec![subworkflow]);

        assert!(workflow.subworkflow("po").is_some());
    }
}
