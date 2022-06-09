use crate::state::Config;
use cosmwasm_std::{
    BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, HumanAddr, Querier, QueryRequest,
    StdResult, Uint128,
};

pub fn query_token_balance<Q: Querier>(querier: &Q, config: &Config) -> StdResult<u128> {
    let balance_resp: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: config.self_addr.clone(),
        denom: config.tax_denom.clone(),
    }))?;

    Ok(balance_resp.amount.amount.u128())
}

pub fn send_native_token_msg(to: &HumanAddr, amount: u128, config: &Config) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        from_address: config.self_addr.clone(),
        to_address: to.clone(),
        amount: vec![Coin {
            denom: config.tax_denom.clone(),
            amount: Uint128::from(amount),
        }],
    })
}
