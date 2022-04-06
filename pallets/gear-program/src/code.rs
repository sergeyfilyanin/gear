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

use common::{CodeMetadata, CodeStorageTrait};
use gear_core::{code::{CheckedCode, CheckedCodeWithHash}, ids::CodeId};

use super::*;

impl<T: Config> CodeStorageTrait for pallet::CodeStorage<T> {
    fn try_new() -> Option<Self> {
        pallet::Pallet::<T>::try_new_code_storage()
    }

    fn add_code_impl(&mut self, code_and_hash: CheckedCodeWithHash, metadata: CodeMetadata) -> Result<(), common::CodeStorageErrorAlreadyExists> {
        pallet::Pallet::<T>::add_code(code_and_hash, metadata)
            .map_err(|_| common::CodeStorageErrorAlreadyExists)
    }

    fn exists_impl(&self, code_id: CodeId) -> Option<()> {
        if pallet::Pallet::<T>::code_exists(code_id) {
            Some(())
        } else {
            None
        }
    }

    fn remove_code_impl(&mut self, code_id: CodeId) -> Option<()> {
        pallet::Pallet::<T>::remove_code(code_id).ok()
    }

    fn get_checked_code(&self, code_id: CodeId) -> Option<CheckedCode> {
        pallet::Pallet::<T>::get_checked_code(code_id)
    }

    fn get_metadata(&self, code_id: CodeId) -> Option<CodeMetadata> {
        pallet::Pallet::<T>::get_metadata(code_id)
    }
}

impl<T: Config> Drop for pallet::CodeStorage<T> {
    fn drop(&mut self) {
        pallet::Pallet::<T>::enable_export();
    }
}

impl<T: Config> pallet::Pallet<T> {
    fn try_new_code_storage() -> Option<CodeStorage<T>> {
        if Self::storage_exported() {
            None
        } else {
            StorageExport::<T>::put(true);
            Some(CodeStorage(Default::default()))
        }
    }

    fn enable_export() {
        StorageExport::<T>::put(false);
    }

    fn add_code(code: CheckedCodeWithHash, metadata: CodeMetadata) -> Result<(), FailedToAddCode> {
        let (code, key) = code.into_parts();
        ProgramCodes::<T>::mutate(key, |maybe| {
            if maybe.is_some() {
                return Err(FailedToAddCode);
            }

            *maybe = Some(Code { code, metadata });
            Ok(())
        })
    }

    fn remove_code(code_id: CodeId) -> Result<(), PrgoramCodeNotFound> {
        ProgramCodes::<T>::mutate(code_id, |maybe| {
            if maybe.is_none() {
                return Err(PrgoramCodeNotFound);
            }

            *maybe = None;
            Ok(())
        })
    }

    fn code_exists(code_id: CodeId) -> bool {
        ProgramCodes::<T>::contains_key(code_id)
    }

    fn get_code(code_id: CodeId) -> Option<Code> {
        ProgramCodes::<T>::get(code_id)
    }

    fn get_checked_code(code_id: CodeId) -> Option<CheckedCode> {
        Self::get_code(code_id).map(|code| code.code)
    }

    fn get_metadata(code_id: CodeId) -> Option<CodeMetadata> {
        Self::get_code(code_id).map(|code| code.metadata)
    }
}
