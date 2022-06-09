use crate::state::Config;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, HumanAddr, Uint128};

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
