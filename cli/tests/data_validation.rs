/*
 * Copyright 2022 Cargill Incorporated
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

extern crate assert_cmd;
extern crate dirs;

use assert_cmd::prelude::*;
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

use grid_sdk::data_validation::validate_order_xml_3_4;
use grid_sdk::error::InvalidArgumentError;

const CACHE_DIR_ENV: &str = "GRID_TEST_CACHE_DIR";

static KEY_DIR: &str = "keys";
static CACHE_DIR: &str = "cache";
static STATE_DIR: &str = "state";

static INIT: Once = Once::new();

struct Config {
    schema_dir: String,
    manifest_path: PathBuf,
}

#[cfg(feature = "xsd-downloader")]
#[test]
fn test_validate_order_xml_3_4() {
    let config = get_setup().expect("Unable to get setup");

    let test_gdsn_xml_path = config
        .manifest_path
        .join("tests/data_validation/xml/order.xml");

    let mut data = String::new();
    std::fs::File::open(test_gdsn_xml_path)
        .expect("Could not open file")
        .read_to_string(&mut data)
        .expect("Could not convert GDSN path to string");

    let result = validate_order_xml_3_4(&data, false, &config.schema_dir);

    assert!(result.is_ok());
}

#[cfg(feature = "xsd-downloader")]
#[test]
fn test_validate_order_xml_3_4_path() {
    let config = get_setup().expect("Unable to get setup");

    let test_gdsn_xml_path = config
        .manifest_path
        .join("tests/data_validation/xml/order.xml");

    let path_str = test_gdsn_xml_path
        .to_str()
        .expect("Could not convert GDSN path to string");

    let result = validate_order_xml_3_4(path_str, true, &config.schema_dir);

    assert!(result.is_ok());
}

#[cfg(feature = "xsd-downloader")]
#[test]
fn test_validate_order_xml_3_4_invalid() {
    let config = get_setup().expect("Unable to get setup");

    let test_gdsn_xml_path = config
        .manifest_path
        .join("tests/data_validation/xml/order_invalid.xml");

    let mut data = String::new();
    std::fs::File::open(test_gdsn_xml_path)
        .expect("Could not open file")
        .read_to_string(&mut data)
        .expect("Could not convert GDSN path to string");

    let result = validate_order_xml_3_4(&data, false, &config.schema_dir);

    assert!(result.is_err());

    let expected_error = InvalidArgumentError::new(data, "file fails to validate".to_string());

    assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
}

#[cfg(feature = "xsd-downloader")]
#[test]
fn test_validate_order_xml_3_4_path_invalid() {
    let config = get_setup().expect("Unable to get setup");

    let test_gdsn_xml_path = config
        .manifest_path
        .join("tests/data_validation/xml/order_invalid.xml");

    let path_str = test_gdsn_xml_path
        .to_str()
        .expect("Could not convert GDSN path to string");

    let result = validate_order_xml_3_4(path_str, true, &config.schema_dir);

    assert!(result.is_err());

    let expected_error =
        InvalidArgumentError::new(path_str.to_string(), "file fails to validate".to_string());

    assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
}

/// Gets a memoized setup by creating keys, an organization, and an agent.
/// Also downloads necessary XSD files from GS1's website.
///
///     Necessary to run purchase order commands
///
fn get_setup() -> std::io::Result<Config> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let temp_path = env::temp_dir().join("grid-integration-tests");
    fs::create_dir_all(&temp_path).expect("could not create temp path");

    let cache_path = match env::var(CACHE_DIR_ENV) {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            let path = temp_path.join(CACHE_DIR);
            fs::create_dir_all(&path).expect("could not create cache path");
            path
        }
    };

    let state_dir = temp_path.join(STATE_DIR);
    fs::create_dir_all(&state_dir)?;

    let key_name: String = "test_key".to_string();
    let key_dir = temp_path.join(KEY_DIR).join(".grid").join("keys");
    fs::create_dir_all(&key_dir)?;

    let mut public_key_path = key_dir.clone();
    public_key_path.push(&key_name);
    public_key_path.set_extension("pub");

    let mut private_key_path = key_dir.clone();
    private_key_path.push(&key_name);
    private_key_path.set_extension("priv");

    INIT.call_once(|| {
        let key_dir_str = key_dir
            .into_os_string()
            .into_string()
            .expect("Unable to convert key dir to string");

        Command::cargo_bin("grid")
            .unwrap()
            .arg("-vv")
            .arg("keygen")
            .arg(&key_name)
            .arg("--force")
            .args(&["--key-dir", &key_dir_str])
            .assert()
            .success();

        assert!(public_key_path.exists());
        assert!(private_key_path.exists());

        let cmd_download_xsd = Command::cargo_bin("grid")
            .unwrap()
            .arg("-vv")
            .arg("download-xsd")
            .env("GRID_CACHE_DIR", &cache_path)
            .env("GRID_STATE_DIR", &state_dir)
            .output()
            .expect("Error downloading XSD files");

        println!("{:?}", cmd_download_xsd);
        cmd_download_xsd.assert().success();
    });

    assert!(INIT.is_completed());

    let schema_dir_str = state_dir
        .join("xsd/po/gs1/ecom")
        .into_os_string()
        .into_string()
        .expect("Unable to convert po schema dir to string");

    Ok(Config {
        schema_dir: schema_dir_str,
        manifest_path,
    })
}
