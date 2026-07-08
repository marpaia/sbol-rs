"""Phase 2 surface: algorithms, GenBank/FASTA I/O, from_document, generic
construction, and constants."""

import unittest

import sbol
from sbol import Design, RdfFormat

NS = "https://example.org/lab"


class TestAlgorithms(unittest.TestCase):
    def test_compute_sequences_concatenates(self):
        d = Design(NS)
        plac = d.promoter("pLac", "aaa")
        b0034 = d.rbs("B0034", "cc")
        tetr = d.cds("tetR", "gggg")
        d.engineered_region("pLac_tu", [plac, b0034, tetr])
        computed = d.finish().compute_sequences()

        computed.check()
        self.assertIn("pLac_tu", computed.component_display_ids())
        self.assertIn("aaaccgggg", computed.to_string(RdfFormat.NTriples))

    def test_compute_sequence_by_identity(self):
        d = Design(NS)
        p = d.promoter("p", "aaa")
        t = d.terminator("t", "ccc")
        d.engineered_region("tu", [p, t])
        computed = d.finish().compute_sequence(f"{NS}/tu")
        self.assertIn("aaaccc", computed.to_string(RdfFormat.NTriples))

    def test_expand_derivations_noop_without_any(self):
        d = Design(NS)
        d.promoter("p", "aaa")
        expanded = d.finish().expand_derivations()
        self.assertEqual(expanded.component_count(), 1)


class TestIO(unittest.TestCase):
    def test_fasta_round_trip(self):
        doc = sbol.read_fasta(">gene1 a gene\nACGTACGT\n>gene2\nTTTT\n", NS)
        self.assertEqual(doc.component_count(), 2)
        fasta = doc.to_fasta()
        self.assertIn("gene1", fasta)
        self.assertEqual(sbol.read_fasta(fasta, NS).component_count(), 2)

    def test_genbank_round_trip(self):
        d = Design(NS)
        d.promoter("pLac", "acgtacgtac")
        doc = d.finish()
        gb = doc.to_genbank()
        self.assertIn("LOCUS", gb)
        self.assertIn("acgt", gb.lower())
        self.assertGreaterEqual(sbol.read_genbank(gb, NS).component_count(), 1)


class TestGenericConstruction(unittest.TestCase):
    def test_generic_component_and_sequence_with_constants(self):
        d = Design(NS)
        seq = d.sequence("mySeq", "acgt", kind="dna")
        d.component(
            "myPart",
            types=[sbol.SBO_DNA],
            roles=[sbol.SO_PROMOTER],
            sequences=[seq],
        )
        doc = d.finish()
        doc.check()
        self.assertEqual(doc.component_count(), 1)
        self.assertEqual(doc.sequence_count(), 1)

    def test_constants_are_iris(self):
        self.assertTrue(sbol.SO_PROMOTER.startswith("https://identifiers.org/SO:"))
        self.assertTrue(sbol.SBO_DNA.startswith("https://identifiers.org/"))


class TestFromDocument(unittest.TestCase):
    def test_import_lookup_extend(self):
        d = Design(NS)
        a = d.promoter("a", "aaa")
        b = d.promoter("b", "ccc")
        d.engineered_region("tu", [a, b])
        doc = d.finish()
        base = doc.component_count()  # a, b, tu

        d2 = Design.from_document(doc)
        self.assertIsNotNone(d2.component_id(f"{NS}/tu"))
        self.assertIsNone(d2.component_id(f"{NS}/does_not_exist"))

        d2.promoter("c", "ggg")
        doc2 = d2.finish()
        doc2.check()
        self.assertEqual(doc2.component_count(), base + 1)


if __name__ == "__main__":
    unittest.main()
