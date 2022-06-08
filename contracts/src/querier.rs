use crate::state::Config;
use cosmwasm_std::{BalanceResponse, BankQuery, Querier, QueryRequest, StdResult};

pub fn check_token_balance<Q: Querier>(querier: &Q, config: &Config) -> StdResult<u128> {
    let balance_resp: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: config.self_addr.clone(),
        denom: config.tx_denom.clone(),
    }))?;

    Ok(balance_resp.amount.amount.u128())
}
