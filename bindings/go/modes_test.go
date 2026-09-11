package wickra

// Operating-mode equivalence: a claim verifies the same whichever way its data
// arrives. A claim names its candles by reference (`dataset_ref.kind ==
// "files"`, the data supplied with the `verify` command) or inline
// (`dataset_ref.kind == "inline"`, embedded in the claim). The verdict must not
// depend on which: same matches, mismatches, report hashes and engine. Only
// `inputs_hash` may differ, because it binds the dataset *reference* -- and it
// must differ, or the reference was not hashed.
//
// The golden claims are `files` claims; each is re-issued inline here. The
// golden directory and CSV loader are shared with golden_test.go.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

var sameAcrossModes = []string{"matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version"}

func TestSuppliedAndInlineDataVerifyAlike(t *testing.T) {
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
			var claim map[string]any
			if err := json.Unmarshal(claimRaw, &claim); err != nil {
				t.Fatal(err)
			}
			ref := claim["dataset_ref"].(map[string]any)
			if ref["kind"] != "files" {
				t.Fatalf("golden claims reference their data, got kind %v", ref["kind"])
			}

			v := New()
			defer v.Close()

			supplied := verifyJSON(t, v, map[string]any{"cmd": "verify", "claim": json.RawMessage(claimRaw), "data": data})

			inlineData := map[string]any{}
			for _, s := range ref["symbols"].([]any) {
				inlineData[s.(string)] = data[s.(string)]
			}
			claim["dataset_ref"] = map[string]any{"kind": "inline", "data": inlineData}
			inline := verifyJSON(t, v, map[string]any{"cmd": "verify", "claim": claim})

			for _, field := range sameAcrossModes {
				if !reflect.DeepEqual(inline[field], supplied[field]) {
					t.Fatalf("%s differs between modes:\n supplied: %v\n   inline: %v", field, supplied[field], inline[field])
				}
			}
			if inline["inputs_hash"] == supplied["inputs_hash"] {
				t.Fatal("inputs_hash must bind the dataset reference")
			}
		})
	}
}

func verifyJSON(t *testing.T, v *Verifier, envelope map[string]any) map[string]any {
	t.Helper()
	cmd, err := json.Marshal(envelope)
	if err != nil {
		t.Fatal(err)
	}
	got, err := v.Command(string(cmd))
	if err != nil {
		t.Fatal(err)
	}
	var verdict map[string]any
	if err := json.Unmarshal([]byte(got), &verdict); err != nil {
		t.Fatalf("verdict is not JSON: %v\n%s", err, got)
	}
	return verdict
}
