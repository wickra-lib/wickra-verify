/* Cross-language golden parity and operating-mode equivalence, from C.
 *
 * Golden: verify each committed golden/claims/*.json over the shared
 * golden/data and assert the response equals golden/expected/<claim>.json
 * byte-for-byte. The ABI returns the core's canonical command output verbatim,
 * so byte equality is the exact cross-language parity check -- the same one
 * Python, Node, Go, C#, Java, R and WASM make.
 *
 * Operating mode: a claim names its candles by reference (dataset_ref.kind
 * "files", the data supplied with the verify command) or inline (kind
 * "inline", embedded in the claim). The verdict must not depend on which; only
 * inputs_hash may differ, because it binds the dataset *reference* -- and it
 * must differ, or the reference was not hashed. Each golden claim is re-issued
 * inline here and the two verdicts compared with that one field masked.
 *
 * Until this existed the C ABI was the only reach with no test at all: the two
 * examples beside it print a verdict and exit zero. Six of the ten language
 * reaches go through this ABI, so a fault here is a fault in all of them.
 *
 * C has no directory API that is portable between POSIX and Windows, so the
 * claim and data lists are globbed by CMake at configure time and written into
 * golden_claims.h. That keeps the property the other bindings get from a
 * runtime glob: a claim added to the corpus is covered here without editing
 * this file.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_verify.h"

#include "golden_claims.h" /* GOLDEN_DIR, GOLDEN_CLAIMS, GOLDEN_CLAIM_COUNT, GOLDEN_SYMBOLS, GOLDEN_SYMBOL_COUNT */

/* Read a whole file. Caller frees. Returns NULL and reports on failure. */
static char *slurp(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return NULL;
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (!buf) {
        fclose(file);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, file);
    fclose(file);
    buf[got] = '\0';
    return buf;
}

/* Trim ASCII whitespace in place and return the start of the trimmed text. */
static char *trim(char *text) {
    while (*text == ' ' || *text == '\n' || *text == '\r' || *text == '\t') {
        text++;
    }
    size_t len = strlen(text);
    while (len > 0) {
        char last = text[len - 1];
        if (last != ' ' && last != '\n' && last != '\r' && last != '\t') {
            break;
        }
        text[--len] = '\0';
    }
    return text;
}

/* A growable string. */
typedef struct {
    char *buf;
    size_t len;
    size_t cap;
} Str;

static int str_push(Str *s, const char *text, size_t n) {
    if (s->len + n + 1 > s->cap) {
        size_t cap = s->cap ? s->cap : 4096;
        while (cap < s->len + n + 1) {
            cap *= 2;
        }
        char *grown = (char *)realloc(s->buf, cap);
        if (!grown) {
            return 0;
        }
        s->buf = grown;
        s->cap = cap;
    }
    memcpy(s->buf + s->len, text, n);
    s->len += n;
    s->buf[s->len] = '\0';
    return 1;
}

static int str_puts(Str *s, const char *text) { return str_push(s, text, strlen(text)); }

/* One golden/data/<SYMBOL>.csv as a JSON candle array: "[{...},{...}]".
 * Column text is passed through verbatim, as the other bindings do. */
static int csv_to_json(Str *out, const char *path) {
    char *raw = slurp(path);
    if (!raw) {
        return 0;
    }
    if (!str_puts(out, "[")) {
        free(raw);
        return 0;
    }
    static const char *KEYS[] = {"time", "open", "high", "low", "close", "volume"};
    int first = 1;
    char *line = raw;
    while (line && *line) {
        char *next = strpbrk(line, "\r\n");
        if (next) {
            *next++ = '\0';
            while (*next == '\r' || *next == '\n') {
                next++;
            }
        }
        /* Split the line on commas in place; a row has six columns. */
        char *cols[6];
        int n = 0;
        char *col = line;
        while (col && n < 6) {
            char *comma = strchr(col, ',');
            if (comma) {
                *comma++ = '\0';
            }
            cols[n++] = trim(col);
            col = comma;
        }
        if (n == 6 && cols[0][0] >= '0' && cols[0][0] <= '9') {
            if (!first && !str_puts(out, ",")) {
                free(raw);
                return 0;
            }
            first = 0;
            str_puts(out, "{");
            for (int k = 0; k < 6; k++) {
                str_puts(out, k ? ",\"" : "\"");
                str_puts(out, KEYS[k]);
                str_puts(out, "\":");
                str_puts(out, cols[k]);
            }
            str_puts(out, "}");
        }
        line = next;
    }
    free(raw);
    return str_puts(out, "]");
}

/* All golden data as one JSON object keyed by symbol. */
static int load_data(Str *out) {
    if (!str_puts(out, "{")) {
        return 0;
    }
    for (size_t i = 0; i < GOLDEN_SYMBOL_COUNT; i++) {
        char path[1024];
        snprintf(path, sizeof path, "%s/data/%s.csv", GOLDEN_DIR, GOLDEN_SYMBOLS[i]);
        if (i && !str_puts(out, ",")) {
            return 0;
        }
        str_puts(out, "\"");
        str_puts(out, GOLDEN_SYMBOLS[i]);
        str_puts(out, "\":");
        if (!csv_to_json(out, path)) {
            return 0;
        }
    }
    return str_puts(out, "}");
}

/* Apply one command through the two-call length protocol. Caller frees. */
static char *run(WickraVerify *verifier, const char *cmd) {
    int32_t len = wickra_verify_command(verifier, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed with code %d\n", (int)len);
        return NULL;
    }
    char *out = (char *)malloc((size_t)len + 1);
    if (!out) {
        return NULL;
    }
    if (wickra_verify_command(verifier, cmd, out, (size_t)len + 1) < 0) {
        free(out);
        return NULL;
    }
    return out;
}

