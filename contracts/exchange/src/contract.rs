use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    get_key_details, load_config, retrieve_key, save_config, store_key, EncryptedKey, State,
};
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        creator: info.sender.clone(),
        broadcast: msg.broadcast.clone(),
    };
    save_config(deps.storage, &state)?;
    Ok(Response::default()
        .add_attribute("owner", state.creator)
        .add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::StoreKey { key, recipient } => {
            store_key(deps, env, info, key, recipient).map(|key_id| {
                Response::new()
                    .add_attribute("action", "store_key")
                    .add_attribute("key_id", key_id.to_string())
            })
        }
        ExecuteMsg::RetrieveKey { key } => retrieve_key(deps, key).map(|decrypted| {
            Response::new()
                .add_attribute("action", "retrieve_key")
                .add_attribute("key", decrypted)
        }),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Response<EncryptedKey>> {
    match msg {
        QueryMsg::GetKeyDetails { key } => get_key_details(deps.storage, key).map(|key_details| {
            let state = load_config(deps.storage);
            return Response::new()
                .add_attribute("action", "get_key_details")
                .add_attribute("key_id", key.to_string())
                .add_attribute("creator", key_details.creator.clone())
                .add_attribute("recipient", key_details.recipient.clone())
                .add_attribute("retrieved", key_details.retrieved.to_string())
                .add_attribute("timestamp", key_details.timestamp.to_string())
                .add_attribute("encrypted_data", state.unwrap().broadcast);
        }),
    }
}
