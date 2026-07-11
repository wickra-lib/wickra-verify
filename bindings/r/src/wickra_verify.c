/* R .Call glue for the wickra-verify C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_verify.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkverify_finalize(SEXP ext) {
    WickraVerify *h = (WickraVerify *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_verify_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraVerify *handle_of(SEXP ext) {
    WickraVerify *h = (WickraVerify *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-verify: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkverify_version(void) {
    return Rf_mkString(wickra_verify_version());
}

SEXP wkverify_new(void) {
    WickraVerify *h = wickra_verify_new();
    if (!h) {
        Rf_error("wickra-verify: failed to create a verifier");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkverify_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkverify_command(SEXP ext, SEXP cmd_json) {
    WickraVerify *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_verify_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-verify: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_verify_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkverify_version", (DL_FUNC)&wkverify_version, 0},
    {"wkverify_new", (DL_FUNC)&wkverify_new, 0},
    {"wkverify_command", (DL_FUNC)&wkverify_command, 2},
    {NULL, NULL, 0}};

void R_init_wickraverify(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
