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

#[cfg(feature = "circuit-template")]
use std::str::FromStr as _;

#[cfg(feature = "circuit-template")]
/// Get the next base62 string of the same length.
///
/// # Returns
///
/// If the given string is the maximum value (all characters are `z`) or the empty string, `None`
/// is returned; otherwise, the next base62 string is returned.
///
/// # Errors
///
/// If the given string is not valid base62, a `Base62Error` will be returned.
pub fn next_base62_string(string: &str) -> Result<Option<String>, Base62Error> {
    if string.is_empty() {
        return Ok(None);
    }

    if !string.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(Base62Error::new("Invalid string: not base62"));
    }

    let (remaining_string, last_character) = string.split_at(string.len() - 1);

    // This cannot fail because the string is non-empty and only the last character is captured in
    // the line above.
    let last_character = char::from_str(last_character).expect("Failed to get char from str");

    match next_base62_char(last_character) {
        Some(next_char) => Ok(Some(remaining_string.to_owned() + &next_char.to_string())),
        None => Ok(next_base62_string(remaining_string)?.map(|string| string + "0")),
    }
}

#[cfg(feature = "circuit-template")]
/// Get the next base62 character; if the given character is the maximum (`z`), return `None`.
pub fn next_base62_char(c: char) -> Option<char> {
    let char_value = c as u32;

    // 'z' is the highest base62 char
    if char_value >= 'z' as u32 {
        return None;
    }

    // This cannot fail, because all u32 values <= `'z' as u32` are valid chars
    let next_char = std::char::from_u32(char_value + 1).expect("Failed to get char from u32");

    if !next_char.is_ascii_alphanumeric() {
        return next_base62_char(next_char);
    }

    Some(next_char)
}

#[cfg(feature = "circuit-template")]
#[derive(Debug)]
pub struct Base62Error(String);

#[cfg(feature = "circuit-template")]
impl Base62Error {
    pub fn new(err: &str) -> Self {
        Self(err.into())
    }
}

#[cfg(feature = "circuit-template")]
impl std::error::Error for Base62Error {}

#[cfg(feature = "circuit-template")]
impl std::fmt::Display for Base62Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "circuit-template")]
    /// Verify that the `next_base62_string` function correctly gets the next base62 string.
    #[test]
    fn next_string() {
        assert_eq!(
            &next_base62_string("0000")
                .expect("got err")
                .expect("got none"),
            "0001"
        );
        assert_eq!(
            &next_base62_string("000z")
                .expect("got err")
                .expect("got none"),
            "0010"
        );
        assert_eq!(
            &next_base62_string("0010")
                .expect("got err")
                .expect("got none"),
            "0011"
        );
        assert_eq!(
            &next_base62_string("00zz")
                .expect("got err")
                .expect("got none"),
            "0100"
        );
        assert_eq!(
            &next_base62_string("0100")
                .expect("got err")
                .expect("got none"),
            "0101"
        );
        assert_eq!(
            &next_base62_string("0zzz")
                .expect("got err")
                .expect("got none"),
            "1000"
        );
        assert_eq!(
            &next_base62_string("1000")
                .expect("got err")
                .expect("got none"),
            "1001"
        );
        assert!(next_base62_string("zzzz").unwrap().is_none());
        assert!(next_base62_string("").unwrap().is_none());
        assert!(next_base62_string("not_base62").is_err());
    }

    #[cfg(feature = "circuit-template")]
    /// Verify that the `next_base62_char` function correctly gets the next base62 char.
    #[test]
    fn next_char() {
        assert_eq!(next_base62_char('0'), Some('1'));
        assert_eq!(next_base62_char('9'), Some('A'));
        assert_eq!(next_base62_char('A'), Some('B'));
        assert_eq!(next_base62_char('Z'), Some('a'));
        assert_eq!(next_base62_char('a'), Some('b'));
        assert_eq!(next_base62_char('z'), None);
    }
}
