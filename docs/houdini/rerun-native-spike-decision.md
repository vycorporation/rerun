# Rerun-Native houdini graph UI spike decision

Date: 2026-06-28

## Decision

Continue the Houdini graph UI spike inside the native Rerun viewer as a narrow product fork.
Do not return to the superseded Svelte, Tauri, React Flow, or Svelte Flow direction for the next slice.
Do not move curve and polygon output to an external or custom viewer target yet.
Do add a specialized renderer follow-up for high-scale curves and polygons before treating the Rerun-native path as production-ready.

This decision is based on the spike proving that Rerun can host the product workflow end to end.
The current product fork has a dockable Houdini Graph view, an editor panel, app-scoped graph state, graph-owned styles and filters, typed node parameters, native cubic Bezier output, Parquet import, attribute inspection, durable `.rrd` export, demand-driven evaluation status, and renderer-native preview plumbing.

The current renderer result is promising but not complete.
The generic renderer-native line-strip preview is the right direction away from egui painter shapes, but it is not equivalent to Rerun's specialized point-cloud renderer path.

## Path chosen

Keep the native Rerun product fork as the primary spike path.
Keep the graph model independent from Rerun viewer state.
Keep native cubic Beziers as graph geometry.
Use adaptive polylines only as viewer, debug, or export representations.
Use Rerun renderer draw data for high-scale preview work, but treat the current line-strip renderer path as an intermediate prototype.

The product fork should optimize for this product's curve, polygon, graph, Python, and spatial inspection workflows.
Upstream-compatible patterns are useful when they reduce maintenance cost, but upstream contribution is not the goal of this fork.

## Evidence

The graph/editor prototype now lives inside the native viewer and shares graph state through `AppState`.
The output is a dockable `Houdini Graph` `ViewClass` that auto-spawns after resetting to the heuristic blueprint.
The editor panel no longer renders duplicate output.
Nodes are draggable, generated nodes are marked, and node parameters are typed graph metadata.
Layer ordering, multiple layer views, table inspection, table-to-graph filter commits, evaluation status, and durable recording export all work in the native viewer.

`GraphDocument` emits `RerunSceneOutput`.
`RerunSceneOutput` preserves native cubic Beziers and can expose adaptive boundary or debug point counts without storing dense polyline geometry as graph truth.
Sidecar persistence and `.rrd` export keep native cubic records rather than flattening curves into stored polylines.

The checked-in Parquet path supports the current eight-column cubic Bezier control-point contract.
That is enough for the spike because broad source-format support was not the bottleneck.

## Rerun APIs touched

The spike touched native viewer UI integration through `ViewClass`, `ViewState`, `ViewerContext`, bottom panel UI, `AppState`, and shared egui context data.
It touched heuristic blueprint behavior through default view registration and view spawning.
It touched Rerun's data/query side through `ViewQuery`, visible data result summaries, product-fork query bridge metadata, and durable recording export.
It touched Arrow and Parquet loading for the eight-column cubic Bezier sample data.
It touched renderer integration through `re_renderer::LineDrawableBuilder`, `ViewBuilder`, `TargetConfiguration`, and `gpu_bridge::new_renderer_callback`.

The point-cloud comparison is important.
Large `Points3D` scenes run through a specialized renderer pipeline: `Points3DVisualizer`, `PointCloudBuilder`, `PointCloudDrawData`, `PointCloudRenderer`, and `point_cloud.wgsl` on wgpu.
The Houdini renderer-native preview currently uses generic line strips prepared at the viewer boundary.
That is a better architecture than egui painter shapes, but it is not yet the same class of optimized renderer as point clouds.

## Benchmark and rendering findings

The initial native cubic egui painter path slowed sharply at high curve counts.
User testing reported roughly 8 frames per second at 20,000 curves and roughly 9 frames per second around 10,000 curves in that path.

An optimized egui fast-preview path improved the bounded benchmark substantially.
User testing reported 50,000 curves plus 20,000 polygons at the 30 frames-per-second ceiling when no other heavy scene was loaded.
That made egui useful as a control and fallback, but not the preferred high-scale path.

The renderer-native line-strip preview moved large curve and polygon scenes to `re_renderer` draw data.
It keeps graph-owned cubic Beziers native and only samples curves at the viewer boundary.
User testing still observed slowdown when combining the 1.5 million point-cloud scene with maximum benchmark curves and polygons.

The conclusion is not that Rust or Rerun cannot handle this class of geometry.
The conclusion is that high-scale Houdini curves and polygons need a specialized draw-data and renderer path rather than per-frame generic sampled line strips.

## Stability and maintenance cost

The product fork currently depends on internal viewer APIs.
The main maintenance risks are `ViewClass` integration churn, blueprint heuristics, app-scoped state wiring, renderer callback setup, and any changes to `re_renderer` draw-data APIs.

The graph model itself is relatively insulated from those risks.
It remains viewer-agnostic and uses typed model metadata for geometry, styles, layers, filters, evaluation state, and source metadata.

The maintenance cost is acceptable for a product fork because the spike now has product-specific behavior that is unlikely to be upstreamed as-is.
The cost would become unacceptable if renderer-specific details leaked into `GraphDocument` or if Python operators became ad hoc viewer scripts.

## Limitations

The current Houdini view is still a spike UI.
The graph canvas is useful but not a complete Houdini-grade node editor.

The renderer-native preview is still a preview.
It does not implement a custom native-cubic shader or a dedicated curve/polygon renderer.
It still prepares sampled line strips for cubics at the viewer boundary.

High-scale performance with point clouds plus dense curve and polygon overlays is not proven.
The next renderer issue must separate graph evaluation cost, viewer-boundary preparation cost, upload cost, and GPU render cost.

Python operators, uv-managed environments, procedural assets, and trusted native plugin operators remain design follow-ups.
They should build on the graph model and demand-driven evaluation machinery rather than bypass it.

## Next issues

1. #38 - Prototype specialized Houdini curve and polygon renderer path.
2. #31 - Define first-class Houdini Python operator surface.
3. #32 - Track app-managed uv project environment status for Houdini Python.
4. #33 - Prototype graph-backed Houdini procedural asset interfaces.
5. #34 - Define trusted native plugin operator lane for Houdini graph.

The renderer issue should run before declaring high-scale curve and polygon rendering solved.
The Python and plugin issues can proceed as product design work, but they should not distract from the renderer gap.

## Non-Goals

Do not revive the separate frontend stack unless the native viewer path fails a concrete product requirement.
Do not broaden source-format support before the graph/runtime and renderer boundaries are stronger.
Do not store dense curve tessellation as the graph model.
Do not optimize this spike for upstream contribution.
