"""Publish in dependency order; only resume an existing version if its bytes match."""

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


def main():
    value = validate()
    # Workspace packaging resolves both unpublished crates in a temporary registry
    # and verifies that they build before the first irreversible upload.
    subprocess.run(["cargo", "publish", "--workspace", "--dry-run", "--locked"], cwd=ROOT, check=True)
    for name in PACKAGES:
        archive = ROOT / "target/package" / f"{name}-{value}.crate"
        expected = hashlib.sha256(archive.read_bytes()).hexdigest()
        current = existing_checksum(name, value)
        if current is not None:
            if current != expected:
                raise RuntimeError(f"{name} {value} already exists with different bytes; do not overwrite or reuse this version")
            print(f"{name} {value} already published with matching checksum; resuming", flush=True)
            continue
        subprocess.run(["cargo", "publish", "--package", name, "--locked"], cwd=ROOT, check=True)
    print("Both crates published or verified as identical existing releases", flush=True)


if __name__ == "__main__":
    main()
