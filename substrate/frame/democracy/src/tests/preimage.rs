// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The preimage tests.

use frame_support::weights::{Pays, RuntimeDbWeight};
use frame_system::{EventRecord, Phase};

use super::*;

#[test]
fn missing_preimage_should_fail() {
    new_test_ext().execute_with(|| {
        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash(2),
            VoteThreshold::SuperMajorityApprove,
            0,
        );
        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));

        next_block();
        next_block();

        assert_eq!(Balances::free_balance(42), 0);
    });
}

#[test]
fn preimage_deposit_should_be_required_and_returned_if_turnout_is_enough() {
    new_test_ext_execute_with_cond(|operational| {
        // fee of 100 is too much.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 100);
        assert_noop!(
            if operational {
                Democracy::note_preimage_operational(Origin::signed(6), vec![0; 500])
            } else {
                Democracy::note_preimage(Origin::signed(6), vec![0; 500])
            },
            BalancesError::<Test, _>::InsufficientBalance,
        );
        // fee of 1 is reasonable.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash_and_note(2),
            VoteThreshold::SuperMajorityApprove,
            0,
        );

        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));

        assert_eq!(Balances::reserved_balance(6), 12);

        next_block();
        next_block();

        assert_eq!(Balances::reserved_balance(6), 0);
        assert_eq!(Balances::free_balance(6), 60);
        assert_eq!(Balances::free_balance(42), 2);
    });
}

#[test]
fn preimage_deposit_should_be_locked_if_turnout_is_lower_than_required() {
    new_test_ext_execute_with_cond(|operational| {
        // fee of 100 is too much.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 100);
        assert_noop!(
            if operational {
                Democracy::note_preimage_operational(Origin::signed(6), vec![0; 500])
            } else {
                Democracy::note_preimage(Origin::signed(6), vec![0; 500])
            },
            BalancesError::<Test, _>::InsufficientBalance,
        );

        DepositLockStrategy::set(DepositLockConfig::new(2, 3, 20));

        // fee of 1 is reasonable.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash_and_note(2),
            VoteThreshold::SuperMajorityApprove,
            0,
        );

        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));

        assert_eq!(Balances::reserved_balance(6), 12);

        // 2 blocks per referendum and 6 blocks per lock
        for i in 0..8 {
            next_block();

            assert_eq!(Balances::reserved_balance(6), 12);
            assert_eq!(Balances::free_balance(6), 48);
            assert_eq!(Balances::free_balance(42), 2);

            if i >= 2 {
                assert_eq!(
                    LockedDeposits::<Test>::iter_prefix(
                        DepositLockStrategy::get().target_block_from(2)
                    )
                    .collect::<Vec<_>>(),
                    vec![(DepositPaybackTarget::Provider(6), 12)]
                );
            }
        }

        // payback deposit on 7th block from the referendum
        next_block();

        assert!(System::events()
            .iter()
            .find(|event| event
                == &&EventRecord {
                    phase: Phase::Initialization,
                    topics: Default::default(),
                    event: Event::Democracy(
                        crate::Event::LockedDepositUnreserved {
                            recipient: 6,
                            deposit: 12
                        }
                        .into()
                    )
                })
            .is_some());

        assert_eq!(LockedDeposits::<Test>::iter().collect::<Vec<_>>(), vec![]);

        assert_eq!(Balances::reserved_balance(6), 0);
        assert_eq!(Balances::free_balance(6), 60);
        assert_eq!(Balances::free_balance(42), 2);
    });
}

