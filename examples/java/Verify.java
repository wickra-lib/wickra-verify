// A runnable Java example: submit a claim whose report has been doctored and
// assert the binding refutes it. Verification recomputes the report from
// (strategy, data) and compares, so a fabricated number cannot pass.
//
//   cargo build -p wickra-verify-c
//   mvn -f bindings/java/pom.xml -q package -DskipTests
//   javac -cp bindings/java/target/classes examples/java/Verify.java -d examples/java/out
//   java --enable-native-access=ALL-UNNAMED \
//        -Dnative.lib.dir=target/debug \
//        -cp "bindings/java/target/classes;examples/java/out" Verify
import org.wickra.verify.Verifier;

public final class Verify {
    private static final String STRATEGY =
            "{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
                    + "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
                    + "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
                    + "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
                    + "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
                    + "\"risk\":{}}";

    // A short V-shaped price path so the fast/slow EMA cross fires at least once.
    private static final int[] CLOSES = {120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128};

    private static String data() {
        StringBuilder sb = new StringBuilder("{\"AAA\":[");
        for (int i = 0; i < CLOSES.length; i++) {
            int close = CLOSES[i];
            int open = i == 0 ? close : CLOSES[i - 1];
            if (i > 0) {
                sb.append(',');
            }
            sb.append("{\"time\":").append(1_700_000_000L + i * 3600L)
                    .append(",\"open\":").append(open)
                    .append(",\"high\":").append(Math.max(open, close) + 1)
                    .append(",\"low\":").append(Math.min(open, close) - 1)
                    .append(",\"close\":").append(close)
                    .append(",\"volume\":1000}");
        }
        return sb.append("]}").toString();
    }

    public static void main(String[] args) {
        // An inline claim carrying a fabricated report (an inflated fees figure).
        String claim = "{\"strategy\":" + STRATEGY
                + ",\"dataset_ref\":{\"kind\":\"inline\",\"data\":" + data() + "}"
                + ",\"claimed_report\":{\"fees_paid\":99999.0}}";

        try (Verifier verifier = new Verifier()) {
            String verdict = verifier.command("{\"cmd\":\"verify\",\"claim\":" + claim + "}");

            System.out.println("wickra-verify " + Verifier.version());
            if (!verdict.contains("\"matches\":false")) {
                throw new IllegalStateException("a doctored report must be refuted, got: " + verdict);
            }
            System.out.println("doctored claim: REFUTED (tamper caught)");
        }
    }
}
