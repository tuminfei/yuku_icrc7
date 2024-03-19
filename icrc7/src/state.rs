use std::{cell::RefCell, collections::HashMap};

use crate::{
    errors::{ApprovalError, TransferError},
    ext_types::{
        AccountIdentifier, ExtAllowanceArg, ExtAllowanceResult, ExtApproveArg, ExtBalanceArg,
        ExtBalanceResult, ExtBearerResult, ExtCommonError, ExtMetadata, ExtMetadataResult,
        ExtMetadataType, ExtTransferArg, ExtTransferError, ExtTransferResult, TokenIdentifier,
    },
    icrc7_types::{
        BurnResult, Icrc7TokenMetadata, MintArg, MintError, MintResult, Transaction,
        TransactionType, TransferArg, TransferResult,
    },
    memory::{get_log_memory, get_token_map_memory, Memory},
    utils::{account_transformer, burn_account, user_transformer},
    Approval, ApprovalArg, ApproveResult, BurnArg, BurnError,
};
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{
    memory_manager::MemoryManager, storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable,
};
use icrc_ledger_types::{
    icrc::generic_metadata_value::MetadataValue,
    icrc1::account::{Account, DEFAULT_SUBACCOUNT},
};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Icrc7Token {
    pub token_id: u128,
    pub token_name: String,
    pub token_description: Option<String>,
    pub token_logo: Option<String>,
    pub token_owner: Account,
    pub approvals: Vec<Approval>,
}

impl Storable for Icrc7Token {
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Icrc7Token {
    fn new(
        token_id: u128,
        token_name: String,
        token_description: Option<String>,
        token_logo: Option<String>,
        token_owner: Account,
    ) -> Self {
        Self {
            token_id,
            token_name,
            token_logo,
            token_owner,
            token_description,
            approvals: vec![],
        }
    }

    fn transfer(&mut self, to: Account) {
        self.token_owner = to;
        self.approvals.clear();
    }

    fn approve(&mut self, approval: Approval) {
        self.approvals.push(approval);
    }

    fn approval_check(&self, current_time: u64, account: &Account) -> bool {
        for approval in self.approvals.iter() {
            if approval.account == *account {
                if approval.expires_at.is_none() {
                    return true;
                } else if approval.expires_at >= Some(current_time) {
                    return true;
                }
            }
        }
        false
    }

    fn token_metadata(&self) -> Icrc7TokenMetadata {
        let mut metadata = HashMap::<String, MetadataValue>::new();
        metadata.insert("Name".into(), MetadataValue::Text(self.token_name.clone()));
        metadata.insert(
            "Symbol".into(),
            MetadataValue::Text(self.token_name.clone()),
        );
        if let Some(ref description) = self.token_description {
            metadata.insert(
                "Description".into(),
                MetadataValue::Text(description.clone()),
            );
        }
        if let Some(ref logo) = self.token_logo {
            metadata.insert("logo".into(), MetadataValue::Text(logo.clone()));
        }
        metadata
    }

    fn burn(&mut self, burn_address: Account) {
        self.token_owner = burn_address;
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub minting_authority: Option<Account>,
    pub icrc7_symbol: String,
    pub icrc7_name: String,
    pub icrc7_description: Option<String>,
    pub icrc7_logo: Option<String>,
    pub icrc7_total_supply: u128,
    pub icrc7_supply_cap: Option<u128>,
    pub icrc7_max_query_batch_size: Option<u128>,
    pub icrc7_max_update_batch_size: Option<u128>,
    pub icrc7_max_take_value: Option<u128>,
    pub icrc7_default_take_value: Option<u128>,
    pub icrc7_max_memo_size: Option<u128>,
    pub icrc7_atomic_batch_transfers: Option<bool>,
    pub tx_window: Option<u64>,
    pub permitted_drift: Option<u64>,
    #[serde(skip, default = "get_token_map_memory")]
    pub tokens: StableBTreeMap<u128, Icrc7Token, Memory>,
    pub txn_count: u128,
    #[serde(skip, default = "get_log_memory")]
    pub txn_log: StableBTreeMap<u128, Transaction, Memory>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            minting_authority: None,
            icrc7_symbol: "ICRC7".into(),
            icrc7_name: "ICRC7 Collection".into(),
            icrc7_description: None,
            icrc7_logo: None,
            icrc7_total_supply: 0,
            icrc7_supply_cap: None,
            icrc7_max_query_batch_size: None,
            icrc7_max_update_batch_size: None,
            icrc7_max_take_value: None,
            icrc7_default_take_value: None,
            icrc7_max_memo_size: None,
            icrc7_atomic_batch_transfers: None,
            tx_window: None,
            permitted_drift: None,
            tokens: get_token_map_memory(),
            txn_count: 0,
            txn_log: get_log_memory(),
        }
    }
}