#[test]
fn preimage_deposit_should_be_locked_if_turnout_is_lower_than_required_and_then_force_paid() {
    new_test_ext_execute_with_cond(|operational| {
        // fee of 100 is too much.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 100);
        assert_noop!(
            if operational {
                Democracy::note_preimage_operational(Origin::signed(6), vec![0; 500])
            } else {
                Democracy::note_preimage(Origin::signed(6), vec![0; 500])
            },
            BalancesError::<Test, _>::InsufficientBalance,
        );

        DepositLockStrategy::set(DepositLockConfig::new(2, 3, 20));
        // fee of 1 is reasonable.
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash_and_note(2),
            VoteThreshold::SuperMajorityApprove,
            0,
        );

        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));

        assert_eq!(Balances::reserved_balance(6), 12);

        let locked_in = frame_system::Pallet::<Test>::block_number() + 2;

        // 2 blocks per referendum and 6 blocks per lock
        for _ in 0..8 {
            assert_noop!(
                Democracy::unreserve_locked_deposits(
                    Origin::signed(3),
                    DepositLockStrategy::get().target_block_from(locked_in)
                ),
                DispatchErrorWithPostInfo {
                    post_info: PostDispatchInfo {
                        actual_weight:
                            Some(
                                <<Test as frame_system::Config>::DbWeight as Get<
                                    RuntimeDbWeight,
                                >>::get()
                                .reads(1)
                            ),
                        pays_fee: Pays::Yes
                    },
                    error: "Deposits for the given block number are still locked".into()
                }
            );
            next_block();

            assert_eq!(Balances::reserved_balance(6), 12);
            assert_eq!(Balances::free_balance(6), 48);
            assert_eq!(Balances::free_balance(42), 2);
            assert_eq!(
                LockedDeposits::<Test>::iter_prefix(
                    DepositLockStrategy::get().target_block_from(locked_in)
                )
                .collect::<Vec<_>>(),
                vec![(DepositPaybackTarget::Provider(6), 12)]
            );
        }

        // deposits are allowed to be paid from 7th block from the referendum
        System::set_block_number(System::block_number() + 1);

        assert_ok!(Democracy::unreserve_locked_deposits(
            Origin::signed(3),
            DepositLockStrategy::get().target_block_from(locked_in)
        ));

        assert_eq!(LockedDeposits::<Test>::iter().collect::<Vec<_>>(), vec![]);

        assert_eq!(Balances::reserved_balance(6), 0);
        assert_eq!(Balances::free_balance(6), 60);
        assert_eq!(Balances::free_balance(42), 2);
    });
}

#[test]
fn preimage_deposit_should_be_reapable_earlier_by_owner() {
    new_test_ext_execute_with_cond(|operational| {
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        assert_ok!(if operational {
            Democracy::note_preimage_operational(Origin::signed(6), set_balance_proposal(2))
        } else {
            Democracy::note_preimage(Origin::signed(6), set_balance_proposal(2))
        });

        assert_eq!(
            LockDepositFor::<Test>::iter().collect::<Vec<_>>(),
            vec![(
                hex_literal::hex![
                    "89048bee58ae4ebff9b9800e3d0e396efc0306629a3b02fa4e0f3c9039c12e68"
                ]
                .into(),
                true
            )]
        );

        assert_eq!(Balances::reserved_balance(6), 12);

        next_block();
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(6), set_balance_proposal_hash(2), u32::MAX),
            Error::<Test>::TooEarly
        );
        next_block();
        assert_ok!(Democracy::reap_preimage(
            Origin::signed(6),
            set_balance_proposal_hash(2),
            u32::MAX
        ));

        assert_eq!(LockDepositFor::<Test>::iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            LockedDeposits::<Test>::iter_prefix(
                DepositLockStrategy::get().target_block_from_current(),
            )
            .collect::<Vec<_>>(),
            vec![(DepositPaybackTarget::Provider(6), 12)]
        );

        for _ in 0..7 {
            assert_eq!(Balances::reserved_balance(6), 12);
            assert_eq!(Balances::free_balance(6), 48);
            next_block();
        }

        assert_eq!(Balances::free_balance(6), 60);
        assert_eq!(Balances::reserved_balance(6), 0);
    });
}

