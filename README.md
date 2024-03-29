# ICRC7 (Compatible with EXT standard)

```bash
# starts replica in background
dfx start --clean --background

chmod +x gen_candid.sh
./gen_candid.sh
```

#### Deploying Factory Canister

```bash
dfx deploy factory --with-cycles 90000000000000
```

#### Deploying Icrc7 Canister
```bash
dfx deploy icrc7 --argument '(record{                                  
minting_account= opt record {
    owner = principal "3yyxm-t5fpe-v32em-ac6lr-xyort-wuscb-dvl4x-3wnwi-hqkyj-xortw-oqe";                                     
    subaccount = opt blob "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00";
  };                  
icrc7_supply_cap= null;
icrc7_description= opt "ICP Flower Collection";
tx_window= null;                        
permitted_drift= null;                  
icrc7_max_take_value= null;
icrc7_max_memo_size= null;
icrc7_symbol= "ICFL";
icrc7_max_update_batch_size= null;
icrc7_max_query_batch_size= null;
icrc7_atomic_batch_transfers= null;
icrc7_default_take_value= null;
icrc7_logo= null;
icrc7_name= "ICP Flower"
})'
```

#### Minting NFT
```bash
dfx canister call icrc7 icrc7_mint '(record{                                  
to= record {
    owner = principal "3yyxm-t5fpe-v32em-ac6lr-xyort-wuscb-dvl4x-3wnwi-hqkyj-xortw-oqe";                                     
    subaccount = opt blob "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00";
  };          
token_id=1;
memo= null;
from_subaccount= null;                  
token_description= opt "Token Number 1";
token_logo= null;
token_name= null
})'
```

#### Transferring tokens
```bash
dfx canister call icrc7 icrc7_transfer '(vec{
record{
to=record {
owner = principal "t4egw-clf4w-qbpli-svryg-7yqq6-jt2yj-7v755-mabir-zmx6i-vp4fr-fqe";
subaccount = opt blob "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00";
};
token_id= 1;
from_subaccount= null;
memo= null;
created_at_time= opt 1710480202901000000
};
record{
to=record {
owner = principal "t4egw-clf4w-qbpli-svryg-7yqq6-jt2yj-7v755-mabir-zmx6i-vp4fr-fqe";
subaccount = opt blob "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00";
};
token_id= 2;
from_subaccount= null;
memo= null;
created_at_time= opt 1710480202901000000
}
})'
```

#### Approve NFT
```bash
dfx canister call icrc7 icrc7_approve '(vec{record{                                  
spender= record {
    owner = principal "t4egw-clf4w-qbpli-svryg-7yqq6-jt2yj-7v755-mabir-zmx6i-vp4fr-fqe";                                     
    subaccount = opt blob "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00";
  };          
token_id=1;
memo= null;
expires_at= null;
from_subaccount= null;
}
})'
```

#### Deploying Tansaction Log Canister

```bash
dfx deploy transaction_log --argument '(record{                                  
parent_canister_id= principal "bkyz2-fmaaa-aaaaa-qaaaq-cai";
})'
```

#### Set ICRC7 Archive Tansaction Log Canister

```bash
dfx canister call icrc7 icrc7_set_archive_log_canister '(principal "be2us-64aaa-aaaaa-qaabq-cai")'
```

#### Sync ICRC7 Archive Tansaction Log

```bash
dfx canister call icrc7 icrc7_archive_logs
```

## Compatible with EXT interface

### ext-core

balance: query (request : BalanceRequest) -> async BalanceResponse;

transfer: shared (request : TransferRequest) -> async TransferResponse;

### nonfungible

bearer: shared query (token : TokenIdentifier) -> async Result<AccountIdentifier, CommonError>;

mintNFT: shared (request : MintRequest) -> async ();

### common

metadata: shared query (token : TokenIdentifier) -> async Result<Metadata, CommonError>;

supply: shared query (token : TokenIdentifier) -> async Result<Balance, CommonError>;

### allowance

allowance: shared query (request : AllowanceRequest) -> async async Result<Balance, CommonError>;

approve: shared (request : ApproveRequest) -> async ();