# Maintaining the documentation

## Presentation and source

This site uses Docsify with sidebar navigation, browser-local search, appearance controls, a page index and Mermaid diagrams. Documentation remains ordinary Markdown under `docs/`.

The intended public URL is [cobuilders-xyz.github.io/stylus-compatibility-registry](https://cobuilders-xyz.github.io/stylus-compatibility-registry/). GitHub Pages serves the output of the **Docs** workflow from `main`. The website tracks documentation changes; the dated report stays frozen unless an explicitly documented correction is made.

## Edit or add a page

Edit its Markdown file. Add a new page to `docs/_sidebar.md` and to `search.paths` in `docs/site.js`. Keep page links relative, and use GitHub URLs for source outside `docs/`. The checker rejects links that would escape the deployed documentation directory.

JSON evidence links are file downloads rather than viewer routes. The source link in each footer points to the corresponding document on `main`. Light, Dark and System appearance use browser storage; no telemetry or tracking service is configured.

## Preview and validate

```sh
python3 scripts/registry_report.py --check
python3 scripts/docs.py check
python3 scripts/docs.py build
python3 -m http.server 8043 --directory target/docs-site
```

The build creates the site and download archive under `target/`, inside this repository. Check navigation, search, the page index, mobile menu, themes, evidence downloads and the Mermaid diagram before merging.

`index.html` pins Docsify and its search plugin to 5.0.0; `site.js` pins Mermaid to 11.17.2. These dependencies load from jsDelivr. The documentation download contains Markdown, evidence, license, the report analyzer and build provenance; it is not a vendored offline copy of the browser dependencies.

## Automatic publication

The Docs workflow runs on documentation changes in pull requests and `main`, and supports manual dispatch. All runs validate and produce a reviewable site artifact. Deployment requires `main` and uses `actions/configure-pages`, `upload-pages-artifact` and `deploy-pages` with the `github-pages` environment.

One-time repository configuration: set **Settings → Pages → Source → GitHub Actions**. The deployment job has `pages: write` and `id-token: write`; PR validation only has read access. See [GitHub's custom workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).

## Attach the supplement to beta.2

After the documentation PR is merged and the Docs workflow succeeds, download its `documentation-site` artifact. The `downloads/` directory contains `stylus-registry-docs-v0.1.0-beta.2.tar.gz` and `SHA256SUMS`. Its `BUILD-INFO.json` records both the documentation commit and the original beta commit. The supplement targets the published release recorded in the frozen publication evidence, independently of future workspace version bumps.

A maintainer can attach this archive and its checksum as additional assets to the existing GitHub prerelease, naming them with the documentation commit to distinguish revisions. Add a documentation-site link to the release notes. Do not replace the original binaries, data bundle or original `SHA256SUMS`, and do not move the beta tag. Subsequent executable releases automatically include the expanded `docs/` directory through the existing release workflow.

## Frozen reports

`scripts/registry_report.py --check` reproduces counts from `docs/reports/2026-09-08/snapshot.json`. Future registry data must get a new dated report instead of silently changing this beta's baseline. Capture commands/toolchain for new CLI examples and retain limitations with any published findings.
