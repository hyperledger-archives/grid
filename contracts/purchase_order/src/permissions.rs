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

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Permission::CanCreatePo => write!(f, "can-create-po"),
            Permission::CanUpdatePo => write!(f, "can-update-po"),
            Permission::CanCreatePoVersion => write!(f, "can-create-po-version"),
            Permission::CanUpdatePoVersion => write!(f, "can-update-po-version"),
            Permission::CanUpdatePoVersionResponse => {
                write!(f, "can-update-po-version-response")
            }
            Permission::CanTransitionIssued => write!(f, "can-transition-issued"),
            Permission::CanTransitionClosed => write!(f, "can-transition-closed"),
            Permission::CanTransitionConfirmed => write!(f, "can-transition-confirmed"),
            Permission::CanTransitionComposed => write!(f, "can-transition-composed"),
            Permission::CanTransitionProposed => write!(f, "can-transition-proposed"),
            Permission::CanTransitionObsolete => write!(f, "can-transition-obsolete"),
            Permission::CanTransitionRejected => write!(f, "can-transition-rejected"),
            Permission::CanTransitionAccepted => write!(f, "can-transition-accepted"),
            Permission::CanTransitionDeclined => write!(f, "can-transition-declined"),
            Permission::CanTransitionModified => write!(f, "can-transition-modified"),
            Permission::CanTransitionEditable => write!(f, "can-transition-editable"),
            Permission::CanTransitionReview => write!(f, "can-transition-review"),
            Permission::CanTransitionCancelled => write!(f, "can-transition-cancelled"),
        }
    }
}

impl Permission {
    /// Get the relevant permission for transitioning to a workflow state
    pub fn can_transition(to_status: &str) -> Option<Permission> {
        match to_status {
            "issued" => Some(Permission::CanTransitionIssued),
            "closed" => Some(Permission::CanTransitionClosed),
            "confirmed" => Some(Permission::CanTransitionConfirmed),
            "composed" => Some(Permission::CanTransitionComposed),
            "proposed" => Some(Permission::CanTransitionProposed),
            "obsolete" => Some(Permission::CanTransitionObsolete),
            "rejected" => Some(Permission::CanTransitionRejected),
            "accepted" => Some(Permission::CanTransitionAccepted),
            "declined" => Some(Permission::CanTransitionDeclined),
            "modified" => Some(Permission::CanTransitionModified),
            "editable" => Some(Permission::CanTransitionEditable),
            "review" => Some(Permission::CanTransitionReview),
            "cancelled" => Some(Permission::CanTransitionCancelled),
            _ => None,
        }
    }
}
