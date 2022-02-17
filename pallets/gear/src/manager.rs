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

use crate::{
    pallet::Reason, Authorship, Config, DispatchOutcome, Event, ExecutionResult, MessageInfo,
    Pallet, ProgramsLimbo,
};
use codec::{Decode, Encode};
use common::{
    value_tree::{ConsumeResult, ValueView},
    GasToFeeConverter, Origin, GAS_VALUE_PREFIX, STORAGE_PROGRAM_CANDIDATE_PREFIX,
    STORAGE_PROGRAM_PREFIX,
};
use core_processor::common::{
    CollectState, Dispatch, DispatchOutcome as CoreDispatchOutcome, JournalHandler, State,
};
use frame_support::{
    storage::PrefixIterator,
    traits::{BalanceStatus, Currency, ExistenceRequirement, ReservableCurrency},
};
use gear_core::{
    memory::PageNumber,
    message::{ExitCode, Message, MessageId},
    program::{CodeHash, Program, ProgramId},
};
use primitive_types::H256;
use sp_runtime::traits::{UniqueSaturatedInto, Zero};
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet, vec_deque::VecDeque},
    iter::FromIterator,
    marker::PhantomData,
    prelude::*,
};
use codec::Decode;

pub struct ExtManager<T: Config, GH: GasHandler = ValueTreeGasHandler> {
    _phantom: PhantomData<T>,
    gas_handler: GH,
    skip_messages: BTreeSet<SkipMessageKind>,
}

// todo [sab] REMOVE, forbidden
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SkipMessageKind {
    // Skip message with `MessageId`
    //
    // Such message is skipped, because it tries to initialize
    // a program with existing address. See [`ExtManager::bind_programs_to_code_hash`] for more info.
    WithId(MessageId),
    // Skip all messages to the destination, which doesn't have any code
    //
    // More precisely, code hash which was intended to be bound
    // to the `ProgramId` doesn't have any underlying code
    WithDestination(ProgramId),
}

#[derive(Decode, Encode)]
pub enum HandleKind {
    Init(Vec<u8>),
    Handle(H256),
    Reply(H256, ExitCode),
}

pub trait GasHandler {
    fn spend(&mut self, message_id: H256, amount: u64);
    fn consume(&mut self, message_id: H256) -> ConsumeResult;
    fn split(&mut self, message_id: H256, at: H256, amount: u64);
}

#[derive(Default)]
pub struct ValueTreeGasHandler;

impl GasHandler for ValueTreeGasHandler {
    fn spend(&mut self, message_id: H256, amount: u64) {
        if let Some(mut gas_tree) = ValueView::get(GAS_VALUE_PREFIX, message_id) {
            gas_tree.spend(amount);
        } else {
            log::error!(
                "Message does not have associated gas tree: {:?}",
                message_id
            );
        }
    }

    fn consume(&mut self, message_id: H256) -> ConsumeResult {
        if let Some(gas_tree) = ValueView::get(GAS_VALUE_PREFIX, message_id) {
            gas_tree.consume()
        } else {
            log::error!(
                "Message does not have associated gas tree: {:?}",
                message_id
            );

            ConsumeResult::None
        }
    }

    fn split(&mut self, message_id: H256, at: H256, amount: u64) {
        if let Some(mut gas_tree) = ValueView::get(GAS_VALUE_PREFIX, message_id) {
            let _ = gas_tree.split_off(at, amount);
        } else {
            log::error!(
                "Message does not have associated gas tree: {:?}",
                message_id
            );
        }
    }
}

impl GasHandler for () {
    fn spend(&mut self, _message_id: H256, _amount: u64) {}

    fn consume(&mut self, _message_id: H256) -> ConsumeResult {
        ConsumeResult::None
    }

    fn split(&mut self, _message_id: H256, _at: H256, _amount: u64) {}
}

