use crate::state::Beneficiaries;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct InitMsg {
    pub tax_denom: Option<String>,
    pub beneficiaries: Beneficiaries,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Withdraw {},

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
