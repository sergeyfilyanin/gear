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

use std::path::Path;

use crate::{code::Code, manager::Manager, user::User};

/// The entry point to the testing framework.
pub struct System {
    manager: Manager,
}

impl System {
    pub fn new() -> Self {
        Self {
            manager: Manager::new(),
        }
    }

    pub fn submit_code(&self, _path: impl AsRef<Path>) -> Code {
        Code::new()
    }

    pub fn current_code(&self) -> Code {
        Code::new()
    }

    pub fn user(&self) -> User {
        User::new()
    }
}
