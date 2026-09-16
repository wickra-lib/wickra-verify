# Wickra Verify examples — C#

Runnable C# examples for the [Wickra Verify C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-verify-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Verify
```

## The examples

| Example | What it does |
|---------|--------------|
| `Verify/Program.cs` | A runnable .NET example: submit a claim whose report has been doctored and assert the binding refutes it. |
