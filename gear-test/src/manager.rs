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

use core_processor::common::*;
use gear_core::{
    memory::PageNumber,
    message::{Dispatch, DispatchKind, Message, MessageId},
    program::{CodeHash, Program, ProgramId},
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::check::ExecutionContext;

#[derive(Clone, Default)]
pub struct InMemoryExtManager {
    codes: RefCell<BTreeMap<CodeHash, Vec<u8>>>,
    marked_destinations: BTreeSet<ProgramId>,
    dispatch_queue: VecDeque<Dispatch>,
    log: Vec<Message>,
    programs: RefCell<BTreeMap<ProgramId, Option<Program>>>,
    waiting_init: RefCell<BTreeMap<ProgramId, Vec<MessageId>>>,
    wait_list: BTreeMap<(ProgramId, MessageId), Dispatch>,
    current_failed: bool,
}

impl InMemoryExtManager {
    fn move_waiting_msgs_to_queue(&mut self, program_id: ProgramId) {
        let waiting_messages = self.waiting_init.borrow_mut().remove(&program_id);
        for m_id in waiting_messages.iter().flatten() {
            if let Some(dispatch) = self.wait_list.remove(&(program_id, *m_id)) {
                self.dispatch_queue.push_back(dispatch);
            }
        }
    }
}

impl ExecutionContext for InMemoryExtManager {
    fn store_program(&self, program: gear_core::program::Program, _init_message_id: MessageId) {
        self.waiting_init.borrow_mut().insert(program.id(), vec![]);
        let code_hash = sp_io::hashing::blake2_256(program.code()).into();
        if !self.codes.borrow().contains_key(&code_hash) {
            self.codes
                .borrow_mut()
                .insert(code_hash, program.code().to_vec());
        }
        self.programs
            .borrow_mut()
            .insert(program.id(), Some(program));
    }
}

impl CollectState for InMemoryExtManager {
    fn collect(&self) -> State {
        let InMemoryExtManager {
            dispatch_queue,
            log,
            programs,
            current_failed,
            ..
        } = self.clone();

        let programs = programs
            .into_inner()
            .into_iter()
            .filter_map(|(id, p_opt)| p_opt.map(|p| (id, p)))
            .collect();

        State {
            dispatch_queue,
            log,
            programs,
            current_failed,
        }
    }
}

impl JournalHandler for InMemoryExtManager {
    fn message_dispatched(&mut self, outcome: DispatchOutcome) {
        self.current_failed = match outcome {
            DispatchOutcome::MessageTrap { .. } => true,
            DispatchOutcome::InitFailure { program_id, .. } => {
                self.move_waiting_msgs_to_queue(program_id);
                if let Some(prog) = self.programs.borrow_mut().get_mut(&program_id) {
                    // Program is now considered terminated (in opposite to active). But not deleted from the state.
                    *prog = None;
                }
                true
            }
            DispatchOutcome::Success(_) | DispatchOutcome::NoExecution(_) => false,
            DispatchOutcome::InitSuccess { program_id, .. } => {
                if let Some(Some(program)) = self.programs.borrow_mut().get_mut(&program_id) {
                    program.set_initialized();
                }
                self.move_waiting_msgs_to_queue(program_id);
                false
            }
        };
    }
    fn gas_burned(&mut self, _message_id: MessageId, _origin: ProgramId, _amount: u64) {}

    fn exit_dispatch(&mut self, id_exited: ProgramId, _value_destination: ProgramId) {
        self.programs.borrow_mut().remove(&id_exited);
    }

    fn message_consumed(&mut self, message_id: MessageId) {
        if let Some(index) = self
            .dispatch_queue
            .iter()
            .position(|d| d.message.id() == message_id)
        {
            self.dispatch_queue.remove(index);
        }
    }
    fn send_dispatch(&mut self, _message_id: MessageId, dispatch: Dispatch) {
        let dest = dispatch.message.dest();
        if self.programs.borrow().contains_key(&dest) || self.marked_destinations.contains(&dest) {
            // Find in dispatch queue init message to the destination. By that we recognize
            // messages to not yet initialized programs, whose init messages were executed.
            let init_to_dest = self
                .dispatch_queue
                .iter()
                .find(|d| d.message.dest() == dest && d.kind == DispatchKind::Init);
            if let (DispatchKind::Handle, Some(list), None) = (
                dispatch.kind,
                self.waiting_init.borrow_mut().get_mut(&dest),
                init_to_dest,
            ) {
                let message_id = dispatch.message.id();
                list.push(message_id);
                self.wait_list.insert((dest, message_id), dispatch);
            } else {
                self.dispatch_queue.push_back(dispatch);
            }
        } else {
            self.log.push(dispatch.message);
        }
    }
    fn wait_dispatch(&mut self, dispatch: Dispatch) {
        self.message_consumed(dispatch.message.id());
        self.wait_list
            .insert((dispatch.message.dest(), dispatch.message.id()), dispatch);
    }
    fn wake_message(
        &mut self,
        _message_id: MessageId,
        program_id: ProgramId,
        awakening_id: MessageId,
    ) {
        if let Some(dispatch) = self.wait_list.remove(&(program_id, awakening_id)) {
            self.dispatch_queue.push_back(dispatch);
        }
    }
    fn update_nonce(&mut self, program_id: ProgramId, nonce: u64) {
        let mut programs = self.programs.borrow_mut();
        if let Some(prog) = programs
            .get_mut(&program_id)
            .expect("Program not found in storage")
        {
            prog.set_message_nonce(nonce);
        }
    }
    fn update_page(
        &mut self,
        program_id: ProgramId,
        page_number: PageNumber,
        data: Option<Vec<u8>>,
    ) {
        let mut programs = self.programs.borrow_mut();
        if let Some(prog) = programs
            .get_mut(&program_id)
            .expect("Program not found in storage")
        {
            if let Some(data) = data {
                let _ = prog.set_page(page_number, &data);
            } else {
                prog.remove_page(page_number);
            }
        } else {
            unreachable!("Can't update page for terminated program");
        }
    }
    fn send_value(&mut self, _from: ProgramId, _to: Option<ProgramId>, _value: u128) {
        // TODO https://github.com/gear-tech/gear/issues/644
    }

    fn store_new_programs(&mut self, code_hash: CodeHash, candidates: Vec<(ProgramId, MessageId)>) {
        if let Some(code) = self.codes.borrow().get(&code_hash) {
            for (candidate_id, init_message_id) in candidates {
                if !self.programs.borrow().contains_key(&candidate_id) {
                    let program = Program::new(candidate_id, code.clone())
                        .expect("guaranteed to have constructable code");
                    self.store_program(program, init_message_id);
                } else {
                    log::debug!("Program with id {:?} already exists", candidate_id);
                }
            }
        } else {
            log::debug!(
                "No referencing code with code hash {:?} for candidate programs",
                code_hash
            );
            for (invalid_candidate, _) in candidates {
                self.marked_destinations.insert(invalid_candidate);
            }
        }
    }
}
