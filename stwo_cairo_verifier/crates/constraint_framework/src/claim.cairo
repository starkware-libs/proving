use stwo_verifier_core::channel::Channel;
use crate::RelationUsesDict;

/// Trait that defines the functionality required by a "claim",
/// where a "claim" is an object that holds public information about
/// one or multiple components whose trace needs to be verified.
pub trait ClaimTrait<T> {
    /// Mix this claim’s public data into the verification transcript (`channel`),
    /// ensuring it influences all subsequently derived challenges.
    fn mix_into(self: @T, ref channel: Channel);
    /// Record the lookups used by the components associated with the claim.
    fn accumulate_relation_uses(self: @T, ref relation_uses: RelationUsesDict);
}
