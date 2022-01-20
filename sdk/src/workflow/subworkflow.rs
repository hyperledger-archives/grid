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

//! An API for managing a subprocess within a workflow, which contains the list of states involved
//! in this subprocess

use super::state::{StartWorkflowState, WorkflowState};

/// A smaller more specific version of a workflow used to define a more complicated business
/// process within a workflow
#[derive(Clone)]
pub struct SubWorkflow {
    name: String,
    /// The states an object may be in within this subworkflow
    states: Vec<WorkflowState>,
    /// The state an object begins at within this subworkflow
    start_state: Option<StartWorkflowState>,
}

impl SubWorkflow {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieve a specific workflow state from this subworkflow. Returns `None` if the state
    /// does not exist in this subworkflow.
    pub fn state(&self, name: &str) -> Option<WorkflowState> {
        for state in &self.states {
            if state.name() == name {
                return Some(state.clone());
            }
        }

        None
    }

    /// Return the workflow state an object must enter the subworkflow at
    pub fn start_state(&self) -> Option<&StartWorkflowState> {
        self.start_state.as_ref()
    }
}

/// Builder used to create a `SubWorkflow`
#[derive(Default)]
pub struct SubWorkflowBuilder {
    name: String,
    states: Vec<WorkflowState>,
    start_state: Option<StartWorkflowState>,
}

impl SubWorkflowBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    /// Add a workflow state to this subworkflow
    pub fn add_state(mut self, state: WorkflowState) -> Self {
        self.states.push(state);
        self
    }

    /// Add the name of a workflow state an object must enter this subworkflow at
    pub fn with_start_state(mut self, start_state: StartWorkflowState) -> Self {
        self.start_state = Some(start_state);
        self
    }

    pub fn build(self) -> SubWorkflow {
        SubWorkflow {
            name: self.name,
            states: self.states,
            start_state: self.start_state,
        }
    }
}
