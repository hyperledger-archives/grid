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

use rand::{distributions::Alphanumeric, Rng};

use super::{SecretManager, SecretManagerError};

const SECRET_LENGTH: usize = 64;

/// A SecretManager that generates a random string as a secret and keeps it in memory
pub struct AutoSecretManager {
    secret: String,
}

impl Default for AutoSecretManager {
    fn default() -> Self {
        AutoSecretManager {
            secret: generate_random_secret(),
        }
    }
}

impl SecretManager for AutoSecretManager {
    fn secret(&self) -> Result<String, SecretManagerError> {
        Ok(self.secret.to_owned())
    }

    fn update_secret(&mut self) -> Result<(), SecretManagerError> {
        self.secret = generate_random_secret();
        Ok(())
    }
}

fn generate_random_secret() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(SECRET_LENGTH)
        .collect()
}
