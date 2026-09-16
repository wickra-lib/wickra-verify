# Wickra Verify — C / C++ examples

The Wickra Verify C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_verify.h`](../../bindings/c/include/wickra_verify.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_verify.hpp`](../../bindings/c/include/wickra_verify.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-verify-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_verify.so`     | `-lwickra_verify` |
| macOS    | `libwickra_verify.dylib`  | `-lwickra_verify` |
| Windows (MSVC) | `wickra_verify.dll` | `wickra_verify.dll.lib` (import lib) |

A static library (`libwickra_verify.a` / `wickra_verify.lib`) is emitted alongside.

## Build and run the examples

With CMake, as the CI C ABI job does:

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

## The examples

| Example | What it does |
|---------|--------------|
| `verify.c` | A minimal C example: submit a claim to the wickra-verify C ABI whose report |
| `verify.cpp` | A minimal C++ example: submit a claim to the wickra-verify C ABI whose report has been doctored, and assert that verification refutes it. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_verify.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
