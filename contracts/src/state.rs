use crate::querier::check_token_balance;
use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, ReadonlyStorage, StdError, StdResult, Storage,
    Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use primitive_types::U256;
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static BENEFICIARIES_KEY: &[u8] = b"beneficiaries";
pub static BENEFICIARY_PREFIX: &[u8] = b"beneficiary";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub self_addr: HumanAddr,
    pub admin: HumanAddr,
    pub tx_denom: String,
}

impl Config {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        TypedStore::attach(storage).load(CONFIG_KEY)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(CONFIG_KEY, self)
    }

    pub fn assert_admin(&self, address: &HumanAddr) -> StdResult<()> {
        if address != &self.admin {
            return Err(StdError::generic_err(format!(
                "Address {} is not allowed to perform this operation",
                address
            )));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BeneficiaryWeight {
    rate: u16,
    decimal_places_in_rate: u8,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Beneficiary {
    pub address: HumanAddr,
    weight: BeneficiaryWeight,
}

impl Beneficiary {
    pub fn check_beneficiary_balance(&self, total_balance: u128) -> StdResult<u128> {
        let weight_denom = U256::from(10).pow(U256::from(self.weight.decimal_places_in_rate));
        let balance = U256::from(total_balance * self.weight.rate as u128) / weight_denom;

        Ok(balance.as_u128())
    }
}

pub struct Beneficiaries {}

impl Beneficiaries {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Vec<Beneficiary>> {
        TypedStore::attach(storage).load(BENEFICIARIES_KEY)
    }

    pub fn save<S: Storage>(storage: &mut S, beneficiaries: Vec<Beneficiary>) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(BENEFICIARIES_KEY, &beneficiaries)
    }
}
