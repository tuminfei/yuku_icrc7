use ic_cdk_macros::update;

use crate::ext_types::{ExtApproveArg, ExtTransferArg, ExtTransferResult};
use crate::state::STATE;

#[update(name = "transfer")]
pub fn ext_transfer(arg: ExtTransferArg) -> ExtTransferResult {
    let caller = ic_cdk::caller();
    STATE.with(|s| s.borrow_mut().ext_transfer(&caller, arg))
}

#[update(name = "approve")]
pub fn ext_approve(arg: ExtApproveArg) -> bool {
    let caller = ic_cdk::caller();
    STATE.with(|s| s.borrow_mut().ext_approve(&caller, arg))
}