impl State {
    pub const DEFAULT_MAX_QUERY_BATCH_SIZE: u128 = 32;
    pub const DEFAULT_MAX_UPDATE_BATCH_SIZE: u128 = 32;
    pub const DEFAULT_TAKE_VALUE: u128 = 32;
    pub const DEFAULT_MAX_TAKE_VALUE: u128 = 32;
    pub const DEFAULT_MAX_MEMO_SIZE: u128 = 32;
    pub const DEFAULT_TX_WINDOW: u64 = 24 * 60 * 60 * 1000_000_000;
    pub const DEFAULT_PERMITTED_DRIFT: u64 = 2 * 60 * 1000_000_000;

    pub fn icrc7_symbol(&self) -> String {
        self.icrc7_symbol.clone()
    }

    pub fn icrc7_name(&self) -> String {
        self.icrc7_name.clone()
    }

    pub fn icrc7_description(&self) -> Option<String> {
        self.icrc7_description.clone()
    }

    pub fn icrc7_total_supply(&self) -> u128 {
        self.icrc7_total_supply
    }

    pub fn icrc7_supply_cap(&self) -> Option<u128> {
        self.icrc7_supply_cap
    }

    pub fn icrc7_logo(&self) -> Option<String> {
        self.icrc7_logo.clone()
    }

    pub fn icrc7_minting_authority(&self) -> Option<Account> {
        self.minting_authority.clone()
    }

    pub fn icrc7_max_query_batch_size(&self) -> Option<u128> {
        self.icrc7_max_query_batch_size
    }

    pub fn icrc7_max_update_batch_size(&self) -> Option<u128> {
        self.icrc7_max_update_batch_size
    }

    pub fn icrc7_default_take_value(&self) -> Option<u128> {
        self.icrc7_default_take_value
    }

    pub fn icrc7_max_take_value(&self) -> Option<u128> {
        self.icrc7_max_take_value
    }

    pub fn icrc7_max_memo_size(&self) -> Option<u128> {
        self.icrc7_max_memo_size
    }

    pub fn icrc7_atomic_batch_transfers(&self) -> Option<bool> {
        self.icrc7_atomic_batch_transfers
    }

    pub fn icrc7_owner_of(&self, token_id: &[u128]) -> Vec<Option<Account>> {
        let mut res = vec![None; token_id.len()];
        for (index, id) in token_id.iter().enumerate() {
            if let Some(ref token) = self.tokens.get(id) {
                res.insert(index, Some(token.token_owner))
            }
        }
        res
    }

    fn txn_deduplication_check(
        &self,
        allowed_past_time: &u64,
        caller: &Account,
        args: &TransferArg,
    ) -> Result<(), TransferError> {
        let mut count = self.txn_count;
        while count != 0 {
            let txn = self.txn_log.get(&count).unwrap();
            if txn.at < *allowed_past_time {
                return Ok(());
            }
            match txn.txn_type {
                TransactionType::Transfer {
                    ref tid,
                    ref from,
                    ref to,
                } => {
                    if &args.token_id == tid
                        && caller == from
                        && &args.to == to
                        && args.memo == txn.memo
                        && args.created_at_time == Some(txn.at)
                    {
                        return Err(TransferError::Duplicate {
                            duplicate_of: count,
                        });
                    } else {
                        count -= 1;
                        continue;
                    }
                }
                _ => {
                    count -= 1;
                    continue;
                }
            }
        }
        Ok(())
    }

    fn get_txn_id(&mut self) -> u128 {
        self.txn_count += 1;
        self.txn_count
    }

    fn log_transaction(
        &mut self,
        txn_type: TransactionType,
        at: u64,
        memo: Option<Vec<u8>>,
    ) -> u128 {
        let txn_id = self.get_txn_id();
        let txn = Transaction::new(txn_id, txn_type, at, memo);
        self.txn_log.insert(txn_id, txn);
        txn_id
    }

