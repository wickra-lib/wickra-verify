package wickra

// Cross-language golden parity: for each committed golden/claims/*.json, verify
// over the shared golden/data and assert the response equals
// golden/expected/<claim>.json byte-for-byte. The binding returns the core's
// canonical command_json string verbatim, so byte equality is the exact
// cross-language parity check — the same blake3 hashes in every language. The
// fixtures arrive in a later phase; until then the test skips cleanly.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
)

func goldenDir() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for i := 0; i < 8; i++ {
		g := filepath.Join(dir, "golden")
		if _, err := os.Stat(filepath.Join(g, "claims")); err == nil {
			return g
		}
		dir = filepath.Dir(dir)
	}
	return ""
}

func loadGoldenData(g string) map[string][]map[string]float64 {
	data := map[string][]map[string]float64{}
	dataDir := filepath.Join(g, "data")
	entries, err := os.ReadDir(dataDir)
	if err != nil {
		return data
	}
	for _, e := range entries {
		if !strings.HasSuffix(e.Name(), ".csv") {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(dataDir, e.Name()))
		if err != nil {
			continue
		}
		var series []map[string]float64
		for idx, line := range strings.Split(string(raw), "\n") {
			line = strings.TrimSpace(line)
			if line == "" {
				continue
			}
			cols := strings.Split(line, ",")
			if len(cols) < 6 {
				continue
			}
			t, err := strconv.ParseInt(strings.TrimSpace(cols[0]), 10, 64)
			if err != nil {
				if idx == 0 {
					continue // header
				}
				continue
			}
			f := func(i int) float64 { x, _ := strconv.ParseFloat(strings.TrimSpace(cols[i]), 64); return x }
			series = append(series, map[string]float64{
				"time": float64(t), "open": f(1), "high": f(2), "low": f(3), "close": f(4), "volume": f(5),
			})
		}
		data[strings.TrimSuffix(e.Name(), ".csv")] = series
	}
	return data
}

func TestGoldenParity(t *testing.T) {
	g := goldenDir()
	if g == "" {
		t.Skip("golden fixtures not present yet")
	}
	claims, err := os.ReadDir(filepath.Join(g, "claims"))
	if err != nil {
		t.Skip("golden claims not present yet")
	}
	data := loadGoldenData(g)
	for _, entry := range claims {
		if !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}
		name := entry.Name()
		t.Run(name, func(t *testing.T) {
			claimRaw, err := os.ReadFile(filepath.Join(g, "claims", name))
			if err != nil {
				t.Fatal(err)
			}
			expected, err := os.ReadFile(filepath.Join(g, "expected", name))
			if err != nil {
				t.Fatal(err)
			}
			envelope := map[string]any{"cmd": "verify", "claim": json.RawMessage(claimRaw)}
			if len(data) > 0 {
				envelope["data"] = data
			}
			cmd, err := json.Marshal(envelope)
			if err != nil {
				t.Fatal(err)
			}
			v := New()
			defer v.Close()
			got, err := v.Command(string(cmd))
			if err != nil {
				t.Fatal(err)
			}
			if got != strings.TrimSpace(string(expected)) {
				t.Fatalf("golden mismatch for %s:\n got: %s\nwant: %s", name, got, expected)
			}
		})
	}
}
