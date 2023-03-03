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
use sp_runtime::traits::Bounded;

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
    where_clause { where
        types::BalanceOf<T>: Bounded,
    }

    start_new_crowdloan {
        let crowdloan_id = 1_u32;
        let caller = make_account::<T>(2);
        let params = types::CreateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: None,
            end_target: 100_u32.into(),
        };
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into(), params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller.clone(),
            reward_source: caller,
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
        let new_params = types::UpdateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(1_u32.into()),
            end_target: Some(100_u32.into()),
        };
        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.into(),
                types::CreateCampaignParamFor::<T> {
                    hoster: None,
                    instant_percentage: types::SmallRational::new(0, 0),
                    starts_from: None,
                    end_target: 0_u32.into(),
                }
            )
        );
    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into(), new_params)
    verify {
        let info = types::CrowdloanRewardFor::<T> {
            hoster: caller.clone(),
            reward_source: caller,
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
        let params = types::CreateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: Some(0_u32.into()),
            end_target: 100_u32.into(),
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
        let params = types::CreateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: Some(0_u32.into()),
            end_target: 100_u32.into(),
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
        let params = types::CreateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: types::SmallRational::new(1, 1),
            starts_from: Some(0_u32.into()),
            end_target: 100_u32.into(),
        };

        let deposit_amount = <T as crate::Config>::Currency::minimum_balance() * 2_u32.into();
        assert_eq!(<T as crate::Config>::Currency::deposit_creating(&caller, Bounded::max_value()).peek(), Bounded::max_value());
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
                deposit_amount,
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
        let params = types::CreateCampaignParamFor::<T> {
            hoster: None,
            instant_percentage: types::SmallRational::new(5, 10),
            starts_from: Some(1_u32.into()),
            end_target: 10_u32.into(),
        };

        assert_eq!(<T as crate::Config>::Currency::deposit_creating(&caller, Bounded::max_value()).peek(), Bounded::max_value());
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
                <types::BalanceOf<T> as Bounded>::max_value() - 10_000_000_u32.into(),
            )
        );
        assert_ok!(
            CrowdloanReward::<T>::lock_campaign(RawOrigin::Signed(caller.clone()).into(), crowdloan_id.clone())
        );
        log::info!("Vesting init complete with amount: total,per_block: {:?}", {
            let info = CrowdloanReward::<T>::get_contribution(crowdloan_id, contributer.clone()).unwrap();
            (info.vesting_amount, info.per_block)
        });
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
                types::CreateCampaignParamFor::<T> {
                    hoster: None,
                    instant_percentage: types::SmallRational::new(1, 1),
                    starts_from: None,
                    end_target: 10_u32.into(),
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

    // TODO:
    // make weight dependent on number of contributer
    // that exists in this campaign
    wipe_campaign {
        let crowdloan_id: types::CrowdloanIdOf<T> = 3_u32.into();
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.clone(),
                types::CreateCampaignParamFor::<T> {
                    hoster: None,
                    instant_percentage: types::SmallRational::new(1, 1),
                    starts_from: None,
                    end_target: 10_u32.into(),
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

    discard_campaign {
        let crowdloan_id: types::CrowdloanIdOf<T> = 3_u32.into();
        let caller = make_account::<T>(2);

        assert_ok!(
            CrowdloanReward::<T>::start_new_crowdloan(
                RawOrigin::Signed(caller.clone()).into(),
                crowdloan_id.clone(),
                types::CreateCampaignParamFor::<T> {
                    hoster: None,
                    instant_percentage: types::SmallRational::new(1, 1),
                    starts_from: None,
                    end_target: 10_u32.into(),
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

    }: _(RawOrigin::Signed(caller.clone()), crowdloan_id.into())
    verify {
        assert_eq!(
            CrowdloanReward::<T>::get_campaign_status(crowdloan_id.clone()),
            None
        );
    }

    impl_benchmark_test_suite!(CrowdloanReward, crate::mock::new_test_ext(), crate::mock::Test);
}
