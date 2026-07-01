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
- Issue `#104`: add undo/redo for coalesced annotation drag and resize commands.
- Issue `#106`: add undo/redo for annotation title, text, and collapse edits.
- Issue `#108`: add undo/redo for ordinary node duplication with new stable node identities.
- Issue `#110`: add undo/redo for node manual cook flag edits.
- Issue `#111`: add undo/redo for network box and sticky note creation.
- Issue `#112`: add undo/redo for network box resize-to-contents edits.
- Issue `#113`: add undo/redo for ordinary node deletion.

## Implementation notes

- Project command history restores graph/project intent and marks affected outputs stale.
- Runtime evaluation state, work items, running work, and cached outputs are not restored by undo/redo.
- The first slice records node parameter edits with stable node identity, parameter label, old value, and new value.
- Layer visibility and layer order commands restore presentation intent only; they do not replay work items or cached output.
- Node rename commands restore user-facing node names while references continue to resolve by stable node ID.
- Node flag commands restore output participation and comment visibility intent only; they do not replay running work or cached output.
- Node layout drag commands coalesce high-frequency drag updates into one old-position/new-position command at gesture completion.
- Annotation gesture commands coalesce drag and resize updates; network box drag restores the member node positions moved by the gesture.
- Annotation content commands coalesce repeated title/body text edits for the same annotation and keep collapse-all as one project command.
- Node duplicate commands restore ordinary copied graph material with the captured new stable node ID, while output participation and output operators are not silently copied.
- Node manual cook commands restore project intent for whether a node waits for explicit evaluation without replaying work items or cached outputs.
- Organization creation commands restore network boxes and sticky notes with their captured stable annotation identities and do not change graph data flow.
- Network box fit commands restore old and new organization bounds without moving member nodes or changing data flow.
- Node delete commands restore the captured graph node by stable node ID; references to deleted nodes remain stable missing-reference diagnostics until undo restores the target.
- Sidecar save/load intentionally omits the undo and redo stacks so history does not become a second persistence format.
