use crate::types::TransactionLog;
use candid::{Decode, Encode, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 1000;

// For a type to be used in a `StableBTreeMap`, it needs to implement the `Storable`
// trait, which specifies how the type can be serialized/deserialized.
//
// In this example, we're using candid to serialize/deserialize the struct, but you
// can use anything as long as you're maintaining backward-compatibility. The
// backward-compatibility allows you to change your struct over time (e.g. adding
// new fields).
//
// The `Storable` trait is already implemented for several common types (e.g. u64),
// so you can use those directly without implementing the `Storable` trait for them.
impl Storable for TransactionLog {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(Debug)]
pub struct State {
    pub max_txn_id: u128,
    pub txn_count: u128,
    pub parent: Principal,
    pub owner: Principal,
    pub pending: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            max_txn_id: 0,
            txn_count: 0,

            parent: Principal::anonymous(),
            owner: Principal::anonymous(),
            pending: false,
        }
    }
}

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static TRANSACTION_MAP: RefCell<StableBTreeMap<u128, TransactionLog, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static STATE: RefCell<State> = RefCell::default();
}

impl State {
    pub fn page_txn_logs(&self, page_number: u32, page_size: u32) -> Vec<TransactionLog> {
        let offset = (page_number - 1) * page_size;
        if offset as u128 > self.txn_count {
            ic_cdk::trap("Exceeds Max Offset Value")
        }

        TRANSACTION_MAP.with(|p| {
            p.borrow()
                .iter()
                .skip(offset as usize)
                .take(page_size as usize)
                .map(|(_, txn)| txn.clone())
                .collect()
        })
    }
}
