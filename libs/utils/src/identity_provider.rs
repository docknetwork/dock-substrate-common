use sp_runtime::DispatchResult;

/// Provides methods to retrieve an account's identity.
pub trait IdentityProvider<T: frame_system::Config> {
    /// Stored account's identity.
    type Identity;

    /// Returns identity for the supplied account [if it exists].
    fn identity(who: &T::AccountId) -> Option<Self::Identity>;

    /// Returns `true` if the supplied account has an identity.
    fn has_identity(who: &T::AccountId) -> bool {
        Self::identity(who).is_some()
    }
}

/// Provides methods to set an account's identity.
pub trait IdentitySetter<T: frame_system::Config> {
    /// Idenity information to be associated with the account.
    type IdentityInfo: Default;

    /// Attempts to set identity for the account.
    fn set_identity(account: T::AccountId, identity: Self::IdentityInfo) -> DispatchResult;

    /// Attempts to remove identity of the account.
    fn remove_identity(account: &T::AccountId) -> DispatchResult;
}
