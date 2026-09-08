"""Validate and package the Markdown documentation site without build dependencies."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
from urllib.parse import unquote, urlsplit

from registry_report import serialized_metrics, REPORT

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / 'docs'
SITE = ROOT / 'target/docs-site'


def version():
    # The supplement targets an already published release, independently of
    # a future workspace version bump whose tag may not exist yet.
    return json.loads((REPORT / 'publication.json').read_text(encoding='utf-8'))['release']['tag'].removeprefix('v')


def check():
    generated = {f'downloads/stylus-registry-docs-v{version()}.tar.gz', 'downloads/SHA256SUMS'}
    count = 0
    for page in DOCS.rglob('*.md'):
        text = re.sub(r'```.*?```', '', page.read_text(encoding='utf-8'), flags=re.S)
        for target in re.findall(r'\]\(([^)\s]+)(?:\s+"[^"]*")?\)', text):
            url = urlsplit(target)
            if url.scheme or url.netloc or not url.path:
                continue
            relative = unquote(url.path)
            path = ((DOCS / relative.lstrip('/')) if relative.startswith('/') else (page.parent / relative)).resolve()
            if not path.is_relative_to(DOCS):
                raise ValueError(f'{page.relative_to(ROOT)}: link escapes the site: {target}')
            if not path.exists() and path.relative_to(DOCS).as_posix() not in generated:
                raise ValueError(f'{page.relative_to(ROOT)}: missing link: {target}')
            count += 1
    script = (DOCS / 'site.js').read_text(encoding='utf-8')
    section = script.split('paths: [', 1)[1].split(']', 1)[0]
    routes = re.findall(r"'(/[^']*)'", section)
    expected = {'/' if p.name == 'README.md' and p.parent == DOCS else '/' + p.relative_to(DOCS).with_suffix('').as_posix()
                for p in DOCS.rglob('*.md') if not p.name.startswith('_')}
    if set(routes) != expected:
        raise ValueError(f'Search routes differ from pages: {set(routes) ^ expected}')
    sidebar = (DOCS / '_sidebar.md').read_text(encoding='utf-8')
    navigation = {r.removesuffix('.md') for r in re.findall(r'\]\((/[^)]*)\)', sidebar)}
    if navigation != expected:
        raise ValueError(f'Sidebar routes differ from pages: {navigation ^ expected}')
    if (REPORT / 'metrics.json').read_text(encoding='utf-8') != serialized_metrics():
        raise ValueError('Frozen inventory metrics differ; run registry_report.py --check')
    print(f'Documentation validated: {len(expected)} pages, {count} local links, exact frozen inventory')


def build():
    check()
    if SITE.exists():
        shutil.rmtree(SITE)
    shutil.copytree(DOCS, SITE)
    downloads = SITE / 'downloads'
    downloads.mkdir()
    name = f'stylus-registry-docs-v{version()}'
    info = {
        'documentation_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'documentation_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=normal'], cwd=ROOT, text=True).strip()),
        'release': f'v{version()}',
        'release_commit': json.loads((REPORT / 'publication.json').read_text(encoding='utf-8'))['source_commit'],
    }
    provenance = ROOT / 'target/docs-build-info.json'
    provenance.write_text(json.dumps(info, indent=2) + '\n', encoding='utf-8')
    archive = downloads / f'{name}.tar.gz'
    with tarfile.open(archive, 'w:gz') as tar:
        tar.add(DOCS, arcname=f'{name}/docs')
        tar.add(ROOT / 'LICENSE', arcname=f'{name}/LICENSE')
        tar.add(ROOT / 'scripts/registry_report.py', arcname=f'{name}/scripts/registry_report.py')
        tar.add(provenance, arcname=f'{name}/BUILD-INFO.json')
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    (downloads / 'SHA256SUMS').write_text(f'{checksum}  {archive.name}\n', encoding='utf-8')
    print(f'Site and documentation download: {SITE}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['check', 'build'])
    args = parser.parse_args()
    build() if args.command == 'build' else check()
