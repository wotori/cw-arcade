use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::User;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub arcade: String,
    pub admins: Vec<String>,
    pub max_top_score: u8,
    pub denom: String,
    pub price_peer_game: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    AddAdmin { admins: Vec<String> },
    AddTopUser { user: User },
    Leave {},
    Play {},
    UpdatePrice { price: u128 },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub enum QueryMsg {
    AdminsList {},
    ScoreList {},
    GameCounter {},
    Price {},
    PrizePool {},
    TotalDistributed {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryResp {
    Greet {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct ScoreboardListResp {
    pub scores: Vec<User>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct GameCounterResp {
    pub game_counter: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct GamePriceResp {
    pub price: u128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct PrizePoolResp {
    pub prize_pool: u128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct TotalDistributionResp {
    // returns total amouns of token that was distributed across gamers
    pub total_distributed: u128,
}
