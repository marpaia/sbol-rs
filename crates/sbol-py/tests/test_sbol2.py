"""SBOL2 support and SBOL2<->SBOL3 conversion in the single sbol package."""

import unittest

import sbol
from sbol import Design, RdfFormat, Sbol2Document

NS = "https://example.org/lab"


def sbol3_doc():
    d = Design(NS)
    d.promoter("pLac", "acgtacgt")
    d.cds("gfp", "atggtgagcaag")
    return d.finish()


class TestConversion(unittest.TestCase):
    def test_sbol3_downgrade_to_sbol2_text(self):
        text = sbol3_doc().to_sbol2(RdfFormat.Turtle)
        # SBOL2 vocabulary appears; SBOL3 Components become ComponentDefinitions.
        self.assertIn("ComponentDefinition", text)

    def test_downgrade_returns_sbol2_document(self):
        doc2 = sbol3_doc().downgrade()
        self.assertIsInstance(doc2, Sbol2Document)
        self.assertIn("ComponentDefinition", doc2.to_string(RdfFormat.Turtle))

    def test_round_trip_3_to_2_to_3(self):
        doc3 = sbol3_doc()
        base = doc3.component_count()  # 2
        doc2 = doc3.downgrade()
        back = doc2.to_sbol3()
        self.assertEqual(back.component_count(), base)

    def test_upgrade_module_function(self):
        text = sbol3_doc().to_sbol2(RdfFormat.NTriples)
        doc3 = sbol.upgrade_sbol2(text, RdfFormat.NTriples)
        self.assertGreaterEqual(doc3.component_count(), 2)


class TestSbol2Native(unittest.TestCase):
    def test_read_write_sbol2(self):
        # Produce SBOL2 by downgrading, then exercise the native SBOL2 surface.
        text = sbol3_doc().to_sbol2(RdfFormat.Turtle)
        doc2 = Sbol2Document.read_str(text, RdfFormat.Turtle)
        # Re-serialize to another format and reparse.
        nt = doc2.to_string(RdfFormat.NTriples)
        self.assertIn("ComponentDefinition", nt)
        reparsed = Sbol2Document.read_str(nt, RdfFormat.NTriples)
        self.assertIn("ComponentDefinition", reparsed.to_string(RdfFormat.Turtle))


if __name__ == "__main__":
    unittest.main()
