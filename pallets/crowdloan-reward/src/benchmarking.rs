//! Benchmarking setup for pallet-template

use super::*;
use crate::{
    types,
    Pallet as CrowdloanReward,
};
use frame_benchmarking::{account};
use frame_support::{
    assert_ok,
    traits::{
        Currency,
        Imbalance,
    },
};

#[allow(unused)]
use frame_benchmarking::{
    benchmarks,
    whitelisted_caller,
};
use frame_system::RawOrigin;

fn make_account<T: crate::Config>(id: u32) -> types::AccountIdOf<T> {
    account("crowdloan-account", 10, id)
}

benchmarks! {

    start_new_crowdloan {
        let l in 1_u32 .. 100_u32;
        let caller = make_account::<T>(2);
        let reward_source = make_account::<T>(10);
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(reward_source.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: None,
            end_target: Some(100_u32.into()),
        };
    }: _(RawOrigin::Signed(caller.clone()), l.into(), params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            total_pool: Some(10_000_000_u32.into()),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1_u32.into(),
            end_target: 100_u32.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(l.into()),
            Some(info),
        );
    }

    update_campaign {
        let l in 1_u32 .. 100_u32;
        let caller = make_account::<T>(1);
        let reward_source = make_account::<T>(10);
        let new_params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(reward_source.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(1_u32.into()),
            end_target: Some(100_u32.into()),
        };
        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                l.into(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(make_account::<T>(32)),
                    total_pool: Some(None),
                    instant_percentage: Some(types::SmallRational::new(0, 0)),
                    starts_from: None,
                    end_target: Some(0_u32.into()),
                }
            )
        );
    }: _(RawOrigin::Signed(caller.clone()), l.into(), new_params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            total_pool: Some(10_000_000_u32.into()),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1_u32.into(),
            end_target: 100_u32.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(l.into()),
            Some(info),
        );
    }

    add_contributer {
        let l in 1_u32.. 100_u32;
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 2_u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(0_u32.into()),
            end_target: Some(100_u32.into()),
        };

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id,
                params
            )
        );

    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, make_account::<T>(l), amount)
    verify {
        assert!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, make_account::<T>(l)).is_some()
        );
    }

    remove_contributer {
        let l in 1_u32.. 100_u32;
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 2_u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(0_u32.into()),
            end_target: Some(100_u32.into()),
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
                make_account::<T>(l),
                amount
            )
        );
        assert!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, make_account::<T>(l)).is_some(),
        );
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, make_account::<T>(l))
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, make_account::<T>(l)),
            None,
        );
    }

    get_instant_reward {
        let l in 0 .. 100;
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 10_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(0_u32.into()),
            end_target: Some(100_u32.into()),
        };

        assert_eq!(
            <T as crate::Config>::Currency::deposit_creating(&caller, 10_000_000_u32.into()).peek(),
            10_000_000_u32.into(),
        );
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
                make_account::<T>(l),
                10_000_u32.into()
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::lock_campaign(RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone())
        );
    }: _(RawOrigin::Signed(make_account::<T>(l)), crowdloan_id)
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, make_account::<T>(l))
                .map(|p| p.status),
            Some(types::ClaimerStatus::DoneInstant)
        );
    }

    get_vested_reward {
        let l in 0 .. 100;
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 10_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
            total_pool: Some(Some(10_000_000_u32.into())),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(0_u32.into()),
            end_target: Some(100_u32.into()),
        };

        assert_eq!(
            <T as crate::Config>::Currency::deposit_creating(&caller, 10_000_000_u32.into()).peek(),
            10_000_000_u32.into(),
        );
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
                make_account::<T>(l),
                10_000_u32.into()
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::lock_campaign(RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone())
        );
    }: _(RawOrigin::Signed(make_account::<T>(l)), crowdloan_id)
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, make_account::<T>(l))
                .map(|p| p.status),
            Some(types::ClaimerStatus::DoneVesting)
        );
    }

    lock_campaign {
        let l in 0 .. 100;
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                l.into(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(caller.clone()),
                    total_pool: Some(None),
                    instant_percentage: Some(types::SmallRational::new(1, 1)),
                    starts_from: None,
                    end_target: Some(10_u32.into()),
                }
            )
        );

        for contributer in 0_u32 .. 10_u32 {
            assert_ok!(
                CrowdloanReward::<T>::add_contributer(RawOrigin::Signed(caller.clone()).into(),
                    l.into(),
                    make_account::<T>(contributer),
                    10_000_u32.into()
                )
            );
        }
    }: _(RawOrigin::Signed(caller.clone()), l.into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_campaign_status::<types::CrowdloanIdOf<T>>(l.into()),
            Some(types::RewardCampaignStatus::Locked)
        );
    }

    wipe_campaign {
        let l in 0 .. 100;
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                l.into(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(caller.clone()),
                    total_pool: Some(None),
                    instant_percentage: Some(types::SmallRational::new(1, 1)),
                    starts_from: None,
                    end_target: Some(10_u32.into()),
                }
            )
        );
    }: _(RawOrigin::Signed(caller.clone()), l.into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_campaign_status::<types::CrowdloanIdOf<T>>(l.into()),
            Some(types::RewardCampaignStatus::Wiped)
        );
    }

    impl_benchmark_test_suite!(CrowdloanReward, crate::mock::new_test_ext(), crate::mock::Test);
}