#[test]
fn preimage_deposit_should_be_reapable() {
    new_test_ext_execute_with_cond(|operational| {
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(5), set_balance_proposal_hash(2), u32::MAX),
            Error::<Test>::PreimageMissing
        );

        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        assert_ok!(if operational {
            Democracy::note_preimage_operational(Origin::signed(6), set_balance_proposal(2))
        } else {
            Democracy::note_preimage(Origin::signed(6), set_balance_proposal(2))
        });
        assert_eq!(
            LockDepositFor::<Test>::iter().collect::<Vec<_>>(),
            vec![(
                hex_literal::hex![
                    "89048bee58ae4ebff9b9800e3d0e396efc0306629a3b02fa4e0f3c9039c12e68"
                ]
                .into(),
                true
            )]
        );
        assert_eq!(Balances::reserved_balance(6), 12);

        next_block();
        next_block();
        next_block();
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(5), set_balance_proposal_hash(2), u32::MAX),
            Error::<Test>::TooEarly
        );

        next_block();
        assert_ok!(Democracy::reap_preimage(
            Origin::signed(5),
            set_balance_proposal_hash(2),
            u32::MAX
        ));

        assert_eq!(LockDepositFor::<Test>::iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            LockedDeposits::<Test>::iter_prefix(
                DepositLockStrategy::get().target_block_from_current(),
            )
            .collect::<Vec<_>>(),
            vec![(DepositPaybackTarget::Beneficiary { from: 6, to: 5 }, 12)]
        );

        for _ in 0..7 {
            assert_eq!(Balances::reserved_balance(6), 12);
            assert_eq!(Balances::free_balance(6), 48);
            next_block();
        }

        assert_eq!(Balances::reserved_balance(6), 0);
        assert_eq!(Balances::free_balance(6), 48);
        assert_eq!(Balances::free_balance(5), 62);
    });
}

#[test]
fn preimage_deposit_should_be_locked_if_turnout_wasnt_enough() {
    new_test_ext_execute_with_cond(|operational| {
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(5), set_balance_proposal_hash(2), u32::MAX),
            Error::<Test>::PreimageMissing
        );

        DepositLockStrategy::set(DepositLockConfig::new(2, 3, 20));

        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);
        assert_ok!(if operational {
            Democracy::note_preimage_operational(Origin::signed(6), set_balance_proposal(2))
        } else {
            Democracy::note_preimage(Origin::signed(6), set_balance_proposal(2))
        });
        assert_eq!(Balances::reserved_balance(6), 12);

        next_block();
        next_block();
        next_block();
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(5), set_balance_proposal_hash(2), u32::MAX),
            Error::<Test>::TooEarly
        );

        next_block();
        assert_ok!(Democracy::reap_preimage(
            Origin::signed(5),
            set_balance_proposal_hash(2),
            u32::MAX
        ));
        assert_eq!(LockDepositFor::<Test>::iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            LockedDeposits::<Test>::iter_prefix(
                DepositLockStrategy::get().target_block_from_current(),
            )
            .collect::<Vec<_>>(),
            vec![(DepositPaybackTarget::Beneficiary { from: 6, to: 5 }, 12)]
        );

        for _ in 0..7 {
            assert_eq!(Balances::reserved_balance(6), 12);
            assert_eq!(Balances::free_balance(6), 48);
            next_block();
        }

        assert_eq!(Balances::reserved_balance(6), 0);
        assert_eq!(Balances::free_balance(6), 48);
        assert_eq!(Balances::free_balance(5), 62);
    });
}

#[test]
fn noting_imminent_preimage_for_free_should_work() {
    new_test_ext_execute_with_cond(|operational| {
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);

        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash(2),
            VoteThreshold::SuperMajorityApprove,
            1,
        );

        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));

        assert_noop!(
            if operational {
                Democracy::note_imminent_preimage_operational(
                    Origin::signed(6),
                    set_balance_proposal(2),
                )
            } else {
                Democracy::note_imminent_preimage(Origin::signed(6), set_balance_proposal(2))
            },
            Error::<Test>::NotImminent
        );

        next_block();

        // Now we're in the dispatch queue it's all good.
        assert_ok!(Democracy::note_imminent_preimage(
            Origin::signed(6),
            set_balance_proposal(2)
        ));

        next_block();

        assert_eq!(Balances::free_balance(42), 2);
    });
}

