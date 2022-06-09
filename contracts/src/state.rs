use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdError, StdResult, Storage};
use primitive_types::U256;
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub static CONFIG_KEY: &[u8] = b"config";
pub static BENEFICIARIES_KEY: &[u8] = b"beneficiaries";
pub static BENEFICIARY_PREFIX: &[u8] = b"beneficiary";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub self_addr: HumanAddr,
    pub admin: HumanAddr,
    pub tax_denom: String,
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
pub struct Beneficiary {
    pub address: HumanAddr,
    weight: u16,
}

impl Display for Beneficiary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "address: {}, weight: {}", self.address, self.weight)
    }
}

impl Beneficiary {
    pub fn check_beneficiary_balance(
        &self,
        total_balance: u128,
        total_weight: u16,
    ) -> StdResult<u128> {
        let balance =
            U256::from(total_balance) * U256::from(self.weight) / U256::from(total_weight);
        Ok(balance.as_u128())
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Beneficiaries {
    pub list: Vec<Beneficiary>,
    decimal_places_in_rates: u8,
}

impl Beneficiaries {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        TypedStore::attach(storage).load(BENEFICIARIES_KEY)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        self.assert_valid()?;
        TypedStoreMut::attach(storage).store(BENEFICIARIES_KEY, self)
    }

    fn assert_valid(&self) -> StdResult<bool> {
        // Courtesy of @baedrik (https://github.com/baedrik/snip721-reference-impl/blob/632ce04/src/contract.rs#L4696)
        // the allowed message length won't let enough u16 weights to overflow u128
        let total_weights: u128 = self.list.iter().map(|r| r.weight as u128).sum();
        let (weight_den, overflow) =
            U256::from(10).overflowing_pow(U256::from(self.decimal_places_in_rates));
        if overflow {
            return Err(StdError::generic_err(
                "The number of decimal places used in the weights is larger than supported",
            ));
        }
        if U256::from(total_weights) != weight_den {
            return Err(StdError::generic_err(
                "The sum of weights must be exactly 100%",
            ));
        }

        Ok(true)
    }

    pub fn total_weight(&self) -> u16 {
        self.list.iter().map(|b| b.weight).sum()
    }
}
