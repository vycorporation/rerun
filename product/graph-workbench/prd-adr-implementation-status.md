# PRD and ADR implementation status

This ledger tracks how PRD/ADR material maps to issue-backed implementation in the
`vycorporation/rerun` product fork.

## Policy

- Do not delete source PRD or ADR files after implementation.
- Treat ADRs as durable product decisions, including superseded and deferred decisions.
- Mark implementation state here and in focused tracker docs instead of editing source ADR intent.
- Every implementation slice should have a GitHub issue before code changes.
- When an issue merges, update the relevant tracker/status note with the issue number and outcome.

## Status values

- `complete`: the documented v1 slice has closed issue-backed implementation.
- `partial`: the lane has an implemented spine but still has explicit follow-up slices.
- `deferred`: the ADR intentionally says not to implement this in v1 or not yet.
- `superseded`: the ADR is retained for history but should not drive new work.
- `unmined`: the ADR or lane has not yet been broken into issue-backed slices in this fork.

## Issue-backed lanes

| Lane | Status | Tracker | Issues |
| --- | --- | --- | --- |
| Rerun-native spike foundation | complete | `rerun-native-spike-decision.md` | `#1`, `#2`, `#3`, `#4`, `#5`, `#6`, `#7`, `#8`, `#9`, `#10`, `#11`, `#12`, `#13`, `#14`, `#15`, `#16`, `#28`, `#30` |
| Graph source, typed nodes, filters, styles, layers, and output | complete | PRD ADRs plus implementation issues | `#19`, `#20`, `#21`, `#22`, `#23`, `#24`, `#25`, `#26`, `#56`, `#63` |
| Demand-driven evaluation and work items | complete | `execution-work-items.md` | `#27`, `#84`, `#86` |
| Native cubic and renderer-native preview path | partial | `specialized-renderer-path.md` | `#29`, `#36`, `#37`, `#38` |
| Python operator and project environment lane | partial | `python-operator-surface.md`, `python-environment-status.md` | `#31`, `#32`, `#39`, `#40`, `#41`, `#42`, `#43`, `#44`, `#45`, `#46`, `#55` |
| Native plugin operator lane | partial | `native-plugin-operator-lane.md` | `#34`, `#51`, `#52`, `#53`, `#54` |
| Procedural asset interface | partial | `procedural-asset-interface.md` | `#33`, `#47`, `#48`, `#49`, `#50`, `#61`, `#152`, `#156`, `#158`, `#160`, `#162`, `#218`, `#220`, `#228`, `#232` |
| Graph hierarchy and named graph registry | partial | `graph-hierarchy-tracker.md` | `#130`, `#131`, `#132`, `#142`, `#144`, `#146`, `#148`, `#164`, `#166`, `#168`, `#170`, `#172`, `#174`, `#176`, `#178`, `#216`, `#220`, `#222`, `#224`, `#226`, `#228`, `#230` |
| Reference inputs, stable IDs, diagnostics, and target sets | complete for current v1 reference-input spine | PRD ADRs plus command-history tracker | `#57`, `#58`, `#59`, `#60`, `#62`, `#98`, `#113`, `#114`, `#121`, `#122` |
| Workbench layout presets and browser | complete for current v1 workbench slice | `workbench-layout-presets.md` | `#64`, `#68`, `#72`, `#74` |
| Shelf tools | complete for built-in shelf tools; custom shelf tools deferred | `shelf-tools.md` | `#90` |
| Node comments, node info, badges, and organization affordances | complete for current v1 graph organization slice | `node-comment-display-mode.md`, command-history tracker | `#65`, `#66`, `#94`, `#100`, `#102`, `#104`, `#106`, `#111`, `#112`, `#120`, `#123` |
| Node duplication and deletion | complete for ordinary single-node duplicate/delete | `node-duplication.md`, command-history tracker | `#92`, `#108`, `#113` |
| Connection editing semantics | partial | this ledger until a focused tracker is added | `#136`, `#137`, `#138`, `#180`, `#182`, `#184`, `#186`, `#188`, `#190`, `#192`, `#194` |
| Dataset/source breadth and external artifact references | partial | `dataset-source-breadth-tracker.md`, `source-gallery-view-prd.md` | `#196`, `#198`, `#200`, `#202`, `#204`, `#206`, `#208`, `#210`, `#212`, `#214`, `#244`, `#245`, `#246`, `#247`, `#248`, `#249`, `#262`, `#264`, `#266`, `#268` |
| Minimal graph-backed viewport editing | complete for current v1 selection/subset spine | `viewport-editing-tracker.md` | `#256`, `#257`, `#258` |
| Malware byteplot starter workflow | complete for current starter graph and raster output slice | `malware-byteplot-workflow.md` | `#75`, `#79` |
| Project command history | complete through the current command-history batch | `project-command-history.md` | `#88`, `#96`, `#98`, `#100`, `#102`, `#104`, `#106`, `#108`, `#110`, `#111`, `#112`, `#113`, `#114`, `#120`, `#121`, `#122`, `#123` |
| PRD/ADR implementation status tracking | complete for the initial status ledger | `prd-adr-implementation-status.md` | `#128` |

