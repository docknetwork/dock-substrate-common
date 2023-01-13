//! Defines `CurrencyPair` and `StaticCurrencyPair` used to express price relationship between two currencies.
//! Given some from/to pair price `N` should be considered as `1 x from = N x to`.

use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
};
use frame_support::{traits::Get, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

/// A type which implements `EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo`
pub trait EncodableAsString:
    EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo + 'static
{
}
impl<T: EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo + 'static> EncodableAsString
    for T
{
}

/// Symbol of the currency used in `CurrencyPair` limited by the max encoded size.
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxBytesLen))]
struct StoredCurrencySymbol<S: EncodableAsString, MaxBytesLen: Get<u32>> {
    sym: S,
    _marker: PhantomData<MaxBytesLen>,
}

/// Conversion errors happening on `CurrencyPair` -> `StoredCurrencyPair`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StoredCurrencyPairError {
    /// The symbol has an invalid length.
    InvalidSymbolByteLen,
}

impl From<StoredCurrencyPairError> for DispatchError {
    fn from(StoredCurrencyPairError::InvalidSymbolByteLen: StoredCurrencyPairError) -> Self {
        DispatchError::Other("The symbol has an invalid length")
    }
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> EncodeLike<String>
    for StoredCurrencySymbol<S, MaxBytesLen>
{
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>>
    TryFrom<CurrencyPair<From, To>> for StoredCurrencyPair<From, To, MaxSymBytesLen>
{
    type Error = StoredCurrencyPairError;

    fn try_from(CurrencyPair { from, to }: CurrencyPair<From, To>) -> Result<Self, Self::Error> {
        StoredCurrencySymbol::new(from)
            .zip(StoredCurrencySymbol::new(to))
            .map(CurrencyPair::from)
            .map(Self)
            .ok_or(StoredCurrencyPairError::InvalidSymbolByteLen)
    }
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> MaxEncodedLen
    for StoredCurrencySymbol<S, MaxBytesLen>
{
    fn max_encoded_len() -> usize {
        codec::Compact(MaxBytesLen::get())
            .encoded_size()
            .saturating_add(MaxBytesLen::get() as usize)
    }
}

impl<S: EncodableAsString, MaxBytesLen: Get<u32>> StoredCurrencySymbol<S, MaxBytesLen> {
    /// Instantiates `Self` if encoded byte size of the provided currency doesn't exceed `MaxBytesLen`.
    fn new(sym: S) -> Option<Self> {
        (sym.encoded_size() <= Self::max_encoded_len()).then_some(Self {
            sym,
            _marker: PhantomData,
        })
    }
}

/// Represents from/to currency pair.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CurrencyPair<From, To> {
    /// Represents currency being valued.
    from: From,
    /// Used as a unit to express price.
    to: To,
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>> Encode
    for StoredCurrencyPair<From, To, MaxSymBytesLen>
{
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        self.0.encode_to(dest);
    }
}

impl<From: EncodableAsString, To: EncodableAsString, MaxSymBytesLen: Get<u32>>
    EncodeLike<StoredCurrencyPair<String, String, MaxSymBytesLen>>
    for StoredCurrencyPair<From, To, MaxSymBytesLen>
{
}

#[derive(Decode, TypeInfo, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxSymBytesLen))]
pub struct StoredCurrencyPair<From, To, MaxSymBytesLen>(
    CurrencyPair<
        StoredCurrencySymbol<From, MaxSymBytesLen>,
        StoredCurrencySymbol<To, MaxSymBytesLen>,
    >,
)
where
    From: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static;

impl<From, To, MaxSymBytesLen> MaxEncodedLen for StoredCurrencyPair<From, To, MaxSymBytesLen>
where
    From: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static,
{
    fn max_encoded_len() -> usize {
        StoredCurrencySymbol::<From, MaxSymBytesLen>::max_encoded_len()
            .saturating_add(StoredCurrencySymbol::<To, MaxSymBytesLen>::max_encoded_len())
    }
}

impl<FromTy, To, MaxSymBytesLen> From<StoredCurrencyPair<FromTy, To, MaxSymBytesLen>>
    for CurrencyPair<FromTy, To>
where
    FromTy: EncodableAsString,
    To: EncodableAsString,
    MaxSymBytesLen: Get<u32> + 'static,
{
    fn from(
        StoredCurrencyPair(currency_pair): StoredCurrencyPair<FromTy, To, MaxSymBytesLen>,
    ) -> Self {
        currency_pair
            .map_over_from(|StoredCurrencySymbol { sym, .. }| sym)
            .map_over_to(|StoredCurrencySymbol { sym, .. }| sym)
    }
}

impl<From: EncodableAsString, To: EncodableAsString> CurrencyPair<From, To> {
    /// Attempts to instantiate new `CurrencyPair` using given from/to currencies.
    /// Returns `None` if the encoded length of either currency exceeds `MaxSymBytesLen`
    pub fn new(from: From, to: To) -> Self {
        Self { from, to }
    }

    /// Maps given currency pair over `from` member and attempts to create a new `CurrencyPair`.
    pub fn map_over_from<R: EncodableAsString, F: FnMut(From) -> R>(
        self,
        mut map: F,
    ) -> CurrencyPair<R, To> {
        let Self { from, to } = self;

        CurrencyPair::new((map)(from), to)
    }

    /// Maps given currency pair over `to` member and attempts to create a new `CurrencyPair`.
    pub fn map_over_to<R: EncodableAsString, F: FnMut(To) -> R>(
        self,
        mut map: F,
    ) -> CurrencyPair<From, R> {
        let Self { from, to } = self;

        CurrencyPair::new(from, (map)(to))
    }
}

impl<S: EncodableAsString> CurrencyPair<S, S> {
    /// Maps given currency pair over `from`/`to` members and attempts to create a new `CurrencyPair`.
    pub fn map_pair<R: EncodableAsString, F: FnMut(S) -> R>(
        self,
        mut map: F,
    ) -> CurrencyPair<R, R> {
        let Self { from, to } = self;

        CurrencyPair::new((map)(from), (map)(to))
    }
}

impl<FromTy: EncodableAsString, To: EncodableAsString> From<(FromTy, To)>
    for CurrencyPair<FromTy, To>
{
    fn from((from, to): (FromTy, To)) -> Self {
        Self::new(from, to)
    }
}

impl<From, To> Display for CurrencyPair<From, To>
where
    From: EncodableAsString + Display,
    To: EncodableAsString + Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.from, self.to)
    }
}

