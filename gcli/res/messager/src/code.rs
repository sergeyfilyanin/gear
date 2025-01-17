// This file is part of Gear.
//
// Copyright (C) 2021-2022 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{REPLY_REPLY, SEND_REPLY, SENT_VALUE};
use gstd::msg;

#[no_mangle]
extern "C" fn init() {
    let maybe_to = msg::load_bytes().unwrap();
    let to = if maybe_to.len() == 32 {
        let mut to = [0; 32];
        to.copy_from_slice(&maybe_to);
        to.into()
    } else {
        msg::source()
    };
    msg::send_bytes(to, [], SENT_VALUE).expect("Failed to send message");
}

#[no_mangle]
extern "C" fn handle() {
    msg::reply(SEND_REPLY, 0).unwrap();
}

#[no_mangle]
extern "C" fn handle_reply() {
    msg::reply(REPLY_REPLY, 0).unwrap();
}
