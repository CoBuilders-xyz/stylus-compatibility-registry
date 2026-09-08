import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import publish_crates
from release import PACKAGES, ROOT


class PublicationTests(unittest.TestCase):
    def setUp(self):
        base = ROOT / "target/release-tests"
        base.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=base)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        package = self.root / "target/package"
        package.mkdir(parents=True)
        self.checksums = {}
        for name in PACKAGES:
            contents = name.encode()
            (package / f"{name}-0.1.0-beta.2.crate").write_bytes(contents)
            self.checksums[name] = hashlib.sha256(contents).hexdigest()
        for target, replacement in (("ROOT", self.root), ("validate", lambda: "0.1.0-beta.2")):
            patcher = patch.object(publish_crates, target, replacement)
            patcher.start()
            self.addCleanup(patcher.stop)

    @patch.object(publish_crates, "existing_checksum", return_value=None)
    @patch.object(publish_crates.subprocess, "run")
    def test_dry_run_checks_both_packages_without_uploading(self, run, lookup):
        publish_crates.publish(dry_run=True)
        self.assertEqual(lookup.call_count, 2)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[0][:2], ["cargo", "package"])

    @patch.object(publish_crates, "existing_checksum")
    @patch.object(publish_crates.subprocess, "run")
    def test_mismatched_cli_stops_before_publishing_core(self, run, lookup):
        lookup.side_effect = [None, "different-checksum"]
        with self.assertRaisesRegex(RuntimeError, "different bytes"):
            publish_crates.publish()
        self.assertEqual(run.call_count, 1)

    @patch.object(publish_crates, "existing_checksum")
    @patch.object(publish_crates.subprocess, "run")
    def test_partial_publication_resumes_only_missing_cli(self, run, lookup):
        lookup.side_effect = [self.checksums[PACKAGES[0]], None]
        publish_crates.publish()
        self.assertEqual(run.call_count, 2)
        self.assertEqual(run.call_args.args[0], ["cargo", "publish", "--package", PACKAGES[1], "--locked"])

    @patch.object(publish_crates, "existing_checksum", return_value=None)
    @patch.object(publish_crates.subprocess, "run")
    def test_first_publication_uploads_in_dependency_order(self, run, lookup):
        publish_crates.publish()
        uploads = [call.args[0] for call in run.call_args_list[1:]]
        self.assertEqual(uploads, [["cargo", "publish", "--package", name, "--locked"] for name in PACKAGES])


if __name__ == "__main__":
    unittest.main()
