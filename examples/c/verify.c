/* A minimal C example: submit a claim to the wickra-verify C ABI whose report
 * has been doctored, and assert that verification refutes it. This is the whole
 * product in one file — verification recomputes the report from (strategy, data)
 * and compares, so a fabricated `claimed_report` cannot pass.
 *
 * No JSON parser is needed on the C side: the claim is assembled from string
 * literals, and the verdict is inspected with a substring search. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_verify.h"

/* An EMA-cross strategy trading symbol AAA. */
static const char *STRATEGY =
    "{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
    "\"risk\":{}}";

/* A short V-shaped price path so the fast/slow EMA cross fires at least once. */
static const char *DATA =
    "{\"AAA\":["
    "{\"time\":1700000000,\"open\":120,\"high\":121,\"low\":119,\"close\":120,\"volume\":1000},"
    "{\"time\":1700003600,\"open\":120,\"high\":121,\"low\":117,\"close\":118,\"volume\":1000},"
    "{\"time\":1700007200,\"open\":118,\"high\":119,\"low\":115,\"close\":116,\"volume\":1000},"
    "{\"time\":1700010800,\"open\":116,\"high\":117,\"low\":113,\"close\":114,\"volume\":1000},"
    "{\"time\":1700014400,\"open\":114,\"high\":115,\"low\":111,\"close\":112,\"volume\":1000},"
    "{\"time\":1700018000,\"open\":112,\"high\":113,\"low\":109,\"close\":110,\"volume\":1000},"
    "{\"time\":1700021600,\"open\":110,\"high\":111,\"low\":107,\"close\":108,\"volume\":1000},"
    "{\"time\":1700025200,\"open\":108,\"high\":113,\"low\":107,\"close\":112,\"volume\":1000},"
    "{\"time\":1700028800,\"open\":112,\"high\":117,\"low\":111,\"close\":116,\"volume\":1000},"
    "{\"time\":1700032400,\"open\":116,\"high\":121,\"low\":115,\"close\":120,\"volume\":1000},"
    "{\"time\":1700036000,\"open\":120,\"high\":125,\"low\":119,\"close\":124,\"volume\":1000},"
    "{\"time\":1700039600,\"open\":124,\"high\":129,\"low\":123,\"close\":128,\"volume\":1000}]}";

/* A fabricated report: a claimant asserts an inflated fees figure. */
static const char *CLAIMED_REPORT = "{\"fees_paid\":99999.0}";

/* Read a command response into a freshly malloc'd, NUL-terminated buffer using
 * the length-out protocol. Returns NULL on failure. */
static char *run(WickraVerify *verifier, const char *cmd) {
    int len = wickra_verify_command(verifier, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_verify_command(verifier, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraVerify *verifier = wickra_verify_new();
    if (!verifier) {
        fprintf(stderr, "failed to create verifier\n");
        return 1;
    }

    /* Assemble the verify command: an inline claim carrying its own data. */
    size_t cap = strlen(STRATEGY) + strlen(DATA) + strlen(CLAIMED_REPORT) + 128;
    char *cmd = (char *)malloc(cap);
    if (!cmd) {
        wickra_verify_free(verifier);
        return 1;
    }
    snprintf(cmd, cap,
             "{\"cmd\":\"verify\",\"claim\":{\"strategy\":%s,"
             "\"dataset_ref\":{\"kind\":\"inline\",\"data\":%s},"
             "\"claimed_report\":%s}}",
             STRATEGY, DATA, CLAIMED_REPORT);

    char *verdict = run(verifier, cmd);
    if (!verdict) {
        free(cmd);
        wickra_verify_free(verifier);
        return 1;
    }

    printf("wickra-verify %s\n", wickra_verify_version());
    /* The doctored report must be refuted: the verdict says matches:false. */
    int refuted = strstr(verdict, "\"matches\":false") != NULL;
    printf("verdict: %s\n", refuted ? "REFUTED (tamper caught)" : "matched?!");

    free(verdict);
    free(cmd);
    wickra_verify_free(verifier);

    if (!refuted) {
        fprintf(stderr, "a doctored report was not refuted\n");
        return 1;
    }
    return 0;
}
