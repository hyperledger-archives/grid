// Copyright 2018-2020 Cargill Incorporated
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

use std::sync::Arc;

use crate::actix_web::HttpRequest;
use crate::biome::rest_api::BiomeRestConfig;
use crate::biome::secrets::SecretManager;
use crate::biome::sessions::{validate_token, TokenValidationError};
use crate::rest_api::get_authorization_token;

use super::super::resources::authorize::AuthorizationResult;

pub(crate) fn authorize_user(
    request: &HttpRequest,
    user_id: &str,
    secret_manager: &Arc<dyn SecretManager>,
    rest_config: &BiomeRestConfig,
) -> AuthorizationResult {
    let auth_token = match get_authorization_token(&request) {
        Ok(token) => token,
        Err(err) => {
            debug!("Failed to get token: {}", err);
            return AuthorizationResult::Unauthorized("User is not authorized".to_string());
        }
    };

    let secret = match secret_manager.secret() {
        Ok(secret) => secret,
        Err(err) => {
            debug!("Failed to fetch secret {}", err);
            return AuthorizationResult::Failed;
        }
    };

    if let Err(err) = validate_token(&auth_token, &secret, &rest_config.issuer(), |claim| {
        if user_id != claim.user_id() {
            return Err(TokenValidationError::InvalidClaim(format!(
                "User is not update keys for user {}",
                user_id
            )));
        }
        Ok(())
    }) {
        debug!("Invalid token: {}", err);
        return AuthorizationResult::Unauthorized("User is not authorized".to_string());
    };

    AuthorizationResult::Authorized
}
