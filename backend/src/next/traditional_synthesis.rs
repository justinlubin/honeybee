/// The possible reasons a traditional synthesizer may fail.
pub enum TraditionalSynthesizerError {
    Timeout,
    Unsat,
}

/// The type of traditional synthesizers.
pub trait TraditionalSynthesizer {
    type Spec;
    type Exp;
    fn provide_any(&self, spec: Spec, timeout: u128) -> Option<Option<Exp>>;
}
