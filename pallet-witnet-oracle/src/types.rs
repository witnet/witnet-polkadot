use super::*;

pub type BalanceFor<T> = <<T as Config>::Currency as frame_support::traits::Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub type TimestampFor<T> = <<T as Config>::TimeProvider as frame_support::traits::Time>::Moment;

pub type Query<T> = (
    frame_support::BoundedVec<u8, <T as Config>::MaxByteSize>,
    BalanceFor<T>,
);
pub type RequestId = u64;

pub type RequestEntry<T> = (
    Option<Query<T>>,
    Option<Response<T>>,
    Option<<T as frame_system::Config>::AccountId>,
);

pub type Response<T> = (
    TimestampFor<T>,
    [u8; 32],
    frame_support::BoundedVec<u8, <T as Config>::MaxByteSize>,
);
