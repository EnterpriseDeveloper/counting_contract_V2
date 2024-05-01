use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub minimal_donation: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValuerResp)]
    Value {},
}

#[cw_serde]
pub enum ExecMsg {
    Donate {},
    Withdraw {},
}

#[cw_serde]
pub struct ValuerResp {
    pub value: u64,
}
