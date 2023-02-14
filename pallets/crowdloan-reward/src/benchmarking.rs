//! Benchmarking setup for pallet-template

use super::*;
use crate::{
    types,
    Pallet as CrowdloanReward,
};
use frame_support::assert_ok;
use frame_system::Origin;

#[allow(unused)]
use frame_benchmarking::{
    benchmarks,
    whitelisted_caller,
};
use frame_system::RawOrigin;

benchmarks! {
    where_clause { where
        types::BlockNumberOf<T>: From<u64>,
        types::AccountIdOf<T>: From<u64>,
        types::BalanceOf<T>: From<u128>,
        types::VestingBalanceOf<T>: From<u128>,
    }

    start_new_crowdloan {
        let l in 1_u32 .. 100_u32;
        let caller: types::AccountIdOf<T> = 1_u64.into();
        let reward_source: types::AccountIdOf<T> = 10_u64.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(reward_source.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational {
                numenator: 3_u32.into(),
                denomator: 10_u32.into(),
            }),
            starts_from: None,
            end_target: Some(100u64.into()),
        };
    }: _(RawOrigin::Signed(caller.clone()), l.into(), params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            total_pool: Some(10_000_000_u32.into()),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1u64.into(),
            end_target: 100u64.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(l.into()),
            Some(info),
        );
    }

    add_contributer {
        let l in 1_u32.. 100_u32;
        let caller: types::AccountIdOf<T> = 1u64.into();
        let crowdloan_id: types::CrowdloanIdOf<T> = 2u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u128.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(1u64.into()),
            total_pool: Some(Some(10_000_000_u128.into())),
            instant_percentage: Some(types::SmallRational {
                numenator: 3_u32.into(),
                denomator: 10_u32.into(),
            }),
            starts_from: Some(0u64.into()),
            end_target: Some(100_u64.into()),
        };

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id,
                params
            )
        );

    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, (l as u64).into(), amount)
    verify {
        let reward = types::RewardUnitOf::<T> {
            instant_amount: 3_000_000_u128.into(),
            vesting_amount: 7_000_000_u128.into(),
            per_block: 70_000_u128.into(),
            status: types::ClaimerStatus::Unprocessed,
        };
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, (l as u64).into()),
            Some(reward)
        );
    }

    remove_contributer {
        let l in 1_u32.. 100_u32;
        let caller: types::AccountIdOf<T> = 1u64.into();
        let crowdloan_id: types::CrowdloanIdOf<T> = 2u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u128.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(1u64.into()),
            total_pool: Some(Some(10_000_000_u128.into())),
            instant_percentage: Some(types::SmallRational {
                numenator: 3_u32.into(),
                denomator: 10_u32.into(),
            }),
            starts_from: Some(0u64.into()),
            end_target: Some(100_u64.into()),
        };

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id,
                params
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::add_contributer(RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id,
                (l as u64).into(),
                amount
            )
        );
        assert!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, (l as u64).into()).is_some(),
        );
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, (l as u64).into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, (l as u64).into()),
            None,
        );
    }

    impl_benchmark_test_suite!(CrowdloanReward, crate::mock::new_test_ext(), crate::mock::Test);
}
