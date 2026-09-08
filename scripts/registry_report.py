"""Reproduce inventory metrics from a frozen release snapshot (Python 3.12)."""

import argparse
import hashlib
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / 'docs/reports/2026-09-08'


def analyze(snapshot_path):
    raw = snapshot_path.read_bytes()
    snapshot = json.loads(raw)
    records = []
    file_counts = {}
    for source in snapshot['sources']:
        content = source['content'].encode('utf-8')
        if hashlib.sha256(content).hexdigest() != source['sha256']:
            raise ValueError(f"Snapshot source checksum mismatch: {source['path']}")
        entries = tomllib.loads(source['content'])['crate']
        file_counts[source['path']] = len(entries)
        records.extend(entries)
    names = [entry['name'] for entry in records]
    if len(names) != len(set(names)):
        raise ValueError('Snapshot has duplicate names; report requires explicit override analysis')
    return {
        'release': snapshot['release'],
        'source_commit': snapshot['source_commit'],
        'snapshot_sha256': hashlib.sha256(raw).hexdigest(),
        'records_by_file': file_counts,
        'unique_crate_names': len(set(names)),
        'flags': {flag: sum(entry[flag] is True for entry in records)
                  for flag in ('requires_std', 'has_float', 'has_async')},
        'records_with_notes': sum(bool(entry.get('notes')) for entry in records),
        'records_with_alternative': sum(bool(entry.get('alternative')) for entry in records),
        'records_with_max_version': sum(bool(entry.get('max_version')) for entry in records),
        'crate_names': sorted(names),
    }


def serialized_metrics():
    return json.dumps(analyze(REPORT / 'snapshot.json'), indent=2, ensure_ascii=False) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Require committed metrics to match the snapshot')
    args = parser.parse_args()
    result = serialized_metrics()
    path = REPORT / 'metrics.json'
    if args.check:
        if path.read_text(encoding='utf-8') != result:
            raise ValueError('Committed metrics differ from the frozen snapshot')
        print('Frozen registry metrics reproduced exactly')
    else:
        path.write_text(result, encoding='utf-8')
        print(path)


if __name__ == '__main__':
    main()
