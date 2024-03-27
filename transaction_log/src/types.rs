use candid::{CandidType, Principal};
use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

pub type Subaccount = [u8; 32];

#[derive(Serialize, CandidType, Deserialize, Clone, Debug, Copy)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Mint {
        tid: u128,
        from: Account,
        to: Account,
    },
    Burn {
        tid: u128,
        from: Account,
        to: Account,
    },
    Transfer {
        tid: u128,
        from: Account,
        to: Account,
    },
    Approval {
        tid: u128,
        from: Account,
        to: Account,
    },
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct TransactionLog {
    pub at: u64,
    pub txn_id: u128,
    pub op: String,
    pub txn_type: TransactionType,
    pub memo: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug)]
pub enum GetTransactionError {
    Unexpected(String),
    InvalidId,
}

#[derive(CandidType, Clone, Debug)]
pub enum InsertTransactionError {
    Unexpected(String),
    CantWrite,
    InvalidId,
}
