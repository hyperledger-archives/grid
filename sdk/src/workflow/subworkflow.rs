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

use super::state::WorkflowState;

#[derive(Clone)]
pub struct SubWorkflow {
    name: String,
    states: Vec<WorkflowState>,
    starting_states: Vec<String>,
}

impl SubWorkflow {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self, name: &str) -> Option<WorkflowState> {
        for state in &self.states {
            if state.name() == name {
                return Some(state.clone());
            }
        }

        None
    }

    pub fn starting_states(&self) -> &[String] {
        &self.starting_states
    }
}

#[derive(Default)]
pub struct SubWorkflowBuilder {
    name: String,
    states: Vec<WorkflowState>,
    starting_states: Vec<String>,
}

impl SubWorkflowBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    pub fn add_state(mut self, state: WorkflowState) -> Self {
        self.states.push(state);
        self
    }

    pub fn add_starting_state(mut self, starting_state: &str) -> Self {
        self.starting_states.push(starting_state.to_string());
        self
    }

    pub fn build(self) -> SubWorkflow {
        SubWorkflow {
            name: self.name,
            states: self.states,
            starting_states: self.starting_states,
        }
    }
}
