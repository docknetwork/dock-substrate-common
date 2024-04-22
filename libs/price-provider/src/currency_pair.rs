//! Defines `CurrencySymbolPair` and `StaticCurrencySymbolPair` used to express price relationship between two currencies.
//! Given some from/to pair price `N` should be considered as `1 x from = N x to`.

use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
};
use frame_support::{traits::Get, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::{prelude::string::String, TypeInfo};
pub use utils::{BoundedString, BoundedStringConversionError, LikeString};

/// Represents from/to currency symbol pair.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CurrencySymbolPair<From, To> {
    /// Represents currency being valued.
    from: From,
    /// Used as a unit to express price.
    to: To,
}

/// Represents from/to currency pair built atop of two types returning `&'static str`.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(Debug, TypeInfo)]
#[scale_info(skip_type_params(From, To))]
pub struct StaticCurrencySymbolPair<From: Get<&'static str>, To: Get<&'static str>> {
    _marker: PhantomData<(From, To)>,
}

/// Stores `CurrencySymbolPair` and limits each of the symbols by the max length in bytes - `MaxSymBytesLen`.
#[derive(TypeInfo, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxSymBytesLen))]
pub struct BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>(
    CurrencySymbolPair<BoundedString<MaxSymBytesLen, From>, BoundedString<MaxSymBytesLen, To>>,
)
where
    From: LikeString,
    To: LikeString,
    MaxSymBytesLen: Get<u32> + 'static;

impl<From: LikeString, To: LikeString> CurrencySymbolPair<From, To> {
    /// Attempts to instantiate new `CurrencySymbolPair` using given from/to currencies.
    pub fn new(from: From, to: To) -> Self {
        Self { from, to }
    }

    /// Maps given currency pair over `from` member and creates a new `CurrencySymbolPair`.
    pub fn map_over_from<R: LikeString, F: FnOnce(From) -> R>(
        self,
        map: F,
    ) -> CurrencySymbolPair<R, To> {
        let Self { from, to } = self;

        CurrencySymbolPair::new((map)(from), to)
    }

    /// Maps given currency pair over `to` member and creates a new `CurrencySymbolPair`.
    pub fn map_over_to<R: LikeString, F: FnOnce(To) -> R>(
        self,
        map: F,
    ) -> CurrencySymbolPair<From, R> {
        let Self { from, to } = self;

        CurrencySymbolPair::new(from, (map)(to))
    }

    /// Translates given currency pair over `from` member and attempts to create a new `CurrencySymbolPair`.
    pub fn translate_over_from<R: LikeString, E, F: FnOnce(From) -> Result<R, E>>(
        self,
        translate: F,
    ) -> Result<CurrencySymbolPair<R, To>, E> {
        let Self { from, to } = self;

        (translate)(from).map(|from| CurrencySymbolPair::new(from, to))
    }

    /// Translates given currency pair over `to` member and attempts to create a new `CurrencySymbolPair`.
    pub fn translate_over_to<R: LikeString, E, F: FnOnce(To) -> Result<R, E>>(
        self,
        translate: F,
    ) -> Result<CurrencySymbolPair<From, R>, E> {
        let Self { from, to } = self;

        (translate)(to).map(|to| CurrencySymbolPair::new(from, to))
    }
}

impl<S: LikeString> CurrencySymbolPair<S, S> {
    /// Maps given currency pair over `from`/`to` members and creates a new `CurrencySymbolPair`.
    pub fn map_pair<R: LikeString, F: FnMut(S) -> R>(self, mut map: F) -> CurrencySymbolPair<R, R> {
        self.map_over_from(&mut map).map_over_to(map)
    }

    /// Translates given currency pair over `from`/`to` members and attempts to create a new `CurrencySymbolPair`.
    pub fn translate_pair<R: LikeString, E, F: FnMut(S) -> Result<R, E>>(
        self,
        mut translate: F,
    ) -> Result<CurrencySymbolPair<R, R>, E> {
        self.translate_over_from(&mut translate)?
            .translate_over_to(translate)
    }
}

