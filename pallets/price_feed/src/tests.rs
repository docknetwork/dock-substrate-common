use frame_support::{parameter_types, traits::Get};
use price_provider::{
    currency_pair::StaticCurrencySymbolPair, CurrencySymbolPair, PriceProvider, PriceRecord,
    StoredCurrencySymbolPair, StoredCurrencySymbolPairError,
};
use sp_runtime::traits::CheckedConversion;
use sp_std::borrow::ToOwned;

use crate::{mock::*, Prices};

#[test]
fn add_operator() {
    new_test_ext().execute_with(|| {
        assert!(PriceFeedModule::add_operator(
            Origin::signed(1),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_err());
        assert!(PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_ok());

        assert!(PriceFeedModule::remove_operator(
            Origin::signed(1),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_err());
        assert!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencySymbolPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_ok());
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
                    .checked_into::<StoredCurrencySymbolPair<_, _, _>>()
                    .unwrap()
            )
            .unwrap(),
            PriceRecord::new(10, 1, 0)
        );
        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());
        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            2,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencySymbolPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_ok());
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
            Err(StoredCurrencySymbolPairError::InvalidSymbolByteLen)
        );
        assert_eq!(
            PriceFeedModule::pair_price(CurrencySymbolPair::new("A", "BCDEF")),
            Err(StoredCurrencySymbolPairError::InvalidSymbolByteLen)
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
            Err(StoredCurrencySymbolPairError::InvalidSymbolByteLen)
        );
        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, UsdLargeCurrencySymbolPair>>::price(),
            Err(StoredCurrencySymbolPairError::InvalidSymbolByteLen)
        );

        Prices::<Test>::insert(
            CurrencySymbolPair::new("DOCK", "USD")
                .checked_into::<StoredCurrencySymbolPair<_, _, _>>()
                .unwrap(),
            PriceRecord::new(100, 2, 0),
        );

        assert_eq!(
            <PriceFeedModule as StaticPriceProvider<Test, DockUsdPair>>::price(),
            Ok(Some(PriceRecord::new(100, 2, 0)))
        );
    })
}
