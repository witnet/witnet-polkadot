use std::convert::TryInto;

use frame_support::{sp_runtime::traits::Zero, traits::Time};
use frame_system::ensure_signed;

use core::convert::Into;

use crate::prelude::*;

use super::*;

pub trait WitnetOracle<T, O>
where
    T: Config,
{
    fn post_request(origin: O, reward: BalanceOf<T>, bytes: Vec<u8>) -> DispatchResult;
    fn report_result(
        origin: O,
        request_id: u64,
        timestamp: Timestamp<T>,
        dr_tx_hash: [u8; 32],
        result_bytes: Vec<u8>,
    ) -> DispatchResult;
    fn add_operator(origin: O, account_id: T::AccountId) -> DispatchResult;
    fn remove_operator(origin: O, account_id: T::AccountId) -> DispatchResult;
}

impl<T, O> WitnetOracle<T, O> for Pallet<T>
where
    T: Config,
    O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
{
    fn post_request(origin: O, reward: BalanceOf<T>, bytes: Vec<u8>) -> DispatchResult
    where
        O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
    {
        // Ensure that the transaction is signed, and get hold of signer data
        let sender = ensure_signed(origin)?;

        // Reject oversized requests
        let bytes: BoundedVec<_, T::MaxByteSize> = bytes
            .try_into()
            .map_err(|()| Error::<T>::OversizedRequest)?;

        // Check that the report reward foreseeably covers cost of reporting
        let required_reward = estimate_report_reward::<BalanceOf<T>>(bytes.len());
        ensure!(reward >= required_reward, Error::<T>::UnderpayingRequest);

        // Try to put aside the reward to be paid later to the reporter of the result
        T::Currency::reserve(&sender, reward)?;

        // Use next request ID
        let request_id = NextRequestId::<T>::get();

        // Store request and deposit event to signal readiness for fulfillment
        let request_entry: RequestEntry<T> = (Some((bytes, reward)), None);
        Requests::<T>::insert(request_id, request_entry);
        Self::deposit_event(Event::<T>::PostedRequest { request_id, sender });

        // Increase next request ID
        NextRequestId::<T>::put(request_id.wrapping_add(1));

        Ok(())
    }

    fn report_result(
        origin: O,
        request_id: u64,
        timestamp: Timestamp<T>,
        dr_tx_hash: [u8; 32],
        result_bytes: Vec<u8>,
    ) -> DispatchResult
    where
        O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
    {
        // Ensure that the transaction is signed, and get hold of signer data
        let sender = ensure_signed(origin)?;

        // Ensure that the sender is entitled to report
        ensure!(
            Operators::<T>::contains_key(&sender),
            Error::<T>::UnauthorizedOperator
        );

        // Ensure that timestamp is older than current block
        let now = T::TimeProvider::now();
        ensure!(timestamp < now, Error::<T>::ResultFromFuture);

        // Ensure that the CBOR bytes are not empty
        ensure!(!result_bytes.is_empty(), Error::<T>::EmptyResult);

        // Reject oversized results
        let bounded_bytes: BoundedVec<_, T::MaxByteSize> = result_bytes
            .try_into()
            .map_err(|()| Error::<T>::OversizedRequest)?;

        // Do storage related operations in a separate `inner_report_result` function
        // This will allow reusing part that logic in a future batch reporting method
        let payable =
            inner_report_result::<T>(request_id, timestamp, dr_tx_hash, bounded_bytes, true)?;
        if payable > Zero::zero() {
            T::Currency::unreserve(&sender, payable);
        }

        // Deposit event to signal eventual resolution of the data request
        Self::deposit_event(Event::<T>::PostedResult { request_id, sender });

        Ok(())
    }

    fn add_operator(origin: O, account_id: T::AccountId) -> DispatchResult
    where
        O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
    {
        // Ensure that the sender is an operator, and get hold of its account id
        let sender = ensure_operator::<T, O>(origin)?;

        // Ensure that the operator is not trying to add themself
        ensure!(account_id != sender, Error::<T>::OperatorSelfAddition);

        // Insert the account id of the new operator into the operators collection
        Operators::<T>::insert(account_id.clone(), ());

        // Deposit event to signal addition of the new operator
        Self::deposit_event(Event::<T>::AddedOperator {
            added_operator: account_id,
            added_by: sender,
        });

        Ok(())
    }

    fn remove_operator(origin: O, account_id: T::AccountId) -> DispatchResult
    where
        O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
    {
        // Ensure that the sender is an operator, and get hold of its account id
        let sender = ensure_operator::<T, O>(origin)?;

        // Ensure that the operator is not trying to remove themself
        ensure!(account_id != sender, Error::<T>::OperatorSelfRemoval);

        // Try to remove the account id from the operators collection
        Operators::<T>::take(account_id).ok_or_else(|| Error::<T>::UnknownOperator)?;

        Ok(())
    }
}

fn ensure_operator<T, O>(origin: O) -> Result<T::AccountId, DispatchError>
where
    T: Config,
    O: Into<Result<frame_system::RawOrigin<T::AccountId>, O>>,
{
    // Ensure that the transaction is signed, and get hold of signer data
    let sender = ensure_signed(origin)?;

    // Ensure that the sender is entitled to remove an operator
    ensure!(
        Operators::<T>::contains_key(&sender),
        Error::<T>::UnauthorizedOperator
    );

    Ok(sender)
}

fn inner_report_result<T: Config>(
    request_id: u64,
    timestamp: Timestamp<T>,
    dr_tx_hash: [u8; 32],
    result_bytes: BoundedVec<u8, T::MaxByteSize>,
    drop: bool,
) -> Result<BalanceOf<T>, Error<T>> {
    // Retrieve request info from storage, fail if unknown
    let payable = <Requests<T>>::try_mutate(request_id, |entry| {
        match entry {
            // Ensure the request exists
            None => Err(Error::<T>::UnknownRequest),
            // Ensure the request had not been already reported
            Some((_, Some(_))) => Err(Error::<T>::AlreadyReported),
            // If the query is still there, we can operate, otherwise do nothing
            Some((query @ Some(_), report @ None)) => {
                // It is safe to unwrap the query reward here because it's guarded above
                let reward = query.clone().unwrap().1;
                // If drop is set to true, remove query when inserting the report
                if drop {
                    *query = None;
                }
                // Insert the report
                *report = Some((timestamp, dr_tx_hash, result_bytes));

                Ok(reward)
            }
            _ => unreachable!(),
        }
    })?;

    Ok(payable)
}

pub fn estimate_report_reward<Balance: frame_support::sp_runtime::traits::Zero>(
    _bytes_len: usize,
) -> Balance {
    // TODO: use Transaction Payment pallet to predict cost of result reporting
    Balance::zero()
}
