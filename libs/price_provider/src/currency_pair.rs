#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, EncodeLike, Output};
use scale_info::{prelude::string::String, TypeInfo};
use sp_std::prelude::*;

/// Represents from/to currency pair.
/// Used to express price relationship between two currencies.
/// Given some from/to pair price `N` should be considered as `1 x from = N x to`.
#[derive(Decode, TypeInfo, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CurrencyPair<S> {
    /// Represents currency being valued.
    from: S,
    /// Used as a unit to express price.
    to: S,
}

impl<S: AsRef<str>> Encode for CurrencyPair<S> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        self.from.as_ref().as_bytes().encode_to(dest);
        self.to.as_ref().as_bytes().encode_to(dest);
    }
}

impl<S: AsRef<str>> EncodeLike<CurrencyPair<String>> for CurrencyPair<S> {}

impl<S: AsRef<str>> CurrencyPair<S> {
    /// Instantiates new `CurrencyPair` using given from/to currencies.
    pub fn new(from: S, to: S) -> Self {
        Self { from, to }
    }

    /// Maps given currency pair over `from`/`to` members.
    pub fn map_pair<R: AsRef<str>, F: FnMut(S) -> R>(self, mut map: F) -> CurrencyPair<R> {
        CurrencyPair::new((map)(self.from), (map)(self.to))
    }
}

impl<'a> CurrencyPair<&'a str> {
    /// Instantiates new `CurrencyPair` using given from/to currencies.
    pub const fn new_const(from: &'a str, to: &'a str) -> Self {
        Self { from, to }
    }
}

impl<S: AsRef<str>> From<(S, S)> for CurrencyPair<S> {
    fn from((from, to): (S, S)) -> CurrencyPair<S> {
        CurrencyPair::new(from, to)
    }
}

impl<S: AsRef<str>> core::fmt::Debug for CurrencyPair<S> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}/{}", self.from.as_ref(), self.to.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let pair = CurrencyPair::new("ABC", "CDE");
        let encoded = pair.encode();
        let decoded = Decode::decode(&mut &encoded[..]).unwrap();

        assert_eq!(pair.map_pair(ToOwned::to_owned), decoded);

        struct A(String);
        impl AsRef<str> for A {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        let pair = CurrencyPair::new(A("123".to_string()), A("122".to_string()));
        let encoded = pair.encode();
        let decoded = Decode::decode(&mut &encoded[..]).unwrap();

        assert_eq!(pair.map_pair(|A(val)| val), decoded);
    }
}
