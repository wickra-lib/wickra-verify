"""Type stubs for the wickra_verify package."""

__version__: str

class Verifier:
    """A verifier driven by JSON commands."""

    def __init__(self, config_json: str = "{}") -> None:
        """Create a verifier from a config JSON (``{"atol":..,"rtol":..}``).

        Missing fields fall back to the defaults; ``"{}"`` uses the default
        tolerances. Raises ``ValueError`` if the config JSON cannot be parsed.
        """
        ...

    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the resulting response JSON.

        Raises ``ValueError`` if the command envelope cannot be parsed.
        """
        ...

    @staticmethod
    def version() -> str:
        """The library version."""
        ...