    fn mock_transfer(
        &self,
        current_time: &u64,
        caller: &Account,
        arg: &TransferArg,
    ) -> Result<(), TransferError> {
        if let Some(time) = arg.created_at_time {
            let allowed_past_time = *current_time
                - self.tx_window.unwrap_or(State::DEFAULT_TX_WINDOW)
                - self
                    .permitted_drift
                    .unwrap_or(State::DEFAULT_PERMITTED_DRIFT);
            let allowed_future_time = *current_time
                + self
                    .permitted_drift
                    .unwrap_or(State::DEFAULT_PERMITTED_DRIFT);
            if time < allowed_past_time {
                return Err(TransferError::TooOld);
            } else if time > allowed_future_time {
                return Err(TransferError::CreatedInFuture {
                    ledger_time: current_time.clone(),
                });
            }
            self.txn_deduplication_check(&allowed_past_time, caller, arg)?;
        }
        // checking is token for the corresponding ID exists or not
        if let None = self.tokens.get(&arg.token_id) {
            return Err(TransferError::NonExistingTokenId);
        }
        if let Some(ref memo) = arg.memo {
            let max_memo_size = self
                .icrc7_max_memo_size
                .unwrap_or(State::DEFAULT_MAX_MEMO_SIZE);
            if memo.len() as u128 > max_memo_size {
                return Err(TransferError::GenericError {
                    error_code: 3,
                    message: "Exceeds Max Memo Size".into(),
                });
            }
        }
        // checking if receiver and sender have same address
        if arg.to == *caller {
            return Err(TransferError::InvalidRecipient);
        }
        let token = self.tokens.get(&arg.token_id).unwrap();
        // checking if the caller is authorized or is approve to make transaction
        if token.token_owner != *caller && !token.approval_check(current_time.clone(), caller) {
            return Err(TransferError::Unauthorized);
        }
        Ok(())
    }

    pub fn icrc7_transfer(
        &mut self,
        caller: &Principal,
        mut args: Vec<TransferArg>,
    ) -> Vec<Option<TransferResult>> {
        // checking if the argument length in 0
        if args.len() == 0 {
            return vec![Some(Err(TransferError::GenericBatchError {
                error_code: 1,
                message: "No Arguments Provided".into(),
            }))];
        }
        let max_update_batch_size = self
            .icrc7_max_query_batch_size
            .unwrap_or(State::DEFAULT_MAX_UPDATE_BATCH_SIZE);
        let mut txn_results = vec![None; args.len()];
        if args.len() as u128 > max_update_batch_size {
            txn_results[0] = Some(Err(TransferError::GenericBatchError {
                error_code: 2,
                message: "Exceed Max allowed Update Batch Size".into(),
            }));
            return txn_results;
        }
        if *caller == Principal::anonymous() {
            txn_results[0] = Some(Err(TransferError::GenericBatchError {
                error_code: 100,
                message: "Anonymous Identity".into(),
            }));
            return txn_results;
        }
        let current_time = ic_cdk::api::time();
        for (index, arg) in args.iter_mut().enumerate() {
            let caller_account = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            arg.to = account_transformer(arg.to);
            if let Err(e) = self.mock_transfer(&current_time, &caller_account, &arg) {
                txn_results[index] = Some(Err(e));
            }
        }
        if let Some(true) = self.icrc7_atomic_batch_transfers {
            if txn_results
                .iter()
                .any(|res| res.is_some() && res.as_ref().unwrap().is_err())
            {
                return txn_results;
            }
        }
        for (index, arg) in args.iter().enumerate() {
            let caller_account = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            let time = arg.created_at_time.unwrap_or(current_time);
            if let Some(Err(e)) = txn_results.get(index).unwrap() {
                match e {
                    TransferError::GenericBatchError {
                        error_code: _,
                        message: _,
                    } => return txn_results,
                    _ => continue,
                }
            }
            let mut token = self.tokens.get(&arg.token_id).unwrap();
            token.transfer(arg.to.clone());
            self.tokens.insert(arg.token_id, token);
            let txn_id = self.log_transaction(
                TransactionType::Transfer {
                    tid: arg.token_id,
                    from: caller_account.clone(),
                    to: arg.to.clone(),
                },
                time,
                arg.memo.clone(),
            );
            txn_results[index] = Some(Ok(txn_id));
        }
        txn_results
    }

