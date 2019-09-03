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

use std::error::Error;
use std::fmt::{self, Write};

pub fn to_hex(bytes: &[u8]) -> String {
    let mut buf = String::new();
    for b in bytes {
        write!(&mut buf, "{:02x}", b).expect("Unable to write to string");
    }

    buf
}

pub fn parse_hex(hex: &str) -> Result<Vec<u8>, HexError> {
    if hex.len() % 2 != 0 {
        return Err(HexError {
            context: format!("{} is not valid hex: odd number of digits", hex),
            source: None,
        });
    }

    let mut res = vec![];
    for i in (0..hex.len()).step_by(2) {
        res.push(
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|err| HexError {
                context: format!("{} contains invalid hex", hex),
                source: Some(Box::new(err)),
            })?,
        );
    }

    Ok(res)
}

#[derive(Debug)]
pub struct HexError {
    context: String,
    source: Option<Box<dyn Error>>,
}

impl Error for HexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the parse_hex function.
    ///
    /// * test positive cases
    /// * test error cases
    /// * test round trip
    /// * test empty
    #[test]
    fn test_parse_hex() {
        assert_eq!(
            vec![00u8, 10u8],
            parse_hex("000a").expect("unable to parse 000a")
        );
        assert_eq!(
            vec![01u8, 99u8, 255u8],
            parse_hex("0163ff").expect("unable to parse 0163ff")
        );

        // check that odd number of digits fails
        assert!(parse_hex("0").is_err());

        // check that invalid digits fails
        assert!(parse_hex("0G").is_err());

        // check round trip
        assert_eq!(
            "abcdef",
            &to_hex(&parse_hex("abcdef").expect("unable to parse hex for round trip"))
        );
        assert_eq!(
            "012345",
            &to_hex(&parse_hex("012345").expect("unable to parse hex for round trip"))
        );

        // check empty parses
        let empty: Vec<u8> = Vec::with_capacity(0);
        assert_eq!(empty, parse_hex("").expect("unable to parse empty"));
    }
}
