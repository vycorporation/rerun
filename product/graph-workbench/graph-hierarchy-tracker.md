# Graph hierarchy and named graph tracker

This note tracks issue-backed implementation for named graphs, subgraph
navigation, graph-local node paths, and acyclic data-flow rules.

## Source decisions

- `product-prd/source/docs/adr/0008-projects-contain-multiple-named-node-graphs.md`
- `product-prd/source/docs/adr/0035-support-navigable-subgraphs-and-promotable-assets.md`
- `product-prd/source/docs/adr/0036-graphs-allow-exploration-while-boundaries-stay-typed.md`
- `product-prd/source/docs/adr/0074-v1-data-flow-is-acyclic.md`
- `product-prd/source/docs/adr/0075-node-references-use-stable-internal-ids.md`

Source ADRs stay unchanged. This tracker is the mutable implementation ledger
for the graph hierarchy lane.

## Boundaries

- Named graphs are project graph metadata, not viewer state.
- Graph paths and node paths are readable navigation affordances; durable
  references continue to use stable graph and node IDs.
- Network boxes, collapsed network boxes, graph notes, and node comments are
  organization affordances unless the user explicitly creates a subgraph or
  asset.
- Exploratory graph work can remain loose, but graph container boundaries,
  procedural assets, layer promotion, plugin operators, and exports remain typed.
- V1 data flow remains acyclic; invalid or cyclic loaded data should surface as
  diagnostics instead of being silently deleted.

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#130` | complete | Graph registry metadata | `GraphDocument` owns durable graph registry metadata, sidecar JSON round-trips it, and older sidecars default to the main graph. |
| `#131` | complete | Current graph path and graph-local inspection | Shows selected graph path, readable node path, and graph-local name uniqueness in node inspection while preserving stable ID references. |
| `#132` | complete | Focused graph hierarchy tracker | Adds this tracker and points the main PRD/ADR status ledger at it. |
| `#142` | complete | Graph container metadata | Adds serializable subnet-like graph container metadata keyed by stable node ID, pointing to an internal named graph without moving nodes yet. |
| `#144` | complete | Typed graph container boundaries | Adds serializable typed input/output declarations to graph containers and exposes boundary outputs as stable reference targets. |
| `#146` | complete | Graph container boundary anchor mappings | Adds serializable mappings from public boundary ports to internal graph anchors, with model diagnostics for unresolved metadata. |
| `#148` | complete | Graph container collapse manifests | Records selected connected node sets and typed external edge crossings as graph container collapse metadata without moving nodes yet. |
| `#164` | complete | Graph-local node name policy | Adds durable parent graph metadata to nodes and scopes create, rename, duplicate, readable paths, and name collision inspection to the parent graph while preserving stable node IDs. |
| `#166` | complete | Selected graph navigation model seam | Adds graph navigation targets plus safe selected-graph and graph-container enter model actions without moving nodes between graphs. |
| `#168` | complete | Selected graph storage layout scope | Scopes graph-local node index helpers, generated default data-flow edges, and graph layout to selected graph storage while retaining loaded cross-graph diagnostics. |
| `#170` | complete | Physical graph container collapse storage | Moves captured nodes into the new internal graph and rewires external crossings through typed graph-container boundary edges. |
| `#172` | complete | Graph container enter inspector action | Adds a workbench node-info Enter action for resolved navigable graph containers using the selected-graph navigation seam. |
| `#174` | complete | Selected graph data-flow diagnostics | Adds graph-scoped data-flow diagnostic queries and surfaces selected-graph invalid/cyclic connection diagnostics in the workbench Info pane. |
| `#176` | complete | Cross-graph search result navigation | Carries graph IDs and readable node paths through workbench search so node results switch to their parent graph before selection. |
| `#178` | complete | Graph-path reference delete warnings | Adds readable target node paths to reference output change warnings so delete/edit warnings disambiguate graph-local names. |
| `#216` | complete | Selected-node graph container collapse UI | Adds workbench and node context-menu commands that collapse the current non-output, non-container node into a navigable typed graph container using the physical graph-container storage path without faking multi-selection. |
| `#220` | complete | Graph container asset draft promotion | Creates project-local procedural asset drafts from resolved graph containers, preserving typed boundaries, internal graph references, and graph-scoped promoted parameters. |
| `#222` | complete | Drag marquee multi-select for subnet creation | Adds empty-canvas marquee selection for graph nodes and a Collapse Selection to Subnet command that routes multi-node selections through the existing graph-owned physical collapse path. |

## Next implementation candidates

1. Graph path inspection and navigation.
   - Current graph path and readable node path inspection are covered by
     `#131`.
   - Selected graph switching and resolved graph-container enter actions are
     covered by `#166` without turning path display into a second durable
     identity.
   - `#172` wires resolved graph-container enter navigation into the workbench
     inspector.

2. Graph container storage.
   - `#142` adds graph container metadata that can point to an internal named
     graph.
   - `#148` records collapse manifests for selected connected node sets without
     moving nodes before graph-local node storage lands.
   - `#168` scopes layout and generated default data-flow edges to graph-local
     node storage, which gives selected graph navigation an honest model view
     before physical collapse moves nodes.
   - `#170` moves connected collapsed node sets into the internal graph and
     rewires compatible external crossings through typed graph-container
     boundary ports.
   - `#216` exposes the first honest workbench collapse command for the
     currently selected node while leaving multi-selection as a later UI layer.
   - `#220` lets resolved graph containers become procedural asset drafts
     without treating network boxes or viewport selection as asset boundaries.
   - `#222` adds graph-node marquee multi-selection and routes multi-node
     selected sets into the same physical graph container collapse path.
   - Keep selected-node collapse separate from network box organization.

3. Typed graph boundary inputs and outputs.
   - `#144` models typed public graph container inputs and outputs.
   - `#146` maps typed public ports to internal input/output anchors.
   - Preserve ordinary editable graph references while preventing private asset
     internals from becoming external reference targets.

4. Acyclic connection diagnostics.
   - Block obvious cyclic connection attempts once explicit editable edges are
     implemented.
   - Preserve loaded invalid topology as visible diagnostics.
   - `#174` surfaces selected-graph invalid/cyclic diagnostics in the workbench
     Info pane without deleting or normalizing loaded edges.

5. Cross-graph readable paths.
   - Use readable graph/node paths for navigation, search, node info, and delete
     warnings.
   - `#176` makes workbench node search path-aware and switches to a result's
     parent graph before focusing the node.
   - `#178` uses readable target node paths in reference output edit/delete
     warnings while keeping stable IDs as durable references.
   - Keep durable references backed by stable graph IDs and node IDs.

## Maintenance

- Add every graph hierarchy issue here when it is created.
- Change `planned` to `complete` only after the PR is merged and the issue is
  verified closed.
- If a future ADR changes product intent, update this tracker and the main
  implementation status ledger rather than rewriting the historical ADR text.
