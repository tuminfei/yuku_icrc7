use crate::state::STATE;
use ic_cdk::caller;

#[inline(always)]
pub fn owner_guard() -> Result<(), String> {
    let owner = STATE.with(|s| s.borrow().owner);

    if caller() == owner {
        Ok(())
    } else {
        Err(String::from("The caller is not the owner of contract"))
    }
}

#[inline(always)]
pub fn parent_guard() -> Result<(), String> {
    let parent = STATE.with(|s| s.borrow().parent);

    if caller() == parent {
        Ok(())
    } else {
        Err(String::from("The caller is not the parent of contract"))
    }
}
