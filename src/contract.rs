use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{get_key_details, retrieve_key, save_config, store_key, State};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetKeyDetails { key } => to_json_binary(&get_key_details(deps.storage, key)?),
    }
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::StdError;
    use crate::contract::{execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::state::get_key_details;

    #[test]
    fn test_store_and_retrieve_key() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Store a key
        let key = "my_secret_key".to_string();
        let recipient = deps.api.addr_make("recipient");
        let store_msg = ExecuteMsg::StoreKey { key, recipient };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), store_msg).unwrap();
        let key_id = env.block.height;
        assert_eq!(res.attributes[0].value, "store_key");
        assert_eq!(res.attributes[1].value, env.block.height.to_string());

        // Retrieve the key
        let retrieve_msg = ExecuteMsg::RetrieveKey { key: key_id };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), retrieve_msg).unwrap();
        assert_eq!(res.attributes[0].value, "retrieve_key");
        assert_eq!(res.attributes[1].value, "my_secret_key".to_string());
    }

    #[test]
    fn test_retrieve_nonexistent_key() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
            .expect("contract instantiation failed");

        // Try to retrieve a key that does not exist
        let retrieve_msg = ExecuteMsg::RetrieveKey { key: 999 }; // Non-existent key_id
        let err = execute(deps.as_mut(), env.clone(), info.clone(), retrieve_msg).unwrap_err();
        match err {
            StdError::NotFound { .. } => (),
            _ => panic!("Expected NotFound error, got {:?}", err),
        }
    }

    #[test]
    fn test_store_empty_key() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Store an empty key
        let key = "".to_string();
        let recipient = deps.api.addr_make("recipient");
        let store_msg = ExecuteMsg::StoreKey { key, recipient };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), store_msg).unwrap();
        assert_eq!(res.attributes[0].value, "store_key");
        assert_eq!(res.attributes[1].value, env.block.height.to_string());
    }

    #[test]
    fn test_store_long_key() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Store a very long key
        let key = "a".repeat(1000); // 1000 characters long
        let recipient = deps.api.addr_make("recipient");
        let store_msg = ExecuteMsg::StoreKey { key, recipient };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), store_msg).unwrap();
        assert_eq!(res.attributes[0].value, "store_key");
        assert_eq!(res.attributes[1].value, env.block.height.to_string());
    }

    #[test]
    fn test_retrieve_invalid_key_id() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Try to retrieve a key with an invalid key ID
        let retrieve_msg = ExecuteMsg::RetrieveKey { key: 0 }; // Invalid key_id
        let err = execute(deps.as_mut(), env.clone(), info.clone(), retrieve_msg).unwrap_err();
        match err {
            StdError::NotFound { .. } => (),
            _ => panic!("Expected NotFound error, got {:?}", err),
        }
    }

    #[test]
    fn test_store_same_key_twice() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Store a key
        let key = "my_secret_key".to_string();
        let recipient = deps.api.addr_make("recipient");
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::StoreKey {
                key: key.clone(),
                recipient: recipient.clone(),
            },
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "store_key");
        assert_eq!(res.attributes[1].value, env.block.height.to_string());

        // Try to store the same key again
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::StoreKey {
                key: key.clone(),
                recipient: recipient.clone(),
            },
        )
        .unwrap_err();
        match err {
            StdError::GenericErr { msg, .. } if msg == "Key already exists" => (),
            _ => panic!(
                "Expected GenericErr with 'Key already exists', got {:?}",
                err
            ),
        }
    }

    #[test]
    fn test_retrieved_flag() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
        let env = mock_env();

        // Instantiate contract
        let msg = InstantiateMsg {
            broadcast: "dear AI, keep going".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Store a key
        let key = "my_secret_key".to_string();
        let recipient = deps.api.addr_make("recipient");
        let store_msg = ExecuteMsg::StoreKey { key, recipient };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), store_msg).unwrap();
        let key_id = env.block.height;
        assert_eq!(res.attributes[0].value, "store_key");
        assert_eq!(res.attributes[1].value, env.block.height.to_string());

        // Retrieve the key
        let retrieve_msg = ExecuteMsg::RetrieveKey { key: key_id };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), retrieve_msg).unwrap();
        assert_eq!(res.attributes[0].value, "retrieve_key");
        assert_eq!(res.attributes[1].value, "my_secret_key".to_string());

        // Check if the retrieved flag is set
        let key_info = get_key_details(&deps.storage, key_id).unwrap();
        assert!(key_info.retrieved, "The key should be marked as retrieved");
    }
}