    fn mock_mint(&self, caller: &Account, arg: &MintArg) -> Result<(), MintError> {
        if let Some(cap) = self.icrc7_supply_cap {
            if cap == self.icrc7_total_supply {
                return Err(MintError::SupplyCapReached);
            }
        }
        if let None = self.minting_authority {
            return Err(MintError::GenericBatchError {
                error_code: 6,
                message: "Minting Authority Not Set".into(),
            });
        }
        if Some(*caller) != self.minting_authority {
            return Err(MintError::Unauthorized);
        }
        if let Some(ref memo) = arg.memo {
            let allowed_memo_length = self
                .icrc7_max_memo_size
                .unwrap_or(State::DEFAULT_MAX_MEMO_SIZE);
            if memo.len() as u128 > allowed_memo_length {
                return Err(MintError::GenericError {
                    error_code: 7,
                    message: "Exceeds Allowed Memo Length".into(),
                });
            }
        }
        if let Some(_) = self.tokens.get(&arg.token_id) {
            return Err(MintError::TokenIdAlreadyExist);
        }
        Ok(())
    }

    pub fn mint(&mut self, caller: &Principal, mut arg: MintArg) -> MintResult {
        let caller = account_transformer(Account {
            owner: caller.clone(),
            subaccount: arg.from_subaccount,
        });
        arg.to = account_transformer(arg.to);
        self.mock_mint(&caller, &arg)?;
        let token_name = arg.token_name.unwrap_or_else(|| {
            let name = format!("{} {}", self.icrc7_symbol, arg.token_id);
            name
        });
        let token = Icrc7Token::new(
            arg.token_id,
            token_name.clone(),
            arg.token_description.clone(),
            arg.token_logo,
            arg.to.clone(),
        );
        self.tokens.insert(arg.token_id, token);
        let txn_id = self.log_transaction(
            TransactionType::Mint {
                tid: arg.token_id,
                from: caller,
                to: arg.to,
            },
            ic_cdk::api::time(),
            arg.memo,
        );
        Ok(txn_id)
    }

    fn mock_burn(&self, caller: &Account, arg: &BurnArg) -> Result<(), BurnError> {
        if let Some(ref memo) = arg.memo {
            if memo.len() as u128
                > self
                    .icrc7_max_memo_size
                    .unwrap_or(State::DEFAULT_MAX_MEMO_SIZE)
            {
                return Err(BurnError::GenericError {
                    error_code: 3,
                    message: "Exceeds Max Memo Length".into(),
                });
            }
        }
        match self.tokens.get(&arg.token_id) {
            None => Err(BurnError::NonExistingTokenId),
            Some(ref token) => {
                if token.token_owner != *caller {
                    return Err(BurnError::Unauthorized);
                }
                Ok(())
            }
        }
    }

    pub fn burn(&mut self, caller: &Principal, mut args: Vec<BurnArg>) -> Vec<Option<BurnResult>> {
        if args.len() == 0 {
            return vec![Some(Err(BurnError::GenericBatchError {
                error_code: 1,
                message: "No Arguments Provided".into(),
            }))];
        }
        let mut txn_results = vec![None; args.len()];
        if *caller == Principal::anonymous() {
            txn_results[0] = Some(Err(BurnError::GenericBatchError {
                error_code: 100,
                message: "Anonymous Identity".into(),
            }));
            return txn_results;
        }
        for (index, arg) in args.iter_mut().enumerate() {
            let caller = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            if let Err(e) = self.mock_burn(&caller, arg) {
                txn_results.insert(index, Some(Err(e)))
            }
        }
        if let Some(true) = self.icrc7_atomic_batch_transfers {
            if txn_results
                .iter()
                .any(|res| res.is_some() && res.as_ref().unwrap().is_err())
            {
                return txn_results;
            }
        }
        for (index, arg) in args.iter().enumerate() {
            let caller = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            let burn_address = burn_account();
            if let Some(Err(e)) = txn_results.get(index).unwrap() {
                match e {
                    BurnError::GenericBatchError {
                        error_code: _,
                        message: _,
                    } => return txn_results,
                    _ => continue,
                }
            }
            let mut token = self.tokens.get(&arg.token_id).unwrap();
            token.burn(burn_address.clone());
            let tid = self.log_transaction(
                TransactionType::Burn {
                    tid: arg.token_id,
                    from: caller,
                    to: burn_address,
                },
                ic_cdk::api::time(),
                arg.memo.clone(),
            );
            txn_results.insert(index, Some(Ok(tid)))
        }
        txn_results
    }

