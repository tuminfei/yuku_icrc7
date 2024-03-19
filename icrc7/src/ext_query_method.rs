use candid::Principal;
use ic_cdk_macros::query;

use crate::{
    ext_types::{
        ExtAllowanceArg, ExtAllowanceResult, ExtBalanceArg, ExtBalanceResult, ExtBearerResult,
        ExtMetadataResult, TokenIdentifier,
    },
    state::STATE,
};

#[query(name = "balance")]
pub fn ext_balance(arg: ExtBalanceArg) -> ExtBalanceResult {
    STATE.with(|s| s.borrow().ext_balance(arg))
}

#[query(name = "allowance")]
pub fn ext_allowance(arg: ExtAllowanceArg) -> ExtAllowanceResult {
    STATE.with(|s| s.borrow().ext_allowance(arg))
}

#[query(name = "bearer")]
pub fn ext_bearer(token: TokenIdentifier) -> ExtBearerResult {
    STATE.with(|s| s.borrow().ext_bearer(token))
}

#[query(name = "metadata")]
pub fn ext_metadata(token: TokenIdentifier) -> ExtMetadataResult {
    STATE.with(|s| s.borrow().ext_metadata(token))
}

#[query(name = "getMinter")]
pub fn ext_get_minter() -> Principal {
    let minting_authority = STATE.with(|s| s.borrow().icrc7_minting_authority());
    if let Some(minting_authority_info) = minting_authority {
        return minting_authority_info.owner;
    } else {
        return Principal::anonymous();
    }
}
