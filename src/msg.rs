use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::royalties::{DisplayRoyaltyInfo, RoyaltyInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: HumanAddr,

    pub dist_info: RoyaltyInfo,
    
    pub sscrt_addr: HumanAddr,
    pub sscrt_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        #[serde(default)]
        msg: Option<Binary>,
    },
    RegisterToken {
        snip20_addr: HumanAddr,
        snip20_hash: String
    },
    ChangeDistribution {
        dist_info: RoyaltyInfo,
    },
    ChangeAdmin {
        admin_addr: HumanAddr,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryDist {}
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionCheckResponse {
    pub dist: RoyaltyInfo,
}



#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    RoyaltyInfo {
        royalty_info: Option<DisplayRoyaltyInfo>,
    },

}