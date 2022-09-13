use frame_support::{assert_ok, dispatch::DispatchResult};
use sp_runtime::traits::Zero;

use crate::{
    mock::{ExtBuilder, Origin, System, Test, Witnet, MAX_WITNET_BYTE_SIZE},
    prelude::*,
};

fn post_dummy_request(origin: Origin) -> DispatchResult {
    Witnet::post_request(origin, BalanceOf::<Test>::zero(), vec![])
}

#[test]
fn test_post_request() {
    ExtBuilder::default().build_and_execute(|| {
        let account_seven = Origin::signed(7);
        let max_byte_size = usize::from(MAX_WITNET_BYTE_SIZE);

        // This should fail because we are providing too many bytes
        let post = Witnet::post_request(
            account_seven.clone(),
            BalanceOf::<Test>::zero(),
            vec![0; max_byte_size + 1],
        );
        let expected = Err(WitnetError::<Test>::OversizedRequest.into());
        assert_eq!(post, expected);

        // This should work!
        let post = post_dummy_request(account_seven.clone());
        assert_ok!(post);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedRequest {
                request_id: 0,
                sender: 7,
            }
            .into(),
        );

        // A second request should get a different ID
        let post = post_dummy_request(account_seven.clone());
        assert_ok!(post);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedRequest {
                request_id: 1,
                sender: 7,
            }
            .into(),
        );
    });
}

#[test]
fn test_report_result() {
    ExtBuilder::default().build_and_execute(|| {
        let genesis_operator = Origin::signed(5);
        let account_seven = Origin::signed(7);
        let max_byte_size = usize::from(MAX_WITNET_BYTE_SIZE);

        <Test as WitnetConfig>::TimeProvider::set_timestamp(1000);

        post_dummy_request(account_seven.clone()).ok();

        // This should fail because account #7 is not allowed to report
        let report = Witnet::report_result(
            account_seven.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::UnauthorizedOperator.into());
        assert_eq!(report, expected);

        // This should fail because we are reporting a result from the future
        let report = Witnet::report_result(
            genesis_operator.clone(),
            0,
            1000,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::ResultFromFuture.into());
        assert_eq!(report, expected);

        // This should fail because the result cannot be empty
        let report = Witnet::report_result(genesis_operator.clone(), 0, 999, [0; 32], vec![]);
        let expected = Err(WitnetError::<Test>::EmptyResult.into());
        assert_eq!(report, expected);

        // This should fail because the result is oversized
        let report = Witnet::report_result(
            genesis_operator.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size + 1],
        );
        let expected = Err(WitnetError::<Test>::OversizedResult.into());
        assert_eq!(report, expected);

        // This should fail because the request is unknown
        let report = Witnet::report_result(
            genesis_operator.clone(),
            1,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::UnknownRequest.into());
        assert_eq!(report, expected);

        // This should work!
        let report = Witnet::report_result(
            genesis_operator.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        assert_ok!(report);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedResult {
                request_id: 0,
                sender: 5,
            }
            .into(),
        );

        // This should fail because it is a duplicated report
        let report = Witnet::report_result(
            genesis_operator.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::AlreadyReported.into());
        assert_eq!(report, expected);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedResult {
                request_id: 0,
                sender: 5,
            }
            .into(),
        );
    });
}

#[test]
fn test_operators() {
    ExtBuilder::default().build_and_execute(|| {
        let genesis_operator = Origin::signed(5);
        let account_seven = Origin::signed(7);
        let account_nine = Origin::signed(9);
        let max_byte_size = usize::from(MAX_WITNET_BYTE_SIZE);

        <Test as WitnetConfig>::TimeProvider::set_timestamp(1000);

        post_dummy_request(account_seven.clone()).ok();
        post_dummy_request(account_seven.clone()).ok();
        post_dummy_request(account_seven.clone()).ok();

        // This should fail because account #7 is not allowed to report yet
        let report = Witnet::report_result(
            account_seven.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::UnauthorizedOperator.into());
        assert_eq!(report, expected);

        // This should fail because only operators can add more operators
        let add = Witnet::add_operator(account_nine.clone(), 7);
        let expected = Err(WitnetError::<Test>::UnauthorizedOperator.into());
        assert_eq!(add, expected);

        // This should fail because operators cannot add themselves as operators to avoid duplicates
        let add = Witnet::add_operator(genesis_operator.clone(), 5);
        let expected = Err(WitnetError::<Test>::OperatorSelfAddition.into());
        assert_eq!(add, expected);

        // This should work!
        let add = Witnet::add_operator(genesis_operator.clone(), 7);
        assert_ok!(add);
        System::assert_last_event(
            WitnetEvent::<Test>::AddedOperator {
                added_operator: 7,
                added_by: 5,
            }
            .into(),
        );

        // The recently added operator should now be able to report
        let report = Witnet::report_result(
            account_seven.clone(),
            0,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        assert_ok!(report);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedResult {
                request_id: 0,
                sender: 7,
            }
            .into(),
        );

        // The recently added operator should now be able to add another operator
        let add = Witnet::add_operator(account_seven.clone(), 9);
        assert_ok!(add);
        System::assert_last_event(
            WitnetEvent::<Test>::AddedOperator {
                added_operator: 9,
                added_by: 7,
            }
            .into(),
        );

        // This second new operator should also be able to report
        let report = Witnet::report_result(
            account_nine.clone(),
            1,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        assert_ok!(report);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedResult {
                request_id: 1,
                sender: 9,
            }
            .into(),
        );

        // An operator should not be able to remove itself
        let remove = Witnet::remove_operator(account_seven.clone(), 7);
        let expected = Err(WitnetError::<Test>::OperatorSelfRemoval.into());
        assert_eq!(remove, expected);

        // However, one operator should be able to remove a different operator
        let remove = Witnet::remove_operator(genesis_operator.clone(), 7);
        assert_ok!(remove);
        System::assert_last_event(
            WitnetEvent::<Test>::RemovedOperator {
                removed_operator: 7,
                removed_by: 5,
            }
            .into(),
        );

        // The recently removed operator should no longer be able to report
        let report = Witnet::report_result(
            account_seven.clone(),
            2,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        let expected = Err(WitnetError::<Test>::UnauthorizedOperator.into());
        assert_eq!(report, expected);

        // But the third operator (account #9) should still be able to report
        let report = Witnet::report_result(
            account_nine.clone(),
            2,
            999,
            [0; 32],
            vec![0; max_byte_size],
        );
        assert_ok!(report);
        System::assert_last_event(
            WitnetEvent::<Test>::PostedResult {
                request_id: 2,
                sender: 9,
            }
            .into(),
        );

        // You should not be able to remove an operator that never existed
        let remove = Witnet::remove_operator(genesis_operator.clone(), 11);
        let expected = Err(WitnetError::<Test>::UnknownOperator.into());
        assert_eq!(remove, expected);
    })
}
