use frame_support::{
    assert_noop, assert_ok, parameter_types,
    traits::{ConstU32, Get},
};
use price_provider::{
    currency_pair::StaticCurrencySymbolPair, BoundedCurrencySymbolPair,
    BoundedStringConversionError, CurrencySymbolPair, PriceProvider, PriceRecord,
};
use sp_runtime::{traits::CheckedConversion, DispatchError};
use sp_std::borrow::ToOwned;

use crate::{mock::*, Error, Prices};

#[test]
fn add_and_remove_operator() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                1
            ),
            None
        );
        assert_noop!(
            PriceFeedModule::add_operator(
                Origin::signed(1),
                CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
                1
            ),
            DispatchError::BadOrigin
        );
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                1
            ),
            None
        );
        assert_ok!(PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        ));
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                1
            ),
            Some(())
        );
        assert_ok!(PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            2
        ));
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                2
            ),
            Some(())
        );
        assert_ok!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            2
        ));

        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                2,
            ),
            None
        );

        assert_noop!(
            PriceFeedModule::remove_operator(
                Origin::signed(1),
                CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
                1
            ),
            DispatchError::BadOrigin
        );
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                1
            ),
            Some(())
        );
        assert_ok!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        ));
        assert_eq!(
            PriceFeedModule::operators(
                CurrencySymbolPair::new("A", "B")
                    .map_pair(ToOwned::to_owned)
                    .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<4>>>()
                    .unwrap(),
                1
            ),
            None
        );
        assert_noop!(
            PriceFeedModule::remove_operator(
                Origin::root(),
                CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
                1
            ),
            Error::<Test>::OperatorDoesNotExist
        );
        assert_noop!(
            PriceFeedModule::remove_operator(
                Origin::root(),
                CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
                2
            ),
            Error::<Test>::OperatorDoesNotExist
        );
    })
}

#[test]
fn set_price() {
    new_test_ext().execute_with(|| {
        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            10,
            1
        )
        .is_ok());
        assert_eq!(
            PriceFeedModule::price(
                CurrencySymbolPair::new("A", "B")
                    .checked_into::<BoundedCurrencySymbolPair<_, _, _>>()
                    .unwrap()
            )
            .unwrap(),
            PriceRecord::new(10, 1, 0)
        );
        assert_noop!(
            PriceFeedModule::set_price(
                Origin::signed(1),
                CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
                1,
                1
            ),
            Error::<Test>::NotAnOperator
        );
        assert_noop!(
            PriceFeedModule::set_price(
                Origin::signed(2),
                CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
                1,
                1
            ),
            Error::<Test>::NotAnOperator
        );

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            2,
        )
        .unwrap();

        assert_ok!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        ));
        assert_ok!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            2
        ));
        assert_noop!(
            PriceFeedModule::set_price(
                Origin::signed(2),
                CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
                1,
                1
            ),
            Error::<Test>::NotAnOperator
        );
    })
}

#[test]
fn price_provider() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            PriceFeedModule::pair_price(CurrencySymbolPair::new("A", "B")),
            Ok(None)
        );
        assert_eq!(
            PriceFeedModule::pair_price(CurrencySymbolPair::new("ABCDE", "B")),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert_eq!(
            PriceFeedModule::pair_price(CurrencySymbolPair::new("A", "BCDEF")),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
    });
}

#[test]
fn dock_price_provider() {
    use crate::StaticPriceProvider;

    new_test_ext().execute_with(|| {
        parameter_types! {
            pub const DOCKSym: &'static str = "DOCK";
            pub const USDSym: &'static str = "USD";
            pub const LARGESym: &'static str = "ABCDE";
        }

        type DockUsdPair = StaticCurrencySymbolPair<DOCKSym, USDSym>;
        type LargeSymUsdPair = StaticCurrencySymbolPair<LARGESym, USDSym>;
        type UsdLargeCurrencySymbolPair = StaticCurrencySymbolPair<USDSym, LARGESym>;

        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, DockUsdPair>>::pair(),
            CurrencySymbolPair::new("DOCK", "USD")
        );

        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, DockUsdPair>>::pair(),
            DockUsdPair::get()
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, LargeSymUsdPair>>::pair(),
            LargeSymUsdPair::get()
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, UsdLargeCurrencySymbolPair>>::pair(),
            UsdLargeCurrencySymbolPair::get()
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, DockUsdPair>>::price(),
            Ok(None)
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, LargeSymUsdPair>>::price(),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, UsdLargeCurrencySymbolPair>>::price(),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );

        Prices::<Test>::insert(
            CurrencySymbolPair::new("DOCK", "USD")
                .checked_into::<BoundedCurrencySymbolPair<_, _, _>>()
                .unwrap(),
            PriceRecord::new(100, 2, 0),
        );

        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, DockUsdPair>>::price(),
            Ok(Some(PriceRecord::new(100, 2, 0)))
        );
    })
}
