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
use sp_runtime::DispatchError;

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

impl<From: EncodableAsString, To: EncodableAsString> CurrencySymbolPair<From, To> {
    /// Attempts to instantiate new `CurrencySymbolPair` using given from/to currencies.
    pub fn new(from: From, to: To) -> Self {
        Self { from, to }
    }

    /// Maps given currency pair over `from` member and attempts to create a new `CurrencySymbolPair`.
    pub fn map_over_from<R: EncodableAsString, F: FnMut(From) -> R>(
        self,
        mut map: F,
    ) -> CurrencySymbolPair<R, To> {
        let Self { from, to } = self;

        CurrencySymbolPair::new((map)(from), to)
    }

    /// Maps given currency pair over `to` member and attempts to create a new `CurrencySymbolPair`.
    pub fn map_over_to<R: EncodableAsString, F: FnMut(To) -> R>(
        self,
        mut map: F,
    ) -> CurrencySymbolPair<From, R> {
        let Self { from, to } = self;

        CurrencySymbolPair::new(from, (map)(to))
    }
}

impl<S: EncodableAsString> CurrencySymbolPair<S, S> {
    /// Maps given currency pair over `from`/`to` members and attempts to create a new `CurrencySymbolPair`.
    pub fn map_pair<R: EncodableAsString, F: FnMut(S) -> R>(
        self,
        mut map: F,
    ) -> CurrencySymbolPair<R, R> {
        let Self { from, to } = self;

        CurrencySymbolPair::new((map)(from), (map)(to))
    }
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
#[derive(Decode, TypeInfo, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxSymBytesLen))]
pub struct BoundCurrencySymbolPair<From, To, MaxSymBytesLen>(
    CurrencySymbolPair<
        BoundCurrencySymbol<From, MaxSymBytesLen>,
        BoundCurrencySymbol<To, MaxSymBytesLen>,
    >,
)
where
    From: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static;

/// Errors happening on `CurrencySymbolPair` -> `BoundCurrencySymbolPair` conversion.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoundCurrencySymbolPairError {
    /// The symbol has an invalid length.
    InvalidSymbolByteLen,
}

impl From<BoundCurrencySymbolPairError> for DispatchError {
    fn from(
        BoundCurrencySymbolPairError::InvalidSymbolByteLen: BoundCurrencySymbolPairError,
    ) -> Self {
        DispatchError::Other("The symbol has an invalid length")
    }
}

/// Denotes a type which implements `EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo`
pub trait EncodableAsString:
    EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo + 'static
{
}
impl<T: EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo + 'static> EncodableAsString
    for T
{
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>>
    TryFrom<CurrencySymbolPair<From, To>> for BoundCurrencySymbolPair<From, To, MaxSymBytesLen>
{
    type Error = BoundCurrencySymbolPairError;

    /// Attempts to convert `CurrencySymbolPair` to the stored format with `MaxSymBytesLen` limit per symbol bytes.
    /// Returns `Err` if the encoded length of either symbol exceeds `MaxSymBytesLen`.
    fn try_from(
        CurrencySymbolPair { from, to }: CurrencySymbolPair<From, To>,
    ) -> Result<Self, Self::Error> {
        BoundCurrencySymbol::new(from)
            .zip(BoundCurrencySymbol::new(to))
            .map(CurrencySymbolPair::from)
            .map(Self)
            .ok_or(BoundCurrencySymbolPairError::InvalidSymbolByteLen)
    }
}

/// Symbol of the currency used in `CurrencySymbolPair` limited by the max encoded size.
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxBytesLen))]
struct BoundCurrencySymbol<S: EncodableAsString, MaxBytesLen: Get<u32>> {
    sym: S,
    _marker: PhantomData<MaxBytesLen>,
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> EncodeLike<String>
    for BoundCurrencySymbol<S, MaxBytesLen>
{
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> MaxEncodedLen
    for BoundCurrencySymbol<S, MaxBytesLen>
{
    fn max_encoded_len() -> usize {
        codec::Compact(MaxBytesLen::get())
            .encoded_size()
            .saturating_add(MaxBytesLen::get() as usize)
    }
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> BoundCurrencySymbol<S, MaxBytesLen> {
    /// Instantiates `Self` if encoded byte size of the provided currency doesn't exceed `MaxBytesLen`.
    fn new(sym: S) -> Option<Self> {
        (sym.encoded_size() <= Self::max_encoded_len()).then_some(Self {
            sym,
            _marker: PhantomData,
        })
    }
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>> Encode
    for BoundCurrencySymbolPair<From, To, MaxSymBytesLen>
{
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        self.0.encode_to(dest);
    }
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>>
    EncodeLike<BoundCurrencySymbolPair<String, String, MaxSymBytesLen>>
    for BoundCurrencySymbolPair<From, To, MaxSymBytesLen>
{
}

impl<From, To, MaxSymBytesLen> MaxEncodedLen for BoundCurrencySymbolPair<From, To, MaxSymBytesLen>
where
    From: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static,
{
    fn max_encoded_len() -> usize {
        BoundCurrencySymbol::<From, MaxSymBytesLen>::max_encoded_len()
            .saturating_add(BoundCurrencySymbol::<To, MaxSymBytesLen>::max_encoded_len())
    }
}

impl<FromTy, To, MaxSymBytesLen> From<BoundCurrencySymbolPair<FromTy, To, MaxSymBytesLen>>
    for CurrencySymbolPair<FromTy, To>
where
    FromTy: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static,
{
    fn from(
        BoundCurrencySymbolPair(currency_pair): BoundCurrencySymbolPair<FromTy, To, MaxSymBytesLen>,
    ) -> Self {
        currency_pair
            .map_over_from(|BoundCurrencySymbol { sym, .. }| sym)
            .map_over_to(|BoundCurrencySymbol { sym, .. }| sym)
    }
}

impl<FromTy: EncodableAsString, To: EncodableAsString> From<(FromTy, To)>
    for CurrencySymbolPair<FromTy, To>
{
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
    From: EncodableAsString + Display,
    To: EncodableAsString + Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.from, self.to)
    }
}

impl<From: Get<&'static str>, To: Get<&'static str>> Display
    for StaticCurrencySymbolPair<From, To>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}", Self::get())
    }
}

