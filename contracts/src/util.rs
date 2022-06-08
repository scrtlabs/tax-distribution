use crate::state::{Beneficiaries, Beneficiary, Config};
use cosmwasm_std::{
    plaintext_log, BankMsg, Coin, CosmosMsg, HandleResponse, HandleResult, HumanAddr, LogAttribute,
    StdResult, Uint128,
};

pub fn send_native_token_msg(to: HumanAddr, amount: u128, config: &Config) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        from_address: config.self_addr.clone(),
        to_address: to,
        amount: vec![Coin {
            denom: config.tax_denom.clone(),
            amount: Uint128(amount),
        }],
    })
}

pub fn withdraw_tax_for_everyone(
    config: &Config,
    beneficiaries: Vec<Beneficiary>,
    total_balance: u128,
) -> StdResult<(Vec<CosmosMsg>, Vec<LogAttribute>)> {
    let mut messages = vec![];
    let mut log = vec![];

    for b in beneficiaries {
        let balance = b.check_beneficiary_balance(total_balance)?;
        messages.push(send_native_token_msg(b.address.clone(), balance, &config));
        log.extend(vec![
            plaintext_log("tax_redeemed", b.address),
            plaintext_log("amount", balance),
        ]);
    }

    Ok((messages, log))
}
