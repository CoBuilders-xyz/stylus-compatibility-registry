import hashlib
from pathlib import Path
import tempfile
import unittest

from release import PACKAGES, ROOT, TARGETS, archive_name, checksums, validate
from smoke_test import require_compilation


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        base = ROOT / "target/release-tests"
        base.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=base)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion="0.1.0-beta.1"\n[workspace.dependencies]\nstylus-compat-core={version="=0.1.0-beta.1"}\n', encoding="utf-8")
        (self.root / "Cargo.lock").write_text(''.join(f'[[package]]\nname="{name}"\nversion="0.1.0-beta.1"\n' for name in PACKAGES), encoding="utf-8")
        (self.root / "CHANGELOG.md").write_text('# Changelog\n\n## 0.1.0-beta.1\n\nFirst beta.\n', encoding="utf-8")

    def test_refuses_wrong_tag_before_publication(self):
        self.assertEqual(validate("v0.1.0-beta.1", self.root), "0.1.0-beta.1")
        with self.assertRaises(ValueError):
            validate("v0.1.0", self.root)

    def test_refuses_stale_lockfile(self):
        path = self.root / "Cargo.lock"
        path.write_text(path.read_text(encoding="utf-8").replace('0.1.0-beta.1', '0.1.0'), encoding="utf-8")
        with self.assertRaises(ValueError):
            validate(root=self.root)

    def test_refuses_mismatched_core_dependency(self):
        path = self.root / "Cargo.toml"
        path.write_text(path.read_text(encoding="utf-8").replace('=0.1.0-beta.1', '=0.0.9'), encoding="utf-8")
        with self.assertRaises(ValueError):
            validate(root=self.root)

    def test_checksums_require_all_platforms_and_cover_archive_bytes(self):
        dist = self.root / "dist"
        dist.mkdir()
        with self.assertRaises(ValueError):
            checksums(self.root)
        names = [archive_name(target, self.root) for target in TARGETS]
        names.append("stylus-registry-data-v0.1.0-beta.1.tar.gz")
        for name in names:
            (dist / name).write_bytes(name.encode())
        checksums(self.root)
        lines = (dist / "SHA256SUMS").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 5)
        for line in lines:
            checksum, name = line.split("  ")
            self.assertEqual(checksum, hashlib.sha256((dist / name).read_bytes()).hexdigest())

    def test_smoke_does_not_accept_a_passing_fallback(self):
        report = {"results": [{"check_name": "wasm_target", "severity": "Pass", "message": "not in the blocklist"}]}
        with self.assertRaises(AssertionError):
            require_compilation(report)


if __name__ == "__main__":
    unittest.main()
