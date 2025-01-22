use instant::Duration;
use instant::Instant;

pub trait Timer<E> {
    fn tick(&self) -> Result<(), E>;
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

impl<E: Clone> Timer<E> for FiniteTimer<E> {
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

impl<E> Timer<E> for InfiniteTimer {
    fn tick(&self) -> Result<(), E> {
        Ok(())
    }
}
