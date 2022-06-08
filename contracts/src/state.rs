use crate::querier::check_token_balance;
use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, ReadonlyStorage, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use primitive_types::U256;
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static BENEFICIARIES_ADDRESSES_KEY: &[u8] = b"beneficiaries";
pub static BENEFICIARY_PREFIX: &[u8] = b"beneficiary";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub self_addr: HumanAddr,
    pub admin: HumanAddr,
    pub tx_denom: String,
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
    pub address: HumanAddr,
    weight: BeneficiaryWeight,
    debt: u128,
}

impl Beneficiary {
    pub fn load<S: ReadonlyStorage>(storage: &S, address: &HumanAddr) -> StdResult<Option<Self>> {
        let balances_store = ReadonlyPrefixedStorage::new(BENEFICIARY_PREFIX, storage);
        TypedStore::attach(&balances_store).may_load(address.0.as_bytes())
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        let mut balances_store = PrefixedStorage::new(BENEFICIARY_PREFIX, storage);
        TypedStoreMut::attach(&mut balances_store).store(self.address.0.as_bytes(), self)
    }

    pub fn check_beneficiary_balance<Q: Querier>(
        &self,
        querier: &Q,
        config: &Config,
    ) -> StdResult<u128> {
        let total_balance = check_token_balance(querier, config)?;

        let weight_denom = U256::from(10).pow(U256::from(self.weight.decimal_places_in_rate));
        let balance = U256::from(total_balance * self.weight.rate as u128) / weight_denom;

        Ok(balance.as_u128() - self.debt)
    }
}

pub struct Beneficiaries {}

impl Beneficiaries {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Option<Vec<HumanAddr>>> {
        TypedStore::attach(storage).may_load(BENEFICIARIES_ADDRESSES_KEY)
    }

    pub fn save<S: Storage>(beneficiaries: Vec<HumanAddr>, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(BENEFICIARIES_ADDRESSES_KEY, &beneficiaries)
    }
}
