#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Uint128, Querier,
        StdError, StdResult, Storage, CanonicalAddr, QueryResult, testing::mock_dependencies, testing::mock_env
    };
    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
    use secret_toolkit::utils::InitCallback;
    
    use crate::msg::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
    use crate::state::{save, load, may_load, remove, Config, CONFIG_KEY, PREFIX_TOKEN_CONTRACT_INFO, FUNDS_DISTRIBUTION_KEY};
    use crate::royalties::{RoyaltyInfo, StoredRoyaltyInfo, Royalty};
    use crate::contract::{init, receive, register_token, forward_funds};
    

    
    use primitive_types::U256;
    use secret_toolkit::{snip20::handle::{register_receive_msg,transfer_msg}};

    



    #[test]

    pub fn init_test() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("addr1", &[]);
        

        let msg = InitMsg {
            admin: env.message.sender.clone(),
            dist_info: RoyaltyInfo {
                decimal_places_in_rates: 1,
                royalties: vec![ Royalty { recipient: env.message.sender.clone(), rate: 100 }],
            },
            sscrt_addr: HumanAddr::from("Contract Address"),
            sscrt_hash: String::from("Snip20 hash"),
        };

        let receive_amount = Uint128(10);


        init(&mut deps, env.clone(), msg);

        let env = mock_env("Contract Address", &[]);
        forward_funds(&mut deps, env.clone(), receive_amount);

        
        register_token(&mut deps, env.clone(), HumanAddr::from("New Address"), String::from("New hash"));

        let env = mock_env("New Address", &[]);
        forward_funds(&mut deps, env.clone(), receive_amount);
    }


}