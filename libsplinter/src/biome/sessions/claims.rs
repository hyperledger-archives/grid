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

//! Provides a definition and a builder for the payload of a JWT Token

use std::collections::HashMap;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use super::ClaimsBuildError;

/// Defines payload of a JWT Token
#[derive(Serialize, Deserialize)]
pub struct Claims {
    user_id: String,
    iss: String,
    exp: u64,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    custom_claims: HashMap<String, String>,
}

impl Claims {
    /// Returns the user id
    pub fn user_id(&self) -> String {
        self.user_id.to_owned()
    }

    /// Returns the issuer of the token
    pub fn iss(&self) -> String {
        self.iss.to_owned()
    }

    /// Returns the expiration of the token
    pub fn exp(&self) -> u64 {
        self.exp
    }

    /// Returns custom claims
    pub fn custom_claims(&self) -> HashMap<String, String> {
        self.custom_claims.clone()
    }
}
/// Builder for a claim
#[derive(Default)]
pub struct ClaimsBuilder {
    user_id: Option<String>,
    iss: Option<String>,
    duration: Option<Duration>,
    custom_claims: HashMap<String, String>,
}

impl ClaimsBuilder {
    /// User id to be included in the claims
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Issuer to be included in the claims
    pub fn with_issuer(mut self, iss: &str) -> Self {
        self.iss = Some(iss.to_string());
        self
    }

    /// Duration of the JWT token. The token expiration timestamp will be calculated based on
    /// this value upon build.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Adds an custom claim. This method can be called multiple times.
    pub fn with_custom_claim(mut self, key: &str, value: &str) -> Self {
        self.custom_claims
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Consumes the builder and returns Claims. It calculates the expiration token by adding
    /// the duration set in the builder to the current system time. The `exp` field in the claims
    /// is set the resulting value.
    pub fn build(self) -> Result<Claims, ClaimsBuildError> {
        let user_id = self
            .user_id
            .ok_or_else(|| ClaimsBuildError::MissingRequiredField("Missing user_id".to_string()))?;

        let iss = self
            .iss
            .ok_or_else(|| ClaimsBuildError::MissingRequiredField("Missing iss".to_string()))?;

        let duration = self.duration.ok_or_else(|| {
            ClaimsBuildError::MissingRequiredField("Missing claim duration".to_string())
        })?;

        let token_expiration_date = SystemTime::now().checked_add(duration).ok_or_else(|| {
            ClaimsBuildError::InvalidValue(format!("Invalid duration for claim: {:?}", duration))
        })?;

        let token_expiration_timestamp = get_timestamp(token_expiration_date).map_err(|err| {
            ClaimsBuildError::InvalidValue(format!("Invalid duration for claim: {}", err))
        })?;

        Ok(Claims {
            user_id,
            iss,
            exp: token_expiration_timestamp,
            custom_claims: self.custom_claims,
        })
    }
}

fn get_timestamp(time: SystemTime) -> Result<u64, SystemTimeError> {
    Ok(time.duration_since(UNIX_EPOCH)?.as_secs())
}
