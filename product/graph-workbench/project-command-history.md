# Houdini project command history tracker

This note maps the undo/redo and project-command PRD lane to implementation issues in the `vycorporation/rerun` product fork.

## Source PRD

- `product-prd/source/docs/adr/0002-node-graph-is-the-source-of-truth.md`
- `product-prd/source/docs/adr/0025-undo-redo-operates-on-project-commands.md`
- `product-prd/source/docs/adr/0051-build-the-project-model-and-persistence-spine-first.md`
- `product-prd/source/docs/adr/0073-first-malware-artifact-workflow-proves-node-network-mvp.md`

## Issue slices

- Issue `#88`: add the first project-command history slice for undo/redo of graph-owned node parameter edits.
- Issue `#96`: add undo/redo for layer visibility and layer order presentation edits.
- Issue `#98`: add undo/redo for stable-ID-safe node renames.
- Issue `#100`: add undo/redo for node output participation and comment visibility flags.
- Issue `#102`: add undo/redo for coalesced node layout drag commands.

## Implementation notes

- Project command history restores graph/project intent and marks affected outputs stale.
- Runtime evaluation state, work items, running work, and cached outputs are not restored by undo/redo.
- The first slice records node parameter edits with stable node identity, parameter label, old value, and new value.
- Layer visibility and layer order commands restore presentation intent only; they do not replay work items or cached output.
- Node rename commands restore user-facing node names while references continue to resolve by stable node ID.
- Node flag commands restore output participation and comment visibility intent only; they do not replay running work or cached output.
- Node layout drag commands coalesce high-frequency drag updates into one old-position/new-position command at gesture completion.
- Sidecar save/load intentionally omits the undo and redo stacks so history does not become a second persistence format.