## Deferred or superseded ADRs

These files should remain in the PRD corpus, but they should not be mined as
current implementation work unless a later decision reopens them.

- `0045-custom-shelf-tools-are-deferred.md`: deferred until assets, Python permissions, plugin capabilities, and security are more mature.
- `0046-timeline-and-animation-are-deferred.md`: deferred for v1.
- `0052-sveltekit-typescript-frontend.md`: superseded by ADR-0064.
- `0053-use-svelte-flow-as-the-initial-graph-editor.md`: superseded by ADR-0064.
- `0058-graph-ux-starts-beside-rerun-as-orchestration.md`: superseded by ADR-0064.
- `0061-graph-ux-prototype-uses-sveltekit-and-python-driven-rerun.md`: superseded by ADR-0064.
- `0062-rerun-prototype-starts-with-spawned-viewer.md`: superseded by ADR-0064.
- `0076-v1-omits-houdini-style-locked-output-embedding.md`: intentional v1 omission, not a missing implementation.

## Next lanes to mine

These are the highest-signal remaining PRD/ADR areas that should become small
issue-backed implementation slices next.

1. Multiple named graphs and subgraph navigation.
   - Source ADRs: `0008`, `0035`, `0036`, `0074`, `0075`.
   - Issue-backed slices: `#130` graph registry metadata, `#131` current graph path and graph-local inspection, `#132` focused graph hierarchy tracker, `#142` graph container metadata, `#144` typed graph container boundaries, `#146` boundary anchor mappings, `#148` collapse manifests, `#164` graph-local node name policy, `#166` selected graph navigation model actions, `#168` selected graph storage layout scope, `#170` physical graph container collapse storage, `#172` graph container enter inspector action, `#174` selected graph data-flow diagnostics, `#176` cross-graph search result navigation, `#178` graph-path reference delete warnings, `#216` selected-node graph container collapse UI, `#220` graph-container asset draft promotion, `#222` drag marquee multi-select for subnet creation, `#224` Tab palette subnet collapse action, `#226` Houdini-style subnet enter/up navigation, `#228` Tab palette asset draft creation from selected subnet, and `#230` graph-local boxes and notes.
   - Later likely slices: explicit connection-editing cycle blocking, additive selection modifiers, and asset-definition polish after manual subnet creation QA.

