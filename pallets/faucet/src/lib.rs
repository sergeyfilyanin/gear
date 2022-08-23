// This file is part of Gear.

// Copyright (C) 2021-2022 Gear Technologies Inc.
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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchError, pallet_prelude::*};
pub use pallet::*;
pub use primitive_types::H256;
use runtime_primitives::{AccountId, Balance, BlockNumber};
use sp_std::convert::TryInto;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The upper bound of how much funds can be minted for account per time interval
        #[pallet::constant]
        type MintLimit: Get<Balance>;

        #[pallet::constant]
        type ResetInterval: Get<BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FundsMinted {
            account: AccountId,
            amount: Balance,
        },
    }

    // Gas pallet error.
    #[pallet::error]
    pub enum Error<T> {
        LimitPerIntervalExceeded,
    }

    // Private storage for total issuance of gas.
    #[pallet::storage]
    pub type TotalIssuance<T> = StorageValue<_, Balance>;

    // Public wrap of the total issuance of gas.
    common::wrap_storage_value!(
        storage: TotalIssuance,
        name: TotalIssuanceWrap,
        value: Balance
    );

    // ----

    pub type Key = MessageId;
    pub type NodeOf<T> = GasNode<AccountIdOf<T>, Key, Balance>;

    // Private storage for nodes of the gas tree.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type GasNodes<T> = StorageMap<_, Identity, Key, NodeOf<T>>;

    // Public wrap of the nodes of the gas tree.
    common::wrap_storage_map!(
        storage: GasNodes,
        name: GasNodesWrap,
        key: Key,
        value: NodeOf<T>
    );

    // ----

    #[pallet::storage]
    pub type Allowance<T> = StorageValue<_, Balance, ValueQuery, BlockGasLimitOf<T>>;

    pub struct GasAllowance<T: Config>(PhantomData<T>);

    impl<T: Config> common::storage::Limiter for GasAllowance<T> {
        type Value = Balance;

        fn get() -> Self::Value {
            Allowance::<T>::get()
        }

        fn put(gas: Self::Value) {
            Allowance::<T>::put(gas);
        }

        fn decrease(gas: Self::Value) {
            Allowance::<T>::mutate(|v| *v = v.saturating_sub(gas));
        }
    }

    impl<T: Config> GasProvider for Pallet<T> {
        type ExternalOrigin = AccountIdOf<T>;
        type Key = Key;
        type Balance = Balance;
        type PositiveImbalance = PositiveImbalance<Self::Balance, TotalIssuanceWrap<T>>;
        type NegativeImbalance = NegativeImbalance<Self::Balance, TotalIssuanceWrap<T>>;
        type InternalError = Error<T>;
        type Error = DispatchError;

        type GasTree = TreeImpl<
            TotalIssuanceWrap<T>,
            Self::InternalError,
            Self::Error,
            Self::ExternalOrigin,
            GasNodesWrap<T>,
        >;
    }

    impl<T: Config> BlockLimiter for Pallet<T> {
        type BlockGasLimit = BlockGasLimitOf<T>;

        type Balance = Balance;

        type GasAllowance = GasAllowance<T>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Initialization
        fn on_initialize(_bn: BlockNumberFor<T>) -> Weight {
            // Reset block gas allowance
            Allowance::<T>::put(BlockGasLimitOf::<T>::get());

            T::DbWeight::get().writes(1)
        }

        /// Finalization
        fn on_finalize(_bn: BlockNumberFor<T>) {}
    }
}
