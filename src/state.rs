use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, StdError, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DATA_STORE: Map<u64, EncryptedKey> = Map::new("encrypted_keys");
pub const CONFIG: Item<State> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct State {
    pub creator: Addr,
    pub broadcast: String,
}
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct EncryptedKey {
    pub key_id: u64,
    pub creator: Addr,
    pub timestamp: Timestamp,
    pub recipient: Addr,
    pub retrieved: bool,
    pub encrypted_data: Vec<u8>,
}

pub fn save_config(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    let result = CONFIG.save(storage, state);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    Ok(())
}

pub fn load_config(storage: &dyn Storage) -> StdResult<State> {
    CONFIG.load(storage)
}

pub fn store_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    key: String,
    recipient: Addr,
) -> StdResult<u64> {
    let key_id = env.block.height;
    if DATA_STORE.has(deps.storage, key_id) {
        return Err(StdError::generic_err("Key already exists"));
    }
    DATA_STORE.save(
        deps.storage,
        key_id,
        &EncryptedKey {
            key_id,
            timestamp: Timestamp::default(),
            creator: info.sender,
            recipient,
            retrieved: false,
            encrypted_data: encrypt(key),
        },
    )?;
    Ok(key_id)
}

pub fn retrieve_key(deps: DepsMut, key_id: u64) -> StdResult<String> {
    let result = get_key_details(deps.storage, key_id);

    if result.is_ok() {
        let mut key = result?;
        key.retrieved = true;
        let data = key.encrypted_data.clone();
        DATA_STORE.save(deps.storage, key_id, &key.clone())?;
        Ok(decrypt(data))
    } else {
        Err(result.err().unwrap())
    }
}

pub fn get_key_details(storage: &dyn Storage, key_id: u64) -> StdResult<EncryptedKey> {
    DATA_STORE.load(storage, key_id)
}

fn encrypt(data: String) -> Vec<u8> {
    let key = b"32-bytes-long-key-for-best-AES!!";
    let cipher = Aes256Gcm::new_from_slice(key).expect("Failed to create cipher");
    let nonce = GenericArray::from_slice(b"unique nonce"); // 12-bytes; unique per message
    cipher
        .encrypt(nonce, data.as_bytes())
        .expect("Encryption failed")
}

fn decrypt(data: Vec<u8>) -> String {
    let cipher = Aes256Gcm::new_from_slice(b"32-bytes-long-key-for-best-AES!!")
        .expect("Failed to create cipher");
    let nonce = GenericArray::from_slice(b"unique nonce"); // 12-bytes; unique per message
    let decrypted_data = cipher
        .decrypt(nonce, data.as_ref())
        .expect("Decryption failed");
    String::from_utf8(decrypted_data).expect("Failed to convert decrypted data to string")
}
