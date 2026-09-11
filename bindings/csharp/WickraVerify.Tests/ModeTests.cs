using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Verify;
using Xunit;

namespace WickraVerify.Tests;

// Operating-mode equivalence: a claim verifies the same whichever way its data
// arrives. A claim names its candles by reference (`dataset_ref.kind ==
// "files"`, the data supplied with the `verify` command) or inline
// (`dataset_ref.kind == "inline"`, embedded in the claim). The verdict must not
// depend on which: same matches, mismatches, report hashes and engine. Only
// `inputs_hash` may differ, because it binds the dataset *reference* -- and it
// must differ, or the reference was not hashed.
//
// The golden claims are `files` claims; each is re-issued inline here.
public class ModeTests
{
    private static readonly string[] Same =
    {
        "matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version",
    };

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

    private static JsonObject LoadData(string golden)
    {
        var data = new JsonObject();
        string dataDir = Path.Combine(golden, "data");
        if (!Directory.Exists(dataDir))
        {
            return data;
        }
        foreach (string csv in Directory.EnumerateFiles(dataDir, "*.csv"))
        {
            var series = new JsonArray();
            foreach (string raw in File.ReadAllLines(csv))
            {
                string line = raw.Trim();
                if (line.Length == 0)
                {
                    continue;
                }
                string[] cols = line.Split(',');
                if (cols.Length < 6 || !long.TryParse(cols[0].Trim(), out long t))
                {
                    continue; // header or short row
                }
                double F(int i) => double.Parse(cols[i].Trim(), System.Globalization.CultureInfo.InvariantCulture);
                series.Add(new JsonObject
                {
                    ["time"] = t, ["open"] = F(1), ["high"] = F(2), ["low"] = F(3), ["close"] = F(4), ["volume"] = F(5),
                });
            }
            data[Path.GetFileNameWithoutExtension(csv)] = series;
        }
        return data;
    }

    [Fact]
    public void SuppliedAndInlineDataVerifyAlike()
    {
        string? golden = GoldenDir();
        if (golden is null)
        {
            return; // golden fixtures not present yet
        }

        foreach (string claimPath in Directory.EnumerateFiles(Path.Combine(golden, "claims"), "*.json"))
        {
            JsonObject claim = JsonNode.Parse(File.ReadAllText(claimPath))!.AsObject();
            Assert.Equal("files", claim["dataset_ref"]!["kind"]!.GetValue<string>());
            JsonObject data = LoadData(golden);

            using var verifier = new Verifier();
            var suppliedEnvelope = new JsonObject { ["cmd"] = "verify", ["claim"] = claim.DeepClone(), ["data"] = data.DeepClone() };
            JsonObject supplied = JsonNode.Parse(verifier.Command(suppliedEnvelope.ToJsonString()))!.AsObject();

            var inlineData = new JsonObject();
            foreach (JsonNode? symbol in claim["dataset_ref"]!["symbols"]!.AsArray())
            {
                string s = symbol!.GetValue<string>();
                inlineData[s] = data[s]!.DeepClone();
            }
            JsonObject inlineClaim = claim.DeepClone().AsObject();
            inlineClaim["dataset_ref"] = new JsonObject { ["kind"] = "inline", ["data"] = inlineData };
            var inlineEnvelope = new JsonObject { ["cmd"] = "verify", ["claim"] = inlineClaim };
            JsonObject inline = JsonNode.Parse(verifier.Command(inlineEnvelope.ToJsonString()))!.AsObject();

            foreach (string field in Same)
            {
                Assert.True(JsonNode.DeepEquals(inline[field], supplied[field]), $"{Path.GetFileName(claimPath)}: {field}");
            }
            Assert.NotEqual(supplied["inputs_hash"]!.GetValue<string>(), inline["inputs_hash"]!.GetValue<string>());
        }
    }
}
