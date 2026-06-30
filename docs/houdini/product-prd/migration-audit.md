# Houdini clone migration audit

Date: 2026-06-30

## Result

All Markdown source files from `/Users/bit/Documents/houdini-clone` have been imported into `docs/houdini/product-prd/source`.
The imported set contains:

- `CONTEXT.md`
- ADRs `0001` through `0078`
- spike PRD `docs/spikes/0001-rerun-native-graph-ui.md`

The old local folder should no longer be treated as an active product source.

## Active planning surfaces

- GitHub issues `#1` through `#72` track the implementation backlog in `vycorporation/rerun`.
- `docs/houdini/rerun-native-spike-decision.md` records the Rerun-native product-fork decision.
- `docs/houdini/workbench-layout-presets.md` records the Houdini-style workbench layout preset requirement.
- `docs/houdini/python-operator-surface.md`, `python-environment-status.md`, `procedural-asset-interface.md`, `native-plugin-operator-lane.md`, and `specialized-renderer-path.md` distill the larger follow-up lanes.

## Missing item found during migration

The old product docs covered project defaults and user overrides for panel layouts, but the active fork PRD did not explicitly require the Houdini workflow of duplicating a layout, editing it, saving it under a name, and loading it later.
That requirement is now part of `docs/houdini/workbench-layout-presets.md` and issue `#72`.

## Notes

Rerun already has `.rbl` blueprint save/load machinery.
The product gap is the Houdini-facing workbench layer on top of that machinery: named presets, duplicate/edit/save flows, project defaults, and personal overrides.
