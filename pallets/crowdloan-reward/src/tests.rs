use crate::{
    functions,
    mock::*,
    types::{
        self,
        RewardCampaignStatus,
        SmallRational,
    },
};
use frame_support::{
    assert_noop,
    assert_ok,
    assert_storage_noop,
    inherent::BlockT,
    IterableStorageDoubleMap,
};
use frame_system::RawOrigin;

pub const DOLLARS: u128 = 1_000_000_000_000_000_000_u128;
type RewardError = crate::Error<Test>;
type RewardEvent = crate::Event<Test>;

#[test]
fn campaign_creation_success() {
    new_test_ext().execute_with(|| {
        // move the current block number
        let current_block_num = 5;
        run_to_block(current_block_num);

        // initilization data for a crowdloan campaign
        let reward_source = 1u32.into();
        let crowdloan_id = 3u32;
        let hoster = 10u32.into();
        let reward_pool = 10_000_000;
        let info = types::CrowdloanRewardParamFor::<Test> {
            hoster: None,
            reward_source: Some(reward_source),
            instant_percentage: Some(types::SmallRational {
                numenator: 3,
                denomator: 10,
            }),
            starts_from: None,
            end_target: Some(100u32.into()),
        };

        // Put enough balance in creditor
        credit_account::<Test>(&reward_source, reward_pool);

        // Create crowdloan
        assert_ok!(Reward::start_new_crowdloan(Origin::signed(hoster), crowdloan_id, info.clone()));
        // Make sure status is `InProgress`
        assert_eq!(Reward::get_campaign_status(&crowdloan_id), Some(types::RewardCampaignStatus::InProgress));
        // Also check the reward details is filled correctly
        assert_eq!(
            Reward::get_reward_info(&crowdloan_id),
            Some(types::CrowdloanRewardFor::<Test> {
                hoster,
                starts_from: current_block_num,
                instant_percentage: types::SmallRational {
                    numenator: 3,
                    denomator: 10
                },
                end_target: 100u32.into(),
                reward_source,
            })
        );

        // Data of a usual contributer
        let user_a = (11u32.into(), 1_000_000_u128);
        let user_b = (12u32.into(), 2_100_004_u128);
        let user_a_reward = types::RewardUnitOf::<Test> {
            instant_amount: 300_000,
            vesting_amount: 700_000,
            per_block: 7368,
            status: types::ClaimerStatus::Unprocessed,
        };
        let user_b_reward = types::RewardUnitOf::<Test> {
            instant_amount: 630001,
            vesting_amount: 1470003,
            per_block: 15473,
            status: types::ClaimerStatus::Unprocessed,
        };

        assert_eq!(user_a_reward.instant_amount + user_a_reward.vesting_amount, user_a.1);
        assert_eq!(user_b_reward.instant_amount + user_b_reward.vesting_amount, user_b.1);

        // add users
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, user_a.0, user_a.1));
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, user_b.0, user_b.1));

        // Check contributers are added properly
        assert_eq!(Reward::get_contribution(crowdloan_id, user_a.0).as_ref(), Some(&user_a_reward));
        assert_eq!(Reward::get_contribution(crowdloan_id, user_b.0).as_ref(), Some(&user_b_reward));

        // lock campaign
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));
        // check lock status
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::Locked));

        // now user can claim the reward
        assert_ok!(Reward::get_instant_reward(Origin::signed(user_a.0), crowdloan_id));
        assert_ok!(Reward::get_vested_reward(Origin::signed(user_b.0), crowdloan_id));

        // Make sure only status is updated
        assert_eq!(
            Reward::get_contribution(crowdloan_id, user_a.0),
            Some(types::RewardUnitOf::<Test> {
                status: types::ClaimerStatus::DoneInstant,
                ..user_a_reward.clone()
            })
        );
        assert_eq!(
            Reward::get_contribution(crowdloan_id, user_b.0),
            Some(types::RewardUnitOf::<Test> {
                status: types::ClaimerStatus::DoneVesting,
                ..user_b_reward.clone()
            })
        );

        // now user can claim the reward
        assert_ok!(Reward::get_vested_reward(Origin::signed(user_a.0), crowdloan_id));
        assert_ok!(Reward::get_instant_reward(Origin::signed(user_b.0), crowdloan_id));

        // Again status should be updated
        assert_eq!(
            Reward::get_contribution(crowdloan_id, user_a.0),
            Some(types::RewardUnitOf::<Test> {
                status: types::ClaimerStatus::DoneBoth,
                ..user_a_reward
            })
        );
        assert_eq!(
            Reward::get_contribution(crowdloan_id, user_b.0),
            Some(types::RewardUnitOf::<Test> {
                status: types::ClaimerStatus::DoneBoth,
                ..user_b_reward
            })
        );
    });
}

