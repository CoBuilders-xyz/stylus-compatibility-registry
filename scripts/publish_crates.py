"""Publish in dependency order; only resume an existing version if its bytes match."""

import argparse
import hashlib
import json
import subprocess
import urllib.error
import urllib.request

from release import PACKAGES, ROOT, validate


def existing_checksum(name, version):
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}/{version}",
        headers={"User-Agent": "stylus-registry-release (github.com/CoBuilders-xyz/stylus-compatibility-registry)"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)["version"]["checksum"]
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise


def publish(dry_run=False, prepare_only=False):
    value = validate()
    # Workspace packaging resolves both unpublished crates in a temporary registry
    # and verifies that they build before the first irreversible upload.
    # Unlike `cargo publish --dry-run`, `cargo package` retains the final
    # archives in target/package after verifying the workspace.
    subprocess.run(["cargo", "package", "--workspace", "--locked", "--target-dir", str(ROOT / "target")], cwd=ROOT, check=True)
    prepared = {}
    for name in PACKAGES:
        archive = ROOT / "target/package" / f"{name}-{value}.crate"
        prepared[name] = hashlib.sha256(archive.read_bytes()).hexdigest()
    if prepare_only:
        # PR commits have different VCS metadata from an already published
        # version. Verify archive creation without testing publication identity.
        print(f"Prepared and verified archives: {', '.join(prepared)}", flush=True)
        return
    pending = []
    for name, expected in prepared.items():
        current = existing_checksum(name, value)
        if current is not None:
            if current != expected:
                raise RuntimeError(f"{name} {value} already exists with different bytes; do not overwrite or reuse this version")
            print(f"{name} {value} already published with matching checksum; resuming", flush=True)
            continue
        pending.append(name)
    if dry_run:
        print(f"Publication preflight passed; would publish: {', '.join(pending) or 'nothing'}", flush=True)
        return
    for name in pending:
        subprocess.run(["cargo", "publish", "--package", name, "--locked"], cwd=ROOT, check=True)
    print("Both crates published or verified as identical existing releases", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="Verify packages and existing versions without uploading")
    mode.add_argument("--prepare-only", action="store_true", help="Build and checksum archives for PR validation; do not query or publish registry versions")
    args = parser.parse_args()
    publish(dry_run=args.dry_run, prepare_only=args.prepare_only)
