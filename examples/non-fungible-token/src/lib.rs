// Fungible Token Smart Contract.
// Implementation based on https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/ERC20.sol

#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

const GAS_RESERVE: u64 = 500_000_000;

#[derive(Debug)]
struct NonFungibleToken {
    name: String,
    symbol: String,
    base_uri: String,
    token_uri: String,
    token_id: U256,
    token_owner: BTreeMap<U256, ActorId>,
    token_approvals: BTreeMap<U256, ActorId>,
    owned_tokens_count: BTreeMap<ActorId, u128>,
    operator_approval: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

static mut NON_FUNGIBLE_TOKEN: NonFungibleToken = NonFungibleToken {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    token_uri: String::new(),
    token_id: U256::zero(),
    token_owner: BTreeMap::new(),
    token_approvals: BTreeMap::new(),
    owned_tokens_count: BTreeMap::new(),
    operator_approval: BTreeMap::new(),
};

impl NonFungibleToken {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn set_base_uri(&mut self, base_uri: String) {
      self.base_uri = base_uri;
    }
    fn base_uri(&self) -> &str {
      &self.base_uri
    }
    fn token_uri(&self, token_id: U256) -> String {
      let mut temp =  self.base_uri.clone();
      temp.push_str(&token_id.to_string());
      return temp;
    }

    fn exists(&self, token_id: U256) -> bool {
      self.token_owner.contains_key(&token_id)
    }

    fn is_token_owner(&self, token_id: U256, account: &ActorId) -> bool {
      let zero = ActorId::new(H256::zero().to_fixed_bytes());
      account == self.token_owner.get(&token_id).unwrap_or(&zero)
    }

    fn mint(&mut self, account: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("NonFungibleToken: Mint to zero address.");
        }
        if self.exists(token_id) {
          panic!("NonFungibleToken: Token already exists");
        }
        self.token_owner.insert(token_id, *account);
        let balance = *self.owned_tokens_count.get(account).unwrap_or(&0);
        self.owned_tokens_count.insert(*account, balance.saturating_add(1));
        self.token_id = self.token_id.saturating_add(U256::one());
        // unsafe {

