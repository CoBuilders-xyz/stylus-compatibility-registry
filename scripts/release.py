"""Version checks and portable release archives; uses only Python's standard library."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
)
PACKAGES = ("stylus-compat-core", "stylus-registry")


def version(root=ROOT):
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    value = manifest["workspace"]["package"]["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", value):
        raise ValueError("Unsupported release version")
    return value


def validate(tag=None, root=ROOT):
    value = version(root)
    if tag is not None and tag != f"v{value}":
        raise ValueError(f"Tag must be v{value}, received {tag!r}")
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    dependency = manifest["workspace"]["dependencies"]["stylus-compat-core"]
    if dependency["version"] != f"={value}":
        raise ValueError("CLI dependency must pin the released core version")
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    for name in PACKAGES:
        if not any(p["name"] == name and p["version"] == value for p in lock["package"]):
            raise ValueError(f"Cargo.lock is not updated for {name}")
    if f"## {value}\n" not in (root / "CHANGELOG.md").read_text(encoding="utf-8"):
        raise ValueError("Add a changelog entry for the release version")
    return value


def archive_name(target, root=ROOT):
    suffix = ".zip" if target.endswith("windows-msvc") else ".tar.gz"
    return f"stylus-registry-v{version(root)}-{target}{suffix}"


def bundle(target=None, root=ROOT):
    value = validate(root=root)
    dist = root / "dist"
    dist.mkdir(exist_ok=True)
    stem = f"stylus-registry-v{value}-{target}" if target else f"stylus-registry-data-v{value}"
    name = archive_name(target, root) if target else f"{stem}.tar.gz"
    with tempfile.TemporaryDirectory(prefix="stage-", dir=dist) as temporary:
        stage = Path(temporary) / stem
        stage.mkdir()
        for filename in ("README.md", "LICENSE", "CHANGELOG.md", "CONTRIBUTING.md"):
            shutil.copy2(root / filename, stage / filename)
        for directory in ("data", "docs"):
            shutil.copytree(root / directory, stage / directory)
        shutil.copytree(root / "fixtures", stage / "fixtures", ignore=shutil.ignore_patterns("target", "Cargo.lock"))
        example = stage / ".github/examples"
        example.mkdir(parents=True)
        shutil.copy2(root / ".github/examples/stylus-check.yml", example)
        if target:
            executable = "stylus-registry.exe" if target.endswith("windows-msvc") else "stylus-registry"
            shutil.copy2(root / "target" / target / "release" / executable, stage / executable)
        info = {
            "version": value,
            "target": target or "registry-data",
            "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True, encoding="utf-8").strip(),
            "rustc": subprocess.check_output(["rustc", "--version"], cwd=root, text=True, encoding="utf-8").strip(),
        }
        (stage / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        output = dist / name
        if name.endswith(".zip"):
            with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
                for path in sorted(stage.rglob("*")):
                    if path.is_file():
                        archive.write(path, path.relative_to(Path(temporary)))
        else:
            with tarfile.open(output, "w:gz") as archive:
                archive.add(stage, arcname=stem)
    print(output)
    return output


def verify_bundle(target, root=ROOT):
    """Exercise the distributed executable and data, without a second network build."""
    archive = root / "dist" / archive_name(target, root)
    with tempfile.TemporaryDirectory(prefix="verify-", dir=root / "target") as temporary:
        destination = Path(temporary)
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as zipped:
                zipped.extractall(destination)
        else:
            with tarfile.open(archive) as tar:
                tar.extractall(destination, filter="data")
        stage = destination / f"stylus-registry-v{version(root)}-{target}"
        executable = stage / ("stylus-registry.exe" if target.endswith("windows-msvc") else "stylus-registry")
        actual = subprocess.check_output([str(executable), "--version"], text=True, encoding="utf-8").strip()
        if actual != f"stylus-registry {version(root)}":
            raise ValueError(f"Wrong packaged executable version: {actual}")
        env = dict(os.environ, STYLUS_COMPAT_CARGO="stylus-registry-nonexistent-cargo-for-archive-test")
        result = subprocess.run([str(executable), "check", "tokio", "--data-dir", str(stage / "data"), "--json"], env=env, check=True, capture_output=True, text=True, encoding="utf-8", timeout=30)
        report = json.loads(result.stdout)
        if not any("requires std, threads, I/O polling" in item["message"] for item in report["results"]):
            raise ValueError("Packaged registry data did not load")
        for path in ("LICENSE", "docs/usage.md", "docs/architecture.md", "docs/fellowship-report.md", "BUILD-INFO.json"):
            if not (stage / path).is_file():
                raise ValueError(f"Missing release file: {path}")
    print(f"Verified executable, registry and documentation in {archive.name}")


def checksums(root=ROOT):
    dist = root / "dist"
    paths = sorted(p for p in dist.iterdir() if p.name.endswith((".tar.gz", ".zip")))
    expected = {archive_name(target, root) for target in TARGETS}
    expected.add(f"stylus-registry-data-v{version(root)}.tar.gz")
    if {p.name for p in paths} != expected:
        raise ValueError("Release must contain exactly four native archives and one data archive")
    lines = [f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n" for path in paths]
    (dist / "SHA256SUMS").write_text("".join(lines), encoding="utf-8")


def notes(root=ROOT):
    value = validate(root=root)
    changes = (root / "CHANGELOG.md").read_text(encoding="utf-8").split(f"## {value}\n", 1)[1].split("\n## ", 1)[0].strip()
    url = f"https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/v{value}"
    # Relative changelog links are made usable on a GitHub release page.
    changes = changes.replace("](docs/", f"]({url}/docs/")
    text = f"""{changes}

## Install with Cargo

```sh
cargo install stylus-registry --version {value} --locked
rustup target add wasm32-unknown-unknown
stylus-registry check tiny-keccak --features keccak
```

Cargo installs the executable; use the data archive below for `--data-dir`.
Native executable archives include the data, license, docs and build information.
See [installation]({url}/docs/usage.md), [architecture]({url}/docs/architecture.md),
[known limitations]({url}/docs/limitations.md), and the [fellowship report]({url}/docs/fellowship-report.md).

Checksums are in `SHA256SUMS`. This beta's scores are heuristics, not deployment certification.
"""
    (root / "dist/RELEASE_NOTES.md").write_text(text, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("validate")
    check.add_argument("--tag")
    for name in ("bundle", "verify-bundle"):
        command = sub.add_parser(name)
        command.add_argument("--target", choices=TARGETS, required=True)
    sub.add_parser("data-bundle")
    sub.add_parser("checksums")
    sub.add_parser("notes")
    args = parser.parse_args()
    if args.command == "validate":
        value = validate(args.tag)
        print(f"version={value}\nprerelease={'true' if '-' in value else 'false'}")
    elif args.command == "bundle":
        bundle(args.target)
    elif args.command == "verify-bundle":
        verify_bundle(args.target)
    elif args.command == "data-bundle":
        bundle()
    elif args.command == "checksums":
        checksums()
    else:
        notes()


if __name__ == "__main__":
    main()
