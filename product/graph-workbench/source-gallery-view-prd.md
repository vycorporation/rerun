# Source gallery view PRD

## Problem Statement

Users often start visual analysis from a directory, manifest, object-store prefix, or URL that contains many files rather than from a single already-loaded recording. Today the Houdini graph workbench can model source references, source capability reports, bundle previews, asset galleries, and Rerun output targets, but it does not give users a first-class visual gallery for browsing source collections before deciding what to inspect or promote into the graph.

This is especially painful for image-heavy work. A user may have hundreds or thousands of screenshots, byteplot rasters, microscopy tiles, render frames, model outputs, or generated images and needs to scan thumbnails quickly. The same collection may also contain non-image files such as Parquet, CSV, SQLite, FlatGeobuf, LAS/LAZ, Rerun recordings, JSON manifests, model outputs, or polygon tables. Those non-image files still need visible placeholders, lightweight metadata, and useful actions even when they cannot produce image thumbnails.

From the user's perspective, the missing product surface is a gallery they can point at a local source or URL, see thumbnails or generic typed placeholders, filter and select items, and then choose what to do next. The first version should make browsing reliable and inspectable. Sending a selected item directly into the best Rerun view or creating a graph source node from it is valuable, but it should build on the source model rather than silently importing or embedding large files.

## Solution

Add a source gallery workbench view for browsing external source collections. The view accepts local paths, explicit file URLs, HTTP(S) URLs, and manifest-like source lists. It presents a thumbnail grid with stable item identity, file/source metadata, availability status, source kind, and capability status.

Image-like entries render visual thumbnails when the source is available and safe to preview. Non-image entries render generic typed thumbnails that communicate the source family, such as table, polygon table, recording, point cloud, manifest, unknown file, missing reference, or remote URL. The generic thumbnails are not decorative: they should show enough type and status information to help users distinguish a Parquet table with polygon records from an image, a missing local file, or an unverified remote URL.

The gallery should be a Houdini workbench surface, not a separate operating-system file browser. It should feel native to the graph workflow: users can browse, filter, select, inspect metadata, and then take explicit actions. Primary v1 value is source discovery and visual triage. Secondary actions may include opening an image in a suitable 2D Rerun view, opening recordings through the existing Rerun pathways, creating a graph source node, or creating a source collection node that references the selected items.

The product should preserve the existing external-source rule: project files reference source datasets and heavy artifacts by locator unless the user explicitly packages or copies them through a separate bundle/export workflow.

## User Stories

