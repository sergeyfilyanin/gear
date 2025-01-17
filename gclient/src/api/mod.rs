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

pub mod calls;
pub mod error;
pub mod listener;
mod rpc;
pub mod storage;

use crate::{
    node::{ws::WSAddress, Node},
    EventListener,
};
use error::*;
use gsdk::{ext::sp_runtime::AccountId32, signer::Signer, Api};
use std::{marker::PhantomData, ops::Deref};

/// The API instance contains methods to access the node.
#[derive(Clone)]
pub struct GearApi(Signer);

impl GearApi {
    /// Create and init a new `GearApi` specified by its `address` on behalf of
    /// the default `Alice` user.
    pub async fn init(address: WSAddress) -> Result<Self> {
        Self::init_with(address, "//Alice").await
    }

    /// Create and init a new `GearApi` specified by its `address` and `suri`.
    ///
    /// SURI is a Substrate URI that identifies a user by a mnemonic phrase or
    /// provides default users from the keyring (e.g., "//Alice", "//Bob",
    /// etc.). The password for URI should be specified in the same `suri`,
    /// separated by the `':'` char.
    pub async fn init_with(address: WSAddress, suri: impl AsRef<str>) -> Result<Self> {
        let mut suri = suri.as_ref().splitn(2, ':');

        Api::new(Some(&address.url()))
            .await
            .and_then(|api| {
                Ok(Self(
                    api.signer(suri.next().expect("Infallible"), suri.next())?,
                ))
            })
            .map_err(Into::into)
    }

    /// Change SURI to the provided `suri` and return `Self`.
    pub fn with(self, suri: impl AsRef<str>) -> Result<Self> {
        let mut suri = suri.as_ref().splitn(2, ':');

        Ok(Self(
            self.0
                .change(suri.next().expect("Infallible"), suri.next())?,
        ))
    }

    /// Create and init a new `GearApi` instance that will be used with the
    /// local node working in developer mode (running with `--dev` argument).
    pub async fn dev() -> Result<Self> {
        Self::init(WSAddress::dev()).await
    }

    /// Create and init a new `GearApi` instance that will be used with the
    /// local node working in developer mode (running with `--dev` argument)
    /// and spawned programmatically via the `[crate::Node]` struct.
    pub async fn node(node: &Node) -> Result<GearApiWithNode> {
        let api = Self::init(node.ws_address().clone()).await?;
        Ok(GearApiWithNode {
            api,
            phantom: PhantomData,
        })
    }

    /// Create and init a new `GearApi` instance that will be used with the
    /// public Gear testnet.
    pub async fn gear() -> Result<Self> {
        Self::init(WSAddress::gear()).await
    }

    /// Create and init a new `GearApi` instance that will be used with the
    /// public Vara node.
    pub async fn vara() -> Result<Self> {
        Self::init(WSAddress::vara()).await
    }

    /// Create an [`EventListener`] to subscribe and handle continuously
    /// incoming events.
    pub async fn subscribe(&self) -> Result<EventListener> {
        let events = self.0.api().finalized_blocks().await?;
        Ok(EventListener(events))
    }

    /// Set the number used once (`nonce`) that will be used while sending
    /// extrinsics.
    ///
    /// If set, this nonce is added to the extrinsic and provides a unique
    /// identifier of the transaction sent by the current account. The nonce
    /// shows how many prior transactions have occurred from this account. This
    /// helps protect against replay attacks or accidental double-submissions.
    pub fn set_nonce(&mut self, nonce: u32) {
        self.0.set_nonce(nonce)
    }

    /// Get the next number used once (`nonce`) from the node.
    ///
    /// Actually sends the `system_accountNextIndex` RPC to the node.
    pub async fn rpc_nonce(&self) -> Result<u32> {
        self.0
            .api()
            .rpc()
            .system_account_next_index(self.0.account_id())
            .await
            .map_err(Into::into)
    }

    /// Return the signer account address.
    ///
    /// # Examples
    ///
    /// ```
    /// use gclient::{GearApi, Result};
    /// use gsdk::ext::sp_runtime::AccountId32;
    /// # use hex_literal::hex;
    ///
    /// #[tokio::test]
    /// async fn account_test() -> Result<()> {
    ///     let api = GearApi::dev().await?;
    ///     let alice_id = hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
    ///     assert_eq!(api.account_id(), &AccountId32::from(alice_id));
    ///     Ok(())
    /// }
    /// ```
    pub fn account_id(&self) -> &AccountId32 {
        self.0.account_id()
    }
}

/// A struct playing a role of smart pointer to a `GearApi` instance
/// created for working with a programmatically spawn node and preventing
/// the refernced node being destroyed before the instance.
pub struct GearApiWithNode<'a> {
    api: GearApi,
    phantom: PhantomData<&'a Node>,
}

impl Deref for GearApiWithNode<'_> {
    type Target = GearApi;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
