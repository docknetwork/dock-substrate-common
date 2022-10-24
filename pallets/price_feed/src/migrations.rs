#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use codec::{Decode, Encode};

pub mod v1 {
    /// Function and event param types.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    pub enum ParamType {
        /// Address.
        Address,
        /*/// Bytes.
        Bytes,*/
        /// Signed integer. u16 is sufficient as largest EVM integer type is 256 bit
        Int(u16),
        /// Unsigned integer. u16 is sufficient as largest EVM integer type is 256 bit
        Uint(u16),
        /*/// Boolean.
        Bool,
        /// String.
        String,
        /// Array of unknown size.
        Array(Box<ParamType>),
        /// Vector of bytes with fixed size.
        FixedBytes(usize),
        /// Array with fixed size.
        FixedArray(Box<ParamType>, usize),
        /// Tuple containing different types
        Tuple(Vec<ParamType>),*/
    }

    use super::*;
    use frame_support::weights::Weight;
    use scale_info::TypeInfo;
    use sp_core::H160;
    use sp_std::prelude::*;

    const DUMMY_SOURCE: H160 = H160::zero();

    #[derive(codec::Encode, codec::Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    pub struct ContractConfig {
        /// Address of the proxy contract
        pub address: H160,
        /// ABI of the method `aggregator` of the proxy contract. This method is called to get the
        /// address of the Aggregator contract from which price has to be checked. The return value of
        /// this method is a single value which is an address.
        pub query_aggregator_abi_encoded: Vec<u8>,
        /// The ABI of the function to get the price, encoded.
        /// At the time of writing, it is function `latestRoundData` of the contract.
        pub query_price_abi_encoded: Vec<u8>,
        /// ABI of the return type of function corresponding to `query_abi_encoded`.
        /// At the time of writing, this is `[uint(80), int(256), uint(256), uint(256), uint(80)]`
        pub return_val_abi: Vec<ParamType>,
    }

    impl Default for ContractConfig {
        fn default() -> Self {
            ContractConfig {
                address: DUMMY_SOURCE,
                query_aggregator_abi_encoded: vec![],
                query_price_abi_encoded: vec![],
                return_val_abi: vec![],
            }
        }
    }

    decl_storage! {
        trait Store for Module<T: Config> as PriceFeedModule {
            /// Stores contract configuration for DOCK/USD pair. This is the only pair that is relevant right now.
            /// If we need more pairs in future, we can change this to map with a runtime storage migration
            pub ContractConfigStore get(fn contract_config): Option<ContractConfig>;

            /// Price of DOCK/USD pair
            pub Price get(fn price): Option<u32>;

            /// Last update to price by reading from contract was done at this block number
            pub LastPriceUpdateAt get(fn last_price_update_at): Option<T::BlockNumber>;

            /// Price update frequency. After every few blocks the price is read from the contract and
            /// the storage item `Price` is updated unless update frequency is set to `None` or 0.
            pub PriceUpdateFreq get(fn price_update_freq): Option<u32>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {}
    }

    pub fn migrate_to_v2<T: Config>() -> Weight {
        Price::kill();
        ContractConfigStore::kill();
        LastPriceUpdateAt::<T>::kill();
        PriceUpdateFreq::kill();
        StorageVersion::put(Releases::V2MultiPair);

        T::DbWeight::get().writes(5)
    }
}
