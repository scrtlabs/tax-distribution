use cosmwasm_std::{
    debug_print, plaintext_log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, InitResult, Querier, QueryResult,
    StdError, StdResult, Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::querier::check_token_balance;
use crate::state::{Beneficiaries, Beneficiary, Config};
use crate::util::{send_native_token_msg, withdraw_tax_for_everyone};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    Config {
        self_addr: env.contract.address,
        admin: env.message.sender,
        tax_denom: msg.tax_denom.unwrap_or("uscrt".to_string()),
    }
    .save(&mut deps.storage)?;

    msg.beneficiaries.save(&mut deps.storage)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::Withdraw { amount } => withdraw(deps),
        HandleMsg::ChangeAdmin { new_admin } => change_admin(deps, env, new_admin),
        HandleMsg::ChangeBeneficiaries { beneficiaries } => {
            change_beneficiaries(deps, env, beneficiaries)
        }
    }
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>) -> HandleResult {
    let config = Config::load(&deps.storage)?;

    let total_balance = check_token_balance(&deps.querier, &config)?;
    if total_balance == 0 {
        return Ok(HandleResponse::default());
    }

    let beneficiaries = Beneficiaries::load(&deps.storage)?;
    let (messages, log) = withdraw_tax_for_everyone(&config, beneficiaries, total_balance)?;

    Ok(HandleResponse {
        messages,
        log,
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
    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![plaintext_log("changed_admin", new_admin)],
        data: None,
    })
}

pub fn change_beneficiaries<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_beneficiaries: Beneficiaries,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    config.assert_admin(&env.message.sender)?;

    let mut messages = vec![];
    let mut log = vec![];

    let total_balance = check_token_balance(&deps.querier, &config)?;
    if total_balance != 0 {
        let beneficiaries = Beneficiaries::load(&deps.storage)?;
        (messages, log) = withdraw_tax_for_everyone(&config, beneficiaries, total_balance)?;
    }

    for nb in &new_beneficiaries.list {
        log.push(plaintext_log("updated beneficiary", nb));
    }
    new_beneficiaries.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages,
        log,
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetBeneficiaries {} => get_beneficiaries(deps),
        QueryMsg::GetBeneficiaryBalance { address } => get_beneficiary_balance(deps, address),
    }
}

pub fn get_beneficiaries<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let beneficiaries = Beneficiaries::load(&deps.storage)?;
    Ok(to_binary(&beneficiaries)?)
}

pub fn get_beneficiary_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> QueryResult {
    let config = Config::load(&deps.storage)?;
    let beneficiaries = Beneficiaries::load(&deps.storage)?;
    let beneficiary = match beneficiaries.list.iter().find(|b| b.address == address) {
        None => return Err(StdError::generic_err("no such beneficiary exists")),
        Some(b) => b,
    };

    let total_balance = check_token_balance(&deps.querier, &config)?;
    let balance =
        beneficiary.check_beneficiary_balance(total_balance, beneficiaries.total_weight())?;

    Ok(to_binary(&Uint128::from(balance))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