1. As a visual analyst, I want to point the gallery at a local folder, so that I can scan many candidate files without importing them one at a time.
2. As a visual analyst, I want to point the gallery at an explicit image URL, so that I can preview remote images before deciding whether to use them.
3. As a visual analyst, I want to point the gallery at a manifest URL, so that I can browse a curated remote collection without cloning or copying the whole collection first.
4. As a malware researcher, I want byteplot image files to appear as thumbnails, so that I can visually compare texture patterns quickly.
5. As a malware researcher, I want Parquet files that hold polygon records to appear as typed placeholders, so that I can recognize usable geometry sources even when they are not images.
6. As a computer vision researcher, I want generated frames and model output images to appear in a dense grid, so that I can compare runs visually.
7. As a dataset curator, I want non-image files to have distinct generic thumbnails, so that a table, recording, manifest, point cloud, and unknown file do not all look identical.
8. As a dataset curator, I want missing files and failed remote reads to have clear status, so that I can fix locators before building graph workflows.
9. As a user browsing a remote collection, I want the app to avoid surprise recursive crawling, so that opening a URL does not create an unbounded network operation.
10. As a user browsing a large directory, I want thumbnails to load progressively, so that I can start scanning before every item is processed.
11. As a user browsing many files, I want image thumbnails to be cached as lightweight derived previews, so that reopening the gallery is fast without embedding full source files in the project.
12. As a user browsing sensitive data, I want remote fetches to be explicit and visible, so that the app does not contact remote services unexpectedly.
13. As a user browsing a mixed folder, I want filtering by source type, name, extension, status, and capability, so that I can focus on images, tables, recordings, or failed entries.
14. As a user browsing many thumbnails, I want keyboard and pointer selection to feel like other workbench surfaces, so that the gallery supports fast triage.
15. As a user browsing thumbnails, I want a metadata inspector for the selected item, so that I can see locator, source kind, dimensions when known, file size when known, and capability status.
16. As a user browsing image entries, I want to right-click and open an image in a suitable Rerun 2D view, so that I can inspect the item at full fidelity without building a graph first.
17. As a user browsing recording entries, I want to right-click and open a recording through the native Rerun pathway, so that recordings use existing viewer behavior.
18. As a graph user, I want to right-click and create a source node from one selected item, so that the item becomes graph-owned project intent without embedding the file contents.
19. As a graph user, I want to create a source collection node from multiple selected items, so that a graph can operate over a selected subset.
20. As a graph user, I want source nodes created from gallery entries to retain source locator metadata, so that bundle previews and external-reference reports still work.
21. As a graph user, I want generated source nodes to be visible in the graph, so that gallery actions do not hide project behavior in viewer-only state.
22. As a graph user, I want the gallery to coexist with the asset gallery, so that project assets and source files remain different concepts.
23. As a graph user, I want image entries to be distinguishable from source datasets, so that I do not confuse previewable images with graph-ready polygon or table sources.
24. As a data engineer, I want manifest-backed gallery entries to keep original locators, so that downstream packaging and reproducibility warnings remain accurate.
25. As a data engineer, I want the gallery to preserve URL strings rather than copying content automatically, so that remote collections stay lightweight until explicitly packaged.
26. As a user working offline, I want remote entries to remain visible as unverified or unavailable, so that saved source lists can still be inspected.
27. As a user working with local files, I want directory changes to be refreshed explicitly, so that the gallery does not rewrite project state behind my back.
28. As a user reviewing results, I want thumbnails to show enough provenance to separate source images from derived previews, so that I know what I am looking at.
29. As a user with a very large collection, I want the gallery to impose page, limit, or sampling controls, so that accidental large reads do not freeze the viewer.
30. As a user with a project workbench layout, I want Gallery to be a normal movable Rerun workbench view, so that it can be docked beside Graph, Data, Outputs, Assets, or Display.
31. As a user saving a workbench, I want the Gallery layout to save like other workbench views, so that my triage layout can be reused.
32. As a user importing from a URL, I want clear error messages for unsupported listing formats, so that I know whether to provide a manifest or direct file URL.
33. As a user browsing non-image geometry files, I want capability status to say whether a parser exists, is planned, or is only a placeholder, so that generic thumbnails do not imply hidden loader support.
34. As a product maintainer, I want gallery source logic to reuse source locator and capability concepts, so that source browsing does not fork the project model.
35. As a product maintainer, I want thumbnail generation to be testable without launching the full UI, so that source-kind and preview behavior can be validated deterministically.

## Implementation Decisions

