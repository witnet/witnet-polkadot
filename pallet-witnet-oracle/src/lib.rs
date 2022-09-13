#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_support;
extern crate frame_system;
extern crate scale_info;
extern crate sp_runtime;

use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod traits;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
    use crate::prelude::*;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The maximum number of bytes a data request can take.
        #[pallet::constant]
        type MaxByteSize: Get<u32>;

        /// The trait that will be providing timestamps.
        type TimeProvider: frame_support::traits::Time;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Requests<T: Config> =
        StorageMap<_, Twox64Concat, RequestId, RequestEntry<T>, OptionQuery>;

    #[pallet::storage]
    pub(super) type Operators<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    pub(super) type NextRequestId<T> = StorageValue<_, RequestId, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub operators: Vec<T::AccountId>,
    }

    impl<T: Config> GenesisConfig<T> {
        pub fn from_operators(operators: Vec<T::AccountId>) -> Self {
            Self { operators }
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                operators: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for operator in &self.operators {
                <Operators<T>>::insert(operator, ());
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A request has been posted.
        PostedRequest {
            request_id: RequestId,
            requester: T::AccountId,
        },
        /// A request has been resolved.
        PostedResult {
            request_id: RequestId,
            reporter: T::AccountId,
        },
        /// A new operator has been added.
        AddedOperator {
            added_operator: T::AccountId,
            added_by: T::AccountId,
        },
        /// A former operator has been removed.
        RemovedOperator {
            removed_operator: T::AccountId,
            removed_by: T::AccountId,
        },
    }

    /// Error for the Witnet pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// A request is too big or overly complex.
        OversizedRequest,
        /// A request is not paying enough.
        UnderpayingRequest,
        /// Tried to follow up on a request ID that was never posted.
        UnknownRequest,
        /// The signer of the transaction is not an operator.
        UnauthorizedOperator,
        /// Reported an empty byte array as the result to a request.
        EmptyResult,
        /// A result is too big or overly complex.
        OversizedResult,
        /// The reported timestamp comes from the future.
        ResultFromFuture,
        /// The result of this request had already been reported.
        AlreadyReported,
        /// An operator is trying to add themself as operator.
        OperatorSelfAddition,
        /// An operator is trying to remove themself as operator.
        OperatorSelfRemoval,
        /// The specific operator account id is unknown.
        UnknownOperator,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(50_000_000)]
        pub fn post_request(
            origin: OriginFor<T>,
            reward: BalanceFor<T>,
            bytes: Vec<u8>,
        ) -> DispatchResult {
            <Pallet<T> as traits::WitnetOracle<T, OriginFor<T>>>::post_request(
                origin, reward, bytes,
            )
        }

        #[pallet::weight(25_000_000)]
        pub fn report_result(
            origin: OriginFor<T>,
            request_id: u64,
            timestamp: TimestampFor<T>,
            dr_tx_hash: [u8; 32],
            result_bytes: Vec<u8>,
        ) -> DispatchResult {
            <Pallet<T> as traits::WitnetOracle<T, OriginFor<T>>>::report_result(
                origin,
                request_id,
                timestamp,
                dr_tx_hash,
                result_bytes,
            )
        }
    }
}

pub mod prelude {
    pub use crate::pallet::{Config as WitnetConfig, Error as WitnetError, Event as WitnetEvent};
    pub use crate::traits::*;
    pub use crate::types::*;
}
