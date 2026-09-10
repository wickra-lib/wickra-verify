---
name: Bug report (Detailed)
about: Long-form bug report with environment matrix, minimal reproducer, and expected-vs-actual sections.
title: "[Bug] <short description>"
labels: ["bug", "triage"]
assignees: []
---

## Summary

<!-- One or two sentences. What did you expect, what happened instead? -->

## Affected binding

- [ ] Rust crate (`wickra-verify-core`)
- [ ] Python (`pip install wickra-verify`)
- [ ] Node.js (`npm install wickra-verify`)
- [ ] WASM
- [ ] C ABI (`bindings/c`)
- [ ] C# (`WickraVerify` on NuGet)
- [ ] Go (`bindings/go`)
- [ ] Java (`org.wickra:wickra-verify` on Maven Central)
- [ ] R (`bindings/r`)
- [ ] CLI (`wickra-verify`)
- [ ] Docs / examples only

## Environment

| Field                | Value                                  |
| -------------------- | -------------------------------------- |
| Wickra Verify version       | `e.g. 0.4.2`                           |
| Binding version      | `e.g. python 0.4.2 / node 0.4.2`       |
| OS / arch            | `e.g. Windows 11 x86_64, Linux glibc`  |
| Rust toolchain       | `rustc --version` (If building from source) |
| Python / Node.js / .NET version | `python --version` / `node --version` / `dotnet --version` |

## Minimal reproducer

<!--
Paste the smallest possible code snippet that triggers the bug.
If the input data matters, attach a CSV/JSON or paste a few rows inline.
-->

```python
# or rust / js
import wickra_verify as ta
...
```

## Actual output

```
<paste stack trace, panic, wrong values, etc.>
```

## Expected output

<!-- What should the indicator / API have returned? Reference a paper, TA-Lib, or another implementation if possible. -->

## Additional context

<!-- Logs, screenshots, links to related issues, anything else useful. -->
