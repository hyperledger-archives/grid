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

use std::time::{Duration, SystemTime};

use actix_web::{error, web, Error, HttpResponse};
use futures::Future;
use gameroom_database::{helpers, models::XoGame, ConnectionPool};

use crate::rest_api::RestApiResponseError;

use super::{ErrorResponse, SuccessResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiXoGame {
    circuit_id: String,
    game_name: String,
    player_1: String,
    player_2: String,
    game_status: String,
    game_board: String,
    created_time: u64,
    updated_time: u64,
}

impl From<XoGame> for ApiXoGame {
    fn from(game: XoGame) -> Self {
        Self {
            circuit_id: game.circuit_id.to_string(),
            game_name: game.game_name.to_string(),
            player_1: game.player_1.to_string(),
            player_2: game.player_2.to_string(),
            game_status: game.game_status.to_string(),
            game_board: game.game_board.to_string(),
            created_time: game
                .created_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
            updated_time: game
                .updated_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
        }
    }
}

pub fn fetch_xo(
    pool: web::Data<ConnectionPool>,
    circuit_id: web::Path<String>,
    game_name: web::Path<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || fetch_xo_game_from_db(pool, &circuit_id, &game_name)).then(|res| {
            match res {
                Ok(xo_game) => Ok(HttpResponse::Ok().json(SuccessResponse::new(xo_game))),
                Err(err) => match err {
                    error::BlockingError::Error(err) => match err {
                        RestApiResponseError::NotFound(err) => Ok(HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&err.to_string()))),
                        _ => Ok(HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&err.to_string()))),
                    },
                    error::BlockingError::Canceled => {
                        debug!("Internal Server Error: {}", err);
                        Ok(HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error()))
                    }
                },
            }
        }),
    )
}

fn fetch_xo_game_from_db(
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
    game_name: &str,
) -> Result<ApiXoGame, RestApiResponseError> {
    if let Some(xo_game) = helpers::fetch_xo_game(&*pool.get()?, circuit_id, game_name)? {
        return Ok(ApiXoGame::from(xo_game));
    }
    Err(RestApiResponseError::NotFound(format!(
        "XO Game with name {} not found",
        game_name
    )))
}
