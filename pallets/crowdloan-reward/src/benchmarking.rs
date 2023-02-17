//! Benchmarking setup for pallet-template

use super::*;
use crate::{
    types,
    Pallet as CrowdloanReward,
};
use frame_benchmarking::account;
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
        let crowdloan_id = 1_u32;
        let caller = make_account::<T>(2);
        let reward_source = make_account::<T>(10);
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(reward_source.clone()),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: None,
            end_target: Some(100_u32.into()),
        };
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into(), params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1_u32.into(),
            end_target: 100_u32.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(crowdloan_id.into()),
            Some(info),
        );
    }

    update_campaign {
        let crowdloan_id = 33_u32;
        let caller = make_account::<T>(1);
        let reward_source = make_account::<T>(10);
        let new_params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(reward_source.clone()),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(1_u32.into()),
            end_target: Some(100_u32.into()),
        };
        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.into(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(make_account::<T>(32)),
                    instant_percentage: Some(types::SmallRational::new(0, 0)),
                    starts_from: None,
                    end_target: Some(0_u32.into()),
                }
            )
        );
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into(), new_params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller,
            reward_source,
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 1_u32.into(),
            end_target: 100_u32.into(),
        };
        assert_eq!(
            CrowdloanReward::<T>::get_reward_info::<types::CrowdloanIdOf<T>>(crowdloan_id.into()),
            Some(info),
        );
    }

    add_contributer {
        let contributer = make_account::<T>(22);
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 2_u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
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

    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, contributer.clone(), amount)
    verify {
        assert!(
            CrowdloanReward::<T>::get_contribution(crowdloan_id, contributer).is_some()
        );
    }

    remove_contributer {
        let contributer = make_account::<T>(22);
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 2_u32.into();
        let amount: types::BalanceOf<T> = 10_000_000_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
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
                contributer.clone(),
                amount
            )
        );
        assert!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, contributer.clone()).is_some(),
        );
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id, contributer.clone())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution::<_, types::AccountIdOf<T>>(crowdloan_id, contributer.clone()),
            None,
        );
    }

    get_instant_reward {
        let contributer = make_account::<T>(22);
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 10_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
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
                contributer.clone(),
                10_000_u32.into()
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::lock_campaign(RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone())
        );
    }: _(RawOrigin::Signed(contributer.clone()), crowdloan_id)
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution(crowdloan_id, contributer.clone())
                .map(|p| p.status),
            Some(types::ClaimerStatus::DoneInstant)
        );
    }

    get_vested_reward {
        let contributer = make_account::<T>(22);
        let caller = make_account::<T>(1);
        let crowdloan_id: types::CrowdloanIdOf<T> = 10_u32.into();
        let params = types::CrowdloanRewardParamFor::<T> {
            hoster: None,
            reward_source: Some(caller.clone()),
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
                contributer.clone(),
                10_000_u32.into()
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::lock_campaign(RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone())
        );
    }: _(RawOrigin::Signed(contributer.clone()), crowdloan_id)
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_contribution(crowdloan_id, contributer.clone())
                .map(|p| p.status),
            Some(types::ClaimerStatus::DoneVesting)
        );
    }

    lock_campaign {
        let crowdloan_id = 2_u32;
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.into(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(caller.clone()),
                    instant_percentage: Some(types::SmallRational::new(1, 1)),
                    starts_from: None,
                    end_target: Some(10_u32.into()),
                }
            )
        );

        for contributer in 0_u32 .. 10_u32 {
            assert_ok!(
                CrowdloanReward::<T>::add_contributer(RawOrigin::Signed(caller.clone()).into(),
                    crowdloan_id.into(),
                    make_account::<T>(contributer),
                    10_000_u32.into()
                )
            );
        }
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_campaign_status::<types::CrowdloanIdOf<T>>(crowdloan_id.into()),
            Some(types::RewardCampaignStatus::Locked)
        );
    }

    wipe_campaign {
        let crowdloan_id: types::CrowdloanIdOf<T> = 3_u32.into();
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.clone(),
                types::CrowdloanRewardParamFor::<T> {
                    hoster: None,
                    reward_source: Some(caller.clone()),
                    instant_percentage: Some(types::SmallRational::new(1, 1)),
                    starts_from: None,
                    end_target: Some(10_u32.into()),
                }
            )
        );

        for contributer in 0_u32 .. 10_u32 {
            crate::Contribution::<T>::insert(crowdloan_id.clone(), make_account::<T>(contributer), types::RewardUnitOf::<T> {
                instant_amount: 10_000_u32.into(),
                vesting_amount: 10_000_u32.into(),
                per_block: 100_000_u32.into(),
                status: types::ClaimerStatus::DoneBoth,
            });
        }

        assert_ok!(CrowdloanReward::<T>::lock_campaign( RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone()));
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_campaign_status(crowdloan_id.clone()),
            Some(types::RewardCampaignStatus::Wiped)
        );
    }

    impl_benchmark_test_suite!(CrowdloanReward, crate::mock::new_test_ext(), crate::mock::Test);
}
