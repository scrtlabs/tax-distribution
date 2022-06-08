use cosmwasm_std::{
    debug_print, plaintext_log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, InitResult, Querier, QueryResult,
    StdError, StdResult, Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{Beneficiaries, Beneficiary, Config};
use crate::util::send_native_token_msg;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    Config {
        self_addr: env.contract.address,
        admin: env.message.sender,
        tx_denom: msg.tax_denom.unwrap_or("uscrt".to_string()),
    }
    .save(&mut deps.storage)?;

    let mut ben_addresses = vec![];
    for b in msg.beneficiaries {
        b.save(&mut deps.storage)?;
        ben_addresses.push(b.address);
    }
    Beneficiaries::save(ben_addresses, &mut deps.storage)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::Withdraw { amount } => withdraw(deps, env, amount.map(|a| a.u128())),
        HandleMsg::ChangeAdmin { new_admin } => change_admin(deps, env, new_admin),
        HandleMsg::ChangeBeneficiaries { beneficiaries } => {
            change_beneficiaries(deps, env, beneficiaries)
        }
    }
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Option<u128>,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;

    let beneficiary = match Beneficiary::load(&deps.storage, &env.message.sender)? {
        None => return Err(StdError::unauthorized()),
        Some(b) => b,
    };
    let balance = beneficiary.check_beneficiary_balance(&deps.querier, &config)?;
    let amount = amount.unwrap_or(balance);

    if amount > balance {
        return Err(StdError::generic_err(format!(
            "insufficient staked funds to redeem: balance={}, required={}",
            balance, amount,
        )));
    }

    Ok(HandleResponse {
        messages: vec![send_native_token_msg(
            beneficiary.address.clone(),
            amount,
            &config,
        )],
        log: vec![
            plaintext_log("tax_redeemed", beneficiary.address),
            plaintext_log("amount", amount),
        ],
        data: None,
    })
}

pub fn change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_admin: HumanAddr,
) -> HandleResult {
    let mut config = Config::load(&deps.storage)?;
    config.assert_admin(&env.message.sender)?;

    config.admin = new_admin.clone();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![plaintext_log("changed_admin", new_admin)],
        data: None,
    })
}

pub fn change_beneficiaries<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    beneficiaries: Vec<Beneficiary>,
) -> HandleResult {
    // for b in beneficiaries {
    //
    // }

    unimplemented!()
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetBeneficiaries { .. } => get_beneficiaries(deps),
        QueryMsg::GetBeneficiaryBalance { .. } => get_beneficiary_balance(deps),
    }
}

pub fn get_beneficiaries<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    unimplemented!()
}

pub fn get_beneficiary_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> QueryResult {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