- Build the gallery as a first-class Houdini workbench view that can be arranged by the normal Rerun blueprint and workbench layout system.
- Use the existing source locator, source capability, source external-reference, and bundle-preview concepts as the domain vocabulary for gallery entries.
- Introduce one reusable gallery-source model seam that normalizes local paths, explicit file URLs, HTTP(S) item URLs, and manifest entries into stable gallery item records.
- Treat recursive directory walking and URL expansion as explicit bounded operations with user-visible limits.
- Support local directories and explicit file lists before broad remote listing providers.
- Support HTTP(S) direct file URLs and manifest URLs before provider-specific object store browsing.
- Prefer manifest-backed remote collections for v1 URL support because they are reproducible and bounded.
- Keep gallery state divided between durable project intent and runtime cache state. Source locators, saved source lists, and user-created source nodes are durable; decoded thumbnail pixels, fetch attempts, and progressive loading queues are runtime or cache state.
- Do not embed source files or heavy thumbnails in graph sidecar data. Store only lightweight metadata and references unless a separate explicit package/export workflow includes artifacts.
- Render image thumbnails for recognized image formats when the source is available and safe to decode.
- Render generic typed thumbnails for non-image entries. Generic thumbnails should be derived from source kind, capability status, extension, and availability rather than from arbitrary file contents.
- Represent Parquet and GeoParquet-like polygon/table sources as table or polygon-table generic thumbnails unless a future parser produces a visual preview explicitly.
- Preserve native cubic Bezier and polygon semantics when a later parser can read geometry sources; the gallery PRD does not authorize flattening geometry into preview-only polylines as durable graph data.
- Make gallery actions explicit. Opening an item in a Rerun view, creating a source node, creating a source collection node, copying a locator, refreshing metadata, or revealing a local file should all be intentional commands.
- Route image-to-view actions through Rerun-native image or 2D view behavior when feasible, rather than creating a bespoke gallery viewer.
- Route graph creation actions through graph-owned project commands so undo/redo and command history remain coherent.
- Keep the source gallery distinct from the asset gallery. The asset gallery catalogs reusable graph asset definitions; the source gallery catalogs external source items and source collections.
- Avoid GIS/CRS expansion in the gallery PRD. Geometry files can be recognized as source kinds without reopening projection or map semantics.
- Use capability labels to avoid overpromising. A generic thumbnail for a Parquet file means “recognized source candidate,” not “fully loaded geometry.”
- Treat remote credentials, authenticated providers, signed URLs, and object-store browsing as later compatibility work unless a separate issue specifies the provider and trust model.

## Testing Decisions

- Test external behavior at the highest practical seam: given a source locator or manifest, the gallery source model reports stable item records, source kinds, thumbnail intents, capability status, and availability status.
- Add focused tests for local image files, local non-image files, missing local files, direct remote URLs, manifest-backed remote entries, unsupported URL listing, and mixed collections.
- Add focused tests for image thumbnail intent versus generic thumbnail intent without requiring every decoder to be exercised in UI tests.
- Add tests that Parquet or GeoParquet-like entries produce typed generic thumbnails and capability status without claiming a visual geometry preview.
- Add tests that gallery actions produce explicit command intents rather than silently mutating the graph or embedding source content.
- Reuse the existing graph panel and workbench layout test style for the registered Gallery view, bundled layout placement, and context-menu command wiring.
- Reuse source locator, source format inference, external-reference report, bundle-preview, and command-history prior art where the implementation touches those seams.
- Keep network tests deterministic. Prefer fake fetch providers or manifest fixtures over live internet dependencies.
- Include startup smoke validation after registering the new view class, because missing view registration or blueprint wiring can break viewer startup.
- UI screenshots are useful for manual QA, but automated tests should assert model/view contracts rather than pixel-perfect thumbnail layout.

## Out of Scope

- Full provider-specific object-store browsers for S3, Azure Blob, GCS, Hugging Face datasets, SharePoint, or arbitrary web pages.
- Recursive website crawling or unbounded remote directory listing.
- Authenticated remote credential management.
- Automatic import, copy, packaging, or embedding of heavy files.
- Full parsers for every recognized source format.
- Durable storage of decoded thumbnail pixels in graph sidecar data.
- GIS/CRS semantics, map projections, or shapefile compatibility.
- Timeline, video scrubbing, and animation gallery playback.
- ML embedding search, similarity search, or clustering of gallery images.
- Asset-definition publishing or shared asset registries.

## Issue Breakdown

Issue `#244` tracks this PRD slice.

Implementation follow-ups:

1. `#245` - Add the source gallery indexing model.
2. `#246` - Add source gallery thumbnail intents.
3. `#247` - Register the source Gallery workbench view.
4. `#248` - Add explicit source-gallery open-in-Rerun actions.
5. `#249` - Add explicit source-gallery create source node actions.

## Further Notes

The first useful slice is not the right-click action. The first useful slice is a trustworthy gallery index that can describe what is present, what is previewable, what is missing, and what is only a typed source candidate. Once that seam is stable, thumbnail rendering and explicit graph/view actions can land as smaller issues without weakening the project source-of-truth model.

The product should be honest when a file can be previewed but not loaded into the graph, or loaded into the graph but not visually previewed. That distinction is important for image folders mixed with polygon tables, source manifests, recordings, and generated artifacts.
