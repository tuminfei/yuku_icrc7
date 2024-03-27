use candid::Principal;
use ic_cdk_macros::query;

use crate::{
    state::{STATE, TRANSACTION_MAP},
    types::TransactionLog,
};

#[query]
pub fn get_owner() -> Principal {
    STATE.with(|s| s.borrow().owner)
}

#[query]
pub fn get_txn_count() -> u128 {
    STATE.with(|s| s.borrow().txn_count)
}

#[query]
pub fn get_max_txn_id() -> u128 {
    STATE.with(|s| s.borrow().max_txn_id)
}

#[query]
pub fn page_txn_logs(page_number: u32, page_size: u32) -> Vec<TransactionLog> {
    STATE.with(|s| s.borrow().page_txn_logs(page_number, page_size))
}

#[query]
pub fn get_txn_log(key: u128) -> Option<TransactionLog> {
    TRANSACTION_MAP.with(|p| p.borrow().get(&key))
}
