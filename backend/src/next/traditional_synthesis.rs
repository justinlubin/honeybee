use crate::next::timer::Timer;

/// The type of traditional synthesizers.
pub trait TraditionalSynthesizer {
    type Exp;
    fn provide_any<E>(
        &self,
        timer: &impl Timer<E>,
    ) -> Result<Option<Self::Exp>, E>;
}