        // }
        let transfer_token = Transfer {
            from: H256::zero(),
            to: H256::from_slice(account.as_ref()),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn burn(&mut self, account: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("NonFungibleToken: Burn from zero address.");
        }
        if !self.exists(token_id) {
          panic!("NonFungibleToken: Token does not exist");
        }
        if !self.is_token_owner(token_id, account) {
          panic!("NonFungibleToken: account is not owner");
        }

        self.token_approvals.remove(&token_id);
        self.token_owner.remove(&token_id);
        let balance = *self.owned_tokens_count.get(account).unwrap_or(&0);
        self.owned_tokens_count.insert(*account, balance.saturating_sub(1));
        // unsafe {

        // }
        let transfer_token = Transfer {
            from: H256::from_slice(account.as_ref()),
            to: H256::zero(),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn transfer(&mut self, from: &ActorId, to: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if from == &zero {
            panic!("NonFungibleToken: Transfer from zero address.");
        }
        if to == &zero {
            panic!("NonFungibleToken: Transfer to zero address.");
        }
        if !self.is_token_owner(token_id, from) {
          panic!("NonFungibleToken: from is not owner");
        }
        self.token_approvals.remove(&token_id);
        let from_balance = *self.owned_tokens_count.get(from).unwrap_or(&0);
        let to_balance = *self.owned_tokens_count.get(to).unwrap_or(&0);
        self.owned_tokens_count.insert(*from, from_balance.saturating_sub(1));
        self.owned_tokens_count.insert(*to, to_balance.saturating_add(1));
        self.token_owner.insert(token_id, *to);
        let transfer_token = Transfer {
            from: H256::from_slice(from.as_ref()),
            to: H256::from_slice(to.as_ref()),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());

        if owner == &zero {
            panic!("NonFungibleToken: Approval from zero address.");
        }
        if spender == &zero {
            panic!("NonFungibleToken: Approval to zero address.");
        }
        if spender == owner {
          panic!("NonFungibleToken: Approval to current owner");
        }
        if !self.is_token_owner(token_id, owner) {
          panic!("NonFungibleToken: is not owner");
        }
        self.token_approvals.insert(token_id, *spender);

        let approve_token = Approve {
            owner: H256::from_slice(owner.as_ref()),
            spender: H256::from_slice(spender.as_ref()),
            token_id,
        };
        msg::reply(
            Event::Approval(approve_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn get_approved(&self,  token_id: U256) -> ActorId {
        if !self.exists(token_id) {
          panic!("NonFungibleToken: approved query for nonexistent token");
        }
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        *self
            .token_approvals
            .get(&token_id)
            .unwrap_or(&zero)
    }

    fn transfer_from(
        &mut self,
        owner: &ActorId,
        from: &ActorId,
        to: &ActorId,
        token_id: U256,
    ) {
        if !self.is_token_owner(token_id, owner) {
          panic!("NonFungibleToken: is not owner");
        }
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        let approved = self.get_approved(token_id);
        if approved == zero {
          panic!("NonFungibleToken: transfer caller is not approved");
        }
        self.transfer(from, to, token_id);
        self.token_approvals.remove(&token_id);
    }
}

#[derive(Debug, Decode, TypeInfo)]
struct InitConfig {
    name: String,
    symbol: String,
}

#[derive(Debug, Decode, TypeInfo)]
struct MintInput {
    account: H256,
    token_id: U256,
}

#[derive(Debug, Decode, TypeInfo)]
struct BurnInput {
    account: H256,
    token_id: U256,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct Approve {
    owner: H256,
    spender: H256,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
struct Transfer {
    from: H256,
    to: H256,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
struct TransferFrom {
    owner: H256,
    from: H256,
    to: H256,
    token_id: U256,
}

#[derive(Debug, Decode, TypeInfo)]
enum Action {
    Mint(MintInput),
    Burn(BurnInput),
    Transfer(Transfer),
    TransferFrom(TransferFrom),
    Approve(Approve),
}

#[derive(Debug, Encode, TypeInfo)]
enum Event {
    Transfer(Transfer),
    Approval(Approve),
}

gstd::metadata! {
    title: "NonFungibleToken",
        init:
            input : InitConfig,
        handle:
            input : Action,
            output : Event,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");

    match action {
        Action::Mint(mint_input) => {
            let to = ActorId::new(mint_input.account.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.mint(&to, mint_input.token_id);
        }
        Action::Burn(burn_input) => {
            let from = ActorId::new(burn_input.account.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.burn(&from, burn_input.token_id);
        }
        Action::Transfer(transfer) => {
            let from = ActorId::new(transfer.from.to_fixed_bytes());
            let to = ActorId::new(transfer.to.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.transfer(&from, &to, transfer.token_id);
        }
        Action::Approve(approve) => {
            let owner = ActorId::new(approve.owner.to_fixed_bytes());
            let spender = ActorId::new(approve.spender.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.approve(&owner, &spender, approve.token_id);
        }
        Action::TransferFrom(transfer) => {
            let owner = ActorId::new(transfer.owner.to_fixed_bytes());
            let from = ActorId::new(transfer.from.to_fixed_bytes());
            let to = ActorId::new(transfer.to.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.transfer_from(&owner, &from, &to, transfer.token_id);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NON_FUNGIBLE_TOKEN {:?}", config);
    NON_FUNGIBLE_TOKEN.set_name(config.name);
    NON_FUNGIBLE_TOKEN.set_symbol(config.symbol);
    debug!(
        "NON_FUNGIBLE_TOKEN {} SYMBOL {} created",
        NON_FUNGIBLE_TOKEN.name(),
        NON_FUNGIBLE_TOKEN.symbol()
    );
}