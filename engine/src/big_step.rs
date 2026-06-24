//! # Big-step provider for FuseFlow parallelization
//!
//! This module implements a higher-order step provider that collapses
//! sequential par-factor choices into compound "big steps." Instead of
//! choosing par factors one loop at a time, the user is presented with a
//! triangular set of options: for each remaining loop k, a compound step
//! that defaults loops before k to par_factor=1 and sets loop k to a
//! chosen value.

use crate::core;
use crate::top_down::TopDownStep;
use crate::util;

use pbn::{Step, StepProvider};

pub struct BigStepProvider {
    inner: Box<dyn StepProvider<util::Timer, Step = core::Step> + 'static>,
}

impl BigStepProvider {
    pub fn new(
        inner: impl StepProvider<util::Timer, Step = core::Step> + 'static,
    ) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    fn is_default_par_step(step: &core::Step) -> bool {
        match step {
            TopDownStep::Extend(_, f, _) => {
                f.name.0 == "choose_default_par_factor"
            }
            _ => false,
        }
    }

    fn is_non_default_par_step(step: &core::Step) -> bool {
        match step {
            TopDownStep::Extend(_, f, _) => {
                f.name.0.starts_with("choose_par_factor_")
            }
            _ => false,
        }
    }

    fn is_par_factor_step(step: &core::Step) -> bool {
        Self::is_default_par_step(step) || Self::is_non_default_par_step(step)
    }

    fn is_advance_step(step: &core::Step) -> bool {
        match step {
            TopDownStep::Extend(_, f, _) => {
                f.name.0.starts_with("advance_par_factor_level_")
            }
            _ => false,
        }
    }

    fn build_compound_steps(
        &mut self,
        timer: &util::Timer,
        exp: &core::Exp,
        steps: Vec<core::Step>,
    ) -> Result<Vec<core::Step>, util::EarlyCutoff> {
        let mut result: Vec<core::Step> = Vec::new();

        result.extend(
            steps
                .iter()
                .filter(|s| Self::is_non_default_par_step(s))
                .cloned(),
        );

        let default_step =
            match steps.iter().find(|s| Self::is_default_par_step(s)) {
                Some(s) => s.clone(),
                None => return Ok(result),
            };

        let mut sim_exp = exp.clone();
        let mut prefix_steps: Vec<core::Step> = Vec::new();
        prefix_steps.push(default_step);

        loop {
            // Apply all prefix steps to get the simulated expression
            let mut e = exp.clone();
            for s in &prefix_steps {
                e = match s.apply(&e) {
                    Some(e) => e,
                    None => return Ok(result),
                };
            }
            sim_exp = e;

            let sim_steps = self.inner.provide(timer, &sim_exp)?;
            let advance =
                match sim_steps.iter().find(|s| Self::is_advance_step(s)) {
                    Some(a) => a.clone(),
                    None => break,
                };

            sim_exp = match advance.apply(&sim_exp) {
                Some(e) => e,
                None => break,
            };
            prefix_steps.push(advance);

            let next_steps = self.inner.provide(timer, &sim_exp)?;
            let next_default = next_steps
                .iter()
                .find(|s| Self::is_default_par_step(s))
                .cloned();

            let prefix = prefix_steps
                .iter()
                .cloned()
                .reduce(|a, b| TopDownStep::Seq(Box::new(a), Box::new(b)))
                .unwrap();

            for step in next_steps
                .iter()
                .filter(|s| Self::is_non_default_par_step(s))
            {
                result.push(TopDownStep::Seq(
                    Box::new(prefix.clone()),
                    Box::new(step.clone()),
                ));
            }

            match next_default {
                Some(d) => prefix_steps.push(d),
                None => break,
            }
        }

        Ok(result)
    }
}

impl StepProvider<util::Timer> for BigStepProvider {
    type Step = core::Step;

    fn provide(
        &mut self,
        timer: &util::Timer,
        e: &core::Exp,
    ) -> Result<Vec<Self::Step>, util::EarlyCutoff> {
        let steps = self.inner.provide(timer, e)?;

        if steps.iter().any(|s| Self::is_par_factor_step(s)) {
            self.build_compound_steps(timer, e, steps)
        } else {
            Ok(steps)
        }
    }
}
