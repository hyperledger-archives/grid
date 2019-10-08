/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::{error::Error, fmt, time::SystemTime};

use diesel::connection::Connection;
use gameroom_database::{
    error, helpers,
    models::{NewXoGame, XoGame},
    ConnectionPool,
};
use splinter::service::scabbard::StateChangeEvent;

use crate::authorization_handler::sabre::{get_xo_contract_address, XO_PREFIX};

pub struct XoStateDeltaProcessor {
    circuit_id: String,
    node_id: String,
    requester: String,
    contract_address: String,
    db_pool: ConnectionPool,
}

impl XoStateDeltaProcessor {
    pub fn new(circuit_id: &str, node_id: &str, requester: &str, db_pool: &ConnectionPool) -> Self {
        XoStateDeltaProcessor {
            circuit_id: circuit_id.into(),
            node_id: node_id.to_string(),
            requester: requester.to_string(),
            contract_address: get_xo_contract_address(),
            db_pool: db_pool.clone(),
        }
    }

    pub fn handle_state_changes(
        &self,
        changes: Vec<StateChangeEvent>,
    ) -> Result<(), StateDeltaError> {
        changes
            .iter()
            .try_for_each(|change| self.handle_state_change(change))
    }

    fn handle_state_change(&self, change: &StateChangeEvent) -> Result<(), StateDeltaError> {
        debug!("Received state change: {}", change);
        match change {
            StateChangeEvent::Set { key, .. } if key == &self.contract_address => {
                debug!("Xo contract created successfully");
                let time = SystemTime::now();
                let conn = &*self.db_pool.get()?;
                conn.transaction::<_, error::DatabaseError, _>(|| {
                    let notification = helpers::create_new_notification(
                        "circuit_active",
                        &self.requester,
                        &self.node_id,
                        &self.circuit_id,
                    );
                    helpers::insert_gameroom_notification(&conn, &[notification])?;
                    helpers::update_gameroom_status(&conn, &self.circuit_id, &time, "Active")?;
                    helpers::update_gameroom_member_status(
                        &conn,
                        &self.circuit_id,
                        &time,
                        "Ready",
                        "Active",
                    )?;
                    helpers::update_gameroom_service_status(
                        &conn,
                        &self.circuit_id,
                        &time,
                        "Ready",
                        "Active",
                    )?;

                    Ok(())
                })
                .map_err(StateDeltaError::from)
            }
            StateChangeEvent::Set { key, value } if &key[..6] == XO_PREFIX => {
                let time = SystemTime::now();
                let game_state: Vec<String> = String::from_utf8(value.to_vec())
                    .map_err(|err| StateDeltaError::XoPayloadParseError(format!("{:?}", err)))
                    .map(|s| s.split(',').map(String::from).collect())?;

                let conn = &*self.db_pool.get()?;
                conn.transaction::<_, error::DatabaseError, _>(|| {
                    if let Some(game) =
                        helpers::fetch_xo_game(&conn, &self.circuit_id, &game_state[0])?
                    {
                        helpers::update_xo_game(
                            &conn,
                            XoGame {
                                game_board: game_state[1].clone(),
                                game_status: game_state[2].clone(),
                                player_1: game_state[3].clone(),
                                player_2: game_state[4].clone(),
                                updated_time: time,
                                ..game
                            },
                        )?;

                        let notification = helpers::create_new_notification(
                            &format!("game_updated:{}", game_state[0]),
                            &self.requester,
                            &self.node_id,
                            &self.circuit_id,
                        );
                        helpers::insert_gameroom_notification(&conn, &[notification])?;
                    } else {
                        helpers::insert_xo_game(
                            &conn,
                            NewXoGame {
                                circuit_id: self.circuit_id.clone(),
                                game_name: game_state[0].clone(),
                                game_board: game_state[1].clone(),
                                game_status: game_state[2].clone(),
                                player_1: game_state[3].clone(),
                                player_2: game_state[4].clone(),
                                created_time: time,
                                updated_time: time,
                            },
                        )?;
                        let notification = helpers::create_new_notification(
                            &format!("new_game_created:{}", game_state[0]),
                            &self.requester,
                            &self.node_id,
                            &self.circuit_id,
                        );
                        helpers::insert_gameroom_notification(&conn, &[notification])?;
                    }

                    Ok(())
                })
                .map_err(StateDeltaError::from)
            }
            StateChangeEvent::Delete { .. } => {
                debug!("Delete state skipping...");
                Ok(())
            }
            _ => {
                debug!("Unrecognized state change skipping...");
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum StateDeltaError {
    XoPayloadParseError(String),
    DatabaseError(error::DatabaseError),
}

impl Error for StateDeltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StateDeltaError::XoPayloadParseError(_) => None,
            StateDeltaError::DatabaseError(err) => Some(err),
        }
    }
}

impl fmt::Display for StateDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateDeltaError::XoPayloadParseError(err) => {
                write!(f, "Failed to parse xo payload: {}", err)
            }
            StateDeltaError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl From<error::DatabaseError> for StateDeltaError {
    fn from(err: error::DatabaseError) -> Self {
        StateDeltaError::DatabaseError(err)
    }
}
