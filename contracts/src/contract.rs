use cosmwasm_std::{
    plaintext_log, to_binary, Api, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, InitResult, Querier, QueryResult, StdError, Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::querier::query_token_balance;
use crate::state::{BeneficiariesList, Beneficiary, Config, StoredBeneficiary, TaxPool};
use crate::util::send_native_token_msg;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    Config {
        self_addr: env.contract.address,
        admin: env.message.sender,
        tax_denom: msg.tax_denom.unwrap_or_else(|| "uscrt".to_string()),
    }
    .save(&mut deps.storage)?;

    BeneficiariesList::save(
        &mut deps.storage,
        &msg.beneficiaries,
        msg.decimal_places_in_weights,
    )?;

    TaxPool {
        total_weight: 10_u16.pow(msg.decimal_places_in_weights as u32),
        total_withdrawn: 0,
        acc_tax_per_share: 0,
    }
    .save(&mut deps.storage)?;

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
        HandleMsg::SetBeneficiaries {
            beneficiaries,
            decimal_places_in_weights,
        } => set_beneficiaries(deps, env, beneficiaries, decimal_places_in_weights),
        HandleMsg::EmergencyWithdraw {} => emergency_withdraw(deps, env),
    }
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Option<u128>,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    let mut beneficiary =
        StoredBeneficiary::load(&deps.storage, &env.message.sender)?.unwrap_or_default();
    let mut tax_pool = TaxPool::load_updated(deps, &config)?;

    let beneficiary_balance = beneficiary.get_balance(&tax_pool);
    let amount = amount.unwrap_or(beneficiary_balance); // If not specified - get everything

    if amount > beneficiary_balance {
        return Err(StdError::generic_err(format!(
            "insufficient funds to withdraw: balance={}, required={}",
            beneficiary_balance, amount,
        )));
    }

    beneficiary.debt += amount;
    tax_pool.total_withdrawn += amount;
    beneficiary.save(&mut deps.storage, &env.message.sender)?;
    tax_pool.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![send_native_token_msg(&env.message.sender, amount, &config)],
        log: vec![
            plaintext_log("tax_withdrawn", env.message.sender),
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
    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![plaintext_log("changed_admin", new_admin)],
        data: None,
    })
}

pub fn set_beneficiaries<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_beneficiaries: Vec<Beneficiary>,
    decimal_places_in_weights: u8,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    config.assert_admin(&env.message.sender)?;

    let mut messages = vec![];
    let mut log = vec![];
    let tax_pool = TaxPool::load_updated(deps, &config)?;

    // Send all tokens to existing beneficiaries and delete
    let current_beneficiaries = BeneficiariesList::load(&deps.storage)?;
    for b_addr in current_beneficiaries {
        let b = StoredBeneficiary::load(&deps.storage, &b_addr)?.unwrap_or_default();
        let balance = b.get_balance(&tax_pool);

        messages.push(send_native_token_msg(&b_addr, balance, &config));
        log.extend(vec![
            plaintext_log("tax_withdrawn", b_addr.clone()),
            plaintext_log("amount", balance),
        ]);

        StoredBeneficiary::delete(&mut deps.storage, &b_addr);
    }

    // Reset everything
    BeneficiariesList::save(
        &mut deps.storage,
        &new_beneficiaries,
        decimal_places_in_weights,
    )?;
    TaxPool {
        total_weight: 10_u16.pow(decimal_places_in_weights as u32),
        total_withdrawn: 0,
        acc_tax_per_share: 0,
    }
    .save(&mut deps.storage)?;

    log.push(plaintext_log(
        "beneficiaries updated",
        format!("{:?}", new_beneficiaries),
    ));

    Ok(HandleResponse {
        messages,
        log,
        data: None,
    })
}

pub fn emergency_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    config.assert_admin(&env.message.sender)?;

    let balance = query_token_balance(&deps.querier, &config)?;

    Ok(HandleResponse {
        messages: vec![send_native_token_msg(&env.message.sender, balance, &config)],
        log: vec![
            plaintext_log("emergency_withdraw", env.message.sender),
            plaintext_log("amount", balance),
        ],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetBeneficiaries {} => get_beneficiaries(deps),
        QueryMsg::GetBeneficiaryBalance { address } => get_beneficiary_balance(deps, address),
        QueryMsg::GetAdmin {} => get_admin(deps),
    }
}

pub fn get_beneficiaries<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let beneficiaries = BeneficiariesList::load(&deps.storage)?;
    let mut stored_beneficiaries = vec![];
    for b_addr in beneficiaries {
        let stored = StoredBeneficiary::load(&deps.storage, &b_addr)?.unwrap_or_default();
        stored_beneficiaries.push(stored);
    }

    to_binary(&stored_beneficiaries)
}

pub fn get_beneficiary_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> QueryResult {
    let config = Config::load(&deps.storage)?;
    let tax_pool = TaxPool::load_updated(deps, &config /*env.block.height*/)?;
    let beneficiary = StoredBeneficiary::load(&deps.storage, &address)?.unwrap_or_default();
    let balance = beneficiary.get_balance(&tax_pool);

    to_binary(&Uint128::from(balance))
}

pub fn get_admin<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::load(&deps.storage)?;

    to_binary(&config.admin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
