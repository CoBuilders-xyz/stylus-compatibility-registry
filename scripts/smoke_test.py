"""Require real WASM compilation and exercise the installed CLI's documented paths."""

import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run_json(binary, arguments):
    result = subprocess.run([str(binary), *arguments, "--json"], capture_output=True, text=True, encoding="utf-8", timeout=180)
    if result.returncode:
        raise RuntimeError(f"CLI failed ({result.returncode}): {result.stderr}")
    return json.loads(result.stdout)


def require_compilation(report):
    wasm = next(item for item in report["results"] if item["check_name"] == "wasm_target")
    if wasm["severity"] != "Pass" or "compiles for wasm32-unknown-unknown" not in wasm["message"]:
        raise AssertionError(f"Actual compilation was required, received {wasm}")
    if len(report["results"]) != 6:
        raise AssertionError("Expected all six checks")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    subprocess.run([str(binary), "--help"], check=True, capture_output=True)
    data = str(ROOT / "data")
    hex_report = run_json(binary, ["check", "hex", "--version", "0.4.3", "--features", "serde", "--no-default-features", "--data-dir", data])
    require_compilation(hex_report)
    if hex_report["crate_info"]["default_features"] or hex_report["crate_info"]["features"] != ["serde"]:
        raise AssertionError("Dependency feature settings were lost")
    project = run_json(binary, ["check-deps", "--manifest", str(ROOT / "fixtures/release-smoke/Cargo.toml"), "--data-dir", data, "--strict"])
    if project["error_count"] != 0 or project["warning_count"] != 0 or len(project["crate_reports"]) != 2:
        raise AssertionError(f"Unexpected smoke fixture report: {project}")
    for report in project["crate_reports"]:
        require_compilation(report)
    failure = run_json(binary, ["check", "tiny-keccak", "--version", "2.0.2", "--data-dir", data])
    wasm = next(item for item in failure["results"] if item["check_name"] == "wasm_target")
    if wasm["severity"] != "Error" or "hash function" not in wasm["message"]:
        raise AssertionError(f"Expected an actual missing-feature compilation failure: {wasm}")
    print("Smoke passed: real WASM success and failure, features, registry, JSON and strict project mode")


if __name__ == "__main__":
    main()