impl<From: LikeString + 'static, To: LikeString + 'static, MaxSymBytesLen: Get<u32>>
    TryFrom<CurrencySymbolPair<From, To>> for BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>
{
    type Error = BoundedStringConversionError;

    /// Attempts to convert `CurrencySymbolPair` to the stored format with `MaxSymBytesLen` limit per symbol bytes.
    /// Returns `Err` if the encoded length of either symbol exceeds `MaxSymBytesLen`.
    fn try_from(pair: CurrencySymbolPair<From, To>) -> Result<Self, Self::Error> {
        pair.translate_over_from(BoundedString::new)?
            .translate_over_to(BoundedString::new)
            .map(Self)
    }
}

impl<From: LikeString, To: LikeString, MaxSymBytesLen: Get<u32>> Encode
    for BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>
{
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        self.0.encode_to(dest);
    }
}

impl<From, To, MaxSymBytesLen> Decode for BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>
where
    From: LikeString + Decode + 'static,
    To: LikeString + Decode + 'static,
    MaxSymBytesLen: Get<u32>,
{
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        CurrencySymbolPair::<From, To>::decode(input)?
            .try_into()
            .map_err(Into::into)
    }
}

impl<From: LikeString, To: LikeString, MaxSymBytesLen: Get<u32>>
    EncodeLike<BoundedCurrencySymbolPair<String, String, MaxSymBytesLen>>
    for BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>
{
}

impl<From, To, MaxSymBytesLen> MaxEncodedLen for BoundedCurrencySymbolPair<From, To, MaxSymBytesLen>
where
    From: LikeString,
    To: LikeString,
    MaxSymBytesLen: Get<u32>,
{
    fn max_encoded_len() -> usize {
        let from_max_encoded_len = BoundedString::<MaxSymBytesLen, From>::max_encoded_len();
        let to_max_encoded_len = BoundedString::<MaxSymBytesLen, To>::max_encoded_len();

        from_max_encoded_len.saturating_add(to_max_encoded_len)
    }
}

impl<FromTy, To, MaxSymBytesLen> From<BoundedCurrencySymbolPair<FromTy, To, MaxSymBytesLen>>
    for CurrencySymbolPair<FromTy, To>
where
    FromTy: LikeString + 'static,
    To: LikeString + 'static,
    MaxSymBytesLen: Get<u32>,
{
    fn from(
        BoundedCurrencySymbolPair(pair): BoundedCurrencySymbolPair<FromTy, To, MaxSymBytesLen>,
    ) -> Self {
        pair.map_over_from(BoundedString::into_inner)
            .map_over_to(BoundedString::into_inner)
    }
}

impl<FromTy: LikeString, To: LikeString> From<(FromTy, To)> for CurrencySymbolPair<FromTy, To> {
    fn from((from, to): (FromTy, To)) -> Self {
        Self::new(from, to)
    }
}

impl<From: Get<&'static str>, To: Get<&'static str>>
    Get<CurrencySymbolPair<&'static str, &'static str>> for StaticCurrencySymbolPair<From, To>
{
    fn get() -> CurrencySymbolPair<&'static str, &'static str> {
        CurrencySymbolPair::new(From::get(), To::get())
    }
}

impl<From, To> Display for CurrencySymbolPair<From, To>
where
    From: LikeString + Display,
    To: LikeString + Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.from, self.to)
    }
}

#[cfg(test)]
mod tests {
    #[derive(Eq, PartialEq, Clone, Debug, Encode, TypeInfo)]
    struct A(String);
    impl EncodeLike<String> for A {}

    #[derive(Eq, PartialEq, Clone, Debug, Encode, TypeInfo)]
    struct B(String);
    impl EncodeLike<String> for B {}

    use frame_support::traits::ConstU32;
    use sp_runtime::{parameter_types, traits::CheckedConversion};

    use super::*;

    #[test]
    fn debug() {
        assert_eq!(
            format!("{}", CurrencySymbolPair::new("ABC", "CDE")),
            "ABC/CDE"
        );
    }

