use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, HumanAddr, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub static CONFIG_KEY: &[u8] = b"config";
pub static BENEFICIARIES_KEY: &[u8] = b"beneficiaries";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub admin: HumanAddr,
}

impl Config {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Option<Self>> {
        TypedStore::attach(storage).may_load(CONFIG_KEY)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(CONFIG_KEY, self)
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BeneficiaryWeight {
    rate: u16,
    decimal_places_in_rate: u8,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Beneficiary {
    address: HumanAddr,
    weight: BeneficiaryWeight,
}

pub struct Beneficiaries {}

impl Beneficiaries {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Option<Vec<Beneficiary>>> {
        TypedStore::attach(storage).may_load(BENEFICIARIES_KEY)
    }

    pub fn save<S: Storage>(beneficiaries: Vec<Beneficiary>, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(BENEFICIARIES_KEY, &beneficiaries)
    }
}

