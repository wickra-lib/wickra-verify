# WickraVerify — .NET binding

The wickra-verify core for .NET, over the C ABI via P/Invoke. The native library ships
inside the NuGet package for every supported runtime identifier, so there is
nothing to install alongside it.

## Install

```sh
dotnet add package WickraVerify
```

## Use

The binding is a thin, faithful surface over the same command boundary every
other binding drives, so a request built here produces the same canonical bytes
it would in Rust, Python or Go.

```csharp
using WickraVerify;

using var handle = new Verifier();
string response = handle.Command("""{"cmd":"version"}""");
Console.WriteLine(response);
```

`Verifier` owns a native handle and implements `IDisposable`; the `using` above is
what releases it. Dropping the reference without disposing leaks the handle
until the finalizer runs.

## What travels with the package

The managed assembly, the native C ABI library for each supported runtime
identifier, and both licence texts. The package declares `MIT OR Apache-2.0`
and carries `LICENSE-MIT` and `LICENSE-APACHE` to match.

## Building from source

Requires the .NET SDK and a Rust toolchain:

```sh
cargo build -p wickra-verify-c --release
dotnet test bindings/csharp
```

The test project resolves the freshly built native library from `target/release`.
