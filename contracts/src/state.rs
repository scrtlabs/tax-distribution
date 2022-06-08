use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, HumanAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize)]
pub struct Config {
    admin: HumanAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
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

