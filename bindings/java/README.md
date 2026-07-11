# Wickra Verify — Java

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, on the JVM over the Wickra C ABI via the Foreign Function
& Memory API (FFM/Panama, JDK 22+). A doctored `claimed_report` cannot pass,
because verification recomputes rather than trusting the supplied numbers.

## Usage

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```java
import org.wickra.verify.Verifier;

try (Verifier verifier = new Verifier()) {
    String claim = "{"
        + "\"strategy\":" + strategySpec + ","          // a wickra-backtest StrategySpec
        + "\"dataset_ref\":{\"kind\":\"inline\",\"data\":" + data + "},"
        + "\"claimed_report\":" + report                // the report being checked (untrusted)
        + "}";
    String verdict = verifier.command("{\"cmd\":\"verify\",\"claim\":" + claim + "}");
    System.out.println(verdict); // the full Verdict as JSON
}
```

FFM needs native access enabled at runtime:

```sh
java --enable-native-access=ALL-UNNAMED ...
```

The native library is located via the `native.lib.dir` system property (the
Cargo `target/` directory in dev/CI), or the platform library path.

## Commands

| `cmd`          | Payload            | Response                                |
|----------------|--------------------|-----------------------------------------|
| `verify`       | `{claim, data?}`   | the full `Verdict`                      |
| `explain`      | `{verdict}`        | `{ok:true,text:...}`                    |
| `canonicalize` | `{value}`          | `{ok:true,canonical:...}`              |
| `version`      | —                  | `{version:...,engine_version:...}`     |

For `files`-kind claims, supply the candle data under a top-level `data` key;
`inline` claims carry their data already.

Domain errors (a bad claim, an unknown command) come back in-band as
`{ok:false,error:...}`; only null/UTF-8/panic conditions throw.

## Build

```sh
cargo build -p wickra-verify-c        # build the native C ABI library
mvn -Dnative.lib.dir=../../target/debug test
```

## License

MIT OR Apache-2.0.
