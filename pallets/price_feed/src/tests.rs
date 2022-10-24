use price_provider::{CurrencyPair, PriceRecord};

use crate::mock::*;

#[test]
fn add_operator() {
    new_test_ext().execute_with(|| {
        assert!(PriceFeedModule::add_operator(
            Origin::signed(1),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_err());
        assert!(PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_ok());

        assert!(PriceFeedModule::remove_operator(
            Origin::signed(1),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            1
        )
        .is_err());
        assert!(PriceFeedModule::remove_operator(
            Origin::root(),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
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
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            1,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencyPair::new("A", "B").map_pair(ToOwned::to_owned),
            10,
            1
        )
        .is_ok());
        assert_eq!(
            PriceFeedModule::price(CurrencyPair::new("A", "B")).unwrap(),
            PriceRecord::new(10, 1, 0)
        );
        assert!(PriceFeedModule::set_price(
            Origin::signed(1),
            CurrencyPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());
        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencyPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_err());

        PriceFeedModule::add_operator(
            Origin::root(),
            CurrencyPair::new("B", "C").map_pair(ToOwned::to_owned),
            2,
        )
        .unwrap();

        assert!(PriceFeedModule::set_price(
            Origin::signed(2),
            CurrencyPair::new("B", "C").map_pair(ToOwned::to_owned),
            1,
            1
        )
        .is_ok());
    })
}
