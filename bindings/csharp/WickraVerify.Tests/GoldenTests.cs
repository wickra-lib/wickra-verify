using System.Text.Json;
using Wickra.Verify;
using Xunit;

namespace WickraVerify.Tests;

// Cross-language golden parity: for each committed golden/claims/*.json, verify
// over the shared golden/data and assert the response equals
// golden/expected/<claim>.json byte-for-byte. The binding returns the core's
// canonical command_json string verbatim, so byte equality is the exact
// cross-language parity check. The fixtures arrive in a later phase; until then
// the test skips cleanly.
public class GoldenTests
{
    private static string? GoldenDir()
    {
        string? dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10 && !string.IsNullOrEmpty(dir); i++)
        {
            string g = Path.Combine(dir, "golden");
            if (Directory.Exists(Path.Combine(g, "claims")))
            {
                return g;
            }
            dir = Path.GetDirectoryName(dir);
        }
        return null;
    }

    private static Dictionary<string, List<Dictionary<string, double>>> LoadData(string golden)
    {
        var data = new Dictionary<string, List<Dictionary<string, double>>>();
        string dataDir = Path.Combine(golden, "data");
        if (!Directory.Exists(dataDir))
        {
            return data;
        }
        foreach (string csv in Directory.EnumerateFiles(dataDir, "*.csv"))
        {
            var series = new List<Dictionary<string, double>>();
            string[] lines = File.ReadAllLines(csv);
            for (int idx = 0; idx < lines.Length; idx++)
            {
                string line = lines[idx].Trim();
                if (line.Length == 0)
                {
                    continue;
                }
                string[] cols = line.Split(',');
                if (cols.Length < 6 || !long.TryParse(cols[0].Trim(), out long t))
                {
                    continue; // header or short row
                }
                double F(int i) => double.Parse(cols[i].Trim());
                series.Add(new Dictionary<string, double>
                {
                    ["time"] = t, ["open"] = F(1), ["high"] = F(2), ["low"] = F(3), ["close"] = F(4), ["volume"] = F(5),
                });
            }
            data[Path.GetFileNameWithoutExtension(csv)] = series;
        }
        return data;
    }

    [Fact]
    public void GoldenParity()
    {
        string? golden = GoldenDir();
        if (golden is null)
        {
            return; // golden fixtures not present yet
        }

        var data = LoadData(golden);
        foreach (string claimPath in Directory.EnumerateFiles(Path.Combine(golden!, "claims"), "*.json"))
        {
            string name = Path.GetFileName(claimPath);
            var claim = JsonSerializer.Deserialize<JsonElement>(File.ReadAllText(claimPath));
            string expected = File.ReadAllText(Path.Combine(golden!, "expected", name)).Trim();

            object envelope = data.Count > 0
                ? new { cmd = "verify", claim, data }
                : new { cmd = "verify", claim };

            using var verifier = new Verifier();
            string got = verifier.Command(JsonSerializer.Serialize(envelope));
            Assert.Equal(expected, got);
        }
    }
}
