// This file is part of Gear.

// Copyright (C) 2022 Gear Technologies Inc.
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

//! Autogenerated weights for pallet_utility
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
<<<<<<< HEAD
//! DATE: 2022-09-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `epyc-runners-node.hetzner`, CPU: `AMD EPYC 7502P 32-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vara-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/gear benchmark pallet --chain=vara-dev --steps=50 --repeat=20 --pallet=pallet_utility --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./scripts/benchmarking/weights-output/pallet_utility.rs --template=.maintain/frame-weight-template.hbs
=======
//! DATE: 2022-09-07, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vara-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/gear-node benchmark pallet --chain=vara-dev --steps=50 --repeat=20 --pallet=pallet_utility --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./scripts/benchmarking/weights-output/pallet_utility.rs --template=.maintain/frame-weight-template.hbs
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_utility.
pub trait WeightInfo {
    fn batch(c: u32, ) -> Weight;
    fn as_derivative() -> Weight;
    fn batch_all(c: u32, ) -> Weight;
    fn dispatch_as() -> Weight;
    fn force_batch(c: u32, ) -> Weight;
}

/// Weights for pallet_utility using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for SubstrateWeight<T> {
<<<<<<< HEAD
    /// The range of component `c` is `[0, 1000]`.
    fn batch(c: u32, ) -> Weight {
        Weight::from_ref_time(14_950_000 as u64)
            // Standard Error: 792
            .saturating_add(Weight::from_ref_time(4_866_852 as u64).saturating_mul(c as u64))
    }
    fn as_derivative() -> Weight {
        Weight::from_ref_time(7_575_000 as u64)
    }
    /// The range of component `c` is `[0, 1000]`.
    fn batch_all(c: u32, ) -> Weight {
        Weight::from_ref_time(14_940_000 as u64)
            // Standard Error: 978
            .saturating_add(Weight::from_ref_time(5_026_927 as u64).saturating_mul(c as u64))
    }
    fn dispatch_as() -> Weight {
        Weight::from_ref_time(17_245_000 as u64)
    }
    /// The range of component `c` is `[0, 1000]`.
    fn force_batch(c: u32, ) -> Weight {
        Weight::from_ref_time(14_810_000 as u64)
            // Standard Error: 848
            .saturating_add(Weight::from_ref_time(4_867_459 as u64).saturating_mul(c as u64))
=======
    fn batch(c: u32, ) -> Weight {
        (45_711_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((3_966_000 as Weight).saturating_mul(c as Weight))
    }
    fn as_derivative() -> Weight {
        (4_920_000 as Weight)
    }
    fn batch_all(c: u32, ) -> Weight {
        (23_613_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((4_134_000 as Weight).saturating_mul(c as Weight))
    }
    fn dispatch_as() -> Weight {
        (13_527_000 as Weight)
    }
    fn force_batch(c: u32, ) -> Weight {
        (22_929_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((3_998_000 as Weight).saturating_mul(c as Weight))
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
<<<<<<< HEAD
    /// The range of component `c` is `[0, 1000]`.
    fn batch(c: u32, ) -> Weight {
        Weight::from_ref_time(14_950_000 as u64)
            // Standard Error: 792
            .saturating_add(Weight::from_ref_time(4_866_852 as u64).saturating_mul(c as u64))
    }
    fn as_derivative() -> Weight {
        Weight::from_ref_time(7_575_000 as u64)
    }
    /// The range of component `c` is `[0, 1000]`.
    fn batch_all(c: u32, ) -> Weight {
        Weight::from_ref_time(14_940_000 as u64)
            // Standard Error: 978
            .saturating_add(Weight::from_ref_time(5_026_927 as u64).saturating_mul(c as u64))
    }
    fn dispatch_as() -> Weight {
        Weight::from_ref_time(17_245_000 as u64)
    }
    /// The range of component `c` is `[0, 1000]`.
    fn force_batch(c: u32, ) -> Weight {
        Weight::from_ref_time(14_810_000 as u64)
            // Standard Error: 848
            .saturating_add(Weight::from_ref_time(4_867_459 as u64).saturating_mul(c as u64))
    }
}
=======
    fn batch(c: u32, ) -> Weight {
        (45_711_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((3_966_000 as Weight).saturating_mul(c as Weight))
    }
    fn as_derivative() -> Weight {
        (4_920_000 as Weight)
    }
    fn batch_all(c: u32, ) -> Weight {
        (23_613_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((4_134_000 as Weight).saturating_mul(c as Weight))
    }
    fn dispatch_as() -> Weight {
        (13_527_000 as Weight)
    }
    fn force_batch(c: u32, ) -> Weight {
        (22_929_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((3_998_000 as Weight).saturating_mul(c as Weight))
    }
}
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
