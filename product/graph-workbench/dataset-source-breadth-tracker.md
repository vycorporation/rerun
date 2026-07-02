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
| `#212` | complete | Explicit source package manifest writing | Adds an explicit native workbench action and model-owned JSON write seam for source package manifests without copying files, computing hashes, relinking sources, or embedding heavy artifacts. |
| `#214` | complete | Source package manifest inclusion choices | Adds runtime/action-scoped include-available versus reference-only choices for eligible local source manifest artifacts without copying, hashing, relinking, or persisting choice state in sidecar JSON. |
| `#244` | complete | Source gallery view PRD | Adds a product PRD for a Houdini workbench gallery that browses local or URL-backed source collections, renders image thumbnails or generic typed thumbnails, and defines explicit follow-up actions for Rerun views and graph source nodes. |
| `#245` | complete | Source gallery indexing model | Normalizes bounded local paths, explicit URLs, and manifest-like source lists into stable gallery item records with locator, kind, capability, availability, and thumbnail intent without live network fetches or graph mutation. |
| `#246` | complete | Source gallery thumbnail intents | Distinguishes image thumbnail intents from generic typed thumbnails for tables, polygon tables, recordings, point clouds, manifests, missing entries, unknown sources, and runtime-only thumbnail cache state. |
| `#247` | complete | Source Gallery workbench view | Registers a movable Gallery workbench view that presents source entry controls, manifest input, filtered thumbnail tiles, selection, and selected-item metadata. |
| `#248` | complete | Source-gallery open-in-Rerun actions | Adds explicit selected-item actions that route image and recording sources through Rerun's native file/URL loader while leaving missing, live, generated, manifest, and unsupported data sources disabled with explanatory status. |
| `#249` | complete | Source-gallery source-node actions | Adds explicit selected-entry and checked-entry actions that create graph-owned source nodes or source collections with durable locator metadata, undo/redo support, and sidecar persistence without embedding source contents or thumbnails. |
| `#262` | complete | Source package writes with copied local artifacts and hashes | Adds an explicit native package write action that creates a package directory, copies eligible local source artifacts to manifest-owned relative paths, records deterministic content hashes, and leaves missing, remote, generated, live, and reference-only artifacts external with diagnostics. |
| `#264` | complete | Polygon coordinate CSV import | Adds a graph-owned CSV/TSV import path for headered polygon coordinate rows, updates source metadata/capability reporting, and preserves the current source geometry when malformed coordinate files fail to load. |

## Next implementation candidates

1. Source gallery browsing.
   - Build on `#196`, `#198`, `#200`, `#202`, `#208`, and `#210`.
   - Start with a bounded gallery source model that reports item identity,
     locator, source kind, capability, availability, and thumbnail intent.
   - Treat URL support as direct file URLs or explicit manifests before
     provider-specific browsing.
   - Keep decoded thumbnails and fetch attempts out of durable graph sidecar
     state.
   - `#248` routes supported image and recording gallery items through Rerun's
     native file/URL loader without creating graph nodes or copying files.

2. Package/export manifest writing.
   - Build on `#202` bundle metadata and `#206` manifest preview records.
   - `#212` writes manifest records only when an explicit package/export action
     exists.
   - `#214` models explicit include-versus-reference choices before any file
     copy or hash step.
   - `#262` writes explicit package directories for eligible local artifacts and
     records copied-file hashes without changing graph sidecar state.
   - Promote preview placeholders to written manifest entries only after package
     actions have copied files or chosen reference-only entries.
   - Keep content hashing and copying explicit, bounded, and testable.

3. Source-format import breadth.
   - Build on `#198` capability records and `#208` inference reports.
   - Add one parser family at a time.
   - `#264` adds the first CSV/TSV polygon coordinate parser for explicit
     `x0,y0,x1,y1,x2,y2` style coordinate columns.
   - Keep source records and prepared representations inspectable.
   - Avoid GIS/CRS expansion unless a source-specific issue explicitly reopens
     that product scope.

4. External-reference UI actions.
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