/// Represents from/to currency pair built atop of two types returning `&'static str`.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[derive(TypeInfo)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxSymBytesLen))]
pub struct StaticCurrencyPair<From, To> {
    _marker: PhantomData<(From, To)>,
}

impl<From: Get<&'static str>, To: Get<&'static str>> Get<CurrencyPair<&'static str, &'static str>>
    for StaticCurrencyPair<From, To>
{
    fn get() -> CurrencyPair<&'static str, &'static str> {
        CurrencyPair::new(From::get(), To::get())
    }
}

impl<From: Get<&'static str>, To: Get<&'static str>> Display for StaticCurrencyPair<From, To> {
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
        assert_eq!(format!("{}", CurrencyPair::new("ABC", "CDE")), "ABC/CDE");
    }

    #[test]
    fn map() {
        let one_type_pair = CurrencyPair::new("AB".to_string(), "BC".to_string());
        let diff_type_pair = CurrencyPair::new(A("A".to_owned()), B("B".to_owned()));

        assert_eq!(
            one_type_pair.map_pair(|mut v| {
                unsafe { v.as_bytes_mut() }.reverse();
                v
            }),
            CurrencyPair::new("BA".to_string(), "CB".to_string())
        );

        assert_eq!(
            diff_type_pair.clone().map_over_from(|A(a)| a),
            CurrencyPair::new("A".to_owned(), B("B".to_owned()))
        );
        assert_eq!(
            diff_type_pair.map_over_to(|B(b)| b),
            CurrencyPair::new(A("A".to_owned()), "B".to_owned())
        );
    }

    #[test]
    fn encoded_size() {
        assert_eq!("🦅".as_bytes().len(), 4);
        assert_eq!(
            StoredCurrencySymbol::<_, ConstU32<4>>::new("🦅")
                .unwrap()
                .sym,
            "🦅"
        );
        assert_eq!(StoredCurrencySymbol::<_, ConstU32<3>>::new("🦅"), None);
        assert_eq!(StoredCurrencySymbol::<_, ConstU32<2>>::new("ABC"), None);
        assert_eq!(StoredCurrencySymbol::<_, ConstU32<0>>::new("CDE"), None);
        assert!(StoredCurrencySymbol::<_, ConstU32<3>>::new("ABC").is_some());

        assert_eq!(
            StoredCurrencySymbol::<_, ConstU32<3>>::new("ABC")
                .unwrap()
                .sym,
            "ABC"
        );

        assert_eq!(
            StoredCurrencyPair::<_, _, ConstU32<2>>::try_from(CurrencyPair::new("ABC", "CDE")),
            Err(StoredCurrencyPairError::InvalidSymbolByteLen)
        );
        assert_eq!(
            StoredCurrencyPair::<_, _, ConstU32<3>>::try_from(CurrencyPair::new("ABC", "CDE"))
                .unwrap(),
            CurrencyPair::new("ABC", "CDE").try_into().unwrap()
        );
    }

    #[test]
    fn encode_decode() {
        let pair = CurrencyPair::new("ABC", "CDE");
        let encoded = pair
            .clone()
            .checked_into::<StoredCurrencyPair<_, _, ConstU32<3>>>()
            .unwrap()
            .encode();
        let decoded: StoredCurrencyPair<String, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            StoredCurrencySymbol::new("ABC".to_string()).unwrap()
        );
        assert_eq!(
            decoded.0.to,
            StoredCurrencySymbol::new("CDE".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.from,
            StoredCurrencySymbol::new("AB".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.to,
            StoredCurrencySymbol::new("E".to_string()).unwrap()
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

        let pair = CurrencyPair::new(A("123".to_string()), A("122".to_string()));
        let encoded = StoredCurrencyPair::<_, _, ConstU32<3>>::try_from(pair.clone())
            .unwrap()
            .encode();
        let decoded: StoredCurrencyPair<_, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(
            decoded.0.from,
            StoredCurrencySymbol::new("123".to_string()).unwrap(),
        );
        assert_eq!(
            decoded.0.to,
            StoredCurrencySymbol::new("122".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.from,
            StoredCurrencySymbol::new("AB".to_string()).unwrap()
        );
        assert_ne!(
            decoded.0.to,
            StoredCurrencySymbol::new("E".to_string()).unwrap()
        );

        let decoded_pair: CurrencyPair<_, _> = decoded.into();
        assert_eq!(pair.clone().map_pair(|A(val)| val), decoded_pair);
        assert_eq!(pair, decoded_pair.map_pair(A));
    }

    #[test]
    fn static_types() {
        parameter_types! {
            pub const DOCKSym: &'static str = "DOCK";
            pub const USDSym: &'static str = "USD";
            pub const MaxCurrencyLen: u32 = 4;
        }

        type DockUsdPair = StaticCurrencyPair<DOCKSym, USDSym>;

        let cur_pair = CurrencyPair::<_, _>::new("DOCK", "USD");
        assert_eq!(DockUsdPair::get(), cur_pair);

        assert_eq!(format!("{}", DockUsdPair::get()), "DOCK/USD");
    }
}
