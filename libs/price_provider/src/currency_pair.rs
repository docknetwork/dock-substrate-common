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

/// A type which implements `EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo`
pub trait EncodableString: EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo {}
impl<T: EncodeLike<String> + PartialEq + Clone + Debug + TypeInfo> EncodableString for T {}

/// Member of the currency pair used to express currency name limited by the max encoding size.
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxBytesLen))]
struct PairMember<S: EncodableString, MaxBytesLen: Get<u32>> {
    currency: S,
    _marker: PhantomData<MaxBytesLen>,
}

impl<S: EncodableString, MaxBytesLen: Get<u32>> MaxEncodedLen for PairMember<S, MaxBytesLen> {
    fn max_encoded_len() -> usize {
        codec::Compact(MaxBytesLen::get())
            .encoded_size()
            .saturating_add(MaxBytesLen::get() as usize)
    }
}

impl<S: EncodableString, MaxBytesLen: Get<u32>> PairMember<S, MaxBytesLen> {
    /// Instantiates `Self` if encoded byte size of the provided currency doesn't exceed `MaxBytesLen`.
    fn new(currency: S) -> Option<Self> {
        (currency.encoded_size() <= Self::max_encoded_len()).then_some(Self {
            currency,
            _marker: PhantomData,
        })
    }
}

/// Represents from/to currency pair.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(
    Decode, TypeInfo, CloneNoBound, PartialEqNoBound, EqNoBound, MaxEncodedLen, DebugNoBound,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxMemberBytesLen))]
pub struct CurrencyPair<From: EncodableString, To: EncodableString, MaxMemberBytesLen: Get<u32>> {
    /// Represents currency being valued.
    from: PairMember<From, MaxMemberBytesLen>,
    /// Used as a unit to express price.
    to: PairMember<To, MaxMemberBytesLen>,
}

impl<From: EncodableString, To: EncodableString, MaxMemberBytesLen: Get<u32>> Encode
    for CurrencyPair<From, To, MaxMemberBytesLen>
{
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        self.from.encode_to(dest);
        self.to.encode_to(dest);
    }
}

impl<From: EncodableString, To: EncodableString, MaxMemberBytesLen: Get<u32>>
    EncodeLike<CurrencyPair<String, String, MaxMemberBytesLen>>
    for CurrencyPair<From, To, MaxMemberBytesLen>
{
}

impl<From: EncodableString, To: EncodableString, MaxMemberBytesLen: Get<u32>>
    CurrencyPair<From, To, MaxMemberBytesLen>
{
    /// Attempts to instantiate new `CurrencyPair` using given from/to currencies.
    /// Returns `None` if the encoded length of either currency exceeds `MaxMemberBytesLen`
    pub fn new(from: From, to: To) -> Option<Self> {
        PairMember::new(from)
            .zip(PairMember::new(to))
            .map(|(from, to)| Self { from, to })
    }

    /// Maps given currency pair over `from` member and attempts to create a new `CurrencyPair`.
    pub fn map_over_from<R: EncodableString, F: FnMut(From) -> R>(
        self,
        mut map: F,
    ) -> Option<CurrencyPair<R, To, MaxMemberBytesLen>> {
        let Self {
            from: PairMember { currency: from, .. },
            to: PairMember { currency: to, .. },
        } = self;

        CurrencyPair::new((map)(from), to)
    }

    /// Maps given currency pair over `to` member and attempts to create a new `CurrencyPair`.
    pub fn map_over_to<R: EncodableString, F: FnMut(To) -> R>(
        self,
        mut map: F,
    ) -> Option<CurrencyPair<From, R, MaxMemberBytesLen>> {
        let Self {
            from: PairMember { currency: from, .. },
            to: PairMember { currency: to, .. },
        } = self;

        CurrencyPair::new(from, (map)(to))
    }
}

