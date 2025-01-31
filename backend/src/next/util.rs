use indexmap::IndexMap;
use instant::Duration;
use instant::Instant;

////////////////////////////////////////////////////////////////////////////////
// Void

pub enum Void {}

////////////////////////////////////////////////////////////////////////////////
// Timer

pub trait Timer {
    type Expired;
    fn tick(&self) -> Result<(), Self::Expired>;
}

pub struct FiniteTimer<E> {
    end: Instant,
    error: E,
}

impl<E: Clone> FiniteTimer<E> {
    pub fn new(duration: Duration, error: E) -> Self {
        FiniteTimer {
            end: Instant::now() + duration,
            error,
        }
    }
}

impl<E: Clone> Timer for FiniteTimer<E> {
    type Expired = E;
    fn tick(&self) -> Result<(), E> {
        if Instant::now() > self.end {
            return Err(self.error.clone());
        }
        return Ok(());
    }
}

pub struct InfiniteTimer {}

impl InfiniteTimer {
    pub fn new() -> Self {
        InfiniteTimer {}
    }
}

impl Timer for InfiniteTimer {
    type Expired = Void;
    fn tick(&self) -> Result<(), Void> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utilities

pub fn cartesian_product<
    T: Timer,
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
>(
    timer: &T,
    choices: IndexMap<K, Vec<V>>,
) -> Result<Vec<IndexMap<K, V>>, T::Expired> {
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
