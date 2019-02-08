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

use crate::error::CliError;
use std::io::prelude::*;

use std::net::TcpStream;

pub fn do_show(url: &str) -> Result<(), CliError> {
    let mut connection = TcpStream::connect(url)?;
    let request = b"GET /show HTTP/1.1";
    connection.write(request)?;
    connection.flush()?;

    let mut buffer = [0; 512];

    connection.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..]);

    if response.starts_with("HTTP/1.1 200 OK\r\n\r\n") {
        let value = &response["HTTP/1.1 200 OK\r\n\r\n".len()..];
        println!("Value = {}", value);
    } else {
        println!("{}", response);
    }
    Ok(())
}

pub fn do_add(url: &str, value: &str) -> Result<(), CliError> {
    let mut connection = TcpStream::connect(url)?;
    if value.parse::<u32>().is_err() {
        return Err(CliError::UserError(format!(
            "Value {} cannot be parsed to u32",
            value
        )));
    }
    let request = format!("GET /add/{} HTTP/1.1", value);
    connection.write(request.as_bytes())?;
    connection.flush()?;

    let mut buffer = [0; 512];

    connection.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..]);

    if !response.starts_with("HTTP/1.1 204 NO CONTENT") {
        println!("{}", response);
    }
    Ok(())
}
