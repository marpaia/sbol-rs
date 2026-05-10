// Round-trip wrapper used by the cross-implementation conformance
// harness. Reads an SBOL 3 Turtle file via libSBOLj3 and writes the
// parsed document back out in the requested output format. By default
// the output goes to stdout; passing a third argument writes to file.
//
// Invoked from inside the Docker container built by Dockerfile.
// libSBOLj3 emits a few INFO-level log lines on startup (Hibernate
// Validator, SLF4J). Those go to stderr and are filtered by the
// regenerate binary; stdout holds only the serialized output.

import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStream;

import org.sbolstandard.core3.entity.SBOLDocument;
import org.sbolstandard.core3.io.SBOLFormat;
import org.sbolstandard.core3.io.SBOLIO;

public class RoundTrip {
    public static void main(String[] args) throws Exception {
        if (args.length < 2) {
            System.err.println("usage: RoundTrip <input.ttl> <format> [output]");
            System.err.println("formats: turtle, rdfxml, jsonld, ntriples");
            System.exit(2);
        }
        File input = new File(args[0]);
        SBOLFormat outputFormat = parseFormat(args[1]);

        OutputStream out = (args.length >= 3)
                ? new FileOutputStream(args[2])
                : System.out;

        // Source fixtures are always Turtle; the format arg controls the
        // output serialization only.
        SBOLDocument document = SBOLIO.read(input, SBOLFormat.TURTLE);
        SBOLIO.write(document, out, outputFormat);

        if (args.length >= 3) {
            out.close();
        }
    }

    private static SBOLFormat parseFormat(String name) {
        switch (name.toLowerCase()) {
            case "turtle":
                return SBOLFormat.TURTLE;
            case "rdfxml":
                return SBOLFormat.RDFXML;
            case "jsonld":
                return SBOLFormat.JSONLD;
            case "ntriples":
                return SBOLFormat.NTRIPLES;
            default:
                System.err.println("unknown format: " + name);
                System.exit(2);
                return null; // unreachable
        }
    }
}
