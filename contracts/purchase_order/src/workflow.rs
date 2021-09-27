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

use std::fmt;

use grid_sdk::workflow::{
    PermissionAlias, SubWorkflow, SubWorkflowBuilder, Workflow, WorkflowStateBuilder,
};

use crate::permissions::Permission;

pub enum POWorkflow {
    SystemOfRecord,
    Collaborative,
}

pub fn get_workflow(name: &POWorkflow) -> Option<Workflow> {
    match *name {
        POWorkflow::SystemOfRecord => Some(system_of_record_workflow()),
        POWorkflow::Collaborative => Some(collaborative_workflow()),
    }
}

fn system_of_record_workflow() -> Workflow {
    Workflow::new(vec![
        default_sub_workflow(),
        system_of_record_sub_workflow(),
    ])
}

fn collaborative_workflow() -> Workflow {
    Workflow::new(vec![default_sub_workflow(), collaborative_sub_workflow()])
}

fn default_sub_workflow() -> SubWorkflow {
    let create = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanCreatePo.to_string());
        buyer.add_permission(&Permission::CanCreatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionIssued.to_string());
        buyer.add_transition("issued");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission(&Permission::CanCreatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionIssued.to_string());
        buyer.add_transition("issued");

        WorkflowStateBuilder::new("create")
            .add_transition("issued")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let issued = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanCreatePoVersion.to_string());
        buyer.add_permission(&Permission::CanUpdatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionClosed.to_string());
        buyer.add_transition("closed");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission(&Permission::CanCreatePoVersion.to_string());
        seller.add_permission(&Permission::CanUpdatePoVersion.to_string());
        seller.add_permission(&Permission::CanTransitionConfirmed.to_string());
        seller.add_transition("confirmed");

        WorkflowStateBuilder::new("issued")
            .add_transition("confirmed")
            .add_transition("closed")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let confirmed = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanCreatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionIssued.to_string());
        buyer.add_transition("issued");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission(&Permission::CanCreatePoVersion.to_string());
        seller.add_permission(&Permission::CanTransitionClosed.to_string());
        seller.add_transition("confirmed");

        WorkflowStateBuilder::new("confirmed")
            .add_transition("issued")
            .add_transition("closed")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let closed = {
        let buyer = PermissionAlias::new("po::buyer");
        let seller = PermissionAlias::new("po::seller");

        WorkflowStateBuilder::new("closed")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    SubWorkflowBuilder::new("po")
        .add_state(create)
        .add_state(issued)
        .add_state(confirmed)
        .add_state(closed)
        .add_starting_state("create")
        .build()
}

fn system_of_record_sub_workflow() -> SubWorkflow {
    let create = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanCreatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionProposed.to_string());
        buyer.add_transition("proposed");

        let seller = PermissionAlias::new("po::seller");

        WorkflowStateBuilder::new("create")
            .add_transition("proposed")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let proposed = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanUpdatePoVersion.to_string());
        buyer.add_permission(&Permission::CanTransitionObsolete.to_string());
        buyer.add_transition("obsolete");

        let mut seller_confirm = PermissionAlias::new("po::seller");
        seller_confirm.add_permission(&Permission::CanUpdatePoVersion.to_string());
        seller_confirm.add_permission(&Permission::CanTransitionRejected.to_string());
        seller_confirm.add_permission(&Permission::CanTransitionAccepted.to_string());
        seller_confirm.add_transition("rejected");
        seller_confirm.add_transition("accepted");

        let mut seller_modify = PermissionAlias::new("po::seller");
        seller_modify.add_permission(&Permission::CanUpdatePoVersion.to_string());
        seller_modify.add_permission(&Permission::CanUpdatePo.to_string());
        seller_modify.add_permission(&Permission::CanTransitionModified.to_string());
        seller_modify.add_transition("modified");

        WorkflowStateBuilder::new("proposed")
            .add_transition("obsolete")
            .add_transition("rejected")
            .add_transition("accepted")
            .add_transition("modified")
            .add_permission_alias(buyer)
            .add_permission_alias(seller_confirm)
            .add_permission_alias(seller_modify)
            .build()
    };

    let obsolete = {
        let buyer = PermissionAlias::new("po::buyer");
        let seller = PermissionAlias::new("po::seller");

        WorkflowStateBuilder::new("obsolete")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let rejected = {
        let buyer = PermissionAlias::new("po::buyer");
        let seller = PermissionAlias::new("po::seller");

        WorkflowStateBuilder::new("rejected")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let modified = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanTransitionObsolete.to_string());
        buyer.add_transition("obsolete");

        let mut seller_modify = PermissionAlias::new("po::seller");
        seller_modify.add_permission(&Permission::CanUpdatePoVersion.to_string());
        seller_modify.add_permission(&Permission::CanUpdatePo.to_string());
        seller_modify.add_permission(&Permission::CanTransitionModified.to_string());
        seller_modify.add_permission(&Permission::CanUpdatePoVersionResponse.to_string());

        let mut editor = PermissionAlias::new("po::editor");
        editor.add_permission(&Permission::CanTransitionEditable.to_string());
        editor.add_permission(&Permission::CanTransitionReview.to_string());
        editor.add_transition("review");
        editor.add_transition("editable");

        WorkflowStateBuilder::new("modified")
            .add_transition("editable")
            .add_transition("review")
            .add_transition("obsolete")
            .add_permission_alias(buyer)
            .add_permission_alias(seller_modify)
            .add_permission_alias(editor)
            .build()
    };

    let accepted = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission(&Permission::CanTransitionObsolete.to_string());
        buyer.add_transition("obsolete");

        let seller = PermissionAlias::new("po::seller");

        WorkflowStateBuilder::new("accepted")
            .add_transition("obsolete")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let editable = {
        let mut draft = PermissionAlias::new("po::draft");
        draft.add_permission(&Permission::CanUpdatePoVersion.to_string());
        draft.add_permission(&Permission::CanTransitionCancelled.to_string());
        draft.add_permission(&Permission::CanTransitionReview.to_string());
        draft.add_transition("cancelled");
        draft.add_transition("review");

        WorkflowStateBuilder::new("editable")
            .add_transition("review")
            .add_transition("cancelled")
            .add_permission_alias(draft)
            .build()
    };

    let review = {
        let mut draft = PermissionAlias::new("po::draft");
        draft.add_permission(&Permission::CanUpdatePoVersion.to_string());
        draft.add_permission(&Permission::CanTransitionEditable.to_string());
        draft.add_permission(&Permission::CanTransitionComposed.to_string());
        draft.add_permission(&Permission::CanTransitionDeclined.to_string());
        draft.add_transition("editable");
        draft.add_transition("composed");
        draft.add_transition("declined");

        WorkflowStateBuilder::new("review")
            .add_transition("composed")
            .add_transition("declined")
            .add_transition("editable")
            .add_permission_alias(draft)
            .build()
    };

    let declined = {
        let mut draft = PermissionAlias::new("po::draft");
        draft.add_permission(&Permission::CanTransitionEditable.to_string());
        draft.add_permission(&Permission::CanTransitionCancelled.to_string());
        draft.add_transition("editable");
        draft.add_transition("cancelled");

        WorkflowStateBuilder::new("declined")
            .add_transition("editable")
            .add_transition("cancelled")
            .add_permission_alias(draft)
            .build()
    };

    let composed = {
        let draft = PermissionAlias::new("po::draft");

        WorkflowStateBuilder::new("composed")
            .add_permission_alias(draft)
            .build()
    };

    let cancelled = {
        let draft = PermissionAlias::new("po::draft");

        WorkflowStateBuilder::new("cancelled")
            .add_permission_alias(draft)
            .build()
    };

    SubWorkflowBuilder::new("version")
        .add_state(create)
        .add_state(proposed)
        .add_state(obsolete)
        .add_state(rejected)
        .add_state(modified)
        .add_state(accepted)
        .add_state(editable)
        .add_state(review)
        .add_state(declined)
        .add_state(composed)
        .add_state(cancelled)
        .add_starting_state("proposed")
        .add_starting_state("editable")
        .build()
}

