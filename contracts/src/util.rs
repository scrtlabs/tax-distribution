use crate::state::{Beneficiaries, Config};
use cosmwasm_std::{
    plaintext_log, BankMsg, Coin, CosmosMsg, HumanAddr, LogAttribute, StdResult, Uint128,
};

pub fn send_native_token_msg(to: HumanAddr, amount: u128, config: &Config) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        from_address: config.self_addr.clone(),
        to_address: to,
        amount: vec![Coin {
            denom: config.tax_denom.clone(),
            amount: Uint128::from(amount),
        }],
    })
}

pub fn withdraw_tax_for_everyone(
    config: &Config,
    beneficiaries: Beneficiaries,
    total_balance: u128,
) -> StdResult<(Vec<CosmosMsg>, Vec<LogAttribute>)> {
    let mut messages = vec![];
    let mut log = vec![];

    let total_weight = beneficiaries.total_weight();
    for b in &beneficiaries.list {
        let balance = b.check_beneficiary_balance(total_balance, total_weight)?;
        messages.push(send_native_token_msg(b.address.clone(), balance, config));
        log.extend(vec![
            plaintext_log("tax_redeemed", b.address.clone()),
            plaintext_log("amount", balance),
        ]);
    }

    Ok((messages, log))
}
