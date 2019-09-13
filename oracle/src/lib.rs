#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue};
use system::{ensure_signed, ensure_root};
use support::traits::{Get, ChangeMembers};
use sr_primitives::traits::{EnsureOrigin, CheckedSub, CheckedAdd, Zero};
use rstd::prelude::*;

#[cfg(test)]
mod oracle_test;


pub trait Trait: system::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type OracleFee: Get<Self::Balance>;
    type MissReportSlash: Get<Self::Balance>;
    type MaliciousSlash: Get<Self::Balance>;
    type MinStaking: Get<Self::Balance>;

    type MaliciousSlashOrigin: EnsureOrigin<Self::Origin>;

    type Count: Get<u32>;

    type ReportInteval: Get<Self::BlockNumber>;
    type EraDuration: Get<Self::BlockNumber>;

    type ChangeMembers: ChangeMembers<Self::AccountId>;
}

pub trait OnWitnessed<T: Trait> {
    fn on_witnessed(who: &T::AccountId);
}

decl_storage! {
    trait Store for Module<T: Trait> as OracleStorage {
        Oracles get(oracles): Vec<T::AccountId>;
        OracleStakes get(oracle_stakes): map T::AccountId => T::Balance;
        Unbinding get(unbinding): map T::AccountId => T::Balance;

        WitnessReport get(witness_report): map T::AccountId => T::BlockNumber;

        OracleCandidates get(candidates): Vec<T::AccountId>;
        CurrentEra get(current_era): T::BlockNumber;
        
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        const OracleFee: T::Balance = T::OracleFee::get();
        const MissReportSlash: T::Balance = T::MissReportSlash::get();
        const MaliciousSlash: T::Balance = T::MaliciousSlash::get();
        const MinStaking: T::Balance = T::MinStaking::get();
        const Count: u32 = T::Count::get();
        const EraDuration: T::BlockNumber = T::EraDuration::get();
        const ReportInteval: T::BlockNumber = T::ReportInteval::get();


        pub fn bid(origin, staking: T::Balance) -> Result{
            let who = ensure_signed(origin)?;
            // Sub amount in T::Balance
            let already_staked = Self::oracle_stakes(&who);
            let new_staked = already_staked.checked_add(&staking).ok_or("Error calculating new staking")?;

            T::OracleStakes::insert(&who, new_staked);
            let candidates = Self::candidates();
            if !candidates.contains(&who) {
                candidates.push(who.clone());
                T::OracleCandidates::put(candidates);
                Self::deposit_event(RawEvent::CandidatesAdded(who.clone()));
            }
            Self::deposit_event(RawEvent::OracleBonded(who.clone(), staking));
            Ok(())
        }

        pub fn slash(origin, who: T::AccountId, amount: T::Balance) -> Result{
            T::MaliciousSlashOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            let already_staked = Self::oracle_stakes(&who);
            let new_staked = already_staked.checked_sub(&amount).ok_or("Error calculating new staking")?;
            T::OracleStakes::insert(&who, new_staked);
            Ok(())
        }

        pub fn unbond(origin, staking: T::Balance) -> Result{
            let who = ensure_signed(origin)?;

            let already_staked = Self::oracle_stakes(&who);
            if staking > already_staked {
                return Err("staking amount is smaller than unbonding amount");
            }

            let already_unbonding = Self::Unbinding(&who);
            let new_unbonding = already_unbonding.checked_add(&staking).ok_or("error calculating new unbonding")?;
            let new_staked = already_staked.checked_sub(&new_unbonding).ok_or("Error calculating new staking")?;

            T::Unbinding::insert(&who, new_unbonding);
            Self::deposit_event(RawEvent::OracleUnbonded(who.clone(), staking));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T>{
    pub fn on_finalize(block_number: T::BlockNumber) {
        Self::slash_oracles(block_number);

        let current_era = T::current_era();
        if block_number >= current_era + T::EraDuration{
            Self::elect_oracles();
            T::CurrentEr::put(current_era+T::EraDuration);
        }
    }

    fn slash_oracles(block_number: T::BlockNumber){
        let current_oracles = Self::oracles();

        current_oracles.iter().for_each(|o| {
            let last_report_height = Self::witness_report(&o);
            if block_number > last_report_height + Self::ReportInteval{
                Self::slash_oracle(&o);
            }
        });
    }

    fn elect_oracles(){
        let current_oracles = Self::oracles();
        let new_candidates = Self::candidates();
        let all_candidates: Vec<T::AccountId> = Vec::new();

        all_candidates.extend(new_candidates);
        all_candidates.extend(current_oracles);

        let all_candidates = all_candidates.iter().map(|a| {
            let unbonding = Self::unbinding(&a);
            (a, unbonding, Self::oracle_stakes(&a).checked_sub(&unbonding))
        }).collect();

        all_candidates.iter().for_each(|(a, bonded)|{
            if bonded > Zero::zero(){
                // TODO: Add balance
            }
        });

        let all_candidates = all_candidates.iter().
            filter(|(_, _, bonded)| bonded > Zero::zero()).
            sort_by(|(_, _, bonded)| bonded).
            map(|a, _, _| a).
            collect();

        let (chosen_candidates, new_candidates) = all_candidates.split_at(Self::Count);
        chosen_candidates.sort();

        let new_oracles = chosen_candidates.iter().filter(|o| !current_oracles.contains(o)).collect();
        let outgoing_oracles = current_oracles.iter().filter(|o| !new_oracles.contains(o)).collect();
        T::ChangeMembers::change_members(&new_oracles, &outgoing_oracles, chosen_candidates);
        T::Oracles::put(chosen_candidates); 
        T::OracleCandidates::put(new_candidates);
    }

    fn slash_oracle(who: &T::AccountId){
        let already_staked = Self::oracle_stakes(&who);
        if already_staked > T::MissReportSlash{
            let new_staked = already_staked - T::MissReportSlash;
            T::OracleStakes::insert(&who, new_staked);
            Self::deposit_event(RawEvent::OracleSlashed(who.clone(), T::MissReportSlash));
        }else{
            T::OracleStakes::destroy(&who);
            let current_oracles = Self::oracles();
            current_oracles.remove_item(&who);
            T::OracleCandidates::put(current_oracles);
            T::ChangeMembers::change_members(&[], &[who], current_oracles);
            Self::deposit_event(RawEvent::OracleSlashed(who.clone(), already_staked));
        }
    }
}

impl<T:Trait> OnWitnessed<T> for Module<T> {
    fn on_witnessed(who: &T::AccountId){
        let current_height = T::current_height();
        Self::WitnessReport::insert(who, current_height);
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
    {
        OracleBonded(AccountId, Balance),
        OracleUnbonded(AccountId, Balance),
        OracleSlashed(AccountId, Balance),

        CandidatesAdded(AccountId),
        CandidatesRemoved(AccountId),

    }
);