#[test]
fn split_amount() {
    let split_check = |split: functions::SplittedAmount<u128>, input: functions::SplitableAmount<u64, u128>| {
        let instant_amount = input.reward_amount * input.instant_percentage.numenator as u128 /
            input.instant_percentage.denomator as u128;
        let total = split.instant_amount + split.vesting_amount;

        assert_eq!(instant_amount, split.instant_amount);
        assert_eq!(input.reward_amount - instant_amount, split.vesting_amount);
        assert_eq!(total, input.reward_amount);
    };

    {
        let input = functions::SplitableAmount {
            reward_amount: 10_000_204_u128,
            vesting_starts: 20_u64,
            vesting_ends: 53_u64,
            instant_percentage: SmallRational::new(5, 10),
        };
        let split = input.clone().split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        let expected_split = functions::SplittedAmount {
            instant_amount: 5_000_102,
            vesting_amount: 5_000_102,
            per_block: 151518,
        };

        assert_eq!(split.as_ref(), Some(&expected_split));
        split_check(expected_split, input);
    }

    // vesting starts > vesting ends
    {
        let input = functions::SplitableAmount {
            reward_amount: 10_u128,
            vesting_starts: 30_u64,
            vesting_ends: 20_u64,
            instant_percentage: SmallRational::new(1, 2),
        };
        let split = input.split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        assert_eq!(split, None);
    }

    // vesting amount will be 0
    {
        let input = functions::SplitableAmount {
            reward_amount: 10_u128,
            vesting_starts: 30_u64,
            vesting_ends: 20_u64,
            instant_percentage: SmallRational::new(1, 1),
        };
        let split = input.clone().split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        let expected_split = functions::SplittedAmount {
            instant_amount: 10_u128,
            vesting_amount: 0,
            per_block: 0,
        };
        assert_eq!(split.as_ref(), Some(&expected_split));
        split_check(expected_split, input);
    }

    // vesting amount cannot cover whole time range
    {
        let input = functions::SplitableAmount {
            reward_amount: 100_u128,
            vesting_starts: 100_u64,
            vesting_ends: 200,
            instant_percentage: SmallRational::new(1, 2),
        };
        let split = input.clone().split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        let expected_split = functions::SplittedAmount {
            instant_amount: 50,
            vesting_amount: 50,
            per_block: 1,
        };
        assert_eq!(split.as_ref(), Some(&expected_split));
        split_check(expected_split, input);
    }

    {
        let input = functions::SplitableAmount {
            reward_amount: 100_u128,
            vesting_starts: 10,
            vesting_ends: 100,
            instant_percentage: types::SmallRational::new(1, 2),
        };
        let split = input.clone().split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        let expected_split = functions::SplittedAmount {
            instant_amount: 50,
            vesting_amount: 50,
            per_block: 1,
        };
        assert_eq!(split.as_ref(), Some(&expected_split));
        split_check(expected_split, input);
    }

    {
        let input = functions::SplitableAmount {
            reward_amount: 10_000_000_u128,
            vesting_starts: 0,
            vesting_ends: 100,
            instant_percentage: types::SmallRational::new(3, 10),
        };
        let split = input.clone().split_amount::<<Test as crate::Config>::BlockNumberToBalance>();
        let expected_split = functions::SplittedAmount {
            instant_amount: 3_000_000,
            vesting_amount: 7_000_000,
            per_block: 70_000,
        };
        assert_eq!(split.as_ref(), Some(&expected_split));
        split_check(expected_split, input);
    }
}

