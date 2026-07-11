// A runnable .NET example: submit a claim whose report has been doctored and
// assert the binding refutes it. Verification recomputes the report from
// (strategy, data) and compares, so a fabricated number cannot pass.
//
//   cargo build --release -p wickra-verify-c
//   dotnet run --project examples/csharp/Verify

using System.Text.Json;
using Wickra.Verify;

const string strategy =
    "{\"symbol\":\"AAA\",\"timeframe\":\"1h\"," +
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]}," +
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}}," +
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]}," +
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]}," +
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95}," +
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}}," +
    "\"risk\":{}}";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
int[] closes = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128];

static object Candle(long time, int open, int close) => new
{
    time,
    open,
    high = Math.Max(open, close) + 1,
    low = Math.Min(open, close) - 1,
    close,
    volume = 1000,
};

var candles = closes
    .Select((close, i) => Candle(1_700_000_000L + i * 3600, i == 0 ? close : closes[i - 1], close))
    .ToArray();

var claim = new
{
    strategy = JsonDocument.Parse(strategy).RootElement,
    dataset_ref = new { kind = "inline", data = new Dictionary<string, object[]> { ["AAA"] = candles } },
    // A fabricated report: a claimant asserts an inflated fees figure.
    claimed_report = new { fees_paid = 99999.0 },
};

using var verifier = new Verifier();
string verdict = verifier.Command(JsonSerializer.Serialize(new { cmd = "verify", claim }));
using JsonDocument doc = JsonDocument.Parse(verdict);

Console.WriteLine($"wickra-verify {Verifier.Version()}");
if (doc.RootElement.GetProperty("matches").GetBoolean())
{
    throw new InvalidOperationException($"a doctored report must be refuted, got: {verdict}");
}
string field = doc.RootElement.GetProperty("mismatches")[0].GetProperty("field").GetString()!;
Console.WriteLine($"doctored claim: REFUTED (mismatch: {field})");
