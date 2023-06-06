use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::User;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub arcade: String,
    pub admins: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    AddAdmin { admins: Vec<String> },
    AddTopUser { user: User },
    Leave {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryMsg {
    Greet {},
    AdminsList {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GreetResp {
    pub message: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryResp {
    Greet {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}
