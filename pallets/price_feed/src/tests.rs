use price_provider::{CurrencyPair, PriceProvider, PriceRecord};

use crate::{mock::*, Prices};

#[test]
fn add_operator() {
    new_test_ext().execute_with(|| {
        assert!(PriceFeedModule::add_operator(
            Origin::signed(1),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1
        )
        .is_err());
        assert!(PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1
        )
        .is_ok());

        assert!(PriceFeedModule::remove_operator(
            Origin::signed(1),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1
        )
        .is_err());
        assert!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
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
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencyPair::new("A", "B")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            10,
            1
        )
        .is_ok());
        assert_eq!(
            PriceFeedModule::price(CurrencyPair::new("A", "B").unwrap()).unwrap(),
            PriceRecord::new(10, 1, 0)
        );
        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencyPair::new("B", "C")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1,
            1
        )
        .is_err());
        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencyPair::new("B", "C")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("B", "C")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            2,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencyPair::new("B", "C")
                .unwrap()
                .map_pair(ToOwned::to_owned)
                .unwrap(),
            1,
            1
        )
        .is_ok());
    })
}

use frame_support::{parameter_types, traits::Get};

use crate::BoundPriceProvider;

#[test]
fn dock_price_provider() {
    new_test_ext().execute_with(|| {
        parameter_types! {
            pub const DOCK: &'static str = "DOCK";
            pub const USD: &'static str = "USD";
            pub const MaxLen: u32 = 4;
        }

        impl<M: Get<u32>> BoundPriceProvider<Test, M> for PriceFeedModule {
            type From = DOCK;
            type To = USD;

            fn price() -> Option<PriceRecord<<Test as frame_system::Config>::BlockNumber>> {
                Self::pair().and_then(PriceFeedModule::pair_price)
            }
        }

        assert_eq!(
            <PriceFeedModule as BoundPriceProvider<Test, MaxLen>>::pair()
                .unwrap()
                .into_currency_pair(),
            CurrencyPair::new("DOCK", "USD").unwrap()
        );

        assert_eq!(
            <PriceFeedModule as BoundPriceProvider<Test, MaxLen>>::price(),
            None
        );

        Prices::<Test>::insert(
            CurrencyPair::new("DOCK", "USD").unwrap(),
            PriceRecord::new(100, 2, 0),
        );

        assert_eq!(
            <PriceFeedModule as BoundPriceProvider<Test, MaxLen>>::price(),
            Some(PriceRecord::new(100, 2, 0))
        );
    })
}
