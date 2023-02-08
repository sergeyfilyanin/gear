// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use gear_call_gen::{GearCall, SendMessageArgs, UploadProgramArgs};
use gear_common::{storage::Limiter, GasPrice};
use gear_core::{ids::ProgramId, utils::NonEmpty};
use gear_runtime::{Gear, Runtime, RuntimeOrigin};
use once_cell::sync::OnceCell;
use pallet_balances::Pallet as BalancesPallet;
use pallet_gear::{BlockGasLimitOf, Config, GasAllowanceOf};
use parking_lot::Mutex;
use rand::{rngs::SmallRng, Rng, RngCore, SeedableRng};
use runtime::*;
use sp_io::TestExternalities;

mod generator;
mod runtime;

// Saving ext is planned to be multithreaded, so sync primitive is used
// TODO #2189
static TEST_EXT: OnceCell<Mutex<TestExternalities>> = OnceCell::new();
static CONTEXT: OnceCell<Mutex<Context>> = OnceCell::new();

// TODO:
// 1. send_message
// 2. test panic reproduction
// 3. Fix gasAllowance put low value (249...000) before execution.
// 4. Change SmallRng to something more reproducible.
// 5. Logging to file
// 6. Cleanup

struct Context {
    programs: Vec<ProgramId>,
}

impl Context {
    fn new() -> Self {
        Self {
            programs: Vec::new(),
        }
    }
}

/// Runs all the fuzz testing internal machinery.
pub fn run(start_seed: u64) {
    init_logger();

    let sender = runtime::account(ALICE);
    let test_ext = TEST_EXT.get_or_init(|| Mutex::new(new_test_ext()));
    let context = CONTEXT.get_or_init(|| Mutex::new(Context::new()));

    test_ext.lock().execute_with(|| {
        let res = BalancesPallet::<Runtime>::set_balance(
            RuntimeOrigin::root(),
            sender.clone().into(),
            <Runtime as Config>::GasPrice::gas_price(BlockGasLimitOf::<Runtime>::get() * 20),
            BalancesPallet::<Runtime>::reserved_balance(&sender),
        );
        log::info!("Balance update res {:?}", res);

        GasAllowanceOf::<Runtime>::put(250_000_000_000);

        log::info!(
            "Balance of the sender {}",
            BalancesPallet::<Runtime>::free_balance(&sender)
        );
        log::info!(
            "Block gas limit in the beginning {}",
            GasAllowanceOf::<Runtime>::get()
        );

        let mut rand = SmallRng::seed_from_u64(start_seed);

        // Generate args
        let call: GearCall = match rand.gen_range(0..=1) {
            0 => UploadProgramArgs::generate::<SmallRng>(
                rand.next_u64(),
                rand.next_u64(),
                240_000_000_000,
            )
            .into(),
            1 => match NonEmpty::from_vec(context.lock().programs.clone()) {
                Some(existing_programs) => SendMessageArgs::generate::<SmallRng>(
                    existing_programs,
                    rand.next_u64(),
                    240_000_000_000,
                )
                .into(),
                None => UploadProgramArgs::generate::<SmallRng>(
                    rand.next_u64(),
                    rand.next_u64(),
                    240_000_000_000,
                )
                .into(),
            },
            _ => unreachable!(),
        };

        // Execute calls
        let call_res = match call {
            GearCall::UploadProgram(args) => {
                let UploadProgramArgs((code, salt, payload, gas_limit, value)) = args;
                Gear::upload_program(
                    RuntimeOrigin::signed(sender),
                    code,
                    salt,
                    payload,
                    gas_limit,
                    value,
                )
            }
            GearCall::SendMessage(args) => {
                let SendMessageArgs((destination, payload, gas_limit, value)) = args;
                Gear::send_message(
                    RuntimeOrigin::signed(sender),
                    destination,
                    payload,
                    gas_limit,
                    value,
                )
            }
            _ => unreachable!("unsupported currently"),
        };
        log::info!("Extrinsic result {call_res:?}");

        // TODO [sab] catch panic here and save the ext.
        run_to_next_block();

        // TODO [sab] parse events with newly created programs and e.t.c.
    });
}

fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .format_module_path(false)
        .format_level(true)
        .try_init();
}