2. Procedural asset authoring depth.
   - Source ADRs: `0009`, `0010`, `0011`, `0077`, `0078`.
   - Issue-backed slices: `#152` promoted parameter binding metadata, `#156` match-definition and upgrade-to-current-definition model actions, `#158` external artifact reference metadata and warnings, `#160` typed asset input/output boundary editing model actions, `#162` bundle/export artifact inclusion previews, `#218` explicit save-definition actions, `#220` graph-container asset draft promotion, `#222` drag marquee multi-select for subnet creation, `#224` Tab palette subnet collapse action, `#226` Houdini-style subnet enter/up navigation, `#228` Tab palette asset draft creation from selected subnet, `#232` project asset gallery with usage navigation, `#234` subnet asset draft naming/id polish, `#236` selected asset/gallery match and upgrade actions, `#238` project asset gallery search filtering, `#240` asset gallery usage grouping by graph path, and `#242` registered/discoverable Assets workbench view.
   - Later likely slices: deeper promotion controls after manual subnet creation QA, including richer promoted-parameter authoring.

3. Connection editing semantics.
   - Source ADRs: `0004`, `0006`, `0074`.
   - Issue-backed slices: `#136` explicit graph edge metadata spine, `#137` acyclic edge validation, `#138` connection diagnostics, `#180` graph-owned data-flow edge add action, `#182` graph-owned data-flow edge remove action, `#184` explicit edge preservation across node deletion, `#186` reconnect-around node deletion model action, `#188` insert-node-on-connection model action, `#190` edge context-menu removal UI wiring, `#192` primary-port drag-to-connect UI wiring, and `#194` drag-to-connect preview diagnostics.
   - Later likely slices: multi-port expansion.

4. Dataset/source breadth and external artifact references.
   - Source ADRs: `0014`, `0015`, `0016`, `0078`.
   - Issue-backed slices: `#196` source locator metadata model, `#198` source format capability records, `#200` source external-reference status reports, `#202` source bundle preview metadata, `#204` focused dataset/source tracker, `#206` source package manifest preview records, `#208` source format inference reports, `#210` external source reference action hints, `#212` explicit source package manifest writing, `#214` source manifest inclusion choices, `#244` source gallery view PRD, `#245` gallery source indexing, `#246` thumbnail intents, `#247` Gallery workbench view, `#248` open-in-Rerun actions, `#249` source-node actions, `#262` source package writes with copied local artifacts and hashes, `#264` polygon coordinate CSV import, `#266` GeoJSON polygon import, and `#268` copy source locator action.
   - Later likely slices: provider-specific remote collection browsing and
     authenticated URL/provider support.

5. Minimal graph-backed viewport editing.
   - Source ADRs: `0029`, `0047`, `0050`.
   - Issue-backed slices: `#256` transient viewport selection identity bridge,
     `#257` read-only table-to-selection interactions, and `#258` commit
     transient selections as graph-backed subset nodes.
   - Manual edit/source-edit skeletons and direct drawing tools are deferred
     until after the current graph-backed selection/subset spine is validated in
     use.
   - Viewport work should prefer graph-model, workbench, blueprint, and existing
     selection/table seams over broad edits to Rerun core viewport or renderer
     internals, so the product fork can keep updating from upstream with low
     conflict risk.

6. Renderer specialization beyond the current line draw-data preview.
   - Source ADRs: `0020`, `0021`, `0055`, `0067`.
   - First likely slices: profiling result capture, GPU buffer cache keys, and custom Houdini geometry draw-data scaffolding if profiling justifies it.

7. Plugin capability and trust policy hardening.
   - Source ADRs: `0012`, `0013`, `0023`, `0068`.
   - First likely slices: project trust prompts, capability grant persistence, operator enable/disable history, and failure messaging for denied capabilities.

## Maintenance checklist

When creating a new PRD/ADR-derived issue:

1. Add the issue to the relevant tracker doc or this ledger.
2. Keep the source ADR unchanged unless the product decision itself changes.
3. Use `complete`, `partial`, `deferred`, `superseded`, or `unmined` language.
4. When the PR merges, update the tracker line from planned language to closed/implemented language.
5. Prefer adding a new focused tracker note over overloading a source ADR with implementation details.
