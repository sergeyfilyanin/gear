// This file is part of Gear.

// Copyright (C) 2021 Gear Technologies Inc.
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

//! Gear message processor.

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![doc(html_logo_url = "https://docs.gear.rs/logo.svg")]

extern crate alloc;

pub mod common;
pub mod configs;
mod executor;
mod ext;
mod handler;
mod id;
mod lazy_pages;
mod processor;

/// Error exit code.
pub const ERR_EXIT_CODE: ExitCode = 1;
/// Destination doesn't exist anymore for the message
pub const TERMINATED_DEST_EXIT_CODE: ExitCode = 2;
/// A try to init again active program 
pub const RE_INIT_EXIT_CODE: ExitCode = 3;
/// Returned in 2 cases:
/// 1. Program tries to init program with non existing code hash
/// 2. Program tries to init terminated program.
pub const INIT_UNAVAILABLE_EXIT_CODE: ExitCode = 4;

pub use executor::execute_wasm;
pub use ext::Ext;
pub use handler::handle_journal;
pub use id::next_message_id;
pub use processor::{process, process_many};
use gear_core::message::ExitCode;
