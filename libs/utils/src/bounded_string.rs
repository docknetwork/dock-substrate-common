use core::{fmt::Debug, marker::PhantomData};
use frame_support::{traits::Get, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::{prelude::string::String, TypeInfo};
use sp_runtime::DispatchError;

/// String limited by the max encoded byte size.
#[derive(Encode, CloneNoBound, PartialEqNoBound, EqNoBound, DebugNoBound)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo)]
#[scale_info(skip_type_params(MaxBytesLen))]
pub struct BoundedString<MaxBytesLen: Get<u32>, S: LikeString = String> {
    str: S,
    #[codec(skip)]
    #[cfg_attr(feature = "std", serde(skip))]
    _marker: PhantomData<MaxBytesLen>,
}

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
            .then_some(Self {
                str,
                _marker: PhantomData,
            })
            .ok_or(BoundedStringConversionError::InvalidStringByteLen)
    }

    /// Consumes self and returns underlying `S` value.
    pub fn into_inner(self) -> S {
        self.str
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + Default> Default for BoundedString<MaxBytesLen, S> {
    fn default() -> Self {
        Self {
            str: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<MaxBytesLen: Get<u32>> TryFrom<String> for BoundedString<MaxBytesLen, String> {
    type Error = BoundedStringConversionError;

    fn try_from(str: String) -> Result<Self, Self::Error> {
        BoundedString::new(str)
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + PartialOrd> PartialOrd
    for BoundedString<MaxBytesLen, S>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.str.partial_cmp(&other.str)
    }
}

impl<MaxBytesLen: Get<u32>, S: LikeString + Ord> Ord for BoundedString<MaxBytesLen, S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.str.cmp(&other.str)
    }
}

impl From<BoundedStringConversionError> for &'static str {
    fn from(
        BoundedStringConversionError::InvalidStringByteLen: BoundedStringConversionError,
    ) -> Self {
        "The string byte size exceeds max allowed"
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

impl<MaxBytesLen: Get<u32>, S: LikeString> MaxEncodedLen for BoundedString<MaxBytesLen, S> {
    fn max_encoded_len() -> usize {
        codec::Compact(MaxBytesLen::get())
            .encoded_size()
            .saturating_add(MaxBytesLen::get() as usize)
    }
}

/// Denotes a type which implements `EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo`
pub trait LikeString:
    EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo + 'static
{
}
impl<T: EncodeLike<String> + Eq + PartialEq + Clone + Debug + TypeInfo + 'static> LikeString for T {}

#[cfg(test)]
mod tests {
    use codec::{Decode, Encode};
    use sp_runtime::traits::ConstU32;

    use crate::{bounded_string::BoundedString, BoundedStringConversionError};

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
