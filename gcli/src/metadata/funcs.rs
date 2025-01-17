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

//! Host functions
use crate::metadata::{result::Result, StoreData};
use wasmtime::{
    AsContext, AsContextMut, Caller, Extern, Func, Linker, Memory, MemoryType, Store, Trap,
};

pub fn alloc(ctx: impl AsContextMut<Data = StoreData>, memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |mut caller: Caller<'_, StoreData>, pages: i32| {
            memory
                .clone()
                .grow(caller.as_context_mut(), pages as u64)
                .map_or_else(
                    |err| {
                        log::error!("{err:?}");
                        u32::MAX as i32
                    },
                    |pages| pages as i32,
                )
        },
    ))
}

pub fn free(ctx: impl AsContextMut<Data = StoreData>) -> Extern {
    Extern::Func(Func::wrap(ctx, |_: i32| 0))
}

pub fn gr_debug(ctx: impl AsContextMut<Data = StoreData>, memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |caller: Caller<'_, StoreData>, ptr: u32, len: i32| {
            let (ptr, len) = (ptr as usize, len as usize);

            let mut msg = vec![0; len];
            memory
                .clone()
                .read(caller.as_context(), ptr, &mut msg)
                .map_err(|e| {
                    log::error!("{:?}", e);
                    Trap::i32_exit(1)
                })?;

            log::debug!("{:?}", String::from_utf8_lossy(&msg));
            Ok(())
        },
    ))
}

pub fn gr_panic(ctx: impl AsContextMut<Data = StoreData>, memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |caller: Caller<'_, StoreData>, ptr: u32, len: i32| {
            let (ptr, len) = (ptr as usize, len as usize);

            let mut msg = vec![0; len];
            memory
                .clone()
                .read(caller.as_context(), ptr, &mut msg)
                .map_err(|e| {
                    log::error!("{:?}", e);
                    Trap::i32_exit(1)
                })?;

            log::error!("panic occurred: {:?}", String::from_utf8_lossy(&msg));
            Ok(())
        },
    ))
}

pub fn gr_oom_panic(ctx: impl AsContextMut<Data = StoreData>) -> Extern {
    Extern::Func(Func::wrap(ctx, move |_caller: Caller<'_, StoreData>| {
        log::error!("OOM panic occurred");
        Ok(())
    }))
}

pub fn gr_read(ctx: impl AsContextMut<Data = StoreData>, memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |mut caller: Caller<'_, StoreData>, at: u32, len: i32, buff: i32, err: i32| {
            let (at, len, buff, err) = (at as _, len as _, buff as _, err as _);

            let msg = &caller.data().msg;
            let mut payload = vec![0; len];
            if at + len <= msg.len() {
                payload.copy_from_slice(&msg[at..(at + len)]);
            } else {
                log::error!("overflow");
                return Err(Trap::i32_exit(1));
            }

            let len: u32 = memory
                .clone()
                .write(caller.as_context_mut(), buff, &payload)
                .map_err(|e| log::error!("{:?}", e))
                .is_err()
                .into();

            memory
                .clone()
                .write(caller.as_context_mut(), err, &len.to_le_bytes())
                .map_err(|e| {
                    log::error!("{:?}", e);
                    Trap::i32_exit(1)
                })?;

            Ok(())
        },
    ))
}

/// # NOTE
///
/// Just for the compatible with the program metadata
pub fn gr_reply(ctx: impl AsContextMut<Data = StoreData>, _memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |mut _caller: Caller<'_, StoreData>,
              _ptr: u32,
              _len: i32,
              _val: i32,
              _delay: i32,
              _msg: i32| Ok(()),
    ))
}

/// # NOTE
///
/// Just for the compatible with the program metadata
pub fn gr_error(ctx: impl AsContextMut<Data = StoreData>, _memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |mut _caller: Caller<'_, StoreData>, _ptr: u32, _err_ptr: u32| Ok(()),
    ))
}

pub fn gr_size(ctx: impl AsContextMut<Data = StoreData>, memory: Memory) -> Extern {
    Extern::Func(Func::wrap(
        ctx,
        move |mut caller: Caller<'_, StoreData>, size_ptr: u32| {
            let size = caller.data().msg.len() as u32;

            memory
                .clone()
                .write(
                    caller.as_context_mut(),
                    size_ptr as usize,
                    &size.to_le_bytes(),
                )
                .map_err(|e| {
                    log::error!("{:?}", e);
                    Trap::i32_exit(1)
                })?;

            Ok(())
        },
    ))
}
