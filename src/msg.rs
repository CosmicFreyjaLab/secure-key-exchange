use crate::state::EncryptedKey;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct DataEntry {
    pub key_id: u64,
    pub owner: Addr,
    pub encrypted_data: Vec<u8>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub broadcast: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    StoreKey { key: String, recipient: Addr },
    RetrieveKey { key: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(EncryptedKey)]
    GetKeyDetails { key: u64 },
}
