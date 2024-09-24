pub enum Mode<'a> {
    AnyValid,
    AllValid,
    AnySimplyTyped,
    AllSimplyTyped,
    Particular(&'a derivation::Tree),
}
