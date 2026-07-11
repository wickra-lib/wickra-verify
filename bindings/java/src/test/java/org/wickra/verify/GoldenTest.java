package org.wickra.verify;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

// Cross-language golden parity: for each committed golden/claims/*.json, verify
// over the shared golden/data and assert the response equals
// golden/expected/<claim>.json byte-for-byte. The binding returns the core's
// canonical command_json string verbatim, so byte equality is the exact
// cross-language parity check. The fixtures arrive in a later phase; until then
// the test skips cleanly.
class GoldenTest {
    private static Path findGolden() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 10 && dir != null; i++) {
            Path g = dir.resolve("golden");
            if (Files.isDirectory(g.resolve("claims"))) {
                return g;
            }
            dir = dir.getParent();
        }
        return null;
    }

    private static String loadData(Path golden) throws IOException {
        Path dataDir = golden.resolve("data");
        if (!Files.isDirectory(dataDir)) {
            return null;
        }
        var symbols = new TreeMap<String, String>();
        try (Stream<Path> files = Files.list(dataDir)) {
            for (Path csv : files.filter(p -> p.toString().endsWith(".csv")).toList()) {
                List<String> rows = new ArrayList<>();
                List<String> lines = Files.readAllLines(csv);
                for (int idx = 0; idx < lines.size(); idx++) {
                    String line = lines.get(idx).trim();
                    if (line.isEmpty()) {
                        continue;
                    }
                    String[] c = line.split(",");
                    if (c.length < 6) {
                        continue;
                    }
                    try {
                        long t = Long.parseLong(c[0].trim());
                        rows.add("{\"time\":" + t + ",\"open\":" + c[1].trim() + ",\"high\":" + c[2].trim()
                                + ",\"low\":" + c[3].trim() + ",\"close\":" + c[4].trim()
                                + ",\"volume\":" + c[5].trim() + "}");
                    } catch (NumberFormatException e) {
                        // header row
                    }
                }
                String name = csv.getFileName().toString().replaceFirst("\\.csv$", "");
                symbols.put(name, "[" + String.join(",", rows) + "]");
            }
        }
        var parts = new ArrayList<String>();
        symbols.forEach((k, v) -> parts.add("\"" + k + "\":" + v));
        return "{" + String.join(",", parts) + "}";
    }

    @Test
    void goldenParity() throws IOException {
        Path golden = findGolden();
        if (golden == null) {
            return; // golden fixtures not present yet
        }
        String data = loadData(golden);
        try (Stream<Path> claims = Files.list(golden.resolve("claims"))) {
            for (Path claimPath : claims.filter(p -> p.toString().endsWith(".json")).toList()) {
                String name = claimPath.getFileName().toString();
                String claim = Files.readString(claimPath).trim();
                String expected = Files.readString(golden.resolve("expected").resolve(name)).trim();
                String envelope = data != null
                        ? "{\"cmd\":\"verify\",\"claim\":" + claim + ",\"data\":" + data + "}"
                        : "{\"cmd\":\"verify\",\"claim\":" + claim + "}";
                try (Verifier verifier = new Verifier()) {
                    assertEquals(expected, verifier.command(envelope), "golden mismatch for " + name);
                }
            }
        }
    }
}
