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

/// The helper to deal with WASM code.
pub struct Program {
    id: i32,
}

impl Program {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}