#[test]
fn campaign_status() {
    let new_quick_campaign = |hoster, id| {
        Reward::start_new_crowdloan(
            Origin::signed(hoster),
            id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(100_u32.into()),
            },
        )
    };

    // With in-progress status
    // - contributer can be added
    // - contributer can be removed
    // - contributer cannot claim instant reward
    // - contributr cannot claim vesting reward
    // - campaign can be locked
    // - campaign can be discarded
    // - campaign cannot be wiped
    new_test_ext().execute_with(|| {
        let hoster = 10_u32.into();
        let crowdloan_id = 3_u32.into();

        // Initilialize the campaign
        assert_ok!(new_quick_campaign(hoster, crowdloan_id));
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::InProgress));

        // cann add contributer
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 5_u32.into(), 10000_u128.into()));
        // can remove contributer
        assert_ok!(Reward::remove_contributer(Origin::signed(hoster), crowdloan_id, 5_u32.into()));
        // cannot wipe campaign
        assert_noop!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id), RewardError::CampaignNotLocked,);
        // cannot claim instant reward
        assert_noop!(
            Reward::get_instant_reward(Origin::signed(100_u32.into()), crowdloan_id),
            RewardError::NonClaimableCampaign,
        );
        // cannot claim vesting reward
        assert_noop!(
            Reward::get_vested_reward(Origin::signed(100_u32.into()), crowdloan_id),
            RewardError::NonClaimableCampaign,
        );
        // Campaign cannot be wiped
        assert_noop!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id), RewardError::CampaignNotLocked,);
        // Campaign can be locked
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));

        // roll back to in-progress state
        <crate::CampaignStatus<Test>>::insert(crowdloan_id, types::RewardCampaignStatus::InProgress);

        // campaign can be discarded
        assert_ok!(Reward::discard_campaign(Origin::signed(hoster), crowdloan_id));
    });

    // With Locked status
    // - contributer cannot be added
    // - contributer cannot be removed
    // - contributer can claim instant reward
    // - contributer can claim vesting reward
    // - campaign cannot be discarded
    // - campaign cannot be locked
    // - campaign can be wiped ( only if all contributer are rewarded)
    new_test_ext().execute_with(|| {
        let hoster = 1_u32.into();
        let crowdloan_id = 10_u32.into();
        let contributer_a = 1_u32.into();
        let contributer_b = 2_u32.into();

        // initilize the campaign
        assert_ok!(new_quick_campaign(hoster, crowdloan_id));
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, contributer_a, 100_000_u32.into()));
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, contributer_b, 200_000_u32.into()));
        let reward_source = Reward::get_reward_info(crowdloan_id).unwrap().reward_source;
        credit_account::<Test>(&reward_source, 1_000_000_u32.into());

        // lock the campaign
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));

        // cannot add contributer
        assert_noop!(
            Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 33_u32.into(), 100_000_u32.into()),
            RewardError::ReadOnlyCampaign,
        );
        // Cannot remove contributer
        assert_noop!(
            Reward::remove_contributer(Origin::signed(hoster), crowdloan_id, contributer_a),
            RewardError::ReadOnlyCampaign,
        );
        // Cannot lock campaign
        assert_noop!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id), RewardError::NonLockableCampaign,);
        // cannot discard the campaign
        assert_noop!(Reward::discard_campaign(Origin::signed(hoster), crowdloan_id), RewardError::CampaignLocked,);
        // can call get instant reward
        assert_ok!(Reward::get_instant_reward(Origin::signed(contributer_a), crowdloan_id));
        assert_ok!(Reward::get_instant_reward(Origin::signed(contributer_b), crowdloan_id));
        // can call get vesting reward
        assert_ok!(Reward::get_vested_reward(Origin::signed(contributer_a), crowdloan_id));
        // since there is still unclaimed contribution ( vesting reward of contributer_b)
        // cannot wipe campaign
        assert_noop!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id), RewardError::UnclaimedContribution,);
        assert_ok!(Reward::get_vested_reward(Origin::signed(contributer_b), crowdloan_id));
        // can wipe campaign
        assert_ok!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id));
    });

    // With Wiped status
    // - cannot add contributer
    // - cannot remove contributer
    // - cannot lock campaign
    // - cannot discard campaign
    // - contributer cannot claim instant reward
    // - contributer cannot claim vesting reward
    new_test_ext().execute_with(|| {
        let hoster = 10_u32.into();
        let crowdloan_id = 3_u32.into();

        // Initilialize the campaign
        assert_ok!(new_quick_campaign(hoster, crowdloan_id));
        let reward_source = Reward::get_reward_info(crowdloan_id).unwrap().reward_source;
        credit_account::<Test>(&reward_source, 1_000_000_u32.into());
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 100_u32.into(), 100_000_u32.into()));
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));
        assert_ok!(Reward::get_vested_reward(Origin::signed(100_u32.into()), crowdloan_id));
        assert_ok!(Reward::get_instant_reward(Origin::signed(100_u32.into()), crowdloan_id));
        assert_ok!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id));
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::Wiped));

        // cannot add contributer
        assert_noop!(
            Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 33_u32.into(), 100_000_u32.into()),
            RewardError::NoRewardCampaign,
        );
        // Cannot remove contributer
        assert_noop!(
            Reward::remove_contributer(Origin::signed(hoster), crowdloan_id, 33_u32.into()),
            RewardError::NoRewardCampaign,
        );
        // Cannot lock campaign
        assert_noop!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id), RewardError::NoRewardCampaign,);
        // cannot discard the campaign
        assert_noop!(Reward::discard_campaign(Origin::signed(hoster), crowdloan_id), RewardError::NoRewardCampaign,);
        // cannot claim instant reward
        assert_noop!(
            Reward::get_instant_reward(Origin::signed(33_u32.into()), crowdloan_id),
            RewardError::NonClaimableCampaign,
        );
        // cannot claim vesting reward
        assert_noop!(
            Reward::get_vested_reward(Origin::signed(33_u32.into()), crowdloan_id),
            RewardError::NonClaimableCampaign,
        );
        // cannot wipe campaign
        {
            // to bypass the host check let's enter a dummy unit
            <crate::RewardInfo<Test>>::insert(
                crowdloan_id,
                types::CrowdloanRewardFor::<Test> {
                    hoster,
                    ..Default::default()
                },
            );
            assert_noop!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id), RewardError::CampaignNotLocked,);
        }
    });
}

