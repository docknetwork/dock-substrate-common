use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
};
use frame_support::{
    dispatch::DispatchError, traits::Get, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;

/// String limited by the max encoded byte size.
#[derive(CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(transparent))]
pub struct BoundedString<MaxBytesLen: Get<u32>, S: LikeString = String>(
    S,
    #[cfg_attr(feature = "std", serde(skip))] PhantomData<MaxBytesLen>,
);

/// Errors happening on `String` -> `BoundedString` conversion.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoundedStringConversionError {
    /// The string byte size exceeds max allowed.
    InvalidStringByteLen,
}

impl<MaxBytesLen: Get<u32>, S: LikeString> BoundedString<MaxBytesLen, S> {
    /// Instantiates `Self` if encoded byte size of the provided `S` doesn't exceed `MaxBytesLen`.
    pub fn new(str: S) -> Result<Self, BoundedStringConversionError> {
        (str.encoded_size() <= Self::max_encoded_len())
            .then_some(Self(str, PhantomData))
            .ok_or(BoundedStringConversionError::InvalidStringByteLen)
    }

    /// Consumes self and returns underlying `S` value.
    pub fn into_inner(self) -> S {
        self.0
    }

    /// Maps underlying value producing new `BoundedString` carrying result type.
    pub fn map<F, R>(
        self,
        f: F,
    ) -> Result<BoundedString<MaxBytesLen, R>, BoundedStringConversionError>
    where
        R: LikeString,
        F: FnOnce(S) -> R,
    {
        BoundedString::new(f(self.into_inner()))
    }

    /// Attempts to map underlying value producing new `BoundedString` carrying result type.
    pub fn translate<F, R, E>(self, f: F) -> Result<BoundedString<MaxBytesLen, R>, E>
    where
        R: LikeString,
        F: FnOnce(S) -> Result<R, E>,
        E: From<BoundedStringConversionError>,
    {
        let str = f(self.into_inner())?;

        BoundedString::new(str).map_err(Into::into)
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString> Deref for BoundedString<MaxBytesLen, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + Default> Default for BoundedString<MaxBytesLen, S> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<MaxBytesLen: Get<u32>> TryFrom<String> for BoundedString<MaxBytesLen, String> {
    type Error = BoundedStringConversionError;

    fn try_from(str: String) -> Result<Self, Self::Error> {
        BoundedString::new(str)
    }
}

impl<MaxBytesLen: Get<u32>> From<BoundedString<MaxBytesLen, String>> for String {
    fn from(BoundedString(str, _): BoundedString<MaxBytesLen, String>) -> Self {
        str
    }
}

impl<'a, MaxBytesLen: Get<u32>> From<BoundedString<MaxBytesLen, &'a str>> for &'a str {
    fn from(BoundedString(str, _): BoundedString<MaxBytesLen, &'a str>) -> Self {
        str
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + PartialOrd> PartialOrd
    for BoundedString<MaxBytesLen, S>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + Ord> Ord for BoundedString<MaxBytesLen, S> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl From<BoundedStringConversionError> for &'static str {
    fn from(
        BoundedStringConversionError::InvalidStringByteLen: BoundedStringConversionError,
    ) -> Self {
        "The string byte size exceeds max allowed"
    }
}

impl Display for BoundedStringConversionError {
    fn fmt(
        &self,
        f: &mut scale_info::prelude::fmt::Formatter<'_>,
    ) -> scale_info::prelude::fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}

impl From<BoundedStringConversionError> for DispatchError {
    fn from(
        BoundedStringConversionError::InvalidStringByteLen: BoundedStringConversionError,
    ) -> Self {
        DispatchError::Other(BoundedStringConversionError::InvalidStringByteLen.into())
    }
}

impl From<BoundedStringConversionError> for codec::Error {
    fn from(
        BoundedStringConversionError::InvalidStringByteLen: BoundedStringConversionError,
    ) -> Self {
        <&'static str>::from(BoundedStringConversionError::InvalidStringByteLen).into()
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString> EncodeLike<String> for BoundedString<MaxBytesLen, S> {}

impl<MaxBytesLen, S: LikeString> Decode for BoundedString<MaxBytesLen, S>
where
    S: LikeString + Decode,
    MaxBytesLen: Get<u32>,
{
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        S::decode(input).and_then(|decoded| Self::new(decoded).map_err(Into::into))
    }
}

#[cfg(feature = "std")]
impl<'de, MaxBytesLen, S: LikeString> Deserialize<'de> for BoundedString<MaxBytesLen, S>
where
    S: LikeString + Deserialize<'de>,
    MaxBytesLen: Get<u32>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = S::deserialize(deserializer)?;

        Self::new(str).map_err(serde::de::Error::custom)
    }
}

impl<MaxBytesLen, S: LikeString> Encode for BoundedString<MaxBytesLen, S>
where
    S: LikeString + Encode,
    MaxBytesLen: Get<u32>,
{
    fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }
}