#[test]
fn reaping_imminent_preimage_should_fail() {
    new_test_ext().execute_with(|| {
        let h = set_balance_proposal_hash_and_note(2);
        let r = Democracy::inject_referendum(3, h, VoteThreshold::SuperMajorityApprove, 1);
        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));
        next_block();
        next_block();
        assert_noop!(
            Democracy::reap_preimage(Origin::signed(6), h, u32::MAX),
            Error::<Test>::Imminent
        );
    });
}

#[test]
fn note_imminent_preimage_can_only_be_successful_once() {
    new_test_ext().execute_with(|| {
        PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);

        let r = Democracy::inject_referendum(
            2,
            set_balance_proposal_hash(2),
            VoteThreshold::SuperMajorityApprove,
            1,
        );
        assert_ok!(Democracy::vote(Origin::signed(1), r, aye(1)));
        next_block();

        // First time works
        assert_ok!(Democracy::note_imminent_preimage(
            Origin::signed(6),
            set_balance_proposal(2)
        ));

        // Second time fails
        assert_noop!(
            Democracy::note_imminent_preimage(Origin::signed(6), set_balance_proposal(2)),
            Error::<Test>::DuplicatePreimage
        );

        // Fails from any user
        assert_noop!(
            Democracy::note_imminent_preimage(Origin::signed(5), set_balance_proposal(2)),
            Error::<Test>::DuplicatePreimage
        );
    });
}

#[test]
#[should_panic(expected = "`lock_unit` can't be equal to zero")]
fn deposit_lock_invalid_config() {
    DepositLockConfig::<Test>::new(0, 3, 0);
}

#[test]
fn deposit_lock_config() {
    new_test_ext().execute_with(|| {
        assert_eq!(System::block_number(), 1);

        // With zero locking period, we should be able to payback the deposit in the next phase.
        assert_eq!(
            DepositLockConfig::<Test>::new(1, 0, 0).target_block_from_current(),
            2u64
        );
        assert_eq!(
            DepositLockConfig::<Test>::new(10, 0, 0).target_block_from_current(),
            10u64
        );

        assert_eq!(
            DepositLockConfig::<Test>::new(1, 2, 0).target_block_from_current(),
            4u64
        );
        assert_eq!(
            DepositLockConfig::<Test>::new(1, 3, 0).target_block_from_current(),
            5u64
        );
        assert_eq!(
            DepositLockConfig::<Test>::new(2, 3, 0).target_block_from_current(),
            8u64
        );

        assert_eq!(
            DepositLockConfig::<Test>::new(40, 3, 0).target_block_from_current(),
            160
        );
        assert_eq!(
            DepositLockConfig::<Test>::new(40, 40, 0).target_block_from_current(),
            1640
        );
        assert_eq!(
            DepositLockConfig::<Test>::new(40, 40, 0).target_block_from(100),
            1720
        );

        assert!(DepositLockConfig::<Test>::new(40, 40, 0).should_unreserve_in_block(0));
        assert!(DepositLockConfig::<Test>::new(40, 40, 0).should_unreserve_in_block(40));
        assert!(!DepositLockConfig::<Test>::new(40, 40, 0).should_unreserve_in_block(1));
        assert!(
            DepositLockConfig::<Test>::new(40, 40, 10).should_lock_deposit(&ReferendumStatus {
                end: 4,
                proposal_hash: set_balance_proposal_hash(2),
                threshold: VoteThreshold::SuperMajorityAgainst,
                delay: 2,
                tally: Tally {
                    ayes: 0,
                    nays: 0,
                    turnout: 9
                },
            })
        );
        assert!(
            !DepositLockConfig::<Test>::new(40, 40, 10).should_lock_deposit(&ReferendumStatus {
                end: 4,
                proposal_hash: set_balance_proposal_hash(2),
                threshold: VoteThreshold::SuperMajorityAgainst,
                delay: 2,
                tally: Tally {
                    ayes: 0,
                    nays: 0,
                    turnout: 10
                },
            })
        );
    })
}
