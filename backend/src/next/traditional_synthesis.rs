use crate::next::timer::Timer;

/// The type of traditional synthesizers.
pub trait TraditionalSynthesizer {
    type Spec;
    type Exp;
    fn provide_any<E>(
        &self,
        spec: Self::Spec,
        timer: &impl Timer<E>,
    ) -> Result<Option<Self::Exp>, E>;
}
