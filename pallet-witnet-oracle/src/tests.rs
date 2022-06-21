use crate::{
    DispatchResult,
    mock::{ExtBuilder, Origin, Test},
    prelude::*,
};
use sp_runtime::traits::Zero;

#[test]
fn pallet_works() {
    ExtBuilder::default().build_and_execute(|| {
        /*let oracle: DispatchResult = WitnetOracle::<Test, Origin>::post_request(
            Origin::signed(0),
            BalanceOf::<Test>::zero(),
            Vec::<u8>::new(),
        );*/
    });
}
