# Dataset source breadth and external reference tracker

This note tracks issue-backed implementation for source locators, dataset format
capabilities, external-reference status, and package/export preview metadata.

## Source decisions

- `product-prd/source/docs/adr/0014-projects-reference-external-datasets-by-default.md`
- `product-prd/source/docs/adr/0015-v1-source-dataset-formats.md`
- `product-prd/source/docs/adr/0016-preserve-source-records-with-prepared-representations.md`
- `product-prd/source/docs/adr/0078-asset-definitions-do-not-silently-embed-heavy-artifacts.md`

Source ADRs stay unchanged. This tracker is the mutable implementation ledger
for the dataset/source breadth lane.

## Boundaries

- Project files reference external datasets and heavy artifacts by locator unless
  an explicit bundle/export workflow includes them.
- Source locator metadata is graph-owned product data, not viewer state.
- Source availability, missing-file checks, and bundle previews are reports,
  not persisted project state.
- Native cubic Bezier records remain canonical graph geometry; source-format
  support must not flatten them into stored polylines.
- Source-format capability records describe support status before parsers exist;
  they do not imply loader implementation.
- Package/export manifest writing is separate from preview metadata and must not
  copy files without explicit user intent.

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#196` | complete | Source locator metadata model | Adds serializable source locator metadata for local paths, URIs, recording queries, generated sources, and demo fallback while preserving legacy `source_path` compatibility. |
| `#198` | complete | Source format capability records | Adds graph-owned capability records for supported, planned v1, later compatibility, and deferred source formats from ADR-0015 without adding parsers. |
| `#200` | complete | Source external-reference status reports | Adds non-persistent status reports for not-external, local available, local missing, URI/unverified, and recording-query source references and surfaces them in source metadata UI. |
| `#202` | complete | Source bundle preview metadata | Adds preview-only source bundle metadata for inclusion status, expected local size, missing references, remaining external references, and reproducibility warnings without copying or hashing. |
| `#204` | complete | Focused dataset/source tracker | Adds this tracker and points the main PRD/ADR status ledger at it. |
| `#206` | complete | Source package manifest preview records | Adds serializable preview records for package/export manifests with source role, original locator, bundled path placeholder, expected size, hash availability, provenance, and external status without writing or copying files. |
| `#208` | complete | Source format inference reports | Adds graph-owned source format inference reports from locator suffixes to ADR-backed capability records, including generated, live, unknown, supported, planned, later compatibility, and deferred statuses without adding parsers. |
| `#210` | complete | External source reference action hints | Adds graph-owned recommended and secondary action hints for generated, local available, local missing, URI, and recording-query sources without performing OS, clipboard, relink, package, or parser actions. |

## Next implementation candidates

1. Package/export manifest writing.
   - Build on `#202` bundle metadata and `#206` manifest preview records.
   - Write manifest records only when an explicit package/export action exists.
   - Promote preview placeholders to written manifest entries only after package
     actions have copied files or chosen reference-only entries.
   - Keep content hashing and copying explicit, bounded, and testable.

2. Source-format import breadth.
   - Build on `#198` capability records and `#208` inference reports.
   - Add one parser family at a time.
   - Keep source records and prepared representations inspectable.
   - Avoid GIS/CRS expansion unless a source-specific issue explicitly reopens
     that product scope.

3. External-reference UI actions.
   - Build on `#200`, `#202`, and `#210`.
   - Show missing/includable/reference-only status near source inspection and
     future package/export flows.
   - Promote action hints to actual UI commands only behind explicit workflows
     such as relink, clipboard copy, local reveal, or package/export.
   - Do not silently embed large data from asset definitions or source nodes.

## Maintenance

- Add every dataset/source issue here when it is created.
- Change `planned` to `complete` only after the PR is merged and the issue is
  verified closed.
- If a future ADR changes product intent, update this tracker and the main
  implementation status ledger rather than rewriting the historical ADR text.
