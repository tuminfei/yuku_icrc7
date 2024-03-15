use ic_cdk_macros::query;

use crate::{
    ext_types::{ExtBalanceArg, ExtBalanceResult},
    state::STATE,
};

#[query]
pub fn ext_balance(arg: ExtBalanceArg) -> ExtBalanceResult {
    STATE.with(|s| s.borrow().ext_balance(arg))
}
