use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Beneficiary;

#[derive(Deserialize, JsonSchema)]
pub struct InitMsg {
    pub tax_denom: Option<String>,
    pub beneficiaries: Vec<Beneficiary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Withdraw { amount: Option<Uint128> },

    // Admin commands
    ChangeAdmin { new_admin: HumanAddr },
    ChangeBeneficiaries {
        beneficiaries: Vec<Beneficiary>
    },
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBeneficiaries {},
    GetBeneficiaryBalance { address: HumanAddr },
}