impl<S: EncodableString, MaxMemberBytesLen: Get<u32>> CurrencyPair<S, S, MaxMemberBytesLen> {
    /// Maps given currency pair over `from`/`to` members and attempts to create a new `CurrencyPair`.
    pub fn map_pair<R: EncodableString, F: FnMut(S) -> R>(
        self,
        mut map: F,
    ) -> Option<CurrencyPair<R, R, MaxMemberBytesLen>> {
        let Self {
            from: PairMember { currency: from, .. },
            to: PairMember { currency: to, .. },
        } = self;

        CurrencyPair::new((map)(from), (map)(to))
    }
}

impl<FromTy: EncodableString, To: EncodableString, MaxMemberBytesLen: Get<u32>>
    From<(
        PairMember<FromTy, MaxMemberBytesLen>,
        PairMember<To, MaxMemberBytesLen>,
    )> for CurrencyPair<FromTy, To, MaxMemberBytesLen>
{
    fn from(
        (from, to): (
            PairMember<FromTy, MaxMemberBytesLen>,
            PairMember<To, MaxMemberBytesLen>,
        ),
    ) -> Self {
        CurrencyPair { from, to }
    }
}

impl<
        From: EncodableString + Display,
        To: EncodableString + Display,
        MaxMemberBytesLen: Get<u32>,
    > Display for CurrencyPair<From, To, MaxMemberBytesLen>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.from.currency, self.to.currency)
    }
}

/// Represents from/to currency pair built atop of two types returning `&'static str`.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(
    Encode, TypeInfo, CloneNoBound, PartialEqNoBound, EqNoBound, MaxEncodedLen, DebugNoBound,
)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxMemberBytesLen))]
pub struct StaticCurrencyPair<From, To, MaxMemberBytesLen: Get<u32>> {
    pair: CurrencyPair<&'static str, &'static str, MaxMemberBytesLen>,
    _marker: PhantomData<(From, To)>,
}

impl<From, To, MaxMemberBytesLen> StaticCurrencyPair<From, To, MaxMemberBytesLen>
where
    From: Get<&'static str>,
    To: Get<&'static str>,
    MaxMemberBytesLen: Get<u32>,
{
    /// Instantiates new `StaticCurrencyPair` using specified types.
    pub fn new() -> Option<Self> {
        let pair = CurrencyPair::new(From::get(), To::get())?;

        Some(Self {
            pair,
            _marker: PhantomData,
        })
    }

    /// Converts `StaticCurrencyPair` into `CurrencyPair`.
    pub fn into_currency_pair(self) -> CurrencyPair<&'static str, &'static str, MaxMemberBytesLen> {
        self.pair
    }
}

impl<FromTy, To, MaxMemberBytesLen: Get<u32>>
    From<StaticCurrencyPair<FromTy, To, MaxMemberBytesLen>>
    for CurrencyPair<&'static str, &'static str, MaxMemberBytesLen>
{
    fn from(
        StaticCurrencyPair { pair, .. }: StaticCurrencyPair<FromTy, To, MaxMemberBytesLen>,
    ) -> Self {
        pair
    }
}

