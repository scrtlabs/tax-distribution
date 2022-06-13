use crate::state::Beneficiary;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    EmergencyWithdraw {},
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBeneficiaries {},
    GetBeneficiaryBalance { address: HumanAddr },
    GetAdmin {},
}

#[derive(Serialize, JsonSchema)]
pub struct QueryBeneficiary {
    pub address: HumanAddr,
    pub weight: u16,
    pub withdrawn: Uint128,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetBeneficiaries {
        beneficiaries: Vec<QueryBeneficiary>,
    },
    GetBeneficiaryBalance {
        balance: Uint128,
    },
    GetAdmin {
        address: HumanAddr,
    },
}
