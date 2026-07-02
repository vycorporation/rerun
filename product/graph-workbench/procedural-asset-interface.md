# Houdini procedural asset interface

Date: 2026-06-28

## Decision

Define procedural assets as project-local typed subgraphs behind declarative interfaces.
They are a packaging layer over graph truth, not custom plugin code.
They should let a user promote a stable graph or subgraph into a reusable node-like operator without changing the underlying runtime model.

The v1 procedural asset is local to a project.
Shared asset libraries, marketplace-style distribution, custom viewport handles, and native plugin execution are deferred.

## Asset metadata

A procedural asset declaration has an asset id, display name, version, description, labels, help text, author or source, created timestamp, updated timestamp, and compatibility version.
It also declares typed inputs, typed outputs, promoted parameters, documentation, preview metadata, and the wrapped subgraph reference.

Inputs and outputs use the same graph-level data kinds as the rest of the Houdini product fork.
The v1 data kinds should include geometry tables compatible with `HoudiniGeometryRecord`, attribute tables, scalar values, string values, and layer/style metadata.
Native cubic Beziers remain four-control-point geometry records.
Assets must not store dense curve tessellation as their canonical output.

Promoted parameters are graph-owned parameter definitions exposed from internal nodes.
Each promoted parameter records a display label, help text, default value, current value, optional range, optional enum values, optional grouping, and the internal node parameter it binds to.
Changing a promoted parameter invalidates the wrapped subgraph in the same way changing the original internal parameter would.

## Wrapping a subgraph

An asset wraps a graph or subgraph by recording an internal graph fragment plus a declarative boundary.
The boundary maps asset inputs to internal source/input nodes and maps internal output nodes back to asset outputs.

The wrapped graph remains editable graph data.
Expanding or editing the asset should reveal ordinary nodes and edges rather than opaque executable code.
This keeps assets inspectable, diffable, and compatible with demand-driven evaluation.

The first implementation can package a whole `GraphDocument` or a selected connected subgraph.
The selected subgraph path is preferable long term, but a whole-document asset is acceptable as a bounded first slice if it preserves the same metadata contract.

## Runtime behavior

At runtime, an asset instance behaves like a graph node with an internal subgraph.
It receives typed inputs, applies promoted parameters, evaluates the internal graph, and emits typed outputs.
It participates in stale, running, cached, failed, manual, and clean evaluation states.

Asset evaluation should reuse graph evaluation machinery.
It should not create a separate plugin runtime.
It should not call viewer APIs directly.
It should not depend on `ViewerContext`, egui, renderer objects, or `AppState`.

Asset output feeds the same current geometry path as Parquet import and Python operators.
The runtime should convert asset output into graph-owned geometry records and let `GraphDocument` produce `RerunSceneOutput`.

## Persistence

Project-local assets should persist with or alongside the Houdini graph sidecar.
The persistence format should record asset declarations, wrapped graph fragments, promoted parameters, boundary mappings, documentation, and version metadata.

Asset instances inside a graph should store the asset id and version they were created from.
If the local asset declaration changes, existing instances should become stale or show an update warning rather than silently changing behavior.

The sidecar should avoid duplicating large cached outputs inside the asset declaration.
Cached outputs belong to evaluation/cache state, not to the asset definition itself.

## Compatibility with the current geometry path

The v1 procedural asset should preserve the current Parquet-focused geometry compatibility.
It may wrap a source node that reads the eight-column cubic Bezier contract.
It may wrap filters, styles, layer views, and output nodes.
It may emit polygons and native cubic Beziers through the same `HoudiniGeometryRecord` shape used elsewhere.

Assets should not require a new viewer output surface.
They should not require a custom renderer.
They should not require Python or native plugins.
Those capabilities can be composed with assets later, but asset v1 should prove graph packaging first.

## UI surface

The graph UI should show asset instances as ordinary graph nodes with a distinct asset marker.
The inspector should show asset metadata, promoted parameters, input bindings, output summaries, version status, and a way to open or expand the wrapped graph.

The asset creation flow should start from a selected graph or subgraph and prompt for name, description, inputs, outputs, and promoted parameters.
Graph containers can be promoted into project-local asset definitions by reusing their typed boundary and internal graph reference.
The asset editing flow should preserve graph-visible internals.
Unlocked asset instances should expose an explicit Save Asset Definition action.
Saving writes compatible graph-owned metadata back to the project-local declaration, bumps the declaration version, relocks the saved instance to that version, and leaves other exact-version pins requiring explicit upgrade.

The UI should avoid presenting assets as a hidden code plugin system.
An asset is a saved graph pattern with a typed boundary.

The project UI should include a project asset gallery for active project-local
asset definitions. The gallery should show each definition's display metadata,
interface summary, version, and current usage count, then list every graph node
instance using that definition with readable graph/node paths. Usage entries
should be navigable: selecting one switches to the owning graph, selects the
asset instance node, and frames it in the network editor. Missing definitions
that still have live instances should remain visible as repairable project
state instead of disappearing from the gallery.

## Deferred work

Shared asset libraries are deferred.
Remote asset registries are deferred.
Marketplace distribution is deferred.
Custom viewport handles are deferred.
Native plugin-backed assets are deferred.
Python-backed assets are deferred until the Python operator lane is implemented.

## Implementation follow-ups

1. #49 - Add a serializable procedural asset declaration model.
2. #50 - Add asset instance nodes and graph inspection.
3. #48 - Add a create-asset-from-graph flow for project-local assets.
4. #47 - Add asset versioning and stale-instance warnings.
5. #152 - Complete: add promoted parameter labels, current values, groups, and stable internal node parameter bindings.
6. #156 - Complete: add explicit match-definition and upgrade-to-current-definition model actions for asset instances.
7. #158 - Complete: add external artifact reference metadata and asset inspection warnings without embedding heavy artifacts.
8. #160 - Complete: add typed asset input/output boundary editing model actions with duplicate guards and stale-instance marking.
9. #162 - Complete: add artifact bundle/export preview metadata with inclusion choices, expected size, remaining external references, and reproducibility warnings without copying artifacts.
10. #218 - Complete: add explicit Save Asset Definition model and workbench UI actions for unlocked procedural asset instances while preserving exact-version pins.
11. #220 - Complete: create project-local procedural asset drafts from resolved graph containers using their typed boundary and internal graph reference.
12. #228 - Complete: expose asset draft creation from the selected subnet through the Tab/operator palette while reusing the graph-container asset draft path.
13. #232 - Complete: add a project asset gallery that lists active project asset definitions and every graph usage with jump-to-instance navigation.
