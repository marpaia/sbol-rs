"""End-to-end tests for the idiomatic sbol Python bindings.

Run after `maturin develop`:  python -m unittest discover -s crates/sbol3-py/tests
"""

import unittest

import sbol
from sbol import Design, RdfFormat, SbolError


class TestConstruction(unittest.TestCase):
    def test_build_finish_serialize_validate(self):
        d = Design("https://example.org/lab")
        plac = d.promoter("pLac", "caatacg", description="LacI-repressible")
        b0034 = d.rbs("B0034", "ttgaac")
        tetr = d.cds("tetR", "atggtg")
        d.engineered_region("pLac_tu", [plac, b0034, tetr], description="a TU")

        doc = d.finish()
        doc.check()  # raises SbolError on validation failure

        # pLac, B0034, tetR, pLac_tu
        self.assertEqual(doc.component_count(), 4)
        self.assertEqual(doc.sequence_count(), 3)
        self.assertIn("pLac_tu", doc.component_display_ids())

        nt = doc.to_string(RdfFormat.NTriples)
        self.assertIn("pLac_tu", nt)
        self.assertIn("meets", nt)  # engineered_region ordering constraints

    def test_all_verbs(self):
        d = Design("https://example.org/lab")
        d.gene("araC", "atggtgaaacag")
        d.operator("op1", "aattgtgagc")
        d.mrna("gfp_mrna", "auggugagcaag")
        d.transcription_factor("tetR_tf", "MARLNRESVI")
        d.functional_component("LacI", description="LacI tetramer")
        doc = d.finish()
        doc.check()
        self.assertEqual(doc.component_count(), 5)

    def test_round_trip_through_rdf(self):
        d = Design("https://example.org/lab")
        d.promoter("pLac", "caatacg")
        doc = d.finish()
        turtle = doc.to_string(RdfFormat.Turtle)
        reparsed = sbol.Document.read_str(turtle, RdfFormat.Turtle)
        self.assertEqual(reparsed.component_count(), 1)

    def test_finish_consumes_the_arena(self):
        d = Design("https://example.org/lab")
        d.promoter("p", "aaa")
        d.finish()
        with self.assertRaises(SbolError):
            d.finish()

    def test_invalid_namespace_raises(self):
        with self.assertRaises(SbolError):
            Design("not a valid iri")


if __name__ == "__main__":
    unittest.main()
