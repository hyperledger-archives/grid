// Copyright 2022 Cargill Incorporated
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

//! Provides native representations of smart contract actions used to deserialize from JSON

use cylinder::Signer;
use transact::protocol::transaction::Transaction;

use crate::rest_api::resources::error::ErrorResponse;

pub trait TransactionPayload {
    fn build_transaction(&self, signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse>;
}

impl TransactionPayload for Box<dyn TransactionPayload> {
    fn build_transaction(&self, signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        (**self).build_transaction(signer)
    }
}
