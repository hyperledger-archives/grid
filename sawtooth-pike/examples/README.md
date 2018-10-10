#Smart Permission Boilerplate

```
extern crate wasm_sdk;

use wasm_sdk::{WasmPtr, WasmPtrList, execute_entrypoint, WasmSdkError, Request};

fn has_permission(request: Request) -> Result<bool, WasmSdkError> {
    // Code describing permission
}

#[no_mangle]
pub unsafe fn entrypoint(roles: WasmPtrList, org_id: WasmPtr, public_key: WasmPtr) -> i32 {
    execute_entrypoint(roles, org_id, public_key, has_permission)
}
```
