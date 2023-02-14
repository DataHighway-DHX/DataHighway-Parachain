use crate::{
    functions,
    mock::*,
    types::{
        self,
        SmallRational,
    },
};
use frame_support::{
    assert_ok,
    inherent::BlockT,
};

pub const DOLLARS: u128 = 1_000_000_000_000_000_000_u128;

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
            total_pool: Some(Some(reward_pool)),
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
                total_pool: Some(reward_pool),
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
