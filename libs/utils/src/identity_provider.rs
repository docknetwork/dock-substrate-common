use sp_runtime::DispatchResult;

/// Provides methods to work with an account's identity.
pub trait IdentityProvider<T: frame_system::Config> {
    /// Stored account's identity.
    type Identity: Default;

    /// Returns identity for the supplied account [if it exists].
    fn identity(who: &T::AccountId) -> Option<Self::Identity>;

    /// Returns `true` if the supplied account has an identity.
    fn has_identity(who: &T::AccountId) -> bool {
        Self::identity(who).is_some()
    }

    /// Attempts to set identity for the account.
    fn set_identity(account: T::AccountId, identity: Self::Identity) -> DispatchResult;

    /// Attempts to remove identity of the account.
    fn remove_identity(account: &T::AccountId) -> DispatchResult;
}
