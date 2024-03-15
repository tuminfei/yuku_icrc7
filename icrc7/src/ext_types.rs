use candid::CandidType;
use candid::Principal;
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::Deserialize;

#[derive(Debug)]
pub enum ExtCommonError {
    InvalidToken,
}

// b"\x0Atid"
pub static TDS: [u8; 4] = [10, 116, 105, 100];

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct TokenIdentifier(String);

impl TokenIdentifier {
    pub fn parse_token_identifier(canister_id: Principal, index: u128) -> Self {
        let mut array = vec![];
        array.extend_from_slice(&TDS); // 加上前缀
        array.extend_from_slice(canister_id.as_slice());
        array.extend_from_slice(&index.to_be_bytes()); // 加上序号
        TokenIdentifier(candid::Principal::try_from_slice(&array).unwrap().to_text())
    }

    pub fn parse_token_index(&self, canister_id: Principal) -> Result<u128, ExtCommonError> {
        let (canister, index) = self._parse_token_identifier();
        if &canister[..] != canister_id.as_slice() {
            // canister 不是本 canister 的 id，说明 token 不对
            return Err(ExtCommonError::InvalidToken);
        }
        Ok(index)
    }

    fn _parse_token_identifier(&self) -> (Vec<u8>, u128) {
        let array = self.0.as_bytes().to_vec();
        // ic_cdk::println!("parse_token_identifier {:?}", array);
        // 1. 检查前 4 位的前缀是否是 TDS，如果不是直接返回
        if array.len() <= 4 || &array[0..4] != TDS {
            return (array, 0); // 直接返回
        }
        if array.len() <= 8 {
            return (array, 0); // 直接返回
        }
        // 2. 去掉前 4 位的前缀, 剩下的是 canister id 和序号
        let canister = array[4..array.len() - 4].to_vec();
        let index = &array[array.len() - 4..array.len()];
        let index = (index[0] as u128) << 24
            | (index[1] as u128) << 16
            | (index[2] as u128) << 8
            | (index[3] as u128);
        (canister, index)
    }
}

#[derive(CandidType, Clone, Copy, Hash, Debug, Deserialize)]
pub struct AccountIdentifier {
    pub hash: [u8; 28],
}

#[derive(CandidType, Clone, Copy, Deserialize)]
pub enum User {
    Address(AccountIdentifier),
    Principal(Principal),
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ExtTransferArg {
    pub from: User,
    pub to: User,
    pub token: TokenIdentifier,
    pub memo: Option<Vec<u8>>,
    pub amount: u128,
    pub notify: bool,
    pub subaccount: Option<Subaccount>,
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

pub type ExtTransferResult = Result<u128, ExtTransferError>;
