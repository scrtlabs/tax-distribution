use crate::state::{Beneficiaries, Beneficiary};
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct InitMsg {
    pub tax_denom: Option<String>,
    pub beneficiaries: Beneficiaries,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Withdraw { amount: Option<Uint128> },

    // Admin commands
    ChangeAdmin { new_admin: HumanAddr },
    ChangeBeneficiaries { beneficiaries: Beneficiaries },
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBeneficiaries {},
    GetBeneficiaryBalance { address: HumanAddr },
}
