// A minimal C++ example: submit a claim to the wickra-verify C ABI whose report
// has been doctored, and assert that verification refutes it. Verification
// recomputes the report from (strategy, data) and compares, so a fabricated
// `claimed_report` cannot pass.
//
// No JSON parser is needed: the claim is assembled from string literals and the
// verdict is inspected with a substring search.
#include <cstddef>
#include <iostream>
#include <string>
#include <vector>

#include "wickra_verify.h"

namespace {
// An EMA-cross strategy trading symbol AAA.
const char *STRATEGY =
    R"({"symbol":"AAA","timeframe":"1h",)"
    R"("indicators":{"ema_fast":{"type":"Ema","params":[3]},)"
    R"("ema_slow":{"type":"Ema","params":[8]}},)"
    R"("entry":{"cross_above":["ema_fast","ema_slow"]},)"
    R"("exit":{"cross_below":["ema_fast","ema_slow"]},)"
    R"("sizing":{"type":"fixed_fraction","fraction":0.95},)"
    R"("costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},)"
    R"("risk":{}})";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const char *DATA =
    R"({"AAA":[)"
    R"({"time":1700000000,"open":120,"high":121,"low":119,"close":120,"volume":1000},)"
    R"({"time":1700003600,"open":120,"high":121,"low":117,"close":118,"volume":1000},)"
    R"({"time":1700007200,"open":118,"high":119,"low":115,"close":116,"volume":1000},)"
    R"({"time":1700010800,"open":116,"high":117,"low":113,"close":114,"volume":1000},)"
    R"({"time":1700014400,"open":114,"high":115,"low":111,"close":112,"volume":1000},)"
    R"({"time":1700018000,"open":112,"high":113,"low":109,"close":110,"volume":1000},)"
    R"({"time":1700021600,"open":110,"high":111,"low":107,"close":108,"volume":1000},)"
    R"({"time":1700025200,"open":108,"high":113,"low":107,"close":112,"volume":1000},)"
    R"({"time":1700028800,"open":112,"high":117,"low":111,"close":116,"volume":1000},)"
    R"({"time":1700032400,"open":116,"high":121,"low":115,"close":120,"volume":1000},)"
    R"({"time":1700036000,"open":120,"high":125,"low":119,"close":124,"volume":1000},)"
    R"({"time":1700039600,"open":124,"high":129,"low":123,"close":128,"volume":1000}]})";

// A fabricated report: a claimant asserts an inflated fees figure.
const char *CLAIMED_REPORT = R"({"fees_paid":99999.0})";

// Run a command and return its response using the length-out protocol.
std::string run(WickraVerify *verifier, const std::string &cmd) {
    int len = wickra_verify_command(verifier, cmd.c_str(), nullptr, 0);
    if (len < 0) {
        std::cerr << "command failed: code " << len << "\n";
        return {};
    }
    std::vector<char> buf(static_cast<std::size_t>(len) + 1);
    wickra_verify_command(verifier, cmd.c_str(), buf.data(), buf.size());
    return std::string(buf.data());
}
}  // namespace

int main() {
    WickraVerify *verifier = wickra_verify_new();
    if (verifier == nullptr) {
        std::cerr << "failed to create verifier\n";
        return 1;
    }

    const std::string cmd = std::string(R"({"cmd":"verify","claim":{"strategy":)") + STRATEGY +
                            R"(,"dataset_ref":{"kind":"inline","data":)" + DATA +
                            R"(},"claimed_report":)" + CLAIMED_REPORT + "}}";

    const std::string verdict = run(verifier, cmd);
    if (verdict.empty()) {
        wickra_verify_free(verifier);
        return 1;
    }

    std::cout << "wickra-verify " << wickra_verify_version() << "\n";
    // The doctored report must be refuted: the verdict says matches:false.
    bool refuted = verdict.find("\"matches\":false") != std::string::npos;
    std::cout << "verdict: " << (refuted ? "REFUTED (tamper caught)" : "matched?!") << "\n";

    wickra_verify_free(verifier);
    if (!refuted) {
        std::cerr << "a doctored report was not refuted\n";
        return 1;
    }
    return 0;
}
