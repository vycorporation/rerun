# Houdini native plugin operator lane

Date: 2026-06-28

## Decision

Define a narrow trusted native plugin lane for Houdini graph operators.
Native plugins are local, trusted, product-fork operator implementations with typed graph contracts.
They are not frontend plugins, browser extensions, marketplace plugins, remote operators, or arbitrary viewer scripting.

The lane exists for cases where Rust/native code is the right implementation vehicle for performance, hardware access, existing native libraries, or deeper integration than Python should own.
It should remain smaller and more explicit than the Python operator lane.

## Native operator declaration

A native operator declaration must include operator id, display name, version, ABI or host compatibility version, implementation location, typed input kinds, typed output kinds, parameter schema, capability declarations, provenance fields, failure modes, and documentation.

Input and output kinds should use the same graph-level vocabulary as the rest of the Houdini product fork.
The v1 kinds should include geometry tables compatible with `HoudiniGeometryRecord`, attribute tables, scalar values, string values, layer/style metadata, and optional artifact references.
Native cubic Beziers remain native geometry records.
Dense curve tessellation is allowed only as viewer, debug, or export representation.

Parameters must be graph-owned and inspectable.
Each parameter declares type, label, help text, default value, optional range, optional enum values, and cache invalidation behavior.
Parameter values must not be hidden inside plugin-local configuration.

## Capabilities and permissions

Every native operator declares the capabilities it needs.
The initial capability set should include file reads, file writes, network access, subprocess access, GPU access, native library loading, environment mutation, and long-running execution.

The v1 lane is trusted desktop execution.
It should require explicit project trust or explicit operator enablement before running native code.
Opening a project should not silently execute a native plugin.

The product should not promise sandboxing in v1.
It should communicate that native operators run with trusted local privileges and are therefore a narrower, more deliberate lane than graph assets or Python declarations.

## Provenance and versioning

Every successful native operator output should record operator id, operator version, implementation digest, host compatibility version, parameter digest, input cache keys, capability settings, execution timestamp, and output counts.
If the native implementation changes, dependent nodes become stale.
If the host compatibility version changes, the operator should be disabled or require revalidation until compatibility is confirmed.

Version and provenance must be visible in node inspection.
The graph should be able to explain which native operator produced a result and with which inputs.

## Failure modes

Native operators should report failures through the same demand-driven evaluation vocabulary as the rest of the graph.
Failures include load failure, incompatible host version, missing capability grant, dependency/library load failure, invalid input type, runtime error, timeout, cancellation, and output schema mismatch.

Failure reporting should preserve previous valid cached output when available.
The inspector should show a concise failure summary plus diagnostic detail.
Unbounded native logs should not be dumped into the graph panel.

## Demand-driven evaluation

Native operator nodes participate in stale, running, cached, failed, manual, and clean states.
Cache keys should include implementation digest, declaration version, parameter digest, input cache keys, host compatibility version, and capability settings.

Manual-run controls are appropriate by default.
Automatic evaluation should require the operator to declare itself cheap or the user to opt in.

Native operators should receive typed inputs and produce typed outputs.
They should not mutate graph state outside their declared output.
They should not call `ViewerContext`, egui, renderer resources, or `AppState` directly.

## Distinction from procedural assets

Procedural assets wrap graph or subgraph data behind a declarative boundary.
They require no custom plugin code.
They are inspectable by expanding the graph internals.

Native plugins provide custom native implementation code behind a typed operator contract.
They are appropriate when a graph asset cannot express the operation efficiently or safely.
They are not a replacement for graph-backed procedural assets.

If a reusable operation can be represented as a graph or subgraph, it should be a procedural asset first.
If it needs native performance, hardware access, or native library integration, it can be a native plugin operator.

## Deferred surfaces

Frontend plugins are deferred.
Remote operators are deferred.
Marketplace distribution is deferred.
Untrusted sandboxing is deferred.
Custom viewport handles are deferred.
Native renderer extensions are deferred unless they are handled through the specialized renderer workstream.

Deferring these surfaces keeps the product fork focused on graph-visible dataflow and avoids opening a broad extension platform before the core graph runtime is stable.

## Implementation follow-ups

1. #52 - Add a serializable trusted native operator declaration model.
2. #51 - Add native operator node inspection and demand-driven state integration.
3. #53 - Prototype native operator loading with explicit project trust and capability checks.
4. #54 - Add native operator cache/provenance records and compatibility invalidation.
