//! # Programming By Navigation
//!
//! This module defines all the necessary high-level components of Programming
//! By Navigation. In particular, it defines the interface that is necessary for
//! the Programming By Navigation interaction and guarantees.

use crate::util::{EarlyCutoff, Timer};

/// The type of steps.
///
/// Steps transform one expression into another and must satisfy the
/// *navigation relation* properties.
pub trait Step {
    type Exp: Clone;
    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

pub trait ValidityChecker {
    type Exp;
    fn check(&self, e: &Self::Exp) -> bool;
}

/// The type of step providers.
///
/// To be a valid solution to the Programming By Navigation Synthesis Problem,
/// step providers must satisfy the *validity*, *strong completeness*, and
/// *strong soundness* conditions.
pub trait StepProvider {
    type Step: Step;
    fn provide(
        &mut self,
        timer: &Timer,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff>;
}

/// A Programming By Navigation controller. Controllers abstract away the
/// actual step provider (and validity checker) and can be used to engage in
/// the Programming By Navigation interactive process in a way that is
/// abstracted from the underlying synthesis algorithm.
pub struct Controller<S: Step> {
    timer: Timer,
    provider: Box<dyn StepProvider<Step = S> + 'static>,
    checker: Box<dyn ValidityChecker<Exp = S::Exp> + 'static>,
    state: S::Exp,
}

impl<S: Step> Controller<S> {
    /// Create a new controller
    pub fn new(
        timer: Timer,
        provider: impl StepProvider<Step = S> + 'static,
        checker: impl ValidityChecker<Exp = S::Exp> + 'static,
        start: S::Exp,
    ) -> Self {
        Self {
            timer,
            provider: Box::new(provider),
            checker: Box::new(checker),
            state: start,
        }
    }

    /// Ask the synthesizer to provide a list of possible next steps (all and
    /// only the valid ones)
    pub fn provide(&mut self) -> Result<Vec<S>, EarlyCutoff> {
        self.provider.provide(&self.timer, &self.state)
    }

    /// Decide which step to take - must be selected from among the ones that
    /// are provided by the [`provide`] function
    pub fn decide(&mut self, step: S) {
        self.state = step.apply(&self.state).unwrap();
    }

    /// Returns the current working expression
    pub fn working_expression(&self) -> S::Exp {
        self.state.clone()
    }

    /// Returns whether or not the current working expression is valid
    pub fn valid(&self) -> bool {
        self.checker.check(&self.state)
    }
}
