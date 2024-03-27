use candid::CandidType;
use serde::Deserialize;

use crate::ext_types::{AccountIdentifier, TokenIdentifier};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransferError {
    NonExistingTokenId,
    InvalidRecipient,
    Unauthorized,
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: u128 },
    GenericError { error_code: u128, message: String },
    GenericBatchError { error_code: u128, message: String },
}

#[derive(CandidType, Clone)]
pub enum ApprovalError {
    Unauthorized { tokens_ids: Vec<u128> },
    TooOld,
    TemporaryUnavailable,
    NonExistingTokenId,
    InvalidSpender,
    GenericError { error_code: u128, message: String },
    GenericBatchError { error_code: u128, message: String },
}

#[derive(CandidType, Clone)]
pub enum BurnError {
    Unauthorized,
    NonExistingTokenId,
    GenericError { error_code: u128, message: String },
    GenericBatchError { error_code: u128, message: String },
}

#[derive(CandidType, Clone)]
pub enum MintError {
    SupplyCapReached,
    Unauthorized,
    TokenIdAlreadyExist,
    TokenIdMinimumLimit,
    GenericError { error_code: u128, message: String },
    GenericBatchError { error_code: u128, message: String },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ExtCommonError {
    InvalidToken(TokenIdentifier),
    Other(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ExtTransferError {
    Unauthorized(AccountIdentifier),
    InsufficientBalance,
    Rejected,
    InvalidToken(TokenIdentifier),
    CannotNotify(AccountIdentifier),
    Other(String),
}


#[derive(CandidType, Debug, PartialEq, Deserialize)]
pub enum InsertTransactionError {
    SyncPending,
    NotSetArchiveCanister,
    RemoteError,
    Unexpected(String),
    CantWrite,
    InvalidId,
}

