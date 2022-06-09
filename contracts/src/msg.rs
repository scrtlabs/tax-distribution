use crate::state::Beneficiary;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct InitMsg {
    pub tax_denom: Option<String>,
    pub beneficiaries: Vec<Beneficiary>,
    pub decimal_places_in_weights: u8,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Withdraw {
        amount: Option<Uint128>,
    },

    // Admin commands
    ChangeAdmin {
        new_admin: HumanAddr,
    },
    SetBeneficiaries {
        beneficiaries: Vec<Beneficiary>,
        decimal_places_in_weights: u8,
    },
    // todo add emergency redeem
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBeneficiaries {},
    GetBeneficiaryBalance { address: HumanAddr },
}
