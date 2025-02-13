use indexmap::IndexMap;
use instant::Duration;
use instant::Instant;

////////////////////////////////////////////////////////////////////////////////
// Timer

#[derive(Debug)]
enum TimerInner {
    Finite { end: Instant },
    Infinite,
}

#[derive(Debug)]
pub struct TimerExpired;

#[derive(Debug)]
pub struct Timer(TimerInner);

impl Timer {
    pub fn finite(duration: Duration) -> Self {
        Timer(TimerInner::Finite {
            end: Instant::now() + duration,
        })
    }

    pub fn infinite() -> Self {
        Timer(TimerInner::Infinite)
    }

    pub fn tick(&self) -> Result<(), TimerExpired> {
        match self.0 {
            TimerInner::Finite { end } => {
                if Instant::now() > end {
                    Err(TimerExpired)
                } else {
                    Ok(())
                }
            }
            TimerInner::Infinite => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utilities

pub fn cartesian_product<K: Clone + Eq + std::hash::Hash, V: Clone>(
    timer: &Timer,
    choices: IndexMap<K, Vec<V>>,
) -> Result<Vec<IndexMap<K, V>>, TimerExpired> {
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
