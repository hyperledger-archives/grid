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

use grid_sdk::workflow::{
    PermissionAlias, SubWorkflow, SubWorkflowBuilder, Workflow, WorkflowStateBuilder,
};

#[allow(dead_code)]
pub enum POWorkflow {
    SystemOfRecord,
    Collaborative,
}

#[allow(dead_code)]
pub fn get_workflow(name: POWorkflow) -> Option<Workflow> {
    match name {
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
        buyer.add_permission("can-create-po");
        buyer.add_permission("can-create-po-version");
        buyer.add_permission("can-transition-issued");
        buyer.add_transition("issued");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission("can-create-po-version");
        buyer.add_permission("can-transition-issued");
        buyer.add_transition("issued");

        WorkflowStateBuilder::new("create")
            .add_transition("issued")
            .add_permission_alias(buyer)
            .add_permission_alias(seller)
            .build()
    };

    let issued = {
        let mut buyer = PermissionAlias::new("po::buyer");
        buyer.add_permission("can-create-po-version");
        buyer.add_permission("can-update-po-version");
        buyer.add_permission("can-transition-closed");
        buyer.add_transition("closed");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission("can-create-po-version");
        seller.add_permission("can-update-po-version");
        seller.add_permission("can-transition-confirmed");
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
        buyer.add_permission("can-create-po-version");
        buyer.add_permission("can-transition-issued");
        buyer.add_transition("issued");

        let mut seller = PermissionAlias::new("po::seller");
        seller.add_permission("can-create-po-version");
        seller.add_permission("can-transition-closed");
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
        buyer.add_permission("can-create-po-version");
        buyer.add_permission("can-transition-proposed");
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
        buyer.add_permission("can-update-po-version");
        buyer.add_permission("can-transition-obsolete");
        buyer.add_transition("obsolete");

        let mut seller_confirm = PermissionAlias::new("po::seller");
        seller_confirm.add_permission("can-update-po-version");
        seller_confirm.add_permission("can-transition-rejected");
        seller_confirm.add_permission("can-transition-accepted");
        seller_confirm.add_transition("rejected");
        seller_confirm.add_transition("accepted");

        let mut seller_modify = PermissionAlias::new("po::seller");
        seller_modify.add_permission("can-update-po-version");
        seller_modify.add_permission("can-update-po");
        seller_modify.add_permission("can-transition-modified");
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
        buyer.add_permission("can-transition-obsolete");
        buyer.add_transition("obsolete");

        let mut seller_modify = PermissionAlias::new("po::seller");
        seller_modify.add_permission("can-update-po-version");
        seller_modify.add_permission("can-update-po");
        seller_modify.add_permission("can-transition-modified");
        seller_modify.add_permission("can-update-po-version-response");

        let mut editor = PermissionAlias::new("po::editor");
        editor.add_permission("can-transition-editable");
        editor.add_permission("can-transition-review");
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
        buyer.add_permission("can-transition-obsolete");
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
        draft.add_permission("can-update-po-version");
        draft.add_permission("can-transition-cancelled");
        draft.add_permission("can-transition-review");
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
        draft.add_permission("can-update-po-version");
        draft.add_permission("can-transition-editable");
        draft.add_permission("can-transition-composed");
        draft.add_permission("can-transition-declined");
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
        draft.add_permission("can-transition-editable");
        draft.add_permission("can-transition-cancelled");
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
        partner.add_permission("can-create-po-version");
        partner.add_permission("can-transition-proposed");
        partner.add_transition("proposed");

        WorkflowStateBuilder::new("create")
            .add_transition("proposed")
            .add_permission_alias(partner)
            .build()
    };

    let proposed = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission("can-update-po-version");
        partner.add_permission("can-transition-rejected");
        partner.add_permission("can-transition-accepted");
        partner.add_permission("can-transition-modified");
        partner.add_permission("can-transition-obsolete");
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
        partner.add_permission("can-transition-obsolete");
        partner.add_transition("obsolete");

        WorkflowStateBuilder::new("proposed")
            .add_transition("obsolete")
            .add_permission_alias(partner)
            .build()
    };

    let modified = {
        let mut partner = PermissionAlias::new("po::partner");
        partner.add_permission("can-update-po-version");
        partner.add_permission("can-update-po");
        partner.add_permission("can-update-po-version-response");
        partner.add_permission("can-transition-proposed");
        partner.add_permission("can-transition-accepted");
        partner.add_permission("can-transition-obsolete");
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
