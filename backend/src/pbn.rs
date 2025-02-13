//! # Programming By Navigation
//!
//! This module defines all the necessary high-level components of Programming
//! By Navigation. In particular, it defines the interface that is necessary for
//! the Programming By Navigation interaction and guarantees.

use crate::util::Timer;

/// The type of steps.
///
/// Steps transform one expression into another and must satisfy the
/// *navigation relation* properties.
pub trait Step {
    type Exp: Clone;
    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

/// The type of step providers.
///
/// To be a valid solution to the Programming By Navigation Synthesis Problem,
/// step providers must satisfy the *validity*, *strong completeness*, and
/// *strong soundness* conditions.
pub trait StepProvider<T: Timer> {
    type Step: Step;
    fn provide(
        &mut self,
        timer: &T,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, T::Expired>;

    // fn valid(&mut self, e: &<Self::Step as Step>::Exp) -> bool;
}

pub trait ValidityChecker {
    type Exp;
    fn check(&self, e: &Self::Exp) -> bool;
}

pub struct Controller<T: Timer, S: Step> {
    timer: T,
    provider: Box<dyn StepProvider<T, Step = S> + 'static>,
    checker: Box<dyn ValidityChecker<Exp = S::Exp> + 'static>,
    state: S::Exp,
}

impl<T: Timer, S: Step> Controller<T, S> {
    pub fn new(
        timer: T,
        provider: impl StepProvider<T, Step = S> + 'static,
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

    pub fn provide(&mut self) -> Result<Vec<S>, T::Expired> {
        self.provider.provide(&self.timer, &self.state)
    }

    pub fn decide(&mut self, step: S) {
        self.state = step.apply(&self.state).unwrap();
    }

    pub fn working_expression(&self) -> S::Exp {
        self.state.clone()
    }

    pub fn valid(&self) -> bool {
        self.checker.check(&self.state)
    }
}
