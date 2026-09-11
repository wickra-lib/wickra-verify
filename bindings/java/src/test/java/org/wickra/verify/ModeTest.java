package org.wickra.verify;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

// Operating-mode equivalence: a claim verifies the same whichever way its data
// arrives. A claim names its candles by reference (`dataset_ref.kind ==
// "files"`, the data supplied with the `verify` command) or inline
// (`dataset_ref.kind == "inline"`, embedded in the claim). The verdict must not
// depend on which: same matches, mismatches, report hashes and engine. Only
// `inputs_hash` may differ, because it binds the dataset *reference* -- and it
// must differ, or the reference was not hashed.
//
// The golden claims are `files` claims; each is re-issued inline here. The
// binding carries no JSON library, so the claim is rewritten textually: the
// `dataset_ref` object is located by brace matching and replaced. The verdict
// is canonical JSON with sorted keys, so the two responses are compared whole
// after the one field allowed to differ is masked.
class ModeTest {
    private static final Pattern INPUTS_HASH = Pattern.compile("\"inputs_hash\":\"([0-9a-f]{64})\"");
    private static final Pattern SYMBOL = Pattern.compile("\"([^\"]+)\"");

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

    private static TreeMap<String, String> loadData(Path golden) throws IOException {
        var symbols = new TreeMap<String, String>();
        Path dataDir = golden.resolve("data");
        if (!Files.isDirectory(dataDir)) {
            return symbols;
        }
        try (Stream<Path> files = Files.list(dataDir)) {
            for (Path csv : files.filter(p -> p.toString().endsWith(".csv")).toList()) {
                List<String> rows = new ArrayList<>();
                for (String raw : Files.readAllLines(csv)) {
                    String line = raw.trim();
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
        return symbols;
    }

    private static String joinObject(TreeMap<String, String> members) {
        var parts = new ArrayList<String>();
        members.forEach((k, v) -> parts.add("\"" + k + "\":" + v));
        return "{" + String.join(",", parts) + "}";
    }

    /** The [start, end) span of the JSON object that is the value of {@code key}. */
    private static int[] objectSpan(String json, String key) {
        int at = json.indexOf("\"" + key + "\"");
        assertTrue(at >= 0, "claim has no " + key);
        int open = json.indexOf('{', at);
        int depth = 0;
        boolean inString = false;
        for (int i = open; i < json.length(); i++) {
            char ch = json.charAt(i);
            if (inString) {
                if (ch == '\\') {
                    i++;
                } else if (ch == '"') {
                    inString = false;
                }
            } else if (ch == '"') {
                inString = true;
            } else if (ch == '{') {
                depth++;
            } else if (ch == '}') {
                depth--;
                if (depth == 0) {
                    return new int[] {open, i + 1};
                }
            }
        }
        throw new AssertionError("unbalanced object for " + key);
    }

    @Test
    void suppliedAndInlineDataVerifyAlike() throws IOException {
        Path golden = findGolden();
        if (golden == null) {
            return; // golden fixtures not present yet
        }
        TreeMap<String, String> data = loadData(golden);
        try (Stream<Path> claims = Files.list(golden.resolve("claims"))) {
            for (Path claimPath : claims.filter(p -> p.toString().endsWith(".json")).toList()) {
                String name = claimPath.getFileName().toString();
                String claim = Files.readString(claimPath).trim();

                int[] span = objectSpan(claim, "dataset_ref");
                String ref = claim.substring(span[0], span[1]);
                assertTrue(ref.contains("\"files\""), name + ": golden claims reference their data");
                int symbolsAt = ref.indexOf("\"symbols\"");
                String symbolList = ref.substring(ref.indexOf('[', symbolsAt), ref.indexOf(']', symbolsAt) + 1);
                var inlineData = new TreeMap<String, String>();
                Matcher m = SYMBOL.matcher(symbolList);
                while (m.find()) {
                    inlineData.put(m.group(1), data.get(m.group(1)));
                }
                String inlineClaim = claim.substring(0, span[0])
                        + "{\"kind\":\"inline\",\"data\":" + joinObject(inlineData) + "}"
                        + claim.substring(span[1]);

                try (Verifier verifier = new Verifier()) {
                    String supplied = verifier.command(
                            "{\"cmd\":\"verify\",\"claim\":" + claim + ",\"data\":" + joinObject(data) + "}");
                    String inline = verifier.command("{\"cmd\":\"verify\",\"claim\":" + inlineClaim + "}");

                    Matcher suppliedHash = INPUTS_HASH.matcher(supplied);
                    Matcher inlineHash = INPUTS_HASH.matcher(inline);
                    assertTrue(suppliedHash.find() && inlineHash.find(), name + ": verdict carries inputs_hash");
                    assertNotEquals(suppliedHash.group(1), inlineHash.group(1),
                            name + ": inputs_hash binds the dataset reference");
                    assertEquals(suppliedHash.replaceFirst("\"inputs_hash\":\"-\""),
                            inlineHash.replaceFirst("\"inputs_hash\":\"-\""),
                            name + ": verdict differs between modes");
                }
            }
        }
    }
}
