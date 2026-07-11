using System.Text.Json;
using Wickra.Verify;
using Xunit;

namespace WickraVerify.Tests;

public class VerifierTests
{
    private static object Strategy() => new
    {
        symbol = "BTCUSDT",
        timeframe = "1h",
        indicators = new
        {
            ema_fast = new { type = "Ema", @params = new[] { 5 } },
            ema_slow = new { type = "Ema", @params = new[] { 15 } },
        },
        entry = new { cross_above = new[] { "ema_fast", "ema_slow" } },
        exit = new { cross_below = new[] { "ema_fast", "ema_slow" } },
        sizing = new { type = "fixed_fraction", fraction = 0.95 },
        costs = new { taker_bps = 5, slippage = new { type = "fixed_bps", bps = 2 } },
        risk = new { trailing_stop_pct = 5.0 },
    };

    private static object[] Candles()
    {
        var list = new List<object>();
        for (int i = 0; i < 40; i++)
        {
            double b = 100.0 + Math.Sin(i * 0.4) * 8.0;
            list.Add(new { time = 1_700_000_000 + i * 3600, open = b, high = b + 1.0, low = b - 1.0, close = b + 0.5, volume = 1000.0 });
        }
        return [.. list];
    }

    private static string VerifyRequest(object claimedReport)
    {
        var claim = new
        {
            strategy = Strategy(),
            dataset_ref = new { kind = "inline", data = new Dictionary<string, object[]> { ["BTCUSDT"] = Candles() } },
            claimed_report = claimedReport,
        };
        return JsonSerializer.Serialize(new { cmd = "verify", claim });
    }

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(Verifier.Version()));
    }

    [Fact]
    public void FudgedClaim_IsRefuted()
    {
        using var verifier = new Verifier();
        JsonElement verdict = JsonDocument.Parse(
            verifier.Command(VerifyRequest(new { fees_paid = 99999.0 }))).RootElement;

        Assert.False(verdict.GetProperty("matches").GetBoolean());
        bool found = false;
        foreach (JsonElement m in verdict.GetProperty("mismatches").EnumerateArray())
        {
            if (m.GetProperty("field").GetString() == "fees_paid")
            {
                found = true;
            }
        }
        Assert.True(found, "expected a fees_paid mismatch");
        Assert.Equal(64, verdict.GetProperty("claimed_report_hash").GetString()!.Length);
        Assert.Equal(64, verdict.GetProperty("inputs_hash").GetString()!.Length);
    }

    [Fact]
    public void UnknownCommand_IsInBandError()
    {
        using var verifier = new Verifier();
        // An unknown command is not a hard error: the ABI returns a length and the
        // error surfaces in-band as {"ok":false,...} JSON.
        string raw = verifier.Command("{\"cmd\":\"nope\"}");
        Assert.Contains("\"ok\":false", raw);
    }
}
