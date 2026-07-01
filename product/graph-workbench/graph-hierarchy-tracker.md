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
| `#142` | complete when merged | Graph container metadata | Adds serializable subnet-like graph container metadata keyed by stable node ID, pointing to an internal named graph without moving nodes yet. |

## Next implementation candidates

1. Graph-local node name policy.
   - Enforce uniqueness within the parent graph or subgraph for create, rename,
     duplicate, and paste paths.
   - Keep the operator type visible separately from the editable node name.

2. Graph path inspection and navigation.
   - Current graph path and readable node path inspection are covered by
     `#131`.
   - Later navigation work should add selected graph switching without turning
     path display into a second durable identity.

3. Graph container storage.
   - `#142` adds graph container metadata that can point to an internal named
     graph.
   - Keep selected-node collapse separate from network box organization.

4. Typed graph boundary inputs and outputs.
   - Model typed public graph container inputs and outputs.
   - Preserve ordinary editable graph references while preventing private asset
     internals from becoming external reference targets.

5. Acyclic connection diagnostics.
   - Block obvious cyclic connection attempts once explicit editable edges are
     implemented.
   - Preserve loaded invalid topology as visible diagnostics.

6. Cross-graph readable paths.
   - Use readable graph/node paths for navigation, search, node info, and delete
     warnings.
   - Keep durable references backed by stable graph IDs and node IDs.

## Maintenance

- Add every graph hierarchy issue here when it is created.
- Change `planned` to `complete` only after the PR is merged and the issue is
  verified closed.
- If a future ADR changes product intent, update this tracker and the main
  implementation status ledger rather than rewriting the historical ADR text.
