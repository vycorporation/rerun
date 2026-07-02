# Minimal graph-backed viewport editing tracker

This note tracks issue-backed implementation for minimal viewport and table
selection work. It intentionally keeps manual drawing and geometry editing out
of the current lane.

## Source decisions

- `product-prd/source/docs/adr/0029-selection-is-transient-in-the-viewport-and-durable-in-the-graph.md`
- `product-prd/source/docs/adr/0047-v1-uses-minimal-graph-backed-viewport-editing.md`
- `product-prd/source/docs/adr/0050-attribute-tables-are-read-only-in-v1.md`

Source ADRs stay unchanged. This tracker is the mutable implementation ledger
for selection, table inspection, and later graph-backed edit operations.

## Boundaries

- Viewport and table selections remain transient until a user explicitly
  commits them into graph-owned subset data.
- Durable selection data is based on record identity, not row position,
  viewport pick id, or screen position.
- Selection and mask nodes do not filter records or change appearance by
  themselves; filter and style behavior remains explicit graph data.
- Attribute tables remain read-only in v1. Search, sort, and temporary filters
  are inspection state until explicitly committed.
- Manual drawing, CAD-style editing, GIS editing, and 3D modeling tools are
  deferred until the procedural exploration workflow is stable.
- Prefer graph-model, workbench, blueprint, and existing selection/table seams
  over broad edits to Rerun core viewport or renderer internals. If a core
  Rerun seam must be touched, keep it minimal, isolated, and documented so the
  product fork can continue updating from upstream with low conflict risk.

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#256` | complete | Transient viewport selection identity bridge | Adds a graph-model runtime report that resolves supported transient viewport selections to graph record identities, while rejecting renderer pick ids, row positions, screen positions, missing records, and ambiguous matches with diagnostics. |
| `#257` | complete | Read-only table-to-selection interactions | Adds read-only attribute-table row selection backed by graph record fingerprints, routes selected rows through the transient selection identity bridge from `#256`, and reports selected, stale, or unsupported rows without mutating table values or graph output. |
| `#258` | planned | Commit transient selections as graph-backed subset nodes | Add an explicit commit action that creates or updates visible selection or mask graph data using stable record identities from `#256`. |

## Deferred implementation candidates

1. Manual edit or source-edit node skeleton.
   - Defer until selection identity and committed subset behavior have been
     validated.
   - Treat as a graph-backed operation, not hidden viewport mutation.
   - Keep simple curve or polygon correction narrow if it is reopened.

2. Direct drawing tools.
   - Defer until late in the product path.
   - Do not start this lane as broad CAD, GIS, or 3D modeling functionality.

## Maintenance

- Add every viewport/table selection issue here when it is created.
- Change `planned` to `complete` only after the PR is merged and the issue is
  verified closed.
- If an implementation must touch core Rerun viewport or renderer code, record
  the reason and the containment strategy in this tracker or the PR.
