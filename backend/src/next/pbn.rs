//! # Programming By Navigation
//!
//! This module defines all the necessary high-level components of Programming
//! By Navigation. In particular, it defines the interface that is necessary for
//! the Programming By Navigation interaction and guarantees.

use crate::next::timer::Timer;

/// The type of steps.
///
/// Steps transform one expression into another and must satisfy the
/// *navigation relation* properties.
pub trait Step {
    type Exp;
    fn step(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

/// The type of step providers.
///
/// To be a valid solution to the Programming By Navigation Synthesis Problem,
/// step providers must satisfy the *validity*, *strong completeness*, and
/// *strong soundness* conditions. Thus, step providers implicitly rely on a
/// notion of validity.
pub trait StepProvider {
    type Step: Step;
    fn provide<E>(
        &mut self,
        e: &<Self::Step as Step>::Exp,
        timer: &impl Timer<E>,
    ) -> Result<Vec<Self::Step>, E>;
}

/// The components of a Programming By Navigation interaction.
///
/// To provide this interface, an interaction implementation will likely need
/// to keep track of the "state" of an interaction in the form of the working
/// expression.
pub trait Interaction {
    type Step: Step;

    /// Called at the start of an interaction.
    fn init(&self, start: &<Self::Step as Step>::Exp) -> bool;
    /// Called to get the set of possible next steps.
    fn provide(&self) -> Vec<<Self::Step as Step>::Exp>;
    /// Called to choose among the above provided steps.
    ///
    /// Panics if provided a step that is not returned by a call to [`provide`].
    fn decide(&self, step: &Self::Step);
    /// Return the working expression (e.g. for visualization).
    fn working_expression(&self) -> <Self::Step as Step>::Exp;
    /// Returns whether or not the current working expression is valid.
    fn valid(&self) -> bool;
}
