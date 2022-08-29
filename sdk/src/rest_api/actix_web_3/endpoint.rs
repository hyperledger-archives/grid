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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Backend {
    Splinter,
    Sawtooth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub backend: Backend,
    pub url: String,
}

impl Endpoint {
    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn is_sawtooth(&self) -> bool {
        self.backend == Backend::Sawtooth
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

impl From<&str> for Endpoint {
    fn from(s: &str) -> Self {
        let s = s.to_lowercase();

        if s.starts_with("splinter:") {
            let url = s.replace("splinter:", "");
            Endpoint {
                backend: Backend::Splinter,
                url,
            }
        } else if s.starts_with("sawtooth:") {
            let url = s.replace("sawtooth:", "");
            Endpoint {
                backend: Backend::Sawtooth,
                url,
            }
        } else {
            Endpoint {
                backend: Backend::Sawtooth,
                url: s,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_endpoint_splinter_prefix() {
        let endpoint = Endpoint::from("splinter:tcp://localhost:8080");
        assert_eq!(
            endpoint,
            Endpoint {
                backend: Backend::Splinter,
                url: "tcp://localhost:8080".into()
            }
        );
    }

    #[test]
    fn test_endpoint_sawtooth_prefix() {
        let endpoint = Endpoint::from("sawtooth:tcp://localhost:8080");
        assert_eq!(
            endpoint,
            Endpoint {
                backend: Backend::Sawtooth,
                url: "tcp://localhost:8080".into()
            }
        );
    }

    #[test]
    fn test_endpoint_no_prefix() {
        let endpoint = Endpoint::from("tcp://localhost:8080");
        assert_eq!(
            endpoint,
            Endpoint {
                backend: Backend::Sawtooth,
                url: "tcp://localhost:8080".into()
            }
        );
    }

    #[test]
    fn test_endpoint_capitals() {
        let endpoint = Endpoint::from("SAWTOOTH:TCP://LOCALHOST:8080");
        assert_eq!(
            endpoint,
            Endpoint {
                backend: Backend::Sawtooth,
                url: "tcp://localhost:8080".into()
            }
        );
    }

    #[test]
    fn test_endpoint_no_protocol() {
        let endpoint = Endpoint::from("splinter:localhost:8080");
        assert_eq!(
            endpoint,
            Endpoint {
                backend: Backend::Splinter,
                url: "localhost:8080".into()
            }
        );
    }
}
