# Houdini specialized renderer path

Date: 2026-06-29

## Decision

Keep the first specialized Houdini renderer slice inside the native Rerun viewer.
Do not restart from Svelte, Tauri, React Flow, or Svelte Flow.
Do not move cubic Beziers or polygons into Rerun viewer state.

The first product-fork renderer path is a bounded renderer-native line draw-data path:

- `GraphDocument` remains the source of graph truth.
- `RerunSceneOutput` remains the viewer adapter.
- Small scenes use the detailed egui painter fallback.
- Large scenes use `LineDrawableBuilder`, `ViewBuilder`, `TargetConfiguration`, and `gpu_bridge::new_renderer_callback`.
- Cubic Beziers remain native graph records.
- Cubic expansion is CPU-prepared only at the viewer boundary for this slice.
- Dense line strips are not stored as graph geometry.

## Renderer Objects

The current object split is:

- `GraphDocument`: owns polygons, native cubic Beziers, layers, styles, filters, source metadata, and graph evaluation state.
- `RerunSceneOutput`: contains graph-owned scene items plus optional boundary/debug items.
- `HoudiniGraphView`: chooses the preview mode and owns renderer integration.
- `HoudiniRendererPlan`: records the chosen path, fallback, draw-data shape, and cubic evaluation placement.
- `LineDrawableBuilder`: builds batched renderer line-strip draw data.
- `ViewBuilder`: queues renderer draw data for the viewport callback.

The draw-data shape for the first renderer-native slice is batched 2D line strips:

- Polygons become closed line-strip rings.
- Native cubic Beziers become CPU-sampled line strips only at the viewer boundary.
- Selected items stay included even if future large-scene stride logic skips surrounding items.

## Cost Split

The view now records these per-frame buckets:

- Scene preparation: source/query bridge update plus `RerunSceneOutput` construction.
- Boundary expansion: CPU conversion from graph-native polygons/cubics into preview line points.
- Draw-data build: renderer line draw-data construction and queueing into `ViewBuilder`.
- Renderer callback: enqueueing the `gpu_bridge` callback into egui.
- Draw total: total Houdini view drawing work around those buckets.

This does not yet measure GPU execution time directly.
That should be added when the product fork introduces a true Houdini renderer instead of a generic line-strip renderer preview.

## CPU vs GPU Cubic Evaluation

For this slice, native cubic evaluation stays CPU-side at the viewer boundary.
That is the conservative product choice because it keeps the graph model stable and avoids introducing a custom WGSL path before we know whether line-strip draw-data upload or shader work is the real bottleneck.

Shader-side cubic evaluation remains deferred.
The likely future custom renderer would look closer to the point-cloud path:

- `HoudiniGeometryBuilder`
- `HoudiniGeometryDrawData`
- `HoudiniGeometryRenderer`
- a WGSL path that evaluates cubic Beziers from four control points

That future path should cache GPU buffers by graph/output cache keys and should avoid rebuilding sampled strips every frame.

## Fallback

The fallback remains the egui painter path.
Small scenes intentionally keep native egui cubic shapes because they are readable, simple, and adequate for inspection.
Renderer failures in the large-scene path fall back to the fast egui preview and log once.

## Status

This issue does not claim the final custom Houdini renderer is done.
It closes the first specialized renderer-path spike by making the current renderer-native draw-data boundary explicit, instrumented, test-covered, and documented.

The next renderer-focused issue should only be opened if profiling shows that CPU boundary expansion or draw-data upload is still the high-scale bottleneck after this instrumentation is exercised against mixed point-cloud plus curve/polygon scenes.
