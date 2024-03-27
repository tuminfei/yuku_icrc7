use candid::Principal;
use ic_cdk_macros::update;

use crate::{
    guards::{owner_guard, parent_guard},
    state::{STATE, TRANSACTION_MAP},
    types::{InsertTransactionError, TransactionLog},
};

#[update(guard = "parent_guard")]
pub fn insert_txn_log(log: TransactionLog) -> Result<u128, InsertTransactionError> {
    let exist_log = TRANSACTION_MAP.with(|p| p.borrow().get(&log.txn_id));
    if exist_log.is_some() {
        return Err(InsertTransactionError::InvalidId);
    }

    TRANSACTION_MAP.with(|p| p.borrow_mut().insert(log.txn_id, log.clone()));
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.txn_count += 1;
    });
    return Ok(log.txn_id);
}

#[update(guard = "parent_guard")]
pub fn insert_many_txn_log(logs: Vec<TransactionLog>) -> Result<u32, InsertTransactionError> {
    // Prevent trade competition and duplicate executions
    let is_pending = STATE.with(|s| s.borrow().pending);
    if is_pending {
        return Err(InsertTransactionError::CantWrite);
    }

    // set pending
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.pending = true;
    });

    let logs_size = logs.len() as u32;
    let txn_ids: Vec<u128> = logs.iter().map(|log| log.txn_id).collect();

    for txn_id in &txn_ids {
        let is_exist = TRANSACTION_MAP.with(|p| p.borrow().contains_key(txn_id));
        if is_exist {
            return Err(InsertTransactionError::InvalidId);
        }
    }

    for log in &logs {
        TRANSACTION_MAP.with(|p| p.borrow_mut().insert(log.txn_id, log.clone()));
    }

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.txn_count += logs_size as u128;
        state.pending = false;
    });

    return Ok(logs_size);
}

#[update(guard = "owner_guard")]
pub fn update_parent_canister(parent_id: Principal) -> bool {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.parent = parent_id;
    });
    return true;
}

#[update(guard = "owner_guard")]
pub fn update_owner(owner: Principal) -> bool {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.owner = owner;
    });
    return true;
}
