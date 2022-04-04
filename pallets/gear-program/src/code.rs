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

use common::CodeMetadata;
use gear_core::{code::{CheckedCode, CheckedCodeWithHash}, ids::CodeId};

use super::*;

impl<T: Config> pallet::Pallet<T> {
    pub fn add_code(code: CheckedCodeWithHash, metadata: CodeMetadata) -> Result<(), FailedToAddCode> {
        let (code, key) = code.into_parts();
        ProgramCodes::<T>::mutate(key, |maybe| {
            if maybe.is_some() {
                return Err(FailedToAddCode);
            }

            *maybe = Some(Code { code, metadata });
            Ok(())
        })
    }

    pub fn remove_code(code_id: CodeId) -> Result<(), PrgoramCodeNotFound> {
        ProgramCodes::<T>::mutate(code_id, |maybe| {
            if maybe.is_none() {
                return Err(PrgoramCodeNotFound);
            }

            *maybe = None;
            Ok(())
        })
    }

    pub fn code_exists(code_id: CodeId) -> bool {
        ProgramCodes::<T>::contains_key(code_id)
    }

    pub fn get_code(code_id: CodeId) -> Option<Code> {
        ProgramCodes::<T>::get(code_id)
    }

    pub fn get_checked_code(code_id: CodeId) -> Option<CheckedCode> {
        Self::get_code(code_id).map(|code| code.code)
    }

    pub fn get_metadata(code_id: CodeId) -> Option<CodeMetadata> {
        Self::get_code(code_id).map(|code| code.metadata)
    }
}
