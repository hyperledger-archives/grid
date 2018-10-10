// Copyright 2018 Cargill Incorporated
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

use wasmi::{
    Module,
    ModuleInstance,
    ImportsBuilder,
    RuntimeValue
};
use smart_permission_externals::{SmartPermissionExternals, ExternalsError};
use sawtooth_sdk::processor::handler::{TransactionContext};

pub struct SmartPermissionModule {
    context: TransactionContext,
    module: Module
}

impl SmartPermissionModule {
    pub fn new(wasm: &[u8], context: TransactionContext) -> Result<SmartPermissionModule, ExternalsError> {
        let module = Module::from_buffer(wasm)?;
        Ok(SmartPermissionModule { context, module })
    }

    pub fn entrypoint(
        &self,
        roles: Vec<String>,
        org_id: String,
        public_key: String,
        payload: Vec<u8>
    ) -> Result<Option<i32>, ExternalsError> {
        let mut env =  SmartPermissionExternals::new(None, self.context.clone())?;

        let instance = ModuleInstance::new(&self.module, &ImportsBuilder::new().with_resolver("env", &env))?
            .assert_no_start();

        info!("Writing roles to memory");

        let roles_write_results: Vec<Result<u32, ExternalsError>> = roles
            .into_iter()
            .map(|i| env.write_data(i.into_bytes()))
            .collect();

        let mut role_ptrs = Vec::new();

        for i in roles_write_results {
            if i.is_err() {
                return Err(i.unwrap_err());
            }
            role_ptrs.push(i.unwrap());
        }

        let role_list_ptr = if role_ptrs.len() > 0 {
            env.collect_ptrs(role_ptrs)? as i32
        } else {
            -1
        };

        info!("Roles written to memory: {:?}", role_list_ptr);

        let org_id_ptr = env.write_data(org_id.into_bytes())? as i32;
        info!("Organization ID written to memory");

        let public_key_ptr = env.write_data(public_key.into_bytes())? as i32;
        info!("Public key written to memory");

        let payload_ptr = env.write_data(payload)? as i32;
        info!("Payload written to memory");

        let result = instance
            .invoke_export(
                "entrypoint",
                &vec![
                    RuntimeValue::I32(role_list_ptr),
                    RuntimeValue::I32(org_id_ptr),
                    RuntimeValue::I32(public_key_ptr),
                    RuntimeValue::I32(payload_ptr)
                ],
                &mut env
            )?;

        if let Some(RuntimeValue::I32(i)) = result {
            Ok(Some(i))
        } else {
            Ok(None)
        }
    }

    pub fn execute(&mut self, func_name: &str, args: &[&[u8]]) -> Result<Option<i32>, ExternalsError> {
        let mut env =  SmartPermissionExternals::new(None, self.context.clone())?;

        let instance = ModuleInstance::new(&self.module, &ImportsBuilder::new().with_resolver("env", &env))?
            .assert_no_start();

        info!("Writing arguments to memory");

        let write_results: Vec<Result<u32, ExternalsError>> = args
            .iter()
            .map(|i| env.write_data(i.to_vec()))
            .collect();

        let mut export_args = Vec::new();
        for i in write_results {
            if i.is_err() {
                return Err(i.unwrap_err());
            }
            export_args.push(RuntimeValue::I32(i.unwrap() as i32))
        }

        info!("args written to memory {:?}", export_args);
        let result = instance
            .invoke_export(func_name, &export_args, &mut env)?;

        if let Some(RuntimeValue::I32(i)) = result {
            Ok(Some(i))
        } else {
            Ok(None)
        }
    }
}
