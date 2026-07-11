"""Wickra Verify — recompute a claimed backtest report and confirm or refute it.

Create a :class:`Verifier`, drive it with command JSONs (``verify``, ``explain``,
``canonicalize``, ``version``) and read back response JSONs. The same command
protocol crosses every language binding, so this Python front-end drives the
exact same core as the native CLI.
"""

from ._wickra_verify import Verifier, __version__

__all__ = ["Verifier", "__version__"]