#[test]
fn claimer_status() {
    new_test_ext().execute_with(|| {
        let hoster = 1_u32.into();
        let crowdloan_id = 3_u32.into();
        let contributer = 4_u32.into();
        let reward_source = 100_u32.into();

        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(reward_source),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(10_u32.into()),
            },
        ));
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, contributer, 100_000_u32.into()));
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));
        assert_eq!(
            Reward::get_contribution(crowdloan_id, contributer).map(|p| p.status),
            Some(types::ClaimerStatus::Unprocessed)
        );
        credit_account::<Test>(&reward_source, 10_000_000_u32.into());

        // can claim vesting reward
        assert_ok!(Reward::get_vested_reward(Origin::signed(contributer), crowdloan_id));
        // cannot call again vesting reward
        assert_noop!(Reward::get_vested_reward(Origin::signed(contributer), crowdloan_id), RewardError::RewardTaken);
        // can call instant reward
        assert_ok!(Reward::get_instant_reward(Origin::signed(contributer), crowdloan_id));
        // cannot call again instant reward
        assert_noop!(Reward::get_instant_reward(Origin::signed(contributer), crowdloan_id), RewardError::RewardTaken);
    });
}

#[test]
fn new_crowdloan_creation_sucess() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let current_block = frame_system::Pallet::<Test>::block_number();

        let hoster = 1_u32.into();
        let crowdloan_id = 33_u32.into();
        let crowdloan_params = types::CrowdloanRewardParamFor::<Test> {
            hoster: None,
            reward_source: Some(100_u32.into()),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: None,
            end_target: Some(100_u32.into()),
        };

        // extrinsic call should success
        assert_ok!(Reward::start_new_crowdloan(Origin::signed(hoster), crowdloan_id, crowdloan_params));

        // expect right reward info
        let expected_info = types::CrowdloanRewardFor::<Test> {
            hoster,
            reward_source: 100_u32.into(),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: current_block,
            end_target: 100_u32.into(),
        };
        assert_eq!(Reward::get_reward_info(crowdloan_id), Some(expected_info));

        // check initial status is in-progress
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::InProgress));

        // check correct event is dispatched
        assert_eq!(reward_events().last(), Some(&RewardEvent::CampaignStarted(crowdloan_id)));
    });
}

