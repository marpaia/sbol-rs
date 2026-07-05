// Round-trip wrapper used by the SBOL 2 cross-implementation
// conformance harness. Reads an SBOL 2 RDF/XML file via libSBOLj and
// writes the parsed document back out in the requested serialization.
// By default the output goes to stdout; passing a third argument
// writes to a file.
//
// Invoked from inside the Docker container built by Dockerfile.
// libSBOLj writes the serialized document to stdout; any parser
// diagnostics go to stderr and are filtered by the regenerate binary.

import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStream;

import org.sbolstandard.core2.SBOLDocument;
import org.sbolstandard.core2.SBOLReader;
import org.sbolstandard.core2.SBOLWriter;

public class RoundTrip {
    public static void main(String[] args) throws Exception {
        if (args.length < 2) {
            System.err.println("usage: RoundTrip <input.xml> <format> [output]");
            System.err.println("formats: rdfxml, turtle, json");
            System.exit(2);
        }
        File input = new File(args[0]);
        String format = args[1].toLowerCase();

        OutputStream out = (args.length >= 3)
                ? new FileOutputStream(args[2])
                : System.out;

        // Read without imposing a URI prefix: the corpus fixtures carry
        // absolute URIs. Keep parsing lenient so a best-practice or
        // completeness diagnostic doesn't abort the round trip; the
        // Rust side owns the triple-set comparison.
        SBOLReader.setKeepGoing(true);
        SBOLDocument document = SBOLReader.read(input);

        switch (format) {
            case "rdfxml":
                SBOLWriter.write(document, out);
                break;
            case "turtle":
                SBOLWriter.write(document, out, SBOLDocument.TURTLE);
                break;
            case "json":
                SBOLWriter.write(document, out, SBOLDocument.JSON);
                break;
            default:
                System.err.println("unknown format: " + format);
                System.exit(2);
        }

        if (args.length >= 3) {
            out.close();
        }
    }
}
