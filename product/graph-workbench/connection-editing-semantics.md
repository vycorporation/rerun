# Connection editing semantics tracker

This note tracks issue-backed implementation for graph-owned data-flow
connection editing, validation, diagnostics, and port expansion.

## Source decisions

- `product-prd/source/docs/adr/0004-default-to-houdini-like-graph-ergonomics.md`
- `product-prd/source/docs/adr/0006-node-graphs-use-strong-data-kinds.md`
- `product-prd/source/docs/adr/0036-graphs-allow-exploration-while-boundaries-stay-typed.md`
- `product-prd/source/docs/adr/0074-v1-data-flow-is-acyclic.md`

Source ADRs stay unchanged. This tracker is the mutable implementation ledger
for the connection editing lane.

## Boundaries

- Data-flow edges are graph-owned model data, not renderer or egui state.
- Connection editing must preserve stable node IDs and named ports; readable
  node paths remain navigation and inspection affordances only.
- V1 data flow remains acyclic. Normal edits should reject invalid or cyclic
  additions, while loaded invalid topology can remain visible as diagnostics.
- Automatic repairs such as reconnect-around deletion and insert-on-connection
  must route through the same edge validation semantics as manual connection
  editing.
- Current UI affordances may privilege the primary geometry port, but the model
  should keep port names and data kinds explicit so multi-port work can land in
  narrow slices.

## Issue-backed slices

| Issue | Status | Slice | Outcome |
| --- | --- | --- | --- |
| `#136` | complete | Explicit graph edge metadata spine | Adds stable explicit data-flow edge metadata between graph nodes. |
| `#137` | complete | Acyclic edge validation | Rejects invalid or cyclic edge additions through graph-owned validation. |
| `#138` | complete | Connection diagnostics | Exposes readable graph connection diagnostics for invalid loaded or attempted topology. |
| `#180` | complete | Graph-owned data-flow edge add action | Adds an undoable model action for explicit data-flow edge creation. |
| `#182` | complete | Graph-owned data-flow edge remove action | Adds an undoable model action for explicit data-flow edge removal. |
| `#184` | complete | Edge preservation across node deletion | Preserves unrelated explicit data-flow edges when deleting graph nodes. |
| `#186` | complete | Reconnect-around node deletion | Adds a validated reconnect-around deletion model action for simple compatible chains. |
| `#188` | complete | Insert node on connection | Adds an atomic model action that removes a connection and inserts a compatible node between its endpoints. |
| `#190` | complete | Edge context-menu removal UI | Wires graph connection context-menu removal through the graph-owned remove action. |
| `#192` | complete | Primary-port drag-to-connect UI | Adds primary geometry port drag-to-connect wiring through the graph-owned add action. |
| `#194` | complete | Drag-to-connect preview diagnostics | Shows valid/invalid connection preview diagnostics before mutating graph state. |
| `#296` | complete | Focused connection editing tracker | Adds this focused tracker and points the main PRD/ADR status ledger at it. |
| `#298` | complete | Typed node port inspection | Surfaces selected-node input/output port summaries from existing typed declarations while leaving primary quick-wire behavior unchanged. |
| `#300` | complete | Multi-port hit testing and drawing | Draws stable per-port hit regions for typed ports while preserving primary quick-wire behavior. |

## Next implementation candidates

1. Multi-port drag-to-connect.
   - Let users choose non-primary compatible ports during connection drag.
   - Reuse existing graph-owned edge diagnostics for compatibility, duplicate,
     and cycle checks.

2. Connection organization polish.
   - Track connection routing-dot or bend-point work separately from semantic
     data-flow edges so organization affordances do not change evaluation.

## Maintenance

- Add every connection editing issue here when it is created.
- Change `planned` to `complete` only after the PR is merged and the issue is
  verified closed.
- If a future ADR changes product intent, update this tracker and the main
  implementation status ledger rather than rewriting the historical ADR text.