/* The [start, end) span of the JSON object that is the value of `key`, or 0. */
static int object_span(const char *json, const char *key, size_t *start, size_t *end) {
    char quoted[128];
    snprintf(quoted, sizeof quoted, "\"%s\"", key);
    const char *at = strstr(json, quoted);
    if (!at) {
        return 0;
    }
    const char *open = strchr(at, '{');
    if (!open) {
        return 0;
    }
    int depth = 0;
    int in_string = 0;
    for (const char *p = open; *p; p++) {
        if (in_string) {
            if (*p == '\\') {
                p++;
            } else if (*p == '"') {
                in_string = 0;
            }
        } else if (*p == '"') {
            in_string = 1;
        } else if (*p == '{') {
            depth++;
        } else if (*p == '}') {
            if (--depth == 0) {
                *start = (size_t)(open - json);
                *end = (size_t)(p - json) + 1;
                return 1;
            }
        }
    }
    return 0;
}

/* Blank the inputs_hash value in place so the rest of the verdict can be compared. */
static void mask_inputs_hash(char *verdict) {
    char *at = strstr(verdict, "\"inputs_hash\":\"");
    if (!at) {
        return;
    }
    at += strlen("\"inputs_hash\":\"");
    while (*at && *at != '"') {
        *at++ = '-';
    }
}

int main(void) {
    if (GOLDEN_CLAIM_COUNT == 0 || GOLDEN_SYMBOL_COUNT == 0) {
        fprintf(stderr, "no golden claims or data were configured; this would test nothing\n");
        return 1;
    }

    Str data = {0};
    if (!load_data(&data)) {
        fprintf(stderr, "could not load golden data\n");
        return 1;
    }

    WickraVerify *verifier = wickra_verify_new();
    if (!verifier) {
        fprintf(stderr, "failed to create verifier\n");
        return 1;
    }

    int failures = 0;
    for (size_t i = 0; i < GOLDEN_CLAIM_COUNT; i++) {
        const char *name = GOLDEN_CLAIMS[i];
        char path[1024];

        snprintf(path, sizeof path, "%s/claims/%s", GOLDEN_DIR, name);
        char *claim_raw = slurp(path);
        snprintf(path, sizeof path, "%s/expected/%s", GOLDEN_DIR, name);
        char *expected_raw = slurp(path);
        if (!claim_raw || !expected_raw) {
            free(claim_raw);
            free(expected_raw);
            failures++;
            continue;
        }
        char *claim = trim(claim_raw);
        char *expected = trim(expected_raw);

        /* Golden: the files claim, data supplied with the command. */
        Str supplied_cmd = {0};
        str_puts(&supplied_cmd, "{\"cmd\":\"verify\",\"claim\":");
        str_puts(&supplied_cmd, claim);
        str_puts(&supplied_cmd, ",\"data\":");
        str_puts(&supplied_cmd, data.buf);
        str_puts(&supplied_cmd, "}");
        char *supplied = run(verifier, supplied_cmd.buf);
        free(supplied_cmd.buf);
        if (!supplied) {
            fprintf(stderr, "%s: no verdict\n", name);
            failures++;
            free(claim_raw);
            free(expected_raw);
            continue;
        }
        if (strcmp(trim(supplied), expected) != 0) {
            fprintf(stderr, "%s: golden mismatch\n  expected: %s\n  got:      %s\n", name, expected, supplied);
            failures++;
        }

        /* Operating mode: the same claim with its data embedded inline. The
         * golden data carries exactly the symbols the claims reference. */
        size_t start = 0;
        size_t end = 0;
        if (!object_span(claim, "dataset_ref", &start, &end) || !strstr(claim + start, "\"files\"")) {
            fprintf(stderr, "%s: golden claims reference their data\n", name);
            failures++;
        } else {
            Str inline_cmd = {0};
            str_puts(&inline_cmd, "{\"cmd\":\"verify\",\"claim\":");
            str_push(&inline_cmd, claim, start);
            str_puts(&inline_cmd, "{\"kind\":\"inline\",\"data\":");
            str_puts(&inline_cmd, data.buf);
            str_puts(&inline_cmd, "}");
            str_puts(&inline_cmd, claim + end);
            str_puts(&inline_cmd, "}");
            char *inline_verdict = run(verifier, inline_cmd.buf);
            free(inline_cmd.buf);
            if (!inline_verdict) {
                fprintf(stderr, "%s: no inline verdict\n", name);
                failures++;
            } else {
                const char *sh = strstr(supplied, "\"inputs_hash\":\"");
                const char *ih = strstr(inline_verdict, "\"inputs_hash\":\"");
                if (!sh || !ih || strncmp(sh, ih, 15 + 64) == 0) {
                    fprintf(stderr, "%s: inputs_hash must bind the dataset reference\n", name);
                    failures++;
                }
                mask_inputs_hash(supplied);
                mask_inputs_hash(inline_verdict);
                if (strcmp(trim(supplied), trim(inline_verdict)) != 0) {
                    fprintf(stderr, "%s: verdict differs between modes\n  supplied: %s\n  inline:   %s\n",
                            name, supplied, inline_verdict);
                    failures++;
                }
                free(inline_verdict);
            }
        }

        free(supplied);
        free(claim_raw);
        free(expected_raw);
    }

    wickra_verify_free(verifier);
    free(data.buf);

    if (failures > 0) {
        fprintf(stderr, "%d failure(s) across %zu golden claims\n", failures, GOLDEN_CLAIM_COUNT);
        return 1;
    }
    printf("all %zu golden claims are byte-identical from C, in both operating modes\n", GOLDEN_CLAIM_COUNT);
    return 0;
}