    fn mock_approve(&self, caller: &Account, arg: &ApprovalArg) -> Result<(), ApprovalError> {
        if arg.spender == *caller {
            return Err(ApprovalError::InvalidSpender);
        };
        if let Some(ref memo) = arg.memo {
            let max_memo_size = self
                .icrc7_max_memo_size
                .unwrap_or(State::DEFAULT_MAX_MEMO_SIZE);
            if memo.len() as u128 > max_memo_size {
                return Err(ApprovalError::GenericError {
                    error_code: 3,
                    message: "Exceeds Max Memo Size".into(),
                });
            }
        };
        match self.tokens.get(&arg.token_id) {
            None => Err(ApprovalError::NonExistingTokenId),
            Some(ref token) => {
                if token.token_owner != *caller {
                    return Err(ApprovalError::Unauthorized {
                        tokens_ids: vec![arg.token_id],
                    });
                }
                Ok(())
            }
        }
    }

    pub fn approve(
        &mut self,
        caller: &Principal,
        mut args: Vec<ApprovalArg>,
    ) -> Vec<Option<ApproveResult>> {
        if args.len() == 0 {
            return vec![Some(Err(ApprovalError::GenericBatchError {
                error_code: 1,
                message: "No Arguments Provided".into(),
            }))];
        }
        let mut txn_results = vec![None; args.len()];
        if *caller == Principal::anonymous() {
            txn_results[0] = Some(Err(ApprovalError::GenericBatchError {
                error_code: 100,
                message: "Anonymous Identity".into(),
            }));
            return txn_results;
        }
        for (index, arg) in args.iter_mut().enumerate() {
            let caller = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            if let Err(e) = self.mock_approve(&caller, arg) {
                txn_results.insert(index, Some(Err(e)))
            }
        }
        if let Some(true) = self.icrc7_atomic_batch_transfers {
            if txn_results
                .iter()
                .any(|res| res.is_some() && res.as_ref().unwrap().is_err())
            {
                return txn_results;
            }
        }
        for (index, arg) in args.iter().enumerate() {
            let caller = account_transformer(Account {
                owner: caller.clone(),
                subaccount: arg.from_subaccount,
            });
            if let Some(Err(e)) = txn_results.get(index).unwrap() {
                match e {
                    &ApprovalError::GenericBatchError {
                        error_code: _,
                        message: _,
                    } => return txn_results,
                    _ => continue,
                }
            }
            let mut token = self.tokens.get(&arg.token_id).unwrap();
            let approve_arg = Approval {
                account: arg.spender,
                expires_at: arg.expires_at,
            };
            token.approve(approve_arg);
            let tid = self.log_transaction(
                TransactionType::Approval {
                    tid: arg.token_id,
                    from: caller,
                    to: arg.spender,
                },
                ic_cdk::api::time(),
                arg.memo.clone(),
            );
            txn_results.insert(index, Some(Ok(tid)))
        }
        txn_results
    }

    pub fn icrc7_token_metadata(&self, token_ids: &[u128]) -> Vec<Option<Icrc7TokenMetadata>> {
        if token_ids.len() as u128
            > self
                .icrc7_max_query_batch_size
                .unwrap_or(State::DEFAULT_MAX_QUERY_BATCH_SIZE)
        {
            ic_cdk::trap("Exceeds Max Query Batch Size")
        }
        let mut metadata_list = vec![None; token_ids.len()];
        for (index, tid) in token_ids.iter().enumerate() {
            if let Some(ref token) = self.tokens.get(tid) {
                metadata_list.insert(index, Some(token.token_metadata()))
            }
        }
        metadata_list
    }

    pub fn icrc7_balance_of(&self, accounts: &[Account]) -> Vec<u128> {
        let mut count_list = vec![0; accounts.len()];
        accounts.iter().enumerate().for_each(|(index, account)| {
            self.tokens.iter().for_each(|(_id, ref token)| {
                if token.token_owner == *account {
                    let current_count = count_list[index];
                    count_list[index] = current_count + 1;
                }
            })
        });
        count_list
    }

