# Rerun-Native Graph UI Spike

## Goal

Prove whether a Houdini-like graph, layer, parameter, and node-info workflow can live inside or alongside Rerun's native Rust viewer framework without committing to a maintained fork.

The spike validates the smallest useful workflow:

```text
Source -> Filter -> Style -> Rerun Output
```

The source data should be polygon-heavy and include cubic Bezier curves as a first-class case. Point-cloud scale, procedural assets, plugin systems, Python environments, and custom shelf tools are intentionally outside this first spike.

## Primary Questions

1. Can we add native graph/layer/parameter panels using Rerun's viewer extension or embedding path?
2. Can a graph-owned workflow drive Rerun display without hiding semantics inside viewer state?
3. Can layer visibility, parameter edits, and node info feel close enough to the Houdini UX direction?
4. Can this be done upstream-compatibly, or does it immediately require a hard fork?
5. Can cubic Bezier curves remain native graph data without forcing dense polyline expansion for ordinary viewing?

## Non-Goals

- No SvelteKit, Tauri, React Flow, Svelte Flow, or JavaScript graph shell.
- No full procedural asset authoring.
- No Python dependency management.
- No point-cloud performance gate.
- No editable attribute table.
- No VFX timeline or animation system.
- No permanent fork unless the spike proves it is necessary.

## Milestones

### 1. Rerun Extension Skeleton

Create a minimal Rust app or crate that embeds or extends the Rerun viewer and adds at least one custom panel.

Acceptance criteria:

- The project builds from a clean checkout.
- The Rerun viewer still opens normally.
- A custom graph-related panel can be shown, hidden, and docked or placed consistently with Rerun's UI model.
- The approach is documented as one of: upstream extension, embedding, light patch, or hard fork.

### 2. Minimal Graph Model

Implement an in-memory graph model independent of Rerun viewer state.

Acceptance criteria:

- The model has typed nodes, inputs, outputs, parameters, and connections.
- It supports the four node types: Source, Filter, Style, Rerun Output.
- Disconnected exploratory nodes are allowed.
- Evaluation is demand-driven for the Rerun Output path.

### 3. Polygon And Curve Workflow

Load or generate a small polygon and cubic Bezier dataset and run it through Source, Filter, and Style nodes.

Acceptance criteria:

- Source exposes record count, bounds, attribute names, and geometry kind.
- Filter can include/exclude records by one simple attribute rule.
- Style can set at least color and opacity.
- Parameter edits update downstream output intentionally, not through hidden viewer-only state.
- Cubic Bezier curves remain represented as curves in the graph model.

### 3a. Native Curve Rendering Feasibility

Test whether Rerun can display cubic Bezier curves without converting them into dense durable polylines, either through an upstream-compatible custom view, a narrow renderer extension, adaptive prepared representation, or a separate viewer target.

Acceptance criteria:

- The prototype documents whether Rerun supports the needed curve path directly, through extension, through light patching, or not at all.
- A layer with at least 100,000 cubic Bezier curves is estimated or tested against the chosen representation.
- Any tessellation is adaptive, viewer-specific, and treated as a prepared representation rather than durable graph data.
- If Rerun requires dense line strips for the first pass, the spike identifies whether a custom or external viewer target is required.

### 4. Rerun Output And Layers

Send the styled polygon/curve result to Rerun through a graph-visible output node.

Acceptance criteria:

- The output appears in the Rerun viewer.
- A layer-like control can toggle visibility without deleting graph state.
- Node output metadata remains inspectable after display.
- The Rerun-specific integration is isolated to output-target code.

### 5. Node Info And Parameter UX

Add Houdini-inspired inspection and editing surfaces.

Acceptance criteria:

- Selecting a node shows editable parameters in a linked parameter panel.
- Selecting a node or output shows node info: status, data kind, record count, bounds, attributes, warnings, and provenance.
- System state is visually distinct from user organization.
- Basic failure state is visible when a node cannot evaluate.

### 6. Durable Recording Check

Test whether the graph can optionally produce a durable Rerun recording artifact for the same output.

Acceptance criteria:

- The workflow can stream live to the viewer.
- The same workflow can save or identify the path toward saving a replayable Rerun recording.
- Any limitations are documented.

## Kill Criteria

Stop pursuing the Rerun-native path, or downgrade it to a later option, if any of these are true:

- Adding custom graph/layer/parameter panels requires a broad hard fork before the first workflow works.
- Rerun's extension or embedding APIs are too unstable to support iterative development without constant breakage.
- The graph model must be coupled tightly to Rerun viewer internals to make basic interactions work.
- Layer visibility or styling can only be represented as hidden Rerun viewer state rather than graph-owned data.
- Cubic Bezier layers require exploding ordinary workflows into millions of durable polyline points.
- Basic parameter edits and output refresh feel substantially worse than a separate graph UI would.
- The build and dependency workflow becomes slower or more fragile than the product benefit justifies.

## Continue Criteria

Continue with the Rerun-native path if the spike demonstrates:

- Custom panels are practical.
- The graph model can stay independent.
- Rerun can be treated as a viewer target through explicit output operators.
- Polygon filtering/styling feels interactive enough for the first workflow.
- Cubic Bezier curves can stay native in the graph and have a plausible interactive rendering path.
- Node info and parameter editing feel compatible with the Houdini-inspired UX direction.
- Any required Rerun changes are small enough to propose upstream or maintain narrowly.

## Deliverables

- A runnable prototype or branch.
- A short decision memo: continue upstream extension, create a narrow fork, return to a separate UI, or pause.
- Notes on Rerun APIs touched and stability concerns.
- Screenshots or screen recording of the graph panel, parameter panel, node info, layer visibility, and Rerun output.
- Follow-up issues for the next spike if the path continues.
