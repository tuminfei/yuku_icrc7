use ic_cdk_macros::update;

use crate::ext_types::{ExtTransferArg, ExtTransferResult};
use crate::state::STATE;

#[update]
pub fn ext_transfer(arg: ExtTransferArg) -> ExtTransferResult {
    let caller = ic_cdk::caller();
    STATE.with(|s| s.borrow_mut().ext_transfer(&caller, arg))
}
