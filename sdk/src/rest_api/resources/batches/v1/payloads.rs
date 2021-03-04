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

use crate::submitter;

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<InvalidTransaction>,
    pub status: String,
}

impl From<submitter::BatchStatus> for BatchStatus {
    fn from(batch_status: submitter::BatchStatus) -> Self {
        Self {
            id: batch_status.id,
            invalid_transactions: batch_status
                .invalid_transactions
                .into_iter()
                .map(InvalidTransaction::from)
                .collect(),
            status: batch_status.status,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InvalidTransaction {
    pub id: String,
    pub message: String,
    pub extended_data: String,
}

impl From<submitter::InvalidTransaction> for InvalidTransaction {
    fn from(invalid_transaction: submitter::InvalidTransaction) -> Self {
        Self {
            id: invalid_transaction.id,
            message: invalid_transaction.message,
            extended_data: invalid_transaction.extended_data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

impl From<submitter::BatchStatusLink> for BatchStatusLink {
    fn from(batch_status_link: submitter::BatchStatusLink) -> Self {
        Self {
            link: batch_status_link.link,
        }
    }
}
