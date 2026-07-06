// Round-trip benchmark wrapper for libSBOLj (the SBOL 2 Java
// implementation).
//
// Reads an SBOL 2 RDF document, runs the configured number of warmup
// and timed (parse + serialize) iterations, and writes per-iteration
// nanosecond timings to a JSON file. The output-file convention keeps
// the timings clean even when libSBOLj's loggers print on startup.
//
// Invoked from inside the Docker container built by the sibling
// Dockerfile with the same positional protocol as the other bench
// drivers: <input> <parse_fmt> <serialize_fmt> <warmup> <iters>
// <output_json> [validate] [version]. libSBOLj auto-detects the input
// serialization, so the parse-format argument is bookkeeping only; the
// serialize-format argument selects the output serialization. The
// trailing validate and version arguments are ignored: this driver is
// SBOL 2 by construction and runs no validation phase.

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.FileWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Properties;

import org.sbolstandard.core2.SBOLDocument;
import org.sbolstandard.core2.SBOLReader;
import org.sbolstandard.core2.SBOLWriter;

public class Bench {

    public static void main(String[] args) throws Exception {
        if (args.length < 6) {
            System.err.println(
                "usage: Bench <input> <parse_fmt> <serialize_fmt> <warmup> <iters> <output_json>");
            System.exit(2);
        }

        Path inputPath = Paths.get(args[0]);
        String serializeFormat = serializeFormat(args[2]);
        int warmup = Integer.parseInt(args[3]);
        int iters = Integer.parseInt(args[4]);
        Path outputJson = Paths.get(args[5]);

        byte[] rdfBytes = Files.readAllBytes(inputPath);

        // The corpus fixtures carry absolute URIs; keep parsing lenient
        // so a best-practice or completeness diagnostic doesn't abort
        // the round trip. The Rust side owns triple-set comparison.
        SBOLReader.setKeepGoing(true);

        long[] parseNs = new long[iters];
        long[] serializeNs = new long[iters];
        long lastSerializedBytes = 0L;

        for (int i = 0; i < warmup; i++) {
            SBOLDocument doc = SBOLReader.read(new ByteArrayInputStream(rdfBytes));
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            write(doc, out, serializeFormat);
        }

        for (int i = 0; i < iters; i++) {
            long t0 = System.nanoTime();
            SBOLDocument doc = SBOLReader.read(new ByteArrayInputStream(rdfBytes));
            long t1 = System.nanoTime();
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            write(doc, out, serializeFormat);
            long t2 = System.nanoTime();
            parseNs[i] = t1 - t0;
            serializeNs[i] = t2 - t1;
            lastSerializedBytes = out.size();
        }

        String impl = "libsbolj";
        String version = libsboljVersion();
        StringBuilder json = new StringBuilder(1024 + iters * 16);
        json.append("{");
        appendStringField(json, "impl", impl).append(",");
        appendStringField(json, "version", version).append(",");
        appendStringField(json, "fixture", inputPath.toString()).append(",");
        appendStringField(json, "parse_format", args[1]).append(",");
        appendStringField(json, "serialize_format", args[2]).append(",");
        appendLongField(json, "warmup_iters", warmup).append(",");
        appendLongField(json, "measured_iters", iters).append(",");
        appendLongField(json, "serialized_bytes", lastSerializedBytes).append(",");
        appendLongArrayField(json, "parse_ns", parseNs).append(",");
        appendLongArrayField(json, "serialize_ns", serializeNs);
        json.append("}");

        try (FileWriter writer = new FileWriter(outputJson.toFile(), StandardCharsets.UTF_8)) {
            writer.write(json.toString());
        }
    }

    private static void write(SBOLDocument doc, ByteArrayOutputStream out, String format)
            throws Exception {
        switch (format) {
            case "rdfxml":
                SBOLWriter.write(doc, out);
                break;
            case "turtle":
                SBOLWriter.write(doc, out, SBOLDocument.TURTLE);
                break;
            default:
                System.err.println("unsupported serialize format: " + format);
                System.exit(2);
        }
    }

    // libSBOLj serializes RDF/XML (default), Turtle, and its own JSON.
    // The bench only drives RDF/XML and Turtle; JSON-LD and N-Triples
    // have no libSBOLj serializer.
    private static String serializeFormat(String name) {
        switch (name.toLowerCase()) {
            case "rdfxml":
                return "rdfxml";
            case "turtle":
                return "turtle";
            default:
                System.err.println("unsupported serialize format: " + name);
                System.exit(2);
                return null;
        }
    }

    private static String libsboljVersion() {
        Package pkg = SBOLDocument.class.getPackage();
        if (pkg != null) {
            String v = pkg.getImplementationVersion();
            if (v != null && !v.isEmpty()) {
                return v;
            }
        }
        try (var input = SBOLDocument.class.getResourceAsStream(
                "/META-INF/maven/org.sbolstandard/libSBOLj/pom.properties")) {
            if (input != null) {
                Properties props = new Properties();
                props.load(input);
                String v = props.getProperty("version");
                if (v != null) {
                    return v;
                }
            }
        } catch (Exception ignored) {
            // fall through
        }
        return "unknown";
    }

    private static StringBuilder appendStringField(StringBuilder json, String name, String value) {
        json.append('"').append(escape(name)).append("\":\"").append(escape(value)).append('"');
        return json;
    }

    private static StringBuilder appendLongField(StringBuilder json, String name, long value) {
        json.append('"').append(escape(name)).append("\":").append(value);
        return json;
    }

    private static StringBuilder appendLongArrayField(StringBuilder json, String name, long[] values) {
        json.append('"').append(escape(name)).append("\":[");
        for (int i = 0; i < values.length; i++) {
            if (i > 0) {
                json.append(',');
            }
            json.append(values[i]);
        }
        json.append(']');
        return json;
    }

    private static String escape(String s) {
        StringBuilder out = new StringBuilder(s.length() + 8);
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '\\':
                    out.append("\\\\");
                    break;
                case '"':
                    out.append("\\\"");
                    break;
                case '\n':
                    out.append("\\n");
                    break;
                case '\r':
                    out.append("\\r");
                    break;
                case '\t':
                    out.append("\\t");
                    break;
                default:
                    if (c < 0x20) {
                        out.append(String.format("\\u%04x", (int) c));
                    } else {
                        out.append(c);
                    }
            }
        }
        return out.toString();
    }
}
