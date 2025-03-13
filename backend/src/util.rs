//! # Utilities

use indexmap::IndexMap;
use instant::Duration;
use instant::Instant;

////////////////////////////////////////////////////////////////////////////////
// Early cutoff

/// The type of reasons that a computation may have been cut off early.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EarlyCutoff {
    TimerExpired,
    OutOfMemory,
}

/// The max number of steps to take to avoid getting an actual out of memory
/// (or stack overflow) error.
pub const MAX_STEPS: usize = 2500;

////////////////////////////////////////////////////////////////////////////////
// Timer

#[derive(Debug)]
enum TimerInner {
    Finite { end: Instant },
    Infinite,
}

/// The type of timers; these can be used to cut off a computation early based
/// on a timeout. These are used cooperatively, and [`Timer::tick`] must be
/// called frequently enough so that there is a chance to interrupt the
/// computation.
#[derive(Debug)]
pub struct Timer(TimerInner);

impl Timer {
    /// A finite-duration timer.
    pub fn finite(duration: Duration) -> Self {
        Timer(TimerInner::Finite {
            end: Instant::now() + duration,
        })
    }

    /// An infinite-duration timer (will never cut off the computation).
    pub fn infinite() -> Self {
        Timer(TimerInner::Infinite)
    }

    /// Tick the timer (cooperatively check to see if the computation needs to
    /// stop).
    pub fn tick(&self) -> Result<(), EarlyCutoff> {
        match self.0 {
            TimerInner::Finite { end } => {
                if Instant::now() > end {
                    Err(EarlyCutoff::TimerExpired)
                } else {
                    Ok(())
                }
            }
            TimerInner::Infinite => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Small utilities

/// Take the cartesion product of a set of chocies
pub fn cartesian_product<K: Clone + Eq + std::hash::Hash, V: Clone>(
    timer: &Timer,
    choices: IndexMap<K, Vec<V>>,
) -> Result<Vec<IndexMap<K, V>>, EarlyCutoff> {
    let mut results = vec![IndexMap::new()];
    for (k, vs) in choices.iter() {
        let mut new_results = vec![];
        for map in results {
            timer.tick()?;
            for v in vs {
                let mut new_map = map.clone();
                new_map.insert(k.clone(), v.clone());
                new_results.push(new_map)
            }
        }
        results = new_results;
    }
    Ok(results)
}

/// Subscript numbers in a string
pub fn subscript_numbers(s: &str) -> String {
    s.chars()
        .map(|digit| match digit {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            _ => digit,
        })
        .collect()
}