    pub fn icrc7_tokens(&self, prev: Option<u128>, take: Option<u128>) -> Vec<u128> {
        let take = take.unwrap_or(State::DEFAULT_TAKE_VALUE);
        if take > State::DEFAULT_MAX_TAKE_VALUE {
            ic_cdk::trap("Exceeds Max Take Value")
        }
        let mut list: Vec<u128> = self.tokens.iter().map(|(k, _)| k).collect();
        list.sort();
        match prev {
            Some(prev) => match list.iter().position(|id| *id == prev) {
                None => vec![],
                Some(index) => list
                    .iter()
                    .map(|id| *id)
                    .skip(index)
                    .take(take as usize)
                    .collect(),
            },
            None => list[0..take as usize].to_vec(),
        }
    }

    pub fn icrc7_tokens_of(
        &self,
        account: Account,
        prev: Option<u128>,
        take: Option<u128>,
    ) -> Vec<u128> {
        let take = take.unwrap_or(State::DEFAULT_TAKE_VALUE);
        if take > State::DEFAULT_MAX_TAKE_VALUE {
            ic_cdk::trap("Exceeds Max Take Value")
        }
        let mut owned_tokens = vec![];
        for (id, token) in self.tokens.iter() {
            if token.token_owner == account {
                owned_tokens.push(id);
            }
        }
        owned_tokens.sort();
        match prev {
            None => owned_tokens[0..=take as usize].to_vec(),
            Some(prev) => match owned_tokens.iter().position(|id| *id == prev) {
                None => vec![],
                Some(index) => owned_tokens
                    .iter()
                    .map(|id| *id)
                    .skip(index)
                    .take(take as usize)
                    .collect(),
            },
        }
    }

    pub fn icrc7_txn_logs(&self, page_number: u32, page_size: u32) -> Vec<Transaction> {
        let offset = (page_number - 1) * page_size;
        if offset as u128 > self.txn_count {
            ic_cdk::trap("Exceeds Max Offset Value")
        }
        let tx_logs = self
            .txn_log
            .iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .map(|(_, txn)| txn.clone())
            .collect();

        tx_logs
    }

    pub fn ext_transfer(&mut self, caller: &Principal, arg: ExtTransferArg) -> ExtTransferResult {
        if *caller == Principal::anonymous() {
            return Err(ExtTransferError::Rejected);
        }

        if arg.amount != 1 {
            return Err(ExtTransferError::Other("Must use amount of 1".to_string()));
        };

        let current_time = ic_cdk::api::time();
        let canister_id = ic_cdk::api::id();

        let caller_account = account_transformer(Account {
            owner: caller.clone(),
            subaccount: Some(DEFAULT_SUBACCOUNT.clone()),
        });

        let _from_account = match user_transformer(arg.from) {
            Some(account) => account,
            None => {
                return Err(ExtTransferError::Other(
                    "From User not support address".to_string(),
                ))
            }
        };

        let to_account = match user_transformer(arg.to) {
            Some(account) => account,
            None => {
                return Err(ExtTransferError::Other(
                    "To User not support address".to_string(),
                ))
            }
        };

        let token_id = match arg.token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return Err(ExtTransferError::InvalidToken(arg.token)),
        };

        let icrc7_arg = TransferArg {
            from_subaccount: Some(DEFAULT_SUBACCOUNT.clone()),
            to: to_account,
            token_id,
            memo: arg.memo.clone(),
            created_at_time: Some(current_time),
        };

        if let Err(_) = self.mock_transfer(&current_time, &caller_account, &icrc7_arg) {
            return Err(ExtTransferError::Other("mock_transfer error".to_string()));
        }