/// There's a bug with `BoundedString` in substrate metadata generation.
impl<MaxBytesLen: Get<u32> + 'static, S: LikeString + 'static> scale_info::TypeInfo
    for BoundedString<MaxBytesLen, S>
{
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("BoundedString", "BoundedString"))
            .composite(scale_info::build::Fields::unnamed().field(|f| f.ty::<S>()))
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString> MaxEncodedLen for BoundedString<MaxBytesLen, S> {
    fn max_encoded_len() -> usize {
        codec::Compact(MaxBytesLen::get())
            .encoded_size()
            .saturating_add(MaxBytesLen::get() as usize)
    }
}

/// Denotes a type which implements `EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo`
pub trait LikeString: EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo {}
impl<T: EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo> LikeString for T {}

#[cfg(test)]
mod tests {
    use codec::{Decode, Encode};
    use sp_runtime::traits::ConstU32;

    use crate::{bounded_string::BoundedString, BoundedStringConversionError};

    #[cfg(feature = "std")]
    #[test]
    fn serde() {
        use serde_json;

        let serialized =
            serde_json::to_string(&BoundedString::<ConstU32<10>, _>::new("ABC").unwrap()).unwrap();
        assert_eq!(serialized, "\"ABC\"");
        assert_eq!(serde_json::to_string(&"abc").unwrap(), "\"abc\"");

        let deserialized: BoundedString<ConstU32<3>> = serde_json::from_str(&"\"CDE\"").unwrap();
        assert_eq!(
            deserialized,
            BoundedString::<ConstU32<3>, _>::new("CDE")
                .unwrap()
                .map(ToString::to_string)
                .unwrap()
        );

        assert_eq!(
            serde_json::from_str::<'_, BoundedString<ConstU32<2>>>(&"\"CDE\"")
                .unwrap_err()
                .to_string(),
            <serde_json::Error as serde::de::Error>::custom(
                BoundedStringConversionError::InvalidStringByteLen
            )
            .to_string()
        );
    }

    #[test]
    fn workflow() {
        assert_eq!(
            BoundedString::<ConstU32<10>, _>::new("ABCDE")
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<10>>::new("ABCDE".to_string())
                .unwrap()
                .encode()
                .len()
        );
        assert!(BoundedString::<ConstU32<3>, _>::new("ABDE").is_err());
        assert!(BoundedString::<ConstU32<4>, _>::new("ABDE").is_ok());
        assert_eq!(
            BoundedString::<ConstU32<100>>::new("ABCDE".to_string())
                .unwrap()
                .encoded_size(),
            BoundedString::<ConstU32<100>>::new("ABCDE".to_string())
                .unwrap()
                .encode()
                .len()
        );

        assert!(BoundedString::<ConstU32<4>>::decode(
            &mut &BoundedString::<ConstU32<10>, _>::new("ABCDE")
                .unwrap()
                .encode()[..]
        )
        .is_err());

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
    }
}