impl<T, GH> CollectState for ExtManager<T, GH>
where
    T: Config,
    T::AccountId: Origin,
    GH: GasHandler,
{
    fn collect(&self) -> State {
        let programs: BTreeMap<ProgramId, Program> = PrefixIterator::<H256>::new(
            STORAGE_PROGRAM_PREFIX.to_vec(),
            STORAGE_PROGRAM_PREFIX.to_vec(),
            |key, _| Ok(H256::from_slice(key)),
        )
        .map(|k| {
            let program = self.get_program(k).expect("Can't fail");
            (program.id(), program)
        })
        .collect();

        // todo [sab] to be removed
        let program_candidates = PrefixIterator::<ProgramId>::new(
            STORAGE_PROGRAM_CANDIDATE_PREFIX.to_vec(),
            STORAGE_PROGRAM_CANDIDATE_PREFIX.to_vec(),
            |mut key, _| ProgramId::decode(key.into_mut()),
        )
        .map(|candidate_id| {
            (
                candidate_id,
                common::get_code_for_candidate(candidate_id).expect("no candidates without code"),
            )
        })
        .collect();

        let message_queue: VecDeque<_> = common::message_iter().map(Into::into).collect();

        State {
            message_queue,
            programs,
            program_candidates,
            ..Default::default()
        }
    }
}

impl<T, GH> Default for ExtManager<T, GH>
where
    T: Config,
    T::AccountId: Origin,
    GH: Default + GasHandler,
{
    fn default() -> Self {
        ExtManager {
            _phantom: PhantomData,
            skip_messages: Default::default(),
            gas_handler: GH::default(),
        }
    }
}

impl<T, GH> ExtManager<T, GH>
where
    T: Config,
    T::AccountId: Origin,
    GH: GasHandler,
{
    pub fn program_from_code(
        &self,
        id: H256,
        code: Vec<u8>,
    ) -> Option<gear_core::program::Program> {
        Program::new(ProgramId::from_origin(id), code).ok()
    }

    pub fn get_program(&self, id: H256) -> Option<gear_core::program::Program> {
        common::native::get_program(ProgramId::from_origin(id))
    }

    pub fn set_program(&self, program: gear_core::program::Program, message_id: H256) {
        // TODO: This method is used only before program init, so program has no persistent pages.
        assert!(
            program.get_pages().is_empty(),
            "Must has empty persistent pages, has {:?}",
            program.get_pages()
        );

        let persistent_pages: BTreeMap<u32, Vec<u8>> = program
            .get_pages()
            .iter()
            .map(|(k, v)| (k.raw(), v.as_ref().expect("Must have page data").to_vec()))
            .collect();

        let id = program.id().into_origin();

        let code_hash: H256 = sp_io::hashing::blake2_256(program.code()).into();

        // todo [sab] remove? copy paste from Tx calls
        common::set_code(code_hash, program.code());

        let program = common::Program {
            static_pages: program.static_pages(),
            nonce: program.message_nonce(),
            persistent_pages: persistent_pages.keys().copied().collect(),
            code_hash,
            state: common::ProgramState::Uninitialized { message_id },
        };

        common::set_program(id, program, persistent_pages);
    }
}