impl<
        From: EncodableString + Display,
        To: EncodableString + Display,
        MaxMemberBytesLen: Get<u32>,
    > Display for StaticCurrencyPair<From, To, MaxMemberBytesLen>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.pair.from.currency, self.pair.to.currency)
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
    use sp_runtime::parameter_types;

    use super::*;

    #[test]
    fn debug() {
        assert_eq!(
            format!(
                "{}",
                CurrencyPair::<_, _, ConstU32<3>>::new("ABC", "CDE").unwrap()
            ),
            "ABC/CDE"
        );
    }

    #[test]
    fn map() {
        let one_type_pair =
            CurrencyPair::<_, _, ConstU32<2>>::new("AB".to_string(), "BC".to_string()).unwrap();
        let diff_type_pair =
            CurrencyPair::<_, _, ConstU32<1>>::new(A("A".to_owned()), B("B".to_owned())).unwrap();

        assert_eq!(
            one_type_pair
                .map_pair(|mut v| {
                    unsafe { v.as_bytes_mut() }.reverse();
                    v
                })
                .unwrap(),
            CurrencyPair::new("BA".to_string(), "CB".to_string()).unwrap()
        );

        assert_eq!(
            diff_type_pair.clone().map_over_from(|A(a)| a).unwrap(),
            CurrencyPair::<_, _, ConstU32<1>>::new("A".to_owned(), B("B".to_owned())).unwrap()
        );
        assert_eq!(
            diff_type_pair.map_over_to(|B(b)| b).unwrap(),
            CurrencyPair::<_, _, ConstU32<1>>::new(A("A".to_owned()), "B".to_owned()).unwrap()
        );
    }

    #[test]
    fn encoded_size() {
        assert_eq!("🦅".as_bytes().len(), 4);
        assert_eq!(
            PairMember::<_, ConstU32<4>>::new("🦅").unwrap().currency,
            "🦅"
        );
        assert_eq!(PairMember::<_, ConstU32<3>>::new("🦅"), None);
        assert_eq!(PairMember::<_, ConstU32<2>>::new("ABC"), None);
        assert_eq!(PairMember::<_, ConstU32<0>>::new("CDE"), None);
        assert!(PairMember::<_, ConstU32<3>>::new("ABC").is_some());

        assert_eq!(
            PairMember::<_, ConstU32<3>>::new("ABC").unwrap().currency,
            "ABC"
        );

        assert_eq!(CurrencyPair::<_, _, ConstU32<2>>::new("ABC", "CDE"), None);
        assert_eq!(
            CurrencyPair::<_, _, ConstU32<3>>::new("ABC", "CDE"),
            Some(
                (
                    PairMember::<_, ConstU32<3>>::new("ABC").unwrap(),
                    PairMember::<_, ConstU32<3>>::new("CDE").unwrap()
                )
                    .into()
            )
        );
    }

    #[test]
    fn encode_decode() {
        let pair = CurrencyPair::<_, _, ConstU32<3>>::new("ABC", "CDE").unwrap();
        let encoded = pair.encode();
        let decoded: CurrencyPair<String, _, ConstU32<3>> =
            Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded.from, PairMember::new("ABC".to_string()).unwrap());
        assert_eq!(decoded.to, PairMember::new("CDE".to_string()).unwrap());
        assert_ne!(decoded.from, PairMember::new("AB".to_string()).unwrap());
        assert_ne!(decoded.to, PairMember::new("E".to_string()).unwrap());

        assert_eq!(pair.map_pair(ToOwned::to_owned).unwrap(), decoded);
    }

    #[test]
    fn encode_decode_custom_type() {
        impl AsRef<str> for A {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        let pair =
            CurrencyPair::<_, _, ConstU32<3>>::new(A("123".to_string()), A("122".to_string()))
                .unwrap();
        let encoded = pair.encode();
        let decoded: CurrencyPair<_, _, ConstU32<3>> = Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded.from, PairMember::new("123".to_string()).unwrap());
        assert_eq!(decoded.to, PairMember::new("122".to_string()).unwrap());
        assert_ne!(decoded.from, PairMember::new("AB".to_string()).unwrap());
        assert_ne!(decoded.to, PairMember::new("E".to_string()).unwrap());

        assert_eq!(pair.clone().map_pair(|A(val)| val).unwrap(), decoded);
        assert_eq!(pair, decoded.map_pair(A).unwrap());
    }

    #[test]
    fn static_types() {
        parameter_types! {
            pub const DOCK: &'static str = "DOCK";
            pub const USD: &'static str = "USD";
            pub const MaxLen: u32 = 4;
            pub DockUsd: StaticCurrencyPair<DOCK, USD, MaxLen> = StaticCurrencyPair::new().unwrap();
        }

        let dock_pair: CurrencyPair<_, _, MaxLen> = DockUsd::get().into();
        let cur_pair = CurrencyPair::<_, _, MaxLen>::new("DOCK", "USD").unwrap();
        assert_eq!(dock_pair, cur_pair);
    }
}
