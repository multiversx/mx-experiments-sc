use gas_burner::{
    leaderboard::{LeaderboardEntry, LeaderboardModule},
    work::WorkModule,
};
use gas_burner_setup::{GasBurnerSetup, GAS_BURNER_TOKEN_ID};
use multiversx_sc_scenario::{managed_biguint, managed_buffer, rust_biguint};

pub mod gas_burner_setup;

#[test]
fn init_test() {
    let _ = GasBurnerSetup::new(gas_burner::contract_obj);
}

#[test]
fn work_test() {
    let mut setup = GasBurnerSetup::new(gas_burner::contract_obj);

    setup
        .b_mock
        .execute_tx(
            &setup.first_user,
            &setup.gas_burner_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.work(managed_buffer!(b"signature"));

                assert_eq!(sc.leaderboard(1).len(), 1);
                assert_eq!(
                    sc.leaderboard(1).get(1),
                    LeaderboardEntry {
                        user_id: 1,
                        work_amount: managed_biguint!(100_000_000)
                    }
                );
            },
        )
        .assert_ok();

    // users get 1 token for each 1_000_000 gas they use
    // default gas in testing framework is 100_000_000, so they get 100 tokens
    setup
        .b_mock
        .check_esdt_balance(&setup.first_user, GAS_BURNER_TOKEN_ID, &rust_biguint!(100));
}

#[test]
fn leaderboard_test() {
    let mut setup = GasBurnerSetup::new(gas_burner::contract_obj);

    let first_user = setup.first_user.clone();
    let second_user = setup.second_user.clone();
    setup.work_user(&first_user);
    setup.work_user(&second_user);

    setup
        .b_mock
        .execute_query(&setup.gas_burner_wrapper, |sc| {
            assert_eq!(sc.leaderboard(1).len(), 2);
            assert_eq!(
                sc.leaderboard(1).get(1),
                LeaderboardEntry {
                    user_id: 1,
                    work_amount: managed_biguint!(100_000_000)
                }
            );
            assert_eq!(
                sc.leaderboard(1).get(2),
                LeaderboardEntry {
                    user_id: 2,
                    work_amount: managed_biguint!(100_000_000)
                }
            );
        })
        .assert_ok();

    setup.work_user(&second_user);

    setup
        .b_mock
        .execute_query(&setup.gas_burner_wrapper, |sc| {
            assert_eq!(sc.leaderboard(1).len(), 2);
            assert_eq!(
                sc.leaderboard(1).get(1),
                LeaderboardEntry {
                    user_id: 2,
                    work_amount: managed_biguint!(200_000_000)
                }
            );
            assert_eq!(
                sc.leaderboard(1).get(2),
                LeaderboardEntry {
                    user_id: 1,
                    work_amount: managed_biguint!(100_000_000)
                }
            );
        })
        .assert_ok();

    setup.work_user(&first_user);

    setup
        .b_mock
        .execute_query(&setup.gas_burner_wrapper, |sc| {
            assert_eq!(sc.leaderboard(1).len(), 2);
            assert_eq!(
                sc.leaderboard(1).get(1),
                LeaderboardEntry {
                    user_id: 2,
                    work_amount: managed_biguint!(200_000_000)
                }
            );
            assert_eq!(
                sc.leaderboard(1).get(2),
                LeaderboardEntry {
                    user_id: 1,
                    work_amount: managed_biguint!(200_000_000)
                }
            );
        })
        .assert_ok();

    setup.work_user(&first_user);

    setup
        .b_mock
        .execute_query(&setup.gas_burner_wrapper, |sc| {
            assert_eq!(sc.leaderboard(1).len(), 2);
            assert_eq!(
                sc.leaderboard(1).get(1),
                LeaderboardEntry {
                    user_id: 1,
                    work_amount: managed_biguint!(300_000_000)
                }
            );
            assert_eq!(
                sc.leaderboard(1).get(2),
                LeaderboardEntry {
                    user_id: 2,
                    work_amount: managed_biguint!(200_000_000)
                }
            );
        })
        .assert_ok();
}

// Doesn't work :(
// #[test]
// fn dev_rewards_test() {
//     let mut setup = GasBurnerSetup::new(gas_burner::contract_obj);

//     let first_user = setup.first_user.clone();
//     let second_user = setup.second_user.clone();
//     setup.work_user(&first_user);
//     setup.work_user(&second_user);
//     setup.work_user(&first_user);
//     setup.work_user(&first_user);

//     setup
//         .b_mock
//         .execute_query(&setup.gas_burner_wrapper, |sc| {
//             assert_eq!(sc.leaderboard(1).len(), 2);
//             assert_eq!(
//                 sc.leaderboard(1).get(1),
//                 LeaderboardEntry {
//                     user_id: 1,
//                     work_amount: managed_biguint!(300_000_000)
//                 }
//             );
//             assert_eq!(
//                 sc.leaderboard(1).get(2),
//                 LeaderboardEntry {
//                     user_id: 2,
//                     work_amount: managed_biguint!(100_000_000)
//                 }
//             );
//         })
//         .assert_ok();

//     setup.b_mock.set_developer_rewards(
//         setup.gas_burner_wrapper.address_ref(),
//         rust_biguint!(100_000),
//     );

//     setup.b_mock.set_block_epoch(8);

//     setup
//         .b_mock
//         .execute_tx(
//             &first_user,
//             &setup.gas_burner_wrapper,
//             &rust_biguint!(0),
//             |sc| {
//                 let rewards = sc.claim_rewards(1);
//                 assert_eq!(rewards, managed_biguint!(1));
//             },
//         )
//         .assert_ok();
// }