#[test]
fn new_crowdloan_update_sucess() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let current_block = frame_system::Pallet::<Test>::block_number();

        let hoster = 1_u32.into();
        let crowdloan_id = 33_u32.into();

        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(33_u32.into()),
                instant_percentage: Some(types::SmallRational::new(1, 1)),
                starts_from: None,
                end_target: Some(0_u32.into()),
            }
        ));

        let old_info = types::CrowdloanRewardFor::<Test> {
            hoster,
            reward_source: 33_u32.into(),
            instant_percentage: types::SmallRational::new(1, 1),
            starts_from: current_block,
            end_target: 0_u32.into(),
        };
        let new_crowdloan_params = types::CrowdloanRewardParamFor::<Test> {
            hoster: None,
            reward_source: Some(100_u32.into()),
            instant_percentage: Some(types::SmallRational::new(3, 10)),
            starts_from: Some(20_u32.into()),
            end_target: Some(100_u32.into()),
        };
        let new_info = types::CrowdloanRewardFor::<Test> {
            hoster,
            reward_source: 100_u32.into(),
            instant_percentage: types::SmallRational::new(3, 10),
            starts_from: 20_u32.into(),
            end_target: 100_u32.into(),
        };

        // expect unupdated reward info
        assert_eq!(Reward::get_reward_info(crowdloan_id), Some(old_info));
        // expect the extrinsic to execute sucessfully
        assert_ok!(Reward::update_campaign(Origin::signed(hoster), crowdloan_id, new_crowdloan_params));
        // expect updated new info
        assert_eq!(Reward::get_reward_info(crowdloan_id), Some(new_info));
        // make sure status is still untouched
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(RewardCampaignStatus::InProgress));
        // expect the event
        assert_eq!(reward_events().last(), Some(&RewardEvent::CampaignUpdated(crowdloan_id)));
    });
}

#[test]
fn contributer_addition_removal_success() {
    new_test_ext().execute_with(|| {
        let crowdloan_id = 22_u32.into();
        let hoster = 100_u32.into();

        // Initilization
        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(100_u32.into()),
            }
        ));

        let contributer_a = 101_u32.into();
        let contributer_b = 102_u32.into();
        let contributer_amount = 1_000_000_u32.into();
        let contributer_unit = types::RewardUnitOf::<Test> {
            instant_amount: 300_000_u32.into(),
            vesting_amount: 700_000_u32.into(),
            per_block: 7_000_u32.into(),
            status: types::ClaimerStatus::Unprocessed,
        };

        // add contributer a
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, contributer_a, contributer_amount));
        assert_eq!(Reward::get_contribution(crowdloan_id, contributer_a), Some(contributer_unit.clone()));
        assert_eq!(
            reward_events().last(),
            Some(&RewardEvent::ContributerAdded {
                crowdloan_id,
                contributer: contributer_a,
                amount: contributer_amount
            })
        );
        // add contributer b
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, contributer_b, contributer_amount));
        assert_eq!(Reward::get_contribution(crowdloan_id, contributer_b), Some(contributer_unit.clone()));
        assert_eq!(
            reward_events().last(),
            Some(&RewardEvent::ContributerAdded {
                crowdloan_id,
                contributer: contributer_b,
                amount: contributer_amount
            })
        );
        // remove contributer a
        assert_ok!(Reward::remove_contributer(Origin::signed(hoster), crowdloan_id, contributer_a));
        assert_eq!(Reward::get_contribution(crowdloan_id, contributer_a), None);
        assert_eq!(
            reward_events().last(),
            Some(&RewardEvent::ContributerKicked {
                crowdloan_id,
                contributer: contributer_a
            })
        );
    });
}

#[test]
fn discard_campaign_success() {
    new_test_ext().execute_with(|| {
        let hoster = 101_u32.into();
        let crowdloan_id = 10_u32.into();

        // initilization
        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(100_u32.into()),
            }
        ));

        // add a contributers
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 22_u32.into(), 100_000));

        // attempt to discard
        assert_ok!(Reward::discard_campaign(Origin::signed(hoster), crowdloan_id));

        // there should be no status
        assert_eq!(Reward::get_campaign_status(crowdloan_id), None);
        // there should be no reward info
        assert_eq!(Reward::get_reward_info(crowdloan_id), None);
        // should be no contribution inside this id
        assert_eq!(crate::Contribution::<Test>::iter_prefix(crowdloan_id).next(), None);
        // should be event
        assert_eq!(reward_events().last(), Some(&RewardEvent::CampaignDiscarded(crowdloan_id)));
    });
}

