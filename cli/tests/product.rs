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
use users::get_current_username;

mod integration {
    use super::*;
    static KEY_DIR: &str = "/root";
    static PUB_KEY_FILE: &str = "/root/.grid/keys/root.pub";

    static ORG_ID: &str = "314156";
    static ORG_NAME: &str = "target";
    static ORG_ADDRESS: &str = "target hq";

    static PRODUCT_CREATE_FILE: &str = "tests/yaml/test_product_create.yaml";
    static PRODUCT_UPDATE_FILE: &str = "tests/yaml/test_product_update.yaml";

    static PRODUCT_DELETE_ID: &str = "762111177704";

    /// Verifies a `grid product create` command successfully runs.
    ///
    ///     The product information is read in from a yaml file.
    ///
    #[test]
    fn test_product_create() {
        setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .arg(&PRODUCT_CREATE_FILE);
        cmd_product_create.assert().success();
    }

    /// Verifies a `grid product update` command successfully runs.
    ///
    ///     The product information is read in from a yaml file.
    ///     Products are first created before being updated.
    ///
    #[test]
    fn test_product_update() {
        setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .arg(&PRODUCT_CREATE_FILE);
        cmd_product_create.assert().success();

        //run `grid product update`
        let mut cmd_product_update = make_grid_command();
        cmd_product_update
            .arg("product")
            .arg("update")
            .arg(&PRODUCT_UPDATE_FILE);
        cmd_product_update.assert().success();
    }

    /// Verifies a `grid product delete` command successfully runs.
    ///
    ///     The delete command is supplied the product id and type.
    ///     Products are first created before being deleted.
    ///
    #[test]
    fn test_product_delete() {
        setup();
        //run `grid product create`
        let mut cmd_product_create = make_grid_command();
        cmd_product_create
            .arg("product")
            .arg("create")
            .arg(&PRODUCT_CREATE_FILE);
        cmd_product_create.assert().success();

        //run `grid product delete`
        let mut cmd_product_delete = make_grid_command();
        cmd_product_delete
            .arg("product")
            .arg("delete")
            .arg(&PRODUCT_DELETE_ID)
            .arg("GS1"); //product type
        cmd_product_delete.assert().success();
    }

    /// Creates keys, an organization, and an agent
    ///
    ///     Necessary to run product commands
    ///
    fn setup() {
        //run `grid keygen`
        let key_name: String = get_current_username().unwrap().into_string().unwrap();
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
        cmd_org_create
            .arg("organization")
            .arg("create")
            .arg(&ORG_ID)
            .arg(&ORG_NAME)
            .arg(&ORG_ADDRESS)
            .args(&["--metadata", "gs1_company_prefixes=314"]);
        cmd_org_create.assert().success();

        //run `grid agent create`
        let pub_key = fs::read_to_string(PUB_KEY_FILE).unwrap();
        let mut cmd_agent_create = make_grid_command();
        cmd_agent_create
            .arg("agent")
            .arg("create")
            .arg(&ORG_ID)
            .arg(&pub_key)
            .arg("true")
            .args(&[
                "--roles",
                "admin",
                "can_create_product",
                "can_update_product",
                "can_delete_product",
            ]);
        cmd_agent_create.assert().success();
    }

    /// Makes a grid system command
    ///
    ///     Supplies the command with the grid server's URL from an environment variable
    ///
    fn make_grid_command() -> Command {
        let mut cmd = Command::cargo_bin("grid").unwrap();
        let url = env::var("INTEGRATION_TEST_URL").unwrap_or("http://gridd:8080".to_string());
        cmd.args(&["--url", &url]).arg("-vv");
        return cmd;
    }
}
