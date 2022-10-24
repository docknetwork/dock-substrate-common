#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::traits::CheckedConversion;
use sp_std::prelude::*;

/// Stores price amount with specified decimals and block number when this record was created.
#[derive(Encode, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceRecord<T> {
    /// Raw price amount. This value should be divided by 10^decimals to get a price per 1 unit.
    amount: u64,
    /// Represents precision. Used to allow storing decimal value as an integer.
    decimals: u8,
    /// Block number when this record was published.
    block_number: T,
}

impl<T> PriceRecord<T> {
    /// Constructs new `PriceRecord` with the given amount, decimals and block number.
    ///
    /// - `amount` - raw price amount. This value should be divided by 10^decimals to get a price per 1 unit.
    /// - `decimals` - value representing precision. Used to allow storing decimal value as an integer.
    /// - `block_number` - block number when this record was published.
    pub const fn new(amount: u64, decimals: u8, block_number: T) -> Self {
        Self {
            amount,
            decimals,
            block_number,
        }
    }

    /// Returns raw price amount. This value should be divided by 10^decimals to get a price per 1 unit.
    pub const fn amount(&self) -> u64 {
        self.amount
    }

    /// Returns value representing precision. Used to allow storing decimal value as an integer.
    pub const fn decimals(&self) -> u32 {
        self.decimals as u32
    }

    /// Returns block number when this record was published.
    pub fn block_number(&self) -> T
    where
        T: Copy,
    {
        self.block_number
    }

    /// Returns price per given amount of units.
    ///
    /// The input value will be converted to `U256` and the output price will be created from `U256`.
    ///
    /// In case of arithmetic/conversion failure, `None` is returned.
    pub fn price_per_unit<I, O>(&self, unit_amount: I) -> Option<O>
    where
        I: TryInto<U256>,
        O: TryFrom<U256>,
    {
        let record_amount: U256 = self.amount().into();
        let divisor = U256::from(10u8).checked_pow(self.decimals().into())?;

        record_amount
            .checked_mul(unit_amount.checked_into()?)?
            .checked_div(divisor)?
            .checked_into()
    }

    /// Attempts to increase decimals amount for the given price record.
    pub fn inc_decimals(mut self, decimals: u8) -> Option<Self> {
        self.decimals = self.decimals.checked_add(decimals)?;

        Some(self)
    }

    /// Attempts to decrease decimals amount for the given price record.
    pub fn dec_decimals(mut self, decimals: u8) -> Option<Self> {
        self.decimals = self.decimals.checked_sub(decimals)?;

        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use sp_core::U256;

    use crate::PriceRecord;

    #[test]
    fn getters() {
        let rec = PriceRecord::new(12345, 6, 7);

        assert_eq!(rec.amount(), 12345);
        assert_eq!(rec.decimals(), 6);
        assert_eq!(rec.block_number(), 7);
    }

    #[test]
    fn price_per_unit() {
        let large_price = PriceRecord::new(u64::MAX, 0, 0);
        assert_eq!(large_price.price_per_unit(1_000), None::<u64>);
        assert_eq!(
            large_price.price_per_unit(1_000),
            Some(18446744073709551615000u128)
        );
        assert_eq!(large_price.price_per_unit(0), Some(0u8));

        let mut standard_price = PriceRecord::new(1234, 3, 0);
        assert_eq!(standard_price.price_per_unit(32u128), Some(39u16));
        assert_eq!(standard_price.price_per_unit(32u64), Some(39u32));
        assert_eq!(standard_price.price_per_unit(32u32), Some(39u64));
        assert_eq!(standard_price.price_per_unit(32u16), Some(39u128));
        assert_eq!(
            standard_price.price_per_unit(32u8),
            Some(U256::from(39u128))
        );

        standard_price = standard_price.inc_decimals(1).unwrap();
        assert_eq!(standard_price.price_per_unit(32u64), Some(3u32));

        standard_price = standard_price.dec_decimals(2).unwrap();
        assert_eq!(standard_price.price_per_unit(32u64), Some(394u32));
    }

    #[test]
    fn decimals() {
        assert_eq!(PriceRecord::new(12345, 255, 7).inc_decimals(1), None);
        assert_eq!(PriceRecord::new(12345, 0, 7).dec_decimals(1), None);
        assert_eq!(
            PriceRecord::new(12345, 15, 7).inc_decimals(15),
            Some(PriceRecord::new(12345, 30, 7))
        );
        assert_eq!(
            PriceRecord::new(12345, 15, 7).dec_decimals(15),
            Some(PriceRecord::new(12345, 0, 7))
        );
    }
}