fn collaborative_sub_workflow() -> SubWorkflow {
    let create = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission(&Permission::CanCreatePoVersion.to_string());
        partner.add_permission(&Permission::CanTransitionProposed.to_string());
        partner.add_transition("proposed");

        WorkflowStateBuilder::new("create")
            .add_transition("proposed")
            .add_permission_alias(partner)
            .build()
    };

    let proposed = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission(&Permission::CanUpdatePoVersion.to_string());
        partner.add_permission(&Permission::CanTransitionRejected.to_string());
        partner.add_permission(&Permission::CanTransitionAccepted.to_string());
        partner.add_permission(&Permission::CanTransitionModified.to_string());
        partner.add_permission(&Permission::CanTransitionObsolete.to_string());
        partner.add_transition("rejected");
        partner.add_transition("accepted");
        partner.add_transition("modified");
        partner.add_transition("obsolete");

        WorkflowStateBuilder::new("proposed")
            .add_transition("obsolete")
            .add_transition("rejected")
            .add_transition("accepted")
            .add_transition("modified")
            .add_permission_alias(partner)
            .build()
    };

    let rejected = {
        let partner = PermissionAlias::new("po::partner");

        WorkflowStateBuilder::new("rejected")
            .add_permission_alias(partner)
            .build()
    };

    let accepted = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission(&Permission::CanTransitionObsolete.to_string());
        partner.add_transition("obsolete");

        WorkflowStateBuilder::new("proposed")
            .add_transition("obsolete")
            .add_permission_alias(partner)
            .build()
    };

    let modified = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission(&Permission::CanUpdatePoVersion.to_string());
        partner.add_permission(&Permission::CanUpdatePo.to_string());
        partner.add_permission(&Permission::CanUpdatePoVersionResponse.to_string());
        partner.add_permission(&Permission::CanTransitionProposed.to_string());
        partner.add_permission(&Permission::CanTransitionAccepted.to_string());
        partner.add_permission(&Permission::CanTransitionObsolete.to_string());
        partner.add_transition("proposed");
        partner.add_transition("accepted");
        partner.add_transition("obsolete");

        WorkflowStateBuilder::new("modified")
            .add_transition("proposed")
            .add_transition("accepted")
            .add_transition("obsolete")
            .add_permission_alias(partner)
            .build()
    };

    let obsolete = {
        let partner = PermissionAlias::new("po::partner");

        WorkflowStateBuilder::new("obsolete")
            .add_permission_alias(partner)
            .build()
    };

    SubWorkflowBuilder::new("version")
        .add_state(create)
        .add_state(proposed)
        .add_state(obsolete)
        .add_state(rejected)
        .add_state(modified)
        .add_state(accepted)
        .add_starting_state("create")
        .build()
}

impl fmt::Display for POWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            POWorkflow::SystemOfRecord => write!(f, "system_of_record"),
            POWorkflow::Collaborative => write!(f, "collaborative"),
        }
    }
}
