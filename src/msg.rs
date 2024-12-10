use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct Recipient {
    pub address: String,
    pub amount: Vec<Coin>,
}

#[cw_serde]
pub struct Cw20Recipient {
    pub address: String,
    pub amount: Uint128, // Using Uint128 for amounts
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Disperse { recipients: Vec<Recipient> },
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum ReceiveMsg {
    DisperseCw20 { recipients: Vec<Cw20Recipient> },
}

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {}

// We define a custom struct for each query response
