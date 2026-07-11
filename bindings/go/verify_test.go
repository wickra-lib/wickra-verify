package wickra

import (
	"encoding/json"
	"math"
	"strings"
	"testing"
)

const strategy = `{"symbol":"BTCUSDT","timeframe":"1h",` +
	`"indicators":{"ema_fast":{"type":"Ema","params":[5]},"ema_slow":{"type":"Ema","params":[15]}},` +
	`"entry":{"cross_above":["ema_fast","ema_slow"]},"exit":{"cross_below":["ema_fast","ema_slow"]},` +
	`"sizing":{"type":"fixed_fraction","fraction":0.95},` +
	`"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},` +
	`"risk":{"trailing_stop_pct":5.0}}`

func candles() []map[string]float64 {
	out := make([]map[string]float64, 0, 40)
	for i := 0; i < 40; i++ {
		base := 100.0 + math.Sin(float64(i)*0.4)*8.0
		out = append(out, map[string]float64{
			"time": float64(1_700_000_000 + i*3600), "open": base,
			"high": base + 1.0, "low": base - 1.0, "close": base + 0.5, "volume": 1000.0,
		})
	}
	return out
}

func verifyRequest(claimedReport map[string]any) string {
	claim := map[string]any{
		"strategy":       json.RawMessage(strategy),
		"dataset_ref":    map[string]any{"kind": "inline", "data": map[string]any{"BTCUSDT": candles()}},
		"claimed_report": claimedReport,
	}
	cmd, _ := json.Marshal(map[string]any{"cmd": "verify", "claim": claim})
	return string(cmd)
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestFudgedClaimIsRefuted(t *testing.T) {
	v := New()
	defer v.Close()
	raw, err := v.Command(verifyRequest(map[string]any{"fees_paid": 99999.0}))
	if err != nil {
		t.Fatal(err)
	}
	var verdict struct {
		Matches    bool `json:"matches"`
		Mismatches []struct {
			Field string `json:"field"`
		} `json:"mismatches"`
		ClaimedReportHash string `json:"claimed_report_hash"`
		InputsHash        string `json:"inputs_hash"`
	}
	if err := json.Unmarshal([]byte(raw), &verdict); err != nil {
		t.Fatal(err)
	}
	if verdict.Matches {
		t.Fatal("expected a fudged claim to be refuted")
	}
	found := false
	for _, m := range verdict.Mismatches {
		if m.Field == "fees_paid" {
			found = true
		}
	}
	if !found {
		t.Fatalf("expected a fees_paid mismatch, got %s", raw)
	}
	if len(verdict.ClaimedReportHash) != 64 || len(verdict.InputsHash) != 64 {
		t.Fatalf("expected 64-hex hashes, got %s", raw)
	}
}

func TestUnknownCommandIsInBandError(t *testing.T) {
	v := New()
	defer v.Close()
	raw, err := v.Command(`{"cmd":"nope"}`)
	if err != nil {
		t.Fatalf("unexpected hard error: %v", err)
	}
	if !strings.Contains(raw, `"ok":false`) {
		t.Fatalf("expected an in-band error, got: %s", raw)
	}
}