        let mut token = self.tokens.get(&icrc7_arg.token_id).unwrap();
        token.transfer(icrc7_arg.to.clone());
        self.tokens.insert(icrc7_arg.token_id, token);
        self.log_transaction(
            TransactionType::Transfer {
                tid: icrc7_arg.token_id,
                from: caller_account.clone(),
                to: icrc7_arg.to.clone(),
            },
            current_time,
            arg.memo.clone(),
        );
        Ok(arg.amount)
    }

    pub fn ext_approve(&mut self, caller: &Principal, arg: ExtApproveArg) -> bool {
        if *caller == Principal::anonymous() {
            return false;
        }

        if arg.allowance != 1 {
            return false;
        };

        let current_time = ic_cdk::api::time();
        let canister_id = ic_cdk::api::id();

        let caller_account = account_transformer(Account {
            owner: caller.clone(),
            subaccount: Some(DEFAULT_SUBACCOUNT.clone()),
        });

        let to_account = Account {
            owner: arg.spender.clone(),
            subaccount: Some(DEFAULT_SUBACCOUNT.clone()),
        };

        let token_id = match arg.token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return false,
        };

        let icrc7_arg = ApprovalArg {
            from_subaccount: None,
            spender: to_account,
            token_id,
            expires_at: None,
            memo: None,
        };

        if let Err(_) = self.mock_approve(&caller_account, &icrc7_arg) {
            return false;
        }

        let mut token = self.tokens.get(&icrc7_arg.token_id).unwrap();
        let approve_arg = Approval {
            account: icrc7_arg.spender,
            expires_at: None,
        };
        token.approve(approve_arg);
        self.log_transaction(
            TransactionType::Approval {
                tid: icrc7_arg.token_id,
                from: caller_account,
                to: icrc7_arg.spender,
            },
            current_time,
            None,
        );
        true
    }

    pub fn ext_balance(&self, arg: ExtBalanceArg) -> ExtBalanceResult {
        let canister_id = ic_cdk::api::id();
        let token_id = match arg.token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return Err(ExtCommonError::InvalidToken(arg.token)),
        };

        let user = match user_transformer(arg.user) {
            Some(account) => account,
            None => {
                return Err(ExtCommonError::Other(
                    "User not support address".to_string(),
                ))
            }
        };

        if let Some(ref token) = self.tokens.get(&token_id) {
            if token.token_owner == user {
                return Ok(1);
            } else {
                return Ok(0);
            }
        }

        return Err(ExtCommonError::InvalidToken(arg.token));
    }

    pub fn ext_allowance(&self, arg: ExtAllowanceArg) -> ExtAllowanceResult {
        let canister_id = ic_cdk::api::id();
        let current_time = ic_cdk::api::time();

        let token_id = match arg.token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return Err(ExtCommonError::InvalidToken(arg.token)),
        };

        let user = match user_transformer(arg.owner) {
            Some(account) => account,
            None => {
                return Err(ExtCommonError::Other(
                    "User not support address".to_string(),
                ))
            }
        };

        let to_account = Account {
            owner: arg.spender.clone(),
            subaccount: Some(DEFAULT_SUBACCOUNT.clone()),
        };

        let token = self.tokens.get(&token_id).unwrap();
        if token.token_owner != user {
            return Err(ExtCommonError::Other("Invalid owner".to_string()));
        }

        if token.approval_check(current_time, &to_account) {
            return Ok(1);
        } else {
            return Ok(0);
        }
    }

    pub fn ext_bearer(&self, token: TokenIdentifier) -> ExtBearerResult {
        let canister_id = ic_cdk::api::id();

        let token_id = match token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return Err(ExtCommonError::InvalidToken(token)),
        };

        let token = self.tokens.get(&token_id);

        if let Some(token_info) = token {
            return Ok(AccountIdentifier::from_principal(
                &token_info.token_owner.owner,
                &token_info.token_owner.subaccount,
            ));
        } else {
            return Err(ExtCommonError::Other("Invalid token".to_string()));
        }
    }

    pub fn ext_metadata(&self, token: TokenIdentifier) -> ExtMetadataResult {
        let canister_id = ic_cdk::api::id();

        let token_id = match token.parse_token_index(canister_id) {
            Ok(token_id) => token_id,
            Err(_) => return Err(ExtCommonError::InvalidToken(token)),
        };

        let token = self.tokens.get(&token_id);

        if let Some(token_info) = token {
            let metadata = token_info
                .token_description
                .unwrap_or_else(|| String::from(""));

            return Ok(ExtMetadata::Nonfungible(ExtMetadataType::new(metadata)));
        } else {
            return Err(ExtCommonError::Other("Invalid token".to_string()));
        }
    }
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static STATE: RefCell<State> = RefCell::default();
}