#[test]
fn lock_campaign_success() {
    new_test_ext().execute_with(|| {
        let hoster = 1_u32.into();
        let crowdloan_id = 33_u32.into();

        // initilization
        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(100_u32.into()),
            }
        ));

        // add contributers
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 101_u32.into(), 100_000));
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 102_u32.into(), 100_000));

        // lock the campaign
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));
        // status should be locked
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::Locked));
        // contributer should be as-is
        assert_eq!(crate::Contribution::<Test>::iter_key_prefix(crowdloan_id).collect::<Vec<_>>(), vec![102, 101]);
        // event must be deposited
        assert_eq!(reward_events().last(), Some(&RewardEvent::CampaignLocked(crowdloan_id)));
    });
}

#[test]
fn wipe_campaign_success() {
    new_test_ext().execute_with(|| {
        let hoster = 1_u32.into();
        let crowdloan_id = 33_u32.into();

        // initilization
        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(10_u32.into()),
            }
        ));
        credit_account::<Test>(&100_u32.into(), 100_000_u32.into());
        assert_ok!(Reward::add_contributer(Origin::signed(hoster), crowdloan_id, 101_u32.into(), 10_000));
        assert_ok!(Reward::lock_campaign(Origin::signed(hoster), crowdloan_id));
        assert_ok!(Reward::get_instant_reward(Origin::signed(101_u32.into()), crowdloan_id));
        assert_ok!(Reward::get_vested_reward(Origin::signed(101_u32.into()), crowdloan_id));

        // wipe the campaign
        assert_ok!(Reward::wipe_campaign(Origin::signed(hoster), crowdloan_id));
        // status should be set to wiped
        assert_eq!(Reward::get_campaign_status(crowdloan_id), Some(types::RewardCampaignStatus::Wiped));
        // should be no info in rewardInfo
        assert_eq!(Reward::get_reward_info(crowdloan_id), None);
        // contributers should be cleared
        assert_eq!(crate::Contribution::<Test>::iter_key_prefix(crowdloan_id).next(), None);
        // event should be deposited
        assert_eq!(reward_events().last(), Some(&RewardEvent::CampaignWiped(crowdloan_id)));
    })
}

#[test]
fn hoster_access_control() {
    new_test_ext().execute_with(|| {
        let hoster = 1_u32.into();
        let not_hoster = 2_u32.into();
        let crowdloan_id = 100_u32.into();

        assert_ok!(Reward::start_new_crowdloan(
            Origin::signed(hoster),
            crowdloan_id,
            types::CrowdloanRewardParamFor::<Test> {
                hoster: None,
                reward_source: Some(100_u32.into()),
                instant_percentage: Some(types::SmallRational::new(3, 10)),
                starts_from: Some(0_u32.into()),
                end_target: Some(10_u32.into()),
            }
        ));

        // other user than hoster
        // - cannot update the campaign
        // - cannot discard the campaign
        // - cannot lock the campaign
        // - cannot add contributer
        // - cannot remove contributer
        // - cannot wipe contributer
        assert_noop!(
            Reward::update_campaign(Origin::signed(not_hoster), crowdloan_id, Default::default()),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::discard_campaign(Origin::signed(not_hoster), crowdloan_id),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::lock_campaign(Origin::signed(not_hoster), crowdloan_id),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::discard_campaign(Origin::signed(not_hoster), crowdloan_id),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::add_contributer(Origin::signed(not_hoster), crowdloan_id, 99_u32.into(), 10_000_u32.into()),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::remove_contributer(Origin::signed(not_hoster), crowdloan_id, 99_u32.into()),
            RewardError::PermissionDenied,
        );
        assert_noop!(
            Reward::wipe_campaign(Origin::signed(not_hoster), crowdloan_id),
            RewardError::PermissionDenied,
        );
    });
}
