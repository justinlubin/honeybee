class Controller:
    """
    A Programming by Navigation "controller" to manage a Programming by
    Navigation interactive session.

    This class would typically be called in a loop, oscillating between calls
    to `provide` (the step provider) and `decide` (the step decider).
    """
    def __init__(
        self,
        library: str,
        program: str,
        algorithm: str = "PBNHoneybee",
    ) -> None: ...
    def decide(self, /, index: int) -> None:
        """
        Select between the provided steps.
        *Must* be between `0` and `len(self.provide())`.
        """

    def provide(self, /) -> dict:
        """
        Request the set of possible next steps to take.
        """

    def working_expression(self, /) -> dict:
        """
        Request the current working expression (updated on each call to
        `decide`).
        """
