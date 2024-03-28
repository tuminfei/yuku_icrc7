use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::init;
use serde_derive::{Deserialize, Serialize};

use crate::state::{State, STATE};

#[derive(Serialize, Deserialize, CandidType)]
pub struct InitArgs {
    pub parent_canister_id: Principal,
}

#[init]
#[candid_method(init)]
pub fn init(arg: InitArgs) {
    let owner = ic_cdk::caller();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let state = State {
            max_txn_id: 0,
            txn_count: 0,
            parent: arg.parent_canister_id,
            owner,
            pending: false,
        };
        *s = state;
    });
}
