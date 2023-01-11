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

//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `epyc-runners-node.hetzner`, CPU: `AMD EPYC 7502P 32-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("gear-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/gear benchmark pallet --chain=gear-dev --steps=50 --repeat=20 --pallet=frame_system --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./scripts/benchmarking/weights-output/frame_system.rs --template=.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for frame_system.
pub trait WeightInfo {
    fn remark(b: u32, ) -> Weight;
    fn remark_with_event(b: u32, ) -> Weight;
    fn set_heap_pages() -> Weight;
    fn set_storage(i: u32, ) -> Weight;
    fn kill_storage(i: u32, ) -> Weight;
    fn kill_prefix(p: u32, ) -> Weight;
}

/// Weights for frame_system using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::WeightInfo for SubstrateWeight<T> {
    /// The range of component `b` is `[0, 1310720]`.
    fn remark(b: u32, ) -> Weight {
        Weight::from_ref_time(4_489_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(380 as u64).saturating_mul(b as u64))
    }
    /// The range of component `b` is `[0, 1310720]`.
    fn remark_with_event(b: u32, ) -> Weight {
        Weight::from_ref_time(15_541_000 as u64)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(1_665 as u64).saturating_mul(b as u64))
    }
    fn set_heap_pages() -> Weight {
        Weight::from_ref_time(10_401_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    /// The range of component `i` is `[1, 1000]`.
    fn set_storage(i: u32, ) -> Weight {
        Weight::from_ref_time(5_471_000 as u64)
            // Standard Error: 849
            .saturating_add(Weight::from_ref_time(813_934 as u64).saturating_mul(i as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    /// The range of component `i` is `[1, 1000]`.
    fn kill_storage(i: u32, ) -> Weight {
        Weight::from_ref_time(5_410_000 as u64)
            // Standard Error: 1_191
            .saturating_add(Weight::from_ref_time(692_734 as u64).saturating_mul(i as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    /// The range of component `p` is `[1, 1000]`.
    fn kill_prefix(p: u32, ) -> Weight {
        Weight::from_ref_time(8_156_000 as u64)
            // Standard Error: 1_534
            .saturating_add(Weight::from_ref_time(1_454_271 as u64).saturating_mul(p as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    /// The range of component `b` is `[0, 1310720]`.
    fn remark(b: u32, ) -> Weight {
        Weight::from_ref_time(4_489_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(380 as u64).saturating_mul(b as u64))
    }
    /// The range of component `b` is `[0, 1310720]`.
    fn remark_with_event(b: u32, ) -> Weight {
        Weight::from_ref_time(15_541_000 as u64)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(1_665 as u64).saturating_mul(b as u64))
    }
    fn set_heap_pages() -> Weight {
        Weight::from_ref_time(10_401_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    /// The range of component `i` is `[1, 1000]`.
    fn set_storage(i: u32, ) -> Weight {
        Weight::from_ref_time(5_471_000 as u64)
            // Standard Error: 849
            .saturating_add(Weight::from_ref_time(813_934 as u64).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    /// The range of component `i` is `[1, 1000]`.
    fn kill_storage(i: u32, ) -> Weight {
        Weight::from_ref_time(5_410_000 as u64)
            // Standard Error: 1_191
            .saturating_add(Weight::from_ref_time(692_734 as u64).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    /// The range of component `p` is `[1, 1000]`.
    fn kill_prefix(p: u32, ) -> Weight {
        Weight::from_ref_time(8_156_000 as u64)
            // Standard Error: 1_534
            .saturating_add(Weight::from_ref_time(1_454_271 as u64).saturating_mul(p as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
    }
}
