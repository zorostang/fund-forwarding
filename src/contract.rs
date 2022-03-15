use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Uint128, Querier,
    StdError, StdResult, Storage, CanonicalAddr, QueryResult,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use crate::msg::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use crate::state::{save, load, may_load, remove, Config, CONFIG_KEY, PREFIX_TOKEN_CONTRACT_INFO, FUNDS_DISTRIBUTION_KEY};
use crate::royalties::{RoyaltyInfo, StoredRoyaltyInfo};


use primitive_types::U256;
use secret_toolkit::{snip20::handle::{register_receive_msg,transfer_msg}};

pub const BLOCK_SIZE: usize = 256;


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admin: deps.api.canonical_address(&msg.admin)?,
    };

    store_dist_info(
        &mut deps.storage,
        &deps.api,
        Some(msg.dist_info).as_ref(),
        None,
        FUNDS_DISTRIBUTION_KEY
    )?;



   
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
        HandleMsg::ChangeDistribution { dist_info } => change_dao(deps, env, dist_info),
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


   
    forward_funds(
        deps,
        env,
        amount,      
        )

}





pub fn forward_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
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


    


    //Payment distribution
    let royalty_list = load::<StoredRoyaltyInfo, _>(&deps.storage, FUNDS_DISTRIBUTION_KEY)?;
 
    for royalty in royalty_list.royalties.iter() {
        let decimal_places : u32 = royalty_list.decimal_places_in_rates.into();
        let rate :u128 = (royalty.rate as u128) * (10 as u128).pow(decimal_places);
        let amount = Uint128((amount.u128() * rate) / (100 as u128).pow(decimal_places));
        let recipient = deps.api.human_address(&royalty.recipient).unwrap();
        let cosmos_msg = transfer_msg(
            recipient,
            amount,
            padding.clone(),
            BLOCK_SIZE,
            callback_code_hash.clone(),
            snip20_address.clone(),
        )?;
        msg_list.push(cosmos_msg);
    }





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
    dist_info: RoyaltyInfo,
) -> StdResult<HandleResponse> {
    let config: Config = load(&deps.storage, CONFIG_KEY)?;  
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;

    if config.admin != sender_raw {
        return Err(StdError::generic_err(
            "This function is only usable by the Admin",
        ));
    }

    store_dist_info(
        &mut deps.storage,
        &deps.api,
        Some(dist_info).as_ref(),
        None,
        FUNDS_DISTRIBUTION_KEY
    )?;
    


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



/// Returns StdResult<()>
///
/// verifies the royalty information is valid and if so, stores the royalty info for the token
/// or as default
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage for this RoyaltyInfo
/// * `api` - a reference to the Api used to convert human and canonical addresses
/// * `royalty_info` - an optional reference to the RoyaltyInfo to store
/// * `default` - an optional reference to the default StoredRoyaltyInfo to use if royalty_info is
///               not provided
/// * `key` - the storage key (either token key or default key)
fn store_dist_info<S: Storage, A: Api>(
    storage: &mut S,
    api: &A,
    royalty_info: Option<&RoyaltyInfo>,
    default: Option<&StoredRoyaltyInfo>,
    key: &[u8],
) -> StdResult<()> {
    // if RoyaltyInfo is provided, check and save it
    if let Some(royal_inf) = royalty_info {
        // the allowed message length won't let enough u16 rates to overflow u128
        let total_rates: u128 = royal_inf.royalties.iter().map(|r| r.rate as u128).sum();
        let (royalty_den, overflow) =
            U256::from(10).overflowing_pow(U256::from(royal_inf.decimal_places_in_rates));
        if overflow {
            return Err(StdError::generic_err(
                "The number of decimal places used in the royalty rates is larger than supported",
            ));
        }
        if U256::from(total_rates) != royalty_den {
            return Err(StdError::generic_err(
                "The sum of royalty rates must not exceed 100%",
            ));
        }
        let stored = royal_inf.to_stored(api)?;
        save(storage, key, &stored)
    } else if let Some(def) = default {
        save(storage, key, def)
    } else {
        remove(storage, key);
        Ok(())
    }
}











pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryDist {} => to_binary(&query_distribution(deps)?),
    }
}



fn query_distribution<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {

    let royalty = may_load::<StoredRoyaltyInfo, _>(&deps.storage, FUNDS_DISTRIBUTION_KEY)?;


    to_binary(&QueryAnswer::RoyaltyInfo {
        royalty_info: royalty
            .map(|s| s.to_human(&deps.api, false))
            .transpose()?,
    })

}

