use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Config, Notification};

#[cw_serde]
pub struct InstantiateMsg {
    pub dens_addr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateNotifications {
        token_id: String,
        notifications: Vec<Notification>,
    },
    UpdateConfig {
        owner: Option<Addr>,
        dens_addr: Option<Addr>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(Config)]
    GetConfig {},
    #[returns(Vec<Notification>)]
    GetNotifications { token_id: String },
}

// We define a custom struct for each query response
