// Copyright (c) 2019 Target Brands, Inc.
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

extern crate assert_cmd;
extern crate dirs;

use assert_cmd::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;
use users::get_current_username;

mod integration {
    use super::*;
    static KEY_DIR: &str = "/root";
    static PUB_KEY_FILE: &str = "/root/.grid/keys/root.pub";

    static ORG_ID: &str = "cgl";
    static ORG_NAME: &str = "Cargill";
    static ORG_ADDRESS: &str = "hq";

    static SCHEMA_CREATE_FILE: &str = "tests/products/test_product_schema.yaml";

    static PRODUCT_CREATE_FILE: &str = "tests/products/create_product.yaml";
    static PRODUCT_UPDATE_FILE: &str = "tests/products/update_product.yaml";

    static PRODUCT_DELETE_ID: &str = "762111177704";

    static INIT: Once = Once::new();

    /// Verifies a `grid product create` command successfully runs.
    ///
    ///     The product information is read in from a yaml file.
    ///
    #[test]
    fn test_product_create() {
        get_setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .args(&["--file", &PRODUCT_CREATE_FILE]);
        cmd_product_create.assert().success();
    }

    /// Verifies a `grid product update` command successfully runs.
    ///
    ///     The product information is read in from a yaml file.
    ///     Products are first created before being updated.
    ///
    #[test]
    fn test_product_update() {
        get_setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .args(&["--file", &PRODUCT_CREATE_FILE]);
        cmd_product_create.assert().success();

        //run `grid product update`
        let mut cmd_product_update = make_grid_command();
        cmd_product_update
            .arg("product")
            .arg("update")
            .args(&["--file", &PRODUCT_UPDATE_FILE]);
        cmd_product_update.assert().success();
    }

    /// Verifies a `grid product delete` command successfully runs.
    ///
    ///     The delete command is supplied the product id and type.
    ///     Products are first created before being deleted.
    ///
    #[test]
    fn test_product_delete() {
        get_setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .args(&["--file", &PRODUCT_CREATE_FILE]);
        cmd_product_create.assert().success();

        //run `grid product delete`
        let mut cmd_product_delete = make_grid_command();
        cmd_product_delete
            .arg("product")
            .arg("delete")
            .arg(&PRODUCT_DELETE_ID)
            .args(&["--namespace", "GS1"]); //product type
        cmd_product_delete.assert().success();
    }

    /// Creates keys, an organization, and an agent
    ///
    ///     Necessary to run product commands
    ///
    fn setup(org_suffix: &str) {
        //run `grid keygen`
        let key_name: String = get_current_username().unwrap().into_string().unwrap();
        println!("key name: {}", &key_name);
        let mut key_dir: PathBuf = dirs::home_dir().unwrap();
        assert_eq!(PathBuf::from(KEY_DIR), key_dir);
        key_dir.push(".grid");
        key_dir.push("keys");
        key_dir.push(&key_name);
        let mut cmd_key = make_grid_command();
        cmd_key.arg("keygen").arg("--force");
        cmd_key.assert().success();
        let mut public_key_path = key_dir.clone();
        public_key_path.set_extension("pub");
        let mut private_key_path = key_dir.clone();
        private_key_path.set_extension("priv");
        assert!(public_key_path.exists());
        assert!(private_key_path.exists());

        //run `grid organization create`
        let mut cmd_org_create = make_grid_command();
        let org_id = format!("{}{}", &ORG_ID, &org_suffix);
        cmd_org_create
            .arg("organization")
            .arg("create")
            .arg(&org_id)
            .arg(&ORG_NAME)
            .arg(&ORG_ADDRESS)
            .args(&["--metadata", &format!("gs1_company_prefixes={}", &org_id)]);
        cmd_org_create.assert().success();

        //run `grid agent create`
        let mut pub_key = fs::read_to_string(PUB_KEY_FILE).unwrap();
        // key is read in with a trailing newline. This removes the newline
        pub_key.pop();
        let mut cmd_agent_update = make_grid_command();
        cmd_agent_update
            .arg("agent")
            .arg("update")
            .arg(&org_id)
            .arg(&pub_key)
            .arg("--active")
            .args(&["--role", "admin"])
            .args(&["--role", "can_create_product"])
            .args(&["--role", "can_update_product"])
            .args(&["--role", "can_delete_product"])
            .args(&["--role", "can_create_schema"])
            .args(&["--role", "can_update_schema"]);
        cmd_agent_update.assert().success();

        let mut cmd_schema_create = make_grid_command();
        cmd_schema_create
            .arg("schema")
            .arg("create")
            .arg(&SCHEMA_CREATE_FILE);
        cmd_schema_create.assert().success();
    }

    /// Makes a grid system command
    ///
    ///     Supplies the command with the grid server's URL from an environment variable
    ///
    fn make_grid_command() -> Command {
        let mut cmd = Command::cargo_bin("grid").unwrap();
        let url = env::var("INTEGRATION_TEST_URL").unwrap_or("http://gridd:8080".to_string());
        cmd.args(&["--url", &url])
            .args(&["--wait", "10000"])
            .arg("-vv");
        return cmd;
    }

    /// Gets a memoized setup
    ///
    ///     Ensures that the setup is only called once to avoid conflicts
    ///     between tests.
    fn get_setup() {
        INIT.call_once(|| {
            setup("test");
        })
    }
}
