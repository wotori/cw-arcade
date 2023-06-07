use std::cmp::Reverse;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");
pub const ARCADE: Item<String> = Item::new("arcade");
pub const MAX_TOP_SCORES: Item<u8> = Item::new("max_top_scores");

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct User {
    pub name: String,
    pub address: Addr,
    pub score: Reverse<u16>, // Reverse is used for creating a min-heap instead of a max-heap
}

impl Ord for User {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub const TOP_USERS: Item<Vec<User>> = Item::new("top");
pub const GAME_COUNTER: Item<u32> = Item::new("game_counter");
pub const ARCADE_DENOM: Item<String> = Item::new("denom");
pub const PRICE_PEER_GAME: Item<u128> = Item::new("price");