#[cfg(test)]
mod tests {
    #[derive(PartialEq, Clone, Debug, Encode, TypeInfo)]
    struct A(String);
    impl EncodeLike<String> for A {}

    #[derive(PartialEq, Clone, Debug, Encode, TypeInfo)]
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
    fn encoded_size() {
        assert_eq!("🦅".as_bytes().len(), 4);
        assert_eq!(
            BoundCurrencySymbol::<_, ConstU32<4>>::new("🦅")
                .unwrap()
                .sym,
            "🦅"
        );
        assert_eq!(BoundCurrencySymbol::<_, ConstU32<3>>::new("🦅"), None);
        assert_eq!(BoundCurrencySymbol::<_, ConstU32<2>>::new("ABC"), None);
        assert_eq!(BoundCurrencySymbol::<_, ConstU32<0>>::new("CDE"), None);
        assert!(BoundCurrencySymbol::<_, ConstU32<3>>::new("ABC").is_some());

        assert_eq!(
            BoundCurrencySymbol::<_, ConstU32<3>>::new("ABC")
                .unwrap()
                .sym,
            "ABC"
        );

        assert_eq!(
            BoundCurrencySymbolPair::<_, _, ConstU32<2>>::try_from(CurrencySymbolPair::new(
                "ABC", "CDE"
            )),
            Err(BoundCurrencySymbolPairError::InvalidSymbolByteLen)
        );
        assert_eq!(
            BoundCurrencySymbolPair::<_, _, ConstU32<3>>::try_from(CurrencySymbolPair::new(
                "ABC", "CDE"
            ))
            .unwrap(),
            CurrencySymbolPair::new("ABC", "CDE").try_into().unwrap()
        );
    }

    #[test]
    fn encode_decode() {
        let pair = CurrencySymbolPair::new("ABC", "CDE");
        let encoded = pair
            .clone()
            .checked_into::<BoundCurrencySymbolPair<_, _, ConstU32<3>>>()
            .unwrap()
            .encode();
        let decoded: BoundCurrencySymbolPair<String, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundCurrencySymbol::new("ABC".to_string()).unwrap()
        );
        assert_eq!(
            decoded.0.to,
            BoundCurrencySymbol::new("CDE".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.from,
            BoundCurrencySymbol::new("AB".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.to,
            BoundCurrencySymbol::new("E".to_string()).unwrap()
        );

        assert_eq!(pair.map_pair(ToOwned::to_owned), decoded.into());
    }

    #[test]
    fn encode_decode_custom_type() {
        impl AsRef<str> for A {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        let pair = CurrencySymbolPair::new(A("123".to_string()), A("122".to_string()));
        let encoded = BoundCurrencySymbolPair::<_, _, ConstU32<3>>::try_from(pair.clone())
            .unwrap()
            .encode();
        let decoded: BoundCurrencySymbolPair<_, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            BoundCurrencySymbol::new("123".to_string()).unwrap(),
        );
        assert_eq!(
            decoded.0.to,
            BoundCurrencySymbol::new("122".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.from,
            BoundCurrencySymbol::new("AB".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.to,
            BoundCurrencySymbol::new("E".to_string()).unwrap()
        );

        let decoded_pair: CurrencySymbolPair<_, _> = decoded.into();
        assert_eq!(pair.clone().map_pair(|A(val)| val), decoded_pair);
        assert_eq!(pair, decoded_pair.map_pair(A));
    }

    #[test]
    fn static_types() {
        parameter_types! {
            pub const DOCKSym: &'static str = "DOCK";
            pub const USDSym: &'static str = "USD";
            pub const MaxSymbolBytesLen: u32 = 4;
        }

        type DockUsdPair = StaticCurrencySymbolPair<DOCKSym, USDSym>;

        let cur_pair = CurrencySymbolPair::<_, _>::new("DOCK", "USD");
        assert_eq!(DockUsdPair::get(), cur_pair);

        assert_eq!(format!("{}", DockUsdPair::get()), "DOCK/USD");
    }
}
