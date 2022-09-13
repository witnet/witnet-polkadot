use super::*;

pub type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub type Timestamp<T> = <<T as Config>::TimeProvider as frame_support::traits::Time>::Moment;

pub type Query<T> = (
    frame_support::BoundedVec<u8, <T as Config>::MaxByteSize>,
    BalanceOf<T>,
);
pub type RequestId = u64;

pub type RequestEntry<T> = (
    Option<Query<T>>,
    Option<Response<T>>,
    Option<<T as frame_system::Config>::AccountId>,
);

pub type Response<T> = (
    Timestamp<T>,
    [u8; 32],
    frame_support::BoundedVec<u8, <T as Config>::MaxByteSize>,
);
