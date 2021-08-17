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
    CanCreatePo,
    CanUpdatePo,
    CanCreatePoVersion,
    CanUpdatePoVersion,
    CanUpdatePoVersionResponse,
    CanTransitionIssued,
    CanTransitionClosed,
    CanTransitionConfirmed,
    CanTransitionComposed,
    CanTransitionProposed,
    CanTransitionObsolete,
    CanTransitionRejected,
    CanTransitionAccepted,
    CanTransitionDeclined,
    CanTransitionModified,
    CanTransitionEditable,
    CanTransitionReview,
    CanTransitionCancelled,
}

pub fn permission_to_perm_string(permission: Permission) -> String {
    match permission {
        Permission::CanCreatePo => String::from("can-create-po"),
        Permission::CanUpdatePo => String::from("can-update-po"),
        Permission::CanCreatePoVersion => String::from("can-create-po-version"),
        Permission::CanUpdatePoVersion => String::from("can-update-po-version"),
        Permission::CanUpdatePoVersionResponse => String::from("can-update-po-version-response"),
        Permission::CanTransitionIssued => String::from("can-transition-issued"),
        Permission::CanTransitionClosed => String::from("can-transition-closed"),
        Permission::CanTransitionConfirmed => String::from("can-transition-confirmed"),
        Permission::CanTransitionComposed => String::from("can-transition-composed"),
        Permission::CanTransitionProposed => String::from("can-transition-proposed"),
        Permission::CanTransitionObsolete => String::from("can-transition-obsolete"),
        Permission::CanTransitionRejected => String::from("can-transition-rejected"),
        Permission::CanTransitionAccepted => String::from("can-transition-accepted"),
        Permission::CanTransitionDeclined => String::from("can-transition-declined"),
        Permission::CanTransitionModified => String::from("can-transition-modified"),
        Permission::CanTransitionEditable => String::from("can-transition-editable"),
        Permission::CanTransitionReview => String::from("can-transition-review"),
        Permission::CanTransitionCancelled => String::from("can-transition-cancelled"),
    }
}
