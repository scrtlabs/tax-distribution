use crate::querier::query_token_balance;
use cosmwasm_std::{
    Api, Extern, HumanAddr, Querier, ReadonlyStorage, StdError, StdResult, Storage,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use primitive_types::U256;
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static TAX_POOL_KEY: &[u8] = b"tax_pool";
pub static BENEFICIARY_PREFIX: &[u8] = b"beneficiary";
pub static BENEFICIARIES_LIST_KEY: &[u8] = b"beneficiaries_list";

pub const REWARD_SCALE: u128 = 1_000_000_000_000_000_000; // 10 ^ 18

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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TaxPool {
    total_weight: u16,
    pub total_withdrawn: u128,
    acc_tax_per_share: u128,
}

impl TaxPool {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        TypedStore::attach(storage).load(TAX_POOL_KEY)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(TAX_POOL_KEY, self)
    }

    pub fn update<Q: Querier>(&self, querier: &Q, config: &Config) -> StdResult<Self> {
        let current_balance = query_token_balance(querier, config)?;
        let new_total_income = current_balance + self.total_withdrawn;
        Ok(Self {
            total_weight: self.total_weight,
            total_withdrawn: self.total_withdrawn,
            acc_tax_per_share: new_total_income * REWARD_SCALE / self.total_weight as u128,
        })
    }

    pub fn load_updated<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        config: &Config,
    ) -> StdResult<Self> {
        let tax_pool = Self::load(&deps.storage)?;
        tax_pool.update(&deps.querier, config /*block*/)
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Beneficiary {
    pub address: HumanAddr,
    pub weight: u16,
}

#[derive(Serialize, Deserialize, Default)]
pub struct StoredBeneficiary {
    pub weight: u16,
    pub debt: u128,
}

impl StoredBeneficiary {
    pub fn new(beneficiary: &Beneficiary) -> Self {
        Self {
            weight: beneficiary.weight,
            debt: 0,
        }
    }

    pub fn load<S: ReadonlyStorage>(
        storage: &S,
        beneficiary: &HumanAddr,
    ) -> StdResult<Option<Self>> {
        let beneficiary_store = ReadonlyPrefixedStorage::new(BENEFICIARY_PREFIX, storage);
        TypedStore::attach(&beneficiary_store).may_load(beneficiary.0.as_bytes())
    }

    pub fn save<S: Storage>(&self, storage: &mut S, beneficiary: &HumanAddr) -> StdResult<()> {
        let mut beneficiary_store = PrefixedStorage::new(BENEFICIARY_PREFIX, storage);
        TypedStoreMut::attach(&mut beneficiary_store).store(beneficiary.0.as_bytes(), self)
    }

    pub fn delete<S: Storage>(storage: &mut S, beneficiary: &HumanAddr) {
        let mut beneficiary_store = PrefixedStorage::new(BENEFICIARY_PREFIX, storage);
        TypedStoreMut::<Self, PrefixedStorage<S>>::attach(&mut beneficiary_store)
            .remove(beneficiary.0.as_bytes())
    }

    pub fn get_balance(&self, tax_pool: &TaxPool) -> u128 {
        let cut = U256::from(self.weight) * U256::from(tax_pool.acc_tax_per_share)
            / U256::from(REWARD_SCALE)
            - U256::from(self.debt);
        cut.as_u128()
    }
}

pub struct BeneficiariesList {}

impl BeneficiariesList {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Vec<HumanAddr>> {
        TypedStore::attach(storage).load(BENEFICIARIES_LIST_KEY)
    }

    pub fn save<S: Storage>(
        storage: &mut S,
        beneficiaries: &[Beneficiary],
        decimal_places_in_weights: u8,
    ) -> StdResult<()> {
        Self::assert_valid_beneficiaries(beneficiaries, decimal_places_in_weights)?;

        let addresses: Vec<HumanAddr> = beneficiaries.iter().map(|b| b.address.clone()).collect();
        TypedStoreMut::attach(storage).store(BENEFICIARIES_LIST_KEY, &addresses)?;

        for b in beneficiaries {
            let stored = StoredBeneficiary::new(b);
            stored.save(storage, &b.address)?;
        }

        Ok(())
    }

    fn assert_valid_beneficiaries(
        beneficiaries: &[Beneficiary],
        decimal_places_in_weights: u8,
    ) -> StdResult<()> {
        // Courtesy of @baedrik (https://github.com/baedrik/snip721-reference-impl/blob/632ce04/src/contract.rs#L4696)
        // the allowed message length won't let enough u16 weights to overflow u128
        let total_weights: u128 = beneficiaries.iter().map(|r| r.weight as u128).sum();
        let (weight_den, overflow) =
            U256::from(10).overflowing_pow(U256::from(decimal_places_in_weights));
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

        Ok(())
    }
}
