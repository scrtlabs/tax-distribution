use cosmwasm_std::{
    plaintext_log, to_binary, Api, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, InitResult, Querier, QueryResult, StdError, Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryAnswer, QueryBeneficiary, QueryMsg};
use crate::state::{BeneficiariesList, Beneficiary, Config, StoredBeneficiary, TaxPool};
use crate::util::{query_token_balance, send_native_token_msg};

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
    let mut beneficiary = StoredBeneficiary::load(&deps.storage, &env.message.sender)
        .map_err(|e| StdError::generic_err(format!("cannot load beneficiary: {:?}", e)))?;
    let config = Config::load(&deps.storage)?;
    let mut tax_pool = TaxPool::load_updated(deps, &config)?;

    let beneficiary_balance = beneficiary.get_balance(&tax_pool);
    let amount = amount.unwrap_or(beneficiary_balance); // If not specified - get everything

    if amount > beneficiary_balance {
        return Err(StdError::generic_err(format!(
            "insufficient funds to withdraw: balance={}, required={}",
            beneficiary_balance, amount,
        )));
    }

    beneficiary.withdrawn += amount;
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
        let b = StoredBeneficiary::load(&deps.storage, &b_addr).unwrap_or_default();
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

    let mut messages = vec![];
    if balance > 0 {
        messages.push(send_native_token_msg(&env.message.sender, balance, &config));
    }

    Ok(HandleResponse {
        messages,
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
    let mut result_list = vec![];
    for b_addr in beneficiaries {
        let stored = StoredBeneficiary::load(&deps.storage, &b_addr).unwrap_or_default();
        result_list.push(QueryBeneficiary {
            address: b_addr,
            weight: stored.weight,
            withdrawn: Uint128::from(stored.withdrawn),
        });
    }

    to_binary(&QueryAnswer::GetBeneficiaries {
        beneficiaries: result_list,
    })
}

pub fn get_beneficiary_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> QueryResult {
    let config = Config::load(&deps.storage)?;
    let tax_pool = TaxPool::load_updated(deps, &config)?;
    let beneficiary = StoredBeneficiary::load(&deps.storage, &address)
        .map_err(|e| StdError::generic_err(format!("cannot load beneficiary: {:?}", e)))?;

    to_binary(&QueryAnswer::GetBeneficiaryBalance {
        balance: Uint128::from(beneficiary.get_balance(&tax_pool)),
    })
}

pub fn get_admin<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::load(&deps.storage)?;

    to_binary(&QueryAnswer::GetAdmin {
        address: config.admin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{Coin, StdError, StdResult};
    use std::any::type_name;

    fn add_decimals(amount: u128) -> u128 {
        amount * 10_u128.pow(6)
    }

    fn add_tax(
        amount: u128,
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        total_tax_received: &mut u128,
    ) -> StdResult<()> {
        let config = Config {
            self_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
            admin: HumanAddr::from("admin"),
            tax_denom: "uscrt".to_string(),
        };
        let contract_balance = query_token_balance(&deps.querier, &config)?;
        let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
        deps.querier = MockQuerier::new(&[(
            &contract_addr,
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(contract_balance + amount),
            }],
        )]);

        *total_tax_received += amount;

        Ok(())
    }

    fn withdraw_helper(
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        beneficiary: &str,
        amount: Option<u128>,
    ) -> HandleResult {
        withdraw(deps, mock_env(HumanAddr::from(beneficiary), &[]), amount)
    }

    fn withdraw_tester(
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        beneficiary: &Beneficiary,
        beneficiary_withdrawn: &mut u128,
        amount: Option<u128>,
        total_tax_received: u128,
        total_withdrawn: &mut u128,
        total_weight: u16,
    ) -> StdResult<()> {
        let config = Config {
            self_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
            admin: HumanAddr::from("admin"),
            tax_denom: "uscrt".to_string(),
        };

        let expected = amount.unwrap_or(
            total_tax_received * beneficiary.weight as u128 / total_weight as u128
                - *beneficiary_withdrawn,
        );
        let result = withdraw_helper(deps, beneficiary.address.as_str(), amount)?;
        assert_eq!(
            result,
            HandleResponse {
                messages: vec![send_native_token_msg(
                    &beneficiary.address,
                    expected,
                    &config,
                )],
                log: vec![
                    plaintext_log("tax_withdrawn", beneficiary.address.clone()),
                    plaintext_log("amount", expected),
                ],
                data: None,
            }
        );

        *beneficiary_withdrawn += expected;
        *total_withdrawn += expected;

        let contract_balance = query_token_balance(&deps.querier, &config)?;
        deps.querier = MockQuerier::new(&[(
            &config.self_addr,
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(contract_balance - expected),
            }],
        )]);

        Ok(())
    }

    #[test]
    fn test_sanity() -> StdResult<()> {
        let mut deps = mock_dependencies(20, &[]);
        let a = Beneficiary {
            address: HumanAddr::from("a"),
            weight: 350,
        };
        let b = Beneficiary {
            address: HumanAddr::from("b"),
            weight: 650,
        };
        let total_weight = a.weight + b.weight;

        let init_error = init(
            &mut deps,
            mock_env(HumanAddr::from("admin"), &[]),
            InitMsg {
                tax_denom: None,
                beneficiaries: vec![
                    Beneficiary {
                        address: HumanAddr::from("a"),
                        weight: 350,
                    },
                    Beneficiary {
                        address: HumanAddr::from("b"),
                        weight: 350,
                    },
                ],
                decimal_places_in_weights: 3,
            },
        )
        .unwrap_err();

        assert_eq!(
            init_error,
            StdError::generic_err("The sum of weights must be exactly 100%")
        );

        let init_result = init(
            &mut deps,
            mock_env(HumanAddr::from("admin"), &[]),
            InitMsg {
                tax_denom: None,
                beneficiaries: vec![a.clone(), b.clone()],
                decimal_places_in_weights: 3,
            },
        );

        assert!(init_result.is_ok());

        let withdraw_err = withdraw_helper(&mut deps, "a", Some(add_decimals(2000))).unwrap_err();
        assert_eq!(
            withdraw_err,
            StdError::generic_err(format!(
                "insufficient funds to withdraw: balance={}, required={}",
                0,
                add_decimals(2000),
            ))
        );

        let mut total_tax_received = 0;

        add_tax(add_decimals(1000), &mut deps, &mut total_tax_received)?;
        let withdraw_err = withdraw_helper(&mut deps, "a", Some(add_decimals(2000))).unwrap_err();
        assert_eq!(
            withdraw_err,
            StdError::generic_err(format!(
                "insufficient funds to withdraw: balance={}, required={}",
                add_decimals(350),
                add_decimals(2000),
            ))
        );

        let withdraw_err = withdraw_helper(&mut deps, "c", Some(add_decimals(2000))).unwrap_err();
        assert_eq!(
            withdraw_err,
            StdError::generic_err(format!(
                "cannot load beneficiary: {:?}",
                StdError::not_found(type_name::<StoredBeneficiary>())
            ))
        );

        let mut withdrawn_a = 0;
        let mut withdrawn_b = 0;
        let mut total_withdrawn = 0;

        withdraw_tester(
            &mut deps,
            &a,
            &mut withdrawn_a,
            Some(add_decimals(150)),
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;
        withdraw_tester(
            &mut deps,
            &a,
            &mut withdrawn_a,
            None,
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        add_tax(add_decimals(10_000), &mut deps, &mut total_tax_received)?;

        withdraw_tester(
            &mut deps,
            &b,
            &mut withdrawn_b,
            None,
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        add_tax(add_decimals(10_000), &mut deps, &mut total_tax_received)?;
        add_tax(add_decimals(123), &mut deps, &mut total_tax_received)?;

        withdraw_tester(
            &mut deps,
            &b,
            &mut withdrawn_b,
            Some(add_decimals(70)),
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        add_tax(add_decimals(2_000_000), &mut deps, &mut total_tax_received)?;

        withdraw_tester(
            &mut deps,
            &a,
            &mut withdrawn_a,
            Some(add_decimals(130)),
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        withdraw_tester(
            &mut deps,
            &b,
            &mut withdrawn_b,
            None,
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        withdraw_tester(
            &mut deps,
            &a,
            &mut withdrawn_a,
            None,
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        withdraw_tester(
            &mut deps,
            &b,
            &mut withdrawn_b,
            None,
            total_tax_received,
            &mut total_withdrawn,
            total_weight,
        )?;

        assert_eq!(withdrawn_a + withdrawn_b, total_tax_received);

        Ok(())
    }
}
