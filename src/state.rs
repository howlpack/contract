use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub dens_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub enum Notification {
    Email(EmailNotification),
    Webhook(WebhookNotification),
}

#[cw_serde]
pub struct EmailNotification {
    pub masked_addr: String,
    pub encoded_addr: String,
    pub preferences: String,
}

#[cw_serde]
pub struct WebhookNotification {
    pub masked_url: String,
    pub encoded_url: String,
    pub preferences: String,
}

pub const NOTIFICATIONS: Map<&str, Vec<Notification>> = Map::new("notifications");
