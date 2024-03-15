use ic_cdk_macros::export_candid;

pub mod icrc7_types;
pub mod init_method;
pub mod memory;
pub mod query_method;
pub mod state;
pub mod update_method;
pub mod utils;
pub mod candid_file_generator;

use icrc7_types::*;

export_candid!();
