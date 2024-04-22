use sp_runtime::DispatchResult;

/// Identity-related operations.
pub trait Identity {
    /// Identity information excluding the verification state.
    type Info: Default;
    /// Single part of the verification.
    type Justification: Default;

    /// Returns `true` if the underlying identity is verified.
    fn verified(&self) -> bool;

    /// Returns underlying identity information.
    fn info(&self) -> Self::Info;

    /// Adds justification for the underlying identity.
    fn verify(&mut self, justification: Self::Justification) -> DispatchResult;
}

/// Provides methods to retrieve an account's identity.
pub trait IdentityProvider<T: frame_system::Config> {
    /// Stored account's identity.
    type Identity: Identity;

    /// Returns identity for the supplied account [if it exists].
    fn identity(who: &T::AccountId) -> Option<Self::Identity>;
}

/// Provides methods to set an account's identity.
pub trait IdentitySetter<T: frame_system::Config>: IdentityProvider<T> {
    /// Attempts to set identity for the account.
    fn set_identity(
        who: T::AccountId,
        identity: <Self::Identity as Identity>::Info,
    ) -> DispatchResult;

    /// Verifies identity for the provided account.
    fn verify_identity(
        who: &T::AccountId,
        justification: <Self::Identity as Identity>::Justification,
    ) -> DispatchResult;

    /// Attempts to remove identity of the account.
    fn remove_identity(who: &T::AccountId) -> DispatchResult;
}
