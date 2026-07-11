package org.wickra.verify;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.Test;

class VerifierTest {
    private static final String STRATEGY =
            "{\"symbol\":\"BTCUSDT\",\"timeframe\":\"1h\","
                    + "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[5]},"
                    + "\"ema_slow\":{\"type\":\"Ema\",\"params\":[15]}},"
                    + "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
                    + "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
                    + "\"risk\":{\"trailing_stop_pct\":5.0}}";

    private static String candles() {
        StringBuilder sb = new StringBuilder("[");
        for (int i = 0; i < 40; i++) {
            double b = 100.0 + Math.sin(i * 0.4) * 8.0;
            if (i > 0) {
                sb.append(',');
            }
            sb.append("{\"time\":").append(1_700_000_000L + i * 3600L)
                    .append(",\"open\":").append(b)
                    .append(",\"high\":").append(b + 1.0)
                    .append(",\"low\":").append(b - 1.0)
                    .append(",\"close\":").append(b + 0.5)
                    .append(",\"volume\":1000.0}");
        }
        return sb.append(']').toString();
    }

    private static String verifyRequest(String claimedReport) {
        String datasetRef = "{\"kind\":\"inline\",\"data\":{\"BTCUSDT\":" + candles() + "}}";
        String claim = "{\"strategy\":" + STRATEGY + ",\"dataset_ref\":" + datasetRef
                + ",\"claimed_report\":" + claimedReport + "}";
        return "{\"cmd\":\"verify\",\"claim\":" + claim + "}";
    }

    private static String hexField(String json, String key) {
        Matcher m = Pattern.compile("\"" + key + "\":\"([0-9a-f]{64})\"").matcher(json);
        assertTrue(m.find(), "missing 64-hex " + key + " in " + json);
        return m.group(1);
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(Verifier.version().isEmpty());
    }

    @Test
    void fudgedClaimIsRefuted() {
        try (Verifier verifier = new Verifier()) {
            String verdict = verifier.command(verifyRequest("{\"fees_paid\":99999.0}"));
            assertTrue(verdict.contains("\"matches\":false"), verdict);
            assertTrue(verdict.contains("\"field\":\"fees_paid\""), verdict);
            hexField(verdict, "claimed_report_hash");
            hexField(verdict, "inputs_hash");
        }
    }

    @Test
    void unknownCommandIsInBandError() {
        try (Verifier verifier = new Verifier()) {
            // An unknown command is not a hard error: the ABI returns a length and
            // the error surfaces in-band as {"ok":false,...} JSON.
            String raw = verifier.command("{\"cmd\":\"nope\"}");
            assertTrue(raw.contains("\"ok\":false"), raw);
        }
    }
}
