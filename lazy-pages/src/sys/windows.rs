/*
 * This file is part of Gear.
 *
 * Copyright (C) 2022 Gear Technologies Inc.
 * SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)
use crate::{
    sys::{ExceptionInfo, UserSignalHandler},
    Error,
};
<<<<<<< HEAD
=======
use crate::sys::{ExceptionInfo, UserSignalHandler};
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
=======
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)
use std::io;
use winapi::{
    shared::ntdef::LONG,
    um::{
        errhandlingapi::AddVectoredExceptionHandler, minwinbase::EXCEPTION_ACCESS_VIOLATION,
        winnt::EXCEPTION_POINTERS,
    },
    vc::excpt::{EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH},
};

unsafe extern "system" fn exception_handler<H>(exception_info: *mut EXCEPTION_POINTERS) -> LONG
where
    H: UserSignalHandler,
{
    let exception_record = (*exception_info).ExceptionRecord;

    let is_access_violation = (*exception_record).ExceptionCode == EXCEPTION_ACCESS_VIOLATION;
    let num_params = (*exception_record).NumberParameters;
    if !is_access_violation || num_params != 2 {
        log::trace!(
            "Skip exception in handler: is access violation: {}, parameters: {}",
            is_access_violation,
            num_params
        );
        return EXCEPTION_CONTINUE_SEARCH;
    }

    let addr = (*exception_record).ExceptionInformation[1];
    let is_write = match (*exception_record).ExceptionInformation[0] {
        0 /* read */ => Some(false),
        1 /* write */ => Some(true),
        // we work with WASM memory which is handled by WASM executor
        // (e.g. it reads and writes, but doesn't execute as native code)
        // that's why the case is impossible
        8 /* DEP */ => unreachable!("data execution prevention"),
<<<<<<< HEAD
<<<<<<< HEAD
        // existence of other values is undocumented and I expect they should be treated as reserved
=======
        // existence of other values is undocumented and I expect they should be treated as reserved 
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
=======
        // existence of other values is undocumented and I expect they should be treated as reserved
>>>>>>> 1a441afd (Vara: merge master (#1529))
        _ => None,
    };
    let info = ExceptionInfo {
        fault_addr: addr as *mut _,
        is_write,
    };

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)
    if let Err(err) = H::handle(info) {
        if let Error::SignalFromUnknownMemory { .. } | Error::WasmMemAddrIsNotSet = err {
            return EXCEPTION_CONTINUE_SEARCH;
        } else {
            panic!("Signal handler failed: {}", err);
        }
    }
<<<<<<< HEAD
=======
    H::handle(info)
        .map_err(|err| err.to_string())
        .expect("Memory exception handler failed");
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
=======
>>>>>>> 4ca47efe (Merge branch 'master' into vara-stage-1)

    EXCEPTION_CONTINUE_EXECUTION
}

<<<<<<< HEAD
<<<<<<< HEAD
pub(crate) unsafe fn init_for_thread() -> Result<(), String> {
=======
pub unsafe fn setup_signal_handler<H>() -> io::Result<()>
where
    H: UserSignalHandler,
{
    SetUnhandledExceptionFilter(Some(exception_handler::<H>));
>>>>>>> 4ff7e31a (Vara: Update stage 1 to latest master (#1464))
    Ok(())
=======
pub(crate) unsafe fn init_for_thread() -> Result<(), String> {
    Ok(())
}

pub(crate) unsafe fn setup_signal_handler<H>() -> io::Result<()>
where
    H: UserSignalHandler,
{
    const CALL_FIRST: bool = true;

    let handle = AddVectoredExceptionHandler(CALL_FIRST as _, Some(exception_handler::<H>));
    if handle.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
>>>>>>> 1a441afd (Vara: merge master (#1529))
}

pub(crate) unsafe fn setup_signal_handler<H>() -> io::Result<()>
where
    H: UserSignalHandler,
{
    const CALL_FIRST: bool = true;

    let handle = AddVectoredExceptionHandler(CALL_FIRST as _, Some(exception_handler::<H>));
    if handle.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
