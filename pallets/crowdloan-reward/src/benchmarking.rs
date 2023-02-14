//! Benchmarking setup for pallet-template

use super::*;
use crate::{
    types,
    Pallet as CrowdloanReward,
};

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
            end_target: Some(100u32.into()),
        };
    }: _(RawOrigin::Signed(caller.clone()), l.into(), params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            total_pool: Some(10_000_000_u32.into()),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1u32.into(),
            end_target: 100u32.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(l.into()),
            Some(info),
        );
    }

    impl_benchmark_test_suite!(CrowdloanReward, crate::mock::new_test_ext(), crate::mock::Test);
}