impl<T, GH> JournalHandler for ExtManager<T, GH>
where
    T: Config,
    T::AccountId: Origin,
    GH: GasHandler,
{
    fn message_dispatched(&mut self, outcome: CoreDispatchOutcome) {
        let event = match outcome {
            CoreDispatchOutcome::Success(message_id) => Event::MessageDispatched(DispatchOutcome {
                message_id: message_id.into_origin(),
                outcome: ExecutionResult::Success,
            }),
            CoreDispatchOutcome::MessageTrap {
                message_id,
                program_id,
                trap,
            } => {
                let reason = trap
                    .map(|v| {
                        log::info!(
                            target: "runtime::gear",
                            "🪤 Program {} terminated with a trap: {}",
                            program_id.into_origin(),
                            v
                        );
                        v.as_bytes().to_vec()
                    })
                    .unwrap_or_default();

                Event::MessageDispatched(DispatchOutcome {
                    message_id: message_id.into_origin(),
                    outcome: ExecutionResult::Failure(reason),
                })
            }
            CoreDispatchOutcome::InitSuccess {
                message_id,
                origin,
                program_id,
            } => {
                let program_id = program_id.into_origin();
                let event = Event::InitSuccess(MessageInfo {
                    message_id,
                    origin: origin.into_origin(),
                    program_id,
                });

                common::waiting_init_take_messages(program_id)
                    .into_iter()
                    .for_each(|m_id| {
                        if let Some((m, _)) = common::remove_waiting_message(program_id, m_id) {
                            common::queue_message(m);
                        }
                    });

                common::set_program_initialized(program_id);

                event
            }
            CoreDispatchOutcome::InitFailure {
                message_id,
                origin,
                program_id,
                reason,
            } => {
                // todo issue #567
                let program_id = program_id.into_origin();
                let origin = origin.into_origin();

                // Some messages addressed to the program could be processed
                // in the queue before init message. For example, that could
                // happen when init message had more gas limit then rest block
                // gas allowance, but a dispatch message to the program was
                // dequeued. The other case is async init.
                common::waiting_init_take_messages(program_id)
                    .into_iter()
                    .for_each(|m_id| {
                        if let Some((m, _)) = common::remove_waiting_message(program_id, m_id) {
                            common::queue_message(m);
                        }
                    });    
                ProgramsLimbo::<T>::insert(program_id, origin);
                log::info!(
                    target: "runtime::gear",
                    "👻 Program {} will stay in limbo until explicitly removed",
                    program_id
                );

                Event::InitFailure(
                    MessageInfo {
                        message_id: message_id.into_origin(),
                        origin,
                        program_id,
                    },
                    Reason::Dispatch(reason.as_bytes().to_vec()),
                )
            }
        };

        Pallet::<T>::deposit_event(event);
    }

    fn gas_burned(&mut self, message_id: MessageId, origin: ProgramId, amount: u64) {
        let message_id = message_id.into_origin();

        log::debug!("burned: {:?} from: {:?}", amount, message_id);

        Pallet::<T>::decrease_gas_allowance(amount);

        let charge = T::GasConverter::gas_to_fee(amount);

        self.gas_handler.spend(message_id, amount);

        if let Some(author) = Authorship::<T>::author() {
            let _ = T::Currency::repatriate_reserved(
                &<T::AccountId as Origin>::from_origin(origin.into_origin()),
                &author,
                charge,
                BalanceStatus::Free,
            );
        }
    }

    fn exit_dispatch(&mut self, id_exited: ProgramId, value_destination: ProgramId) {
        let program_id = id_exited.into_origin();
        common::remove_program(program_id);

        let program_account = &<T::AccountId as Origin>::from_origin(program_id);
        let balance = T::Currency::total_balance(program_account);
        if !balance.is_zero() {
            T::Currency::transfer(
                program_account,
                &<T::AccountId as Origin>::from_origin(value_destination.into_origin()),
                balance,
                ExistenceRequirement::AllowDeath,
            )
            .expect("balance is not zero; should not fail");
        }
    }

    fn message_consumed(&mut self, message_id: MessageId) {
        let message_id = message_id.into_origin();

        if let ConsumeResult::RefundExternal(external, gas_left) =
            self.gas_handler.consume(message_id)
        {
            log::debug!("unreserve: {}", gas_left);

            let refund = T::GasConverter::gas_to_fee(gas_left);

            let _ =
                T::Currency::unreserve(&<T::AccountId as Origin>::from_origin(external), refund);
        }
    }

    //todo [sab] проверки для защиты в рамках/разных одного блока
    fn send_message(&mut self, message_id: MessageId, message: Message) {
        let has_skipping_dest = self
            .skip_messages
            .contains(&SkipMessageKind::WithDestination(message.dest()));
        let has_skipping_id = self
            .skip_messages
            .contains(&SkipMessageKind::WithId(message.id()));

        let message_id = message_id.into_origin();
        let mut message: common::Message = message.into();

        if has_skipping_dest || has_skipping_id {
            log::debug!(
                "skipping dest - {}, skipping id - {}",
                has_skipping_dest,
                has_skipping_id,
            );
            log::debug!(
                "Message {:?} sent from {:?} will not be queued",
                message,
                message_id
            );
            return;
        }

        log::debug!("Sending message {:?} from {:?}", message, message_id);

        if common::program_exists(message.dest) {
            self.gas_handler.split(message_id, message.id, message.gas_limit);
            common::queue_message(message);
        } else {
            // Being placed into a user's mailbox means the end of a message life cycle.
            // There can be no further processing whatsoever, hence any gas attempted to be
            // passed along must be returned (i.e. remain in the parent message's value tree).
            if message.gas_limit > 0 {
                message.gas_limit = 0;
            }
            Pallet::<T>::insert_to_mailbox(message.dest, message.clone());
            Pallet::<T>::deposit_event(Event::Log(message));
        }
    }

    fn wait_dispatch(&mut self, dispatch: Dispatch) {
        let message: common::Message = dispatch.message.into();

        common::insert_waiting_message(
            message.dest,
            message.id,
            message.clone(),
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        );

        Pallet::<T>::deposit_event(Event::AddedToWaitList(message));
    }

    fn wake_message(
        &mut self,
        message_id: MessageId,
        program_id: ProgramId,
        awakening_id: MessageId,
    ) {
        let awakening_id = awakening_id.into_origin();

        if let Some((msg, _)) =
            common::remove_waiting_message(program_id.into_origin(), awakening_id)
        {
            common::queue_message(msg);

            Pallet::<T>::deposit_event(Event::RemovedFromWaitList(awakening_id));
        } else {
            log::error!(
                "Attempt to awaken unknown message {:?} from {:?}",
                awakening_id,
                message_id.into_origin()
            );
        }
    }

    fn update_nonce(&mut self, program_id: ProgramId, nonce: u64) {
        common::set_program_nonce(program_id.into_origin(), nonce);
    }

    fn update_page(
        &mut self,
        program_id: ProgramId,
        page_number: PageNumber,
        data: Option<Vec<u8>>,
    ) {
        let program_id = program_id.into_origin();
        let page_number = page_number.raw();

        if let Some(prog) = common::get_program(program_id) {
            let mut persistent_pages = prog.persistent_pages;

            if let Some(data) = data {
                persistent_pages.insert(page_number);
                common::set_program_page(program_id, page_number, data);
            } else {
                persistent_pages.remove(&page_number);
                common::remove_program_page(program_id, page_number);
            }

            common::set_program_persistent_pages(program_id, persistent_pages);
        };
    }

    // todo [sab] double bind possible? yes - when program was deleted, therefore delete values after initializing code
    fn store_new_programs(
        &mut self,
        program_candidates_data: BTreeMap<CodeHash, Vec<(ProgramId, MessageId)>>,
    ) {
        for (code_hash, program_candidates_data) in program_candidates_data {
            let code_hash = code_hash.inner().into();

            let mut msgs_to_be_skipped = if let Some(code) = common::get_code(code_hash) {
                let mut ret = BTreeSet::new();

                'inner: for (candidate_id, init_message_id) in program_candidates_data {
                    if common::program_exists(candidate_id.into_origin()) {
                        ret.insert(SkipMessageKind::WithId(init_message_id));
                        continue 'inner;
                    }
                    // invalid wasm (when `Program` can't be instantiated) is not saved to storage
                    let program =
                        Program::new(candidate_id, code.clone()).expect("guaranteed to be valid");
                    self.set_program(program, init_message_id.into_origin());

                    log::debug!("Submit program with id {:?} from program", candidate_id);
                }

                ret
            } else {
                BTreeSet::from_iter(
                    program_candidates_data
                        .iter()
                        .copied()
                        .map(|(program_id, _)| {
                            log::debug!(
                                "Can't initialize program with id {:?}: code with provided code hash {:?} doesn't exist.",
                                program_id.into_origin(),
                                code_hash,
                            );
                            SkipMessageKind::WithDestination(program_id)
                        }),
                )
            };

            if !msgs_to_be_skipped.is_empty() {
                self.skip_messages.append(&mut msgs_to_be_skipped);
            }
        }
    }
}
