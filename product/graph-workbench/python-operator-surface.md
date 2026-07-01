# Houdini Python operator surface

Date: 2026-06-28

## Decision

Define Python operators as first-class Houdini graph nodes, not as viewer scripts.
The v1 surface should let a trusted desktop project run Python code against typed graph inputs and emit typed graph outputs that feed the existing curve and polygon path.
The first implementation should be narrow, inspectable, cacheable, and reproducible.

Python is a product capability because the target workflows need OpenCV, NumPy, PyTorch, geometry libraries, and project-specific analysis code.
That capability must still preserve the current graph model boundary.
The graph remains the source of truth; Python is one operator implementation lane inside the graph.

## Operator contract

A Python operator declaration has a stable operator id, display name, version, source file or module entry point, typed inputs, typed outputs, typed parameters, dependency declarations, capability declarations, and help text.
The declaration is graph data and must persist with the Houdini graph sidecar or project model.

An operator instance is a graph node.
It stores parameter values, incoming edge bindings, output cache metadata, evaluation state, provenance, warnings, and the last failure summary.
It participates in the same demand-driven evaluation model as source, filter, style, and output nodes.

The v1 operator should be pure from the graph's point of view.
It receives declared inputs and parameters, writes declared outputs, and reports diagnostics.
It must not mutate Rerun viewer state, global graph state outside its declared output, layer visibility, selection, or viewport camera state.

## Typed inputs

The v1 input kinds should match the existing graph data model.
The minimum input is a geometry table containing records equivalent to `HoudiniGeometryRecord`.
Each record has a geometry kind, layer, score, and native geometry payload.
Native cubic Beziers are passed as four control points and a score, not as stored dense polylines.
Polygons are passed as point rings plus score and layer metadata.

Optional v1 inputs can include attribute tables, source metadata, graph parameter values, and a small execution context.
The execution context may include project id, operator node id, input cache keys, and output directory paths.
It must not include direct handles to `ViewerContext`, egui, `AppState`, or renderer resources.

Inputs should be serializable through Arrow-compatible tables or a stable local IPC format.
The implementation may start with a process boundary and temporary Arrow or Parquet files if that is the fastest safe slice.
That is an execution detail; the graph-level contract remains typed records and metadata.

## Typed outputs

The primary v1 output is a geometry table compatible with the current Houdini geometry path.
It can emit polygons and native cubic Beziers.
It may also emit scalar or string attributes that appear in the attribute table and can be used by graph filter nodes.

A Python operator result should feed the same path used by current Parquet import.
The runtime converts the result into `HoudiniGeometryRecord` values, attaches `SourceMetadata`, and then lets `GraphDocument` produce `RerunSceneOutput`.
No special Python-only renderer or viewer output path should exist in v1.

The output contract may include warnings, informational metrics, and lightweight artifacts.
Artifacts can include debug tables, logs, or preview files, but the graph output must remain typed and inspectable.

## Parameters

Parameters must be declared in the operator contract before execution.
The v1 parameter kinds should cover numeric scalars, booleans, strings, enums, file paths, and attribute-rule-like selectors.
Each parameter should declare a default value, display label, help text, optional range, optional allowed values, and whether a change invalidates the output cache.

Parameter values are graph-owned data.
They should show in node inspection and persist in the sidecar or project model.
They should not be hidden inside Python source code or process environment variables.

## Dependencies

Each Python operator declares its Python dependency requirements.
The declaration should support a project-local package requirement list, optional extras, and a compatibility marker for the expected Python version.
The declaration should not use the global system Python as the default path.

Dependency resolution belongs to the project Python environment tracked by #32.
The operator surface should record what it needs, while environment creation, locking, health checks, and repair UI are handled separately.

For reproducibility, an executed operator should record the resolved environment identity.
At minimum this should include a lock digest or environment digest, Python version, package set digest, operator source digest, and entry point.

## Provenance

Every successful output cache entry should record enough provenance to explain how it was produced.
The v1 provenance should include operator id, operator version, node id, source path, source digest, parameter digest, input cache keys, dependency lock digest, execution timestamp, and output record counts.

Provenance should be visible in node inspection.
It should also travel with exported or durable graph artifacts where practical.
This makes Python output auditable rather than a one-off local script side effect.

## Failure reporting

Failures should map into the existing node evaluation states.
A failed Python operator should set the node state to failed, preserve the previous clean cache when available, and show a concise failure message in node inspection.

The detailed failure payload should include exception type, traceback summary, stderr tail, stdout tail if useful, exit status, timeout state, dependency status, and whether the previous cache was reused.
The UI should avoid dumping unbounded logs into the graph panel.

Retry, cancel, stale, manual, running, cached, and clean states should reuse the existing demand-driven evaluation vocabulary.
Python should not get a parallel status model.

## Caching and reproducibility

The cache key should be derived from operator source digest, operator declaration version, parameter digest, input cache keys, dependency lock digest, and declared capability settings.
A change to any cache-key component marks the node stale.

Cached outputs should be typed graph outputs.
They should not be opaque Python object pickles.
If process-local acceleration is needed later, it can be an implementation cache beneath the typed graph cache, not the graph contract.

Manual-run controls are appropriate for expensive Python nodes.
Automatic evaluation is appropriate only when the operator declares itself cheap or when the user opts in.

## Security and permissions

The v1 lane is trusted desktop execution.
It is for local project code the user intentionally enables.
It is not a remote plugin system, marketplace extension system, browser plugin system, or untrusted sandbox.

The UI should clearly distinguish enabled trusted Python operators from inert declarations.
Opening a project should not silently execute Python.
The first execution of project Python should require an explicit user action or a project trust decision.

Capability declarations should cover file access roots, network access, subprocess access, GPU access, and environment mutation.
The v1 implementation can start with coarse trusted permissions, but the operator contract should leave room for narrower enforcement.

## V1 boundaries

Do support local Python functions that transform typed geometry and attributes.
Do support Python libraries through a project-managed environment.
Do support node inspection, diagnostics, cache state, and deterministic output records.

Do not support arbitrary viewer scripting.
Do not expose `ViewerContext`, egui, renderer objects, or `AppState` to Python.
Do not support remote execution in v1.
Do not support frontend plugins in v1.
Do not require custom plugin code for graph-backed procedural assets.

## Implementation follow-ups

1. #39 - Add a serializable Python operator declaration model.
2. #41 - Add a Python operator node kind that participates in graph layout, inspection, and demand-driven evaluation.
3. #40 - Add a process-boundary execution prototype that reads typed geometry input and writes typed geometry output.
4. #42 - Add cache/provenance records for Python operator outputs.
5. #32 - Connect dependency declarations to the project uv environment surface.
