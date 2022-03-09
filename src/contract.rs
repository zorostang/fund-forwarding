use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Uint128, Querier,
    StdError, StdResult, Storage, CanonicalAddr,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use crate::msg::{DaoCheckResponse ,HandleMsg, InitMsg, QueryMsg};
use crate::state::{save, load, may_load, Config, CONFIG_KEY, PREFIX_TOKEN_CONTRACT_INFO};


use secret_toolkit::{snip20::handle::{register_receive_msg,transfer_msg}};

pub const BLOCK_SIZE: usize = 256;


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admin: deps.api.canonical_address(&msg.admin)?,
        dao: deps.api.canonical_address(&msg.dao)?,
    };



   
    save(&mut deps.storage, CONFIG_KEY, &config)?;


    // Store sscrt in registered contracts
    let mut snip_contract_storage = PrefixedStorage::new(PREFIX_TOKEN_CONTRACT_INFO, &mut deps.storage);
    save(&mut snip_contract_storage, msg.sscrt_addr.0.as_bytes(), &msg.sscrt_hash)?;


    Ok(InitResponse {
        messages: vec![
            register_receive_msg(
                env.contract_code_hash,
                None,
                BLOCK_SIZE,
                msg.sscrt_hash,
                msg.sscrt_addr
            )?
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive { sender, from, amount, msg } => receive(deps, env, sender, from, amount, msg),
        HandleMsg::RegisterToken { snip20_addr, snip20_hash } => register_token(deps, env, snip20_addr, snip20_hash),
        HandleMsg::ChangeDao { dao_addr } => change_dao(deps, env, dao_addr),
        HandleMsg::ChangeAdmin { admin_addr } => change_admin(deps, env, admin_addr),
    }
}





/// For receiving SNIP20s
pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> HandleResult {


    let config: Config = load(&deps.storage, CONFIG_KEY)?;

    let recipient = deps.api.human_address(&config.dao)?;


        
    forward_funds(
        deps,
        env,
        recipient,
        amount,      
        )

}





pub fn forward_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    amount: Uint128
) -> StdResult<HandleResponse> {
    
    let mut msg_list: Vec<CosmosMsg> = vec![];
    let padding: Option<String> = None;


    // Finds hash associated with snip20 contract
    let snip_contract_storage = ReadonlyPrefixedStorage::new(PREFIX_TOKEN_CONTRACT_INFO, &mut deps.storage);
    let snip20_address: HumanAddr = env.message.sender;
    let callback_code_wrapped: Option<String> = may_load(&snip_contract_storage, snip20_address.0.as_bytes())?;
    let callback_code_hash: String;
    
    if callback_code_wrapped == None {
        return Err(StdError::generic_err(
            "This token is not registered with this contract. Please register it",
        ));
    }

    else {
        callback_code_hash = callback_code_wrapped.unwrap();
    }


    



    // Send funds to recipient
    let cosmos_msg = transfer_msg(
        recipient,
        amount,
        padding.clone(),
        BLOCK_SIZE,
        callback_code_hash.clone(),
        snip20_address.clone(),
    )?;
    msg_list.push(cosmos_msg);





    Ok(HandleResponse {
        messages: msg_list,
        log: vec![],
        data: None,
    })
}

/// Calls register_receive a snip20 token contract
/// and saves snip20 contract hash keyed to address
/// 
/// # Arguements
/// * `deps` - a mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `snip20_addr` - address of the snip20 contract to be registered
/// * `snip20_hash` - contract callback hash of the snip20 contract
pub fn register_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    snip20_addr: HumanAddr,
    snip20_hash: String
) -> StdResult<HandleResponse> {

    let config: Config = load(&deps.storage, CONFIG_KEY)?;  
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;


    
    if config.admin != sender_raw {
        return Err(StdError::generic_err(
            "This function is only usable by the Admin",
        ));
    }


    let mut snip_contract_storage = PrefixedStorage::new(PREFIX_TOKEN_CONTRACT_INFO, &mut deps.storage);
    
    save(&mut snip_contract_storage, snip20_addr.0.as_bytes(), &snip20_hash)?;


    Ok(HandleResponse {
        messages: vec![
            register_receive_msg(
                env.contract_code_hash,
                None,
                BLOCK_SIZE,
                snip20_hash,
                snip20_addr
            )?
        ],
        log: vec![],
        data: None,
    })
}




pub fn change_dao<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    dao_addr: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;  
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;

    if config.admin != sender_raw {
        return Err(StdError::generic_err(
            "This function is only usable by the Admin",
        ));
    }

    config.dao = deps.api.canonical_address(&dao_addr)?;

    save(&mut deps.storage, CONFIG_KEY, &config)?;


    Ok(HandleResponse::default())
}


pub fn change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin_addr: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;  
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;

    if config.admin != sender_raw {
        return Err(StdError::generic_err(
            "This function is only usable by the Admin",
        ));
    }

    config.admin = deps.api.canonical_address(&admin_addr)?;

    save(&mut deps.storage, CONFIG_KEY, &config)?;


    Ok(HandleResponse::default())
}



pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDao {} => to_binary(&query_dao(deps)?),
    }
}



fn query_dao<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<DaoCheckResponse> {
    let config: Config = load(&deps.storage, CONFIG_KEY)?;

    Ok(DaoCheckResponse { dao: deps.api.human_address(&config.dao)? })
}

