// Copyright 2019 Cargill Incorporated
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

use crate::models::CircuitProposal;
use crate::schema::circuit_proposal;

use diesel::{pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult};

pub fn fetch_proposal_by_id(conn: &PgConnection, id: &str) -> QueryResult<Option<CircuitProposal>> {
    circuit_proposal::table
        .filter(circuit_proposal::id.eq(id))
        .first::<CircuitProposal>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}
