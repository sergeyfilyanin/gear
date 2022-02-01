//! A simple example of `create_program` sys-call.
//!
//! The program is mainly used for testing the sys-call logic in pallet `gear` tests.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(not(feature = "std"))]
use gstd::prelude::*;

#[cfg(feature = "std")]
mod native {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}

#[cfg(feature = "std")]
pub use native::{WASM_BINARY, WASM_BINARY_BLOATY};

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum CreateProgram {
    Default(bool),
    // code hash, salt, gas limit
    Custom(Vec<([u8; 32], Vec<u8>, u64)>),
}

#[cfg(not(feature = "std"))]
mod wasm {
    use gstd::{debug, msg, CodeHash};

    use super::CreateProgram;

    static mut COUNTER: i32 = 0;

    fn increase() {
        unsafe {
            COUNTER += 1;
        }
    }

    fn get() -> i32 {
        unsafe { COUNTER }
    }

    #[no_mangle]
    pub unsafe extern "C" fn handle() {
        match msg::load().expect("provided invalid payload") {
            CreateProgram::Default(is_handle) => {
                let submitted_code = hex_literal::hex!(
                    "abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a"
                )
                .into();
                let new_program_id =
                    msg::create_program(submitted_code, get().to_le_bytes(), [], 10_000, 0);
                if is_handle {
                    msg::send(new_program_id, b"", 10_001, 0);
                } else {
                    msg::reply(b"", 10_001, 0);
                }

                increase();
            }
            CreateProgram::Custom(custom_child_data) => {
                for (code_hash, salt, gas_limit) in custom_child_data {
                    let submitted_code = code_hash.into();
                    let new_program_id =
                        msg::create_program(submitted_code, &salt, [], gas_limit, 0);
                    let msg_id = msg::send(new_program_id, b"", 10_001, 0);
                }
            }
        };
    }

    #[no_mangle]
    pub unsafe extern "C" fn init() {}
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::native;

    use common::RunnerContext;

    // todo [sab] write some simple tests

    #[test]
    fn binary_available() {
        assert!(native::WASM_BINARY.is_some());
        assert!(native::WASM_BINARY_BLOATY.is_some());
    }

    fn wasm_code() -> &'static [u8] {
        native::WASM_BINARY_BLOATY.expect("wasm binary exists")
    }
}
