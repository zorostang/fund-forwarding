use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: HumanAddr,
    pub dao: HumanAddr,
    
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
    ChangeDao {
        dao_addr: HumanAddr,
    },
    ChangeAdmin {
        admin_addr: HumanAddr,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetDao {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DaoCheckResponse {
    pub dao: HumanAddr,
}