    #[test]
    fn map() {
        let one_type_pair = CurrencySymbolPair::new("AB".to_string(), "BC".to_string());
        let diff_type_pair = CurrencySymbolPair::new(A("A".to_owned()), B("B".to_owned()));

        assert_eq!(
            one_type_pair.map_pair(|mut v| {
                unsafe { v.as_bytes_mut() }.reverse();
                v
            }),
            CurrencySymbolPair::new("BA".to_string(), "CB".to_string())
        );

        assert_eq!(
            diff_type_pair.clone().map_over_from(|A(a)| a),
            CurrencySymbolPair::new("A".to_owned(), B("B".to_owned()))
        );
        assert_eq!(
            diff_type_pair.map_over_to(|B(b)| b),
            CurrencySymbolPair::new(A("A".to_owned()), "B".to_owned())
        );
    }

    #[test]
    fn max_bytes_len() {
        assert_eq!("🦅".as_bytes().len(), 4);
        assert_eq!(
            BoundedString::<ConstU32<4>, _>::new("🦅")
                .unwrap()
                .into_inner(),
            "🦅"
        );
        assert_eq!(
            BoundedString::<ConstU32<3>, _>::new("🦅"),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert_eq!(
            BoundedString::<ConstU32<2>, _>::new("ABC"),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert_eq!(
            BoundedString::<ConstU32<0>, _>::new("CDE"),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert!(BoundedString::<ConstU32<3>, _>::new("ABC").is_ok());

        assert_eq!(
            BoundedString::<ConstU32<3>, _>::new("ABC")
                .unwrap()
                .into_inner(),
            "ABC"
        );

        assert_eq!(
            BoundedCurrencySymbolPair::<_, _, ConstU32<2>>::try_from(CurrencySymbolPair::new(
                "ABC", "CDE"
            )),
            Err(BoundedStringConversionError::InvalidStringByteLen)
        );
        assert_eq!(
            BoundedCurrencySymbolPair::<_, _, ConstU32<3>>::try_from(CurrencySymbolPair::new(
                "ABC", "CDE"
            ))
            .unwrap(),
            CurrencySymbolPair::new("ABC", "CDE").try_into().unwrap()
        );
    }

    #[test]
    fn encode_decode_both_same_len() {
        let pair = CurrencySymbolPair::new("ABC", "CDE");
        let bound_pair = pair
            .clone()
            .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<3>>>()
            .unwrap();
        let encoded = bound_pair.encode();
        assert_eq!(encoded.len(), bound_pair.encoded_size());
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundedString::new("ABC".to_string()).unwrap()
        );
        assert_eq!(decoded.0.to, BoundedString::new("CDE".to_string()).unwrap());
        assert_ne!(
            decoded.0.from,
            BoundedString::new("AB".to_string()).unwrap()
        );
        assert_ne!(decoded.0.to, BoundedString::new("E".to_string()).unwrap());
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<4>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundedString::new("ABC".to_string()).unwrap()
        );
        assert_eq!(decoded.0.to, BoundedString::new("CDE".to_string()).unwrap());
        assert_eq!(
            BoundedCurrencySymbolPair::<String, String, ConstU32<2>>::decode(&mut &encoded[..]),
            Err("The string byte size exceeds max allowed".into())
        );
        assert_eq!(
            BoundedCurrencySymbolPair::<String, String, ConstU32<1>>::decode(&mut &encoded[..]),
            Err("The string byte size exceeds max allowed".into())
        );
        assert_eq!(pair.map_pair(ToOwned::to_owned), decoded.into());
    }

    #[test]
    fn encode_decode_first_longer() {
        let pair = CurrencySymbolPair::new("ABCDEF", "X");
        let bound_pair = pair
            .clone()
            .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<6>>>()
            .unwrap();
        let encoded = bound_pair.encode();
        assert_eq!(encoded.len(), bound_pair.encoded_size());
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<6>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundedString::new("ABCDEF".to_string()).unwrap()
        );
        assert_eq!(decoded.0.to, BoundedString::new("X".to_string()).unwrap());
        assert_ne!(
            decoded.0.from,
            BoundedString::new("ABCDE".to_string()).unwrap()
        );
        assert_ne!(decoded.0.to, BoundedString::new("".to_string()).unwrap());
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<6>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundedString::new("ABCDEF".to_string()).unwrap()
        );
        assert_eq!(decoded.0.to, BoundedString::new("X".to_string()).unwrap());
        assert_eq!(
            BoundedCurrencySymbolPair::<String, String, ConstU32<5>>::decode(&mut &encoded[..]),
            Err("The string byte size exceeds max allowed".into())
        );
        assert_eq!(pair.map_pair(ToOwned::to_owned), decoded.into());
    }

    #[test]
    fn encode_decode_second_longer() {
        let pair = CurrencySymbolPair::new("X", "ABCDEF");
        let bound_pair = pair
            .clone()
            .checked_into::<BoundedCurrencySymbolPair<_, _, ConstU32<6>>>()
            .unwrap();
        let encoded = bound_pair.encode();
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<6>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(encoded.len(), bound_pair.encoded_size());
        assert_eq!(decoded.0.from, BoundedString::new("X".to_string()).unwrap());
        assert_eq!(
            decoded.0.to,
            BoundedString::new("ABCDEF".to_string()).unwrap()
        );
        assert_ne!(decoded.0.from, BoundedString::new("".to_string()).unwrap());
        assert_ne!(
            decoded.0.to,
            BoundedString::new("ABCDE".to_string()).unwrap()
        );
        let decoded: BoundedCurrencySymbolPair<String, _, ConstU32<6>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded.0.from, BoundedString::new("X".to_string()).unwrap());
        assert_eq!(
            decoded.0.to,
            BoundedString::new("ABCDEF".to_string()).unwrap()
        );
        assert_eq!(
            BoundedCurrencySymbolPair::<String, String, ConstU32<5>>::decode(&mut &encoded[..]),
            Err("The string byte size exceeds max allowed".into())
        );
        assert_eq!(pair.map_pair(ToOwned::to_owned), decoded.into());
    }

    #[test]
    fn max_encoded_len() {
        assert_eq!(
            BoundedString::<ConstU32<10>>::new("a".repeat(10))
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<10>>::max_encoded_len()
        );
        assert_eq!(BoundedString::<ConstU32<10>>::max_encoded_len(), 11);
        assert_eq!(
            BoundedString::<ConstU32<10>, &'static str>::max_encoded_len(),
            11
        );
        assert_eq!(BoundedString::<ConstU32<1000>>::max_encoded_len(), 1002);
        assert_eq!(
            BoundedString::<ConstU32<1000>, &'static str>::max_encoded_len(),
            1002
        );
        assert_eq!(
            BoundedString::<ConstU32<1000>>::new("a".repeat(1000))
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<1000>>::max_encoded_len()
        );
    }

    #[test]
    fn encoded_size_test() {
        assert_eq!(
            BoundedString::<ConstU32<10>, _>::new("ABCDE")
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<10>>::new("ABCDE".to_string())
                .unwrap()
                .encode()
                .len()
        );
        assert_eq!(
            BoundedString::<ConstU32<100>>::new("ABCDE".to_string())
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<100>>::new("ABCDE".to_string())
                .unwrap()
                .encode()
                .len()
        );
    }

    #[test]
    fn encode_decode_custom_type() {
        impl AsRef<str> for A {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        let pair = CurrencySymbolPair::new(A("123".to_string()), A("122".to_string()));
        let encoded = BoundedCurrencySymbolPair::<_, _, ConstU32<3>>::try_from(pair.clone())
            .unwrap()
            .encode();
        let decoded: BoundedCurrencySymbolPair<_, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundedString::new("123".to_string()).unwrap(),
        );
        assert_eq!(decoded.0.to, BoundedString::new("122".to_string()).unwrap());
        assert_ne!(
            decoded.0.from,
            BoundedString::new("AB".to_string()).unwrap()
        );
        assert_ne!(decoded.0.to, BoundedString::new("E".to_string()).unwrap());

        let decoded_pair: CurrencySymbolPair<_, _> = decoded.into();
        assert_eq!(pair.clone().map_pair(|A(val)| val), decoded_pair);
        assert_eq!(pair, decoded_pair.map_pair(A));
    }

    #[test]
    fn static_types() {
        parameter_types! {
            pub const DockSym: &'static str = "DOCK";
            pub const UsdSym: &'static str = "USD";
            pub const MaxSymbolBytesLen: u32 = 4;
        }

        type DockUsdPair = StaticCurrencySymbolPair<DockSym, UsdSym>;

        let cur_pair = CurrencySymbolPair::<_, _>::new("DOCK", "USD");
        assert_eq!(DockUsdPair::get(), cur_pair);
    }
}
