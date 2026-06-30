# Houdini Clone

Houdini Clone is a procedural spatial exploration workspace for reading, filtering, and styling large 2D and 3D datasets through a node-based workflow. It centers on curve and polygon geometry while supporting large point clouds, reusable procedural tools, and a rearrangeable professional interface.

## Language

**Procedural Spatial Workspace**:
An interactive environment where 2D and 3D spatial data is loaded, transformed, analyzed, and visualized through explicit procedural steps.
_Avoid_: Houdini clone, GIS app, map viewer, geospatial workspace

**Houdini-Like Graph Ergonomics**:
Node-network interaction patterns modeled as closely as practical on Houdini's graph workflow and translated into spatial/CV vocabulary: rich node info, linked parameter panels, operator browsing, subnet-like subgraph navigation, visible generated nodes, network boxes, graph notes, exploratory canvas work, and typed exposed boundaries. Routine subnet, asset, selection, mask, filter, and style operations should follow Houdini behavior unless product domain constraints, strong data kinds, viewer-agnostic output targets, or spatial/CV workflows require translation.
_Avoid_: VFX scope, blind Houdini clone, animation system, simulation workflow, dense-polyline curve storage

**Capability Level**:
The set of workspace features available in a runtime environment, such as browser or desktop.
_Avoid_: Edition, mode, platform type

**Native Backend**:
The trusted local runtime available to desktop builds for local data access, indexing, prepared representations, caching, and heavy operator execution.
_Avoid_: Server, daemon, backend service

**Project**:
A saved workspace that contains named node graphs, layers, assets, viewport state, and references to source datasets.
_Avoid_: Scene, document, file

**Project Command**:
An undoable change to the project model, such as creating, deleting, wiring, renaming, moving, coloring, or commenting nodes; editing parameters; changing layer promotion; editing assets; changing graph layout; or editing panel layout.
_Avoid_: UI event, action, mutation

**Project Command History**:
The undo and redo history of project commands. Project command history restores graph and project intent, but does not store evaluation results, cached outputs, running work, or other runtime evaluation state.
_Avoid_: Runtime history, evaluation replay, cache rollback

**Coalesced Project Command**:
A single undoable project command created from a completed high-frequency gesture, such as dragging a node, resizing a network box, editing a graph note, or scrubbing a parameter slider. Coalescing keeps undo and redo aligned with user intent rather than intermediate pointer or value updates.
_Avoid_: Raw gesture events, every mousemove, runtime history

**First Malware Artifact Workflow**:
The initial graph workflow for inspecting external malware-analysis artifacts: load a defanged malware binary or artifact reference, load one or more analysis substrates, load malware region polygons, filter and style those regions, inspect node information, organize the graph, and send the result to a Rerun output operator.
_Avoid_: Training workflow, dataset browser clone, malware pipeline runner

**Spatial Dataset**:
A collection of records with geometry and attributes in a shared working space.
_Avoid_: File, table

**Record**:
A single item in a spatial dataset, preserving the source geometry and attributes that users reason about.
_Avoid_: Row, feature, entity

**Record Identity**:
A stable identifier or provenance key that lets graph outputs, filtered views, selections, styles, node info, and viewer-ready outputs refer back to the same source record without copying the source dataset.
_Avoid_: Viewer id, row number only, temporary render id

**Derived Record Identity**:
A record identity computed when a source artifact lacks an explicit id, such as from the artifact reference plus row group and row index or from a content hash of the record.
_Avoid_: Render id, unstable row order, hidden fallback

**Record Identity Mapping**:
A graph-visible relationship from input record identities to output record identities produced by an operator that preserves, splits, merges, or replaces records. Without a record identity mapping, downstream selections can only match identities that still exist directly.
_Avoid_: Hidden provenance, row mapping, best-effort selection transfer

**Source Provenance**:
Graph-owned metadata describing where records or node outputs came from, such as source dataset, source node, source output, source path, source layer, or reference target. Source provenance supports inspection, filtering, packaging, and output-target mapping without requiring source data to be copied.
_Avoid_: Hidden origin, copied source column, viewer-only metadata

**Source Dataset**:
A spatial dataset as it exists before the workspace applies filtering, styling, or procedural transformation.
_Avoid_: Raw file, input file, import

**Dataset Reference**:
A stored path, URI, or locator that points to a source dataset outside the project file.
_Avoid_: Embedded data, file copy, data import

**External Artifact Reference**:
A stored locator for an external analysis artifact, model weight file, Python resource, recording, or other large dependency used by a project, node, or asset definition without embedding the artifact itself.
_Avoid_: Embedded artifact, bundled copy, hidden dependency

**Cached Metadata**:
Stored summary information about a dataset or node output used for fast inspection, validation, and reopening.
_Avoid_: Cache, preview data, derived data

**Prepared Representation**:
A derived representation of a source dataset or node output created for faster rendering, filtering, styling, or analysis while preserving the logical records.
_Avoid_: Imported copy, converted dataset, optimized file

**External Analysis Artifact**:
A file, recording, image, table, geometry set, or other output produced by an external pipeline and loaded into the workspace for inspection, filtering, styling, comparison, or annotation.
_Avoid_: In-app analysis job, hidden pipeline result, internal model output

**Analysis Artifact Manifest**:
An optional manifest that groups external analysis artifacts, such as substrate images and polygon parquet layers, and declares how they should be loaded together as a visible starter graph.
_Avoid_: Required binding file, hidden import state, dataset registry

**Starter Graph**:
A visible set of generated nodes created by a manifest import or shelf tool to begin a workflow, such as substrate source nodes, polygon source nodes, default style nodes, layer promotions, and a composed viewer output.
_Avoid_: Hidden import result, wizard state, single magic import node

**FiftyOne-Inspired Inspection Affordance**:
A viewing or filtering interaction inspired by Voxel51 FiftyOne, such as field filters, label or polygon filters, saved subsets, sidebar-style metadata navigation, and fast sample or record browsing, adapted into graph-backed spatial workflows.
_Avoid_: FiftyOne clone, dataset platform, hidden view state

**Analysis Substrate**:
A pixel-addressable representation produced from a source artifact for inspection, comparison, or annotation, such as a byteplot image, Markov transition matrix, entropy image, or other pipeline-produced view.
_Avoid_: Source binary, original image, one-to-one byte map

**Image Substrate Source**:
A source operator that loads an image-like analysis substrate and exposes its substrate pixel space, dimensions, and substrate coordinate contract.
_Avoid_: Generic image import, texture loader, hidden viewer image

**Substrate Pixel Space**:
The pure pixel coordinate frame of an analysis substrate. Polygon geometry associated with a substrate is authored and stored in this pixel space, independent of whether the substrate has a one-to-one mapping back to bytes or records.
_Avoid_: Byte offset space, inferred source coordinates, geospatial coordinates

**Substrate Coordinate Contract**:
The explicit metadata that defines an analysis substrate's dimensions, origin, orientation, pixel coordinate convention, and any optional mapping back to source bytes, records, or other substrates.
_Avoid_: Same-size assumption, implicit mapping, viewer transform

**Substrate Projection**:
An explicit graph operation that transforms polygons or other spatial annotations from one analysis substrate's pixel space into another substrate's pixel space using a substrate coordinate contract.
_Avoid_: Polygon reuse, same-size transfer, implicit overlay, hidden reference transform

**Substrate Overlay Layer**:
A polygon or vector layer loaded into a chosen or manifest-declared substrate pixel space for inspection and styling, relying on user or manifest alignment rather than an intrinsic hard link to the substrate artifact.
_Avoid_: Required substrate binding, geospatial vector layer, hidden coordinate transform

**Polygon Overlay Layer**:
A substrate overlay layer backed by polygon parquet records, with independent visibility, filtering, styling, record identity, and node outputs from other polygon overlay layers in the same substrate pixel space.
_Avoid_: Merged polygon layer, hidden label field, single annotation layer

**Overlay Validation Warning**:
A non-blocking warning shown when a substrate overlay layer appears suspicious for the chosen substrate, such as bounds outside the substrate dimensions or likely scale, origin, or orientation mismatch.
_Avoid_: Hard binding failure, automatic projection, silent mismatch

**Defanged Malware Binary**:
A malware sample made inert enough for safe inspection and analysis workflows while preserving the byte structure users need to reason about.
_Avoid_: Executable sample, live malware, generic binary

**Byteplot Representation**:
An analysis substrate that represents a binary's byte sequence as an image-like pixel space for visual inspection and spatial annotation.
_Avoid_: Screenshot, texture, imported image only

**Markov Transition Matrix Substrate**:
A 256-by-256 analysis substrate representing byte-transition relationships rather than a one-to-one byte-to-pixel layout.
_Avoid_: Byteplot, direct binary image, rescaled byte map

**Malware Region Polygon**:
A polygon stored in one analysis substrate's pixel space that marks a malware-relevant region, detected structure, analyst annotation, or pipeline-produced region of interest.
_Avoid_: Detection overlay, hidden mask, viewer-only annotation

**Polygon Parquet Source**:
A source operator that loads one malware-region polygon parquet file and exposes typed polygon records in a user-selected or manifest-declared substrate pixel space. It prefers explicit record id columns and falls back to derived record identity with a visible warning.
_Avoid_: Generic table import, hidden overlay file, viewer annotation loader

**Render Acceleration**:
A prepared representation or GPU-side operation that speeds up viewport display without changing the durable graph meaning.
_Avoid_: Renderer pipeline, hidden filter, display-only result

**Packaged Project**:
A portable project bundle that includes project graph state, asset definitions, metadata, small manifests, and explicitly selected external source datasets or external artifacts alongside the project definition.
_Avoid_: Asset definition, hidden archive, implicit copy

**Artifact Bundle**:
An explicit portable bundle of selected heavy external artifacts, such as datasets, recordings, model weights, Python resources, or analysis outputs, created separately from reusable asset definitions.
_Avoid_: Asset, hidden dependency copy, automatic data embed

**Artifact Inclusion**:
An explicit user choice to copy a large external artifact into a packaged project or artifact bundle instead of leaving it as an external reference.
_Avoid_: Implicit copy, automatic inclusion, hidden embed

**Content Hash**:
A reproducibility identifier computed from artifact contents when feasible, used to verify bundled or externally referenced artifacts.
_Avoid_: File name, path, row id, display label

**Bundle Manifest**:
A manifest written with a packaged project or artifact bundle that records each artifact's role, original locator, bundled path when copied, size, content hash when feasible, source or output node provenance, and whether the artifact remains external.
_Avoid_: Hidden package index, file list only, asset definition

**Packaging Preview**:
A preflight summary shown before creating a packaged project or artifact bundle, including expected size, included files, references left external, artifact inclusion choices, available content hashes, dependency requirements, and reproducibility warnings.
_Avoid_: Silent package, background copy, export complete message

**Point Cloud**:
A spatial dataset made primarily of many individual point records.
_Avoid_: Points, particles

**Benchmark Dataset**:
A representative dataset used to validate rendering, interaction, filtering, styling, and inspection performance.
_Avoid_: Test file, sample data, demo data

**Performance Gate**:
A pass-or-fail benchmark threshold that determines whether a foundation, feature, or implementation path is viable.
_Avoid_: Performance goal, benchmark, target

**External Prototype**:
A validation build that uses an upstream foundation without modifying it directly.
_Avoid_: Fork, spike branch, throwaway app

**Foundation Fork**:
A maintained fork of an upstream foundation used when required product capabilities cannot be achieved through extension or integration.
_Avoid_: Clone, vendor copy, custom build

**Driven Tool**:
An external viewer, runtime, or application controlled by the node graph as part of a workflow.
_Avoid_: Backend, plugin, embedded view

**Graph Orchestration**:
Using the node graph to coordinate data, parameters, operations, and outputs across one or more driven tools.
_Avoid_: Tool automation, scripting, pipeline

**Output Target Contract**:
The stable typed architectural boundary between graph-owned outputs or commands and a viewer, runtime, file, service, or tool that consumes them. It carries graph-owned semantic payloads, durable presentation intent, status, provenance, record identity, time semantics, capability requirements, and output commands while target-specific APIs, entity paths, timelines, UI state, panel state, transport handles, session handles, and adapter internals remain target-owned.
_Avoid_: Rerun interface, viewer API, integration layer

**Output Command**:
A graph-owned request sent through the output target contract, such as display, stream, save, export, package, record, publish, clear, or update a named output target state.
_Avoid_: Viewer callback, API call, UI event

**Output Target Capability**:
A declared ability of an output target, such as supported geometry kinds, image data, temporal data, annotations, native curves, recordings, interactivity, export formats, or streaming modes.
_Avoid_: Target implementation detail, hidden feature flag, viewer setting

**Capability Negotiation**:
The visible process of matching output target capability requirements against a target's declared capabilities and choosing a supported outcome: native mapping, declared prepared representation, lower-fidelity mapping with warning, or unsupported output.
_Avoid_: Silent fallback, hidden conversion, viewer surprise

**Lower-Fidelity Mapping**:
A target-native mapping that represents graph-owned output with reduced precision, detail, interactivity, or semantic richness because the output target lacks a requested capability.
_Avoid_: Native mapping, silent degradation, graph mutation

**Target Adapter**:
The target-owned component that implements an output target contract for a specific viewer, file format, service, runtime, or tool by mapping graph-owned outputs and commands into target-native concepts.
_Avoid_: Graph runtime, private schema, viewer state

**Target-Native Mapping**:
The translation from graph-owned semantic data kinds and output commands into the native concepts of a specific output target.
_Avoid_: Private viewer schema, graph data model, hidden adapter behavior

**Output Target**:
A viewer, runtime, file, service, or tool that receives typed graph outputs or commands.
_Avoid_: Sink, destination, export target

**Preferred Output Target**:
The output target requested by an output operator or user action before capability negotiation determines whether the target can satisfy the requested output.
_Avoid_: Forced target, adapter default, hidden routing

**Target Choice**:
A user-visible decision when capability negotiation finds multiple, imperfect, or unsupported output targets, such as using the preferred target with a warning, switching target, using a prepared representation, or canceling.
_Avoid_: Silent fallback, automatic target switch, hidden downgrade

**Viewer Target**:
An output target that displays graph outputs interactively.
_Avoid_: Viewport backend, renderer, driven viewer

**Rerun Viewer Target**:
A viewer target backed by Rerun for interactive 2D, 3D, temporal, and computer-vision-oriented inspection.
_Avoid_: Rerun backend, embedded Rerun, Rerun plugin

**Viewer Extension**:
A native extension or embedding layer that adds panels, controls, or workflows to a driven tool using that tool's own UI framework.
_Avoid_: Web wrapper, plugin, fork

**Rerun-Native Graph UI**:
A graph, layer, parameter, and orchestration UI built with Rerun's native Rust viewer framework rather than a separate JavaScript frontend.
_Avoid_: Svelte graph, web graph, external panel

**Spawned Viewer**:
An external viewer process started or targeted by the orchestration UI.
_Avoid_: Embedded viewer, child app, subprocess UI

**Embedded Viewer**:
A viewer displayed inside a workspace panel as part of the unified app interface.
_Avoid_: Webview, iframe, built-in viewport

**Rerun Output Operator**:
A target-specialized output operator whose preferred output target is a Rerun viewer target and that communicates compatible graph outputs, Rerun-targeted options, and output commands through the output target contract.
_Avoid_: Rerun node, viewer backend, display node

**Rerun-Native Mapping**:
A target-native mapping that expresses compatible graph outputs through Rerun ecosystem concepts such as archetypes, components, timelines, annotations, and recordings.
_Avoid_: Custom Rerun bypass, private overlay format, graph-owned Rerun schema

**Viewer-Ready Output**:
A target-specific representation prepared from graph-owned filtered views, styles, and output commands for the current viewer target, containing only what the viewer needs to display or replay the requested state while preserving record identity where applicable.
_Avoid_: Durable graph state, full source dataset, hidden materialized copy

**Composed Viewer Output**:
A viewer-ready output that combines a substrate and its visible styled overlay layers into one requested view for an output target.
_Avoid_: Per-layer output by default, hidden viewer scene, layer-only composition

**Geometry**:
The spatial shape or position carried by a record, such as a curve, polygon, point, or mesh.
_Avoid_: Shape, feature shape

**Curve**:
A one-dimensional geometry defined by one or more connected curve segments.
_Avoid_: Line, path

**Cubic Bezier Curve**:
A curve segment controlled by two endpoints and two handles.
_Avoid_: Bezier, spline

**Native Curve Representation**:
A curve representation that preserves compact curve primitives instead of replacing them with dense line segments.
_Avoid_: Polyline, linestring, tessellated curve

**Curve Tessellation**:
The process of converting a curve into line segments for compatibility, rendering, or export.
_Avoid_: Curve import, curve storage, native curve

**Native Curve Rendering**:
Rendering curves from native curve representations or adaptive prepared representations without making dense polylines the durable graph data.
_Avoid_: Polyline rendering, linestring rendering, curve conversion

**Polygon**:
A two-dimensional closed geometry that represents an area.
_Avoid_: Region, area, shape

**Attribute**:
A named value attached to a spatial record and used for filtering, styling, analysis, or procedural operations.
_Avoid_: Property, metadata, column

**Attribute Table**:
A tabular inspection view of records and attributes for a node output or layer view.
_Avoid_: Spreadsheet, data grid, table editor

**Read-Only Attribute Table**:
An attribute table that supports inspection, search, sort, and temporary filtering without directly editing record values.
_Avoid_: Editable table, spreadsheet, data editor

**Table Filter**:
A temporary filter used inside an attribute table for inspection before the user commits it to the graph.
_Avoid_: Graph filter, layer filter, query node

**Table Selection**:
A temporary set of selected records in an attribute table used for inspection, highlighting, or graph-authoring shortcuts before the user commits it to the graph.
_Avoid_: Graph selection, saved subset, selected dataset

**Committed Filter**:
A temporary inspection filter that the user explicitly turns into graph-backed filter data by creating or editing a visible filter node. It edits an existing managed generated filter node when clearly bound, and creates a new node when no binding exists or the user chooses to branch or compare filters.
_Avoid_: Applied filter, saved query, promoted filter

**Subset-to-Filter Action**:
An explicit graph-authoring action that turns selection or mask subset data into a visible filter operator wired to the subset and its source branch.
_Avoid_: Implicit selection filter, layer-local hide, materialized subset copy

**Subset-to-Style Action**:
An explicit graph-authoring action that turns selection or mask subset data into visible style graph data or a style binding for highlighting, opacity, stroke, fill, labels, or other appearance. The subset remains data; visual appearance belongs to style graph data.
_Avoid_: Implicit selected style, layer-local highlight, selection-owned appearance

**Comparison Branch**:
A visible graph branch created to compare alternate filters, styles, selections, or other parameter choices from a shared upstream source. Comparison actions may create layer promotions, default styles, and a network box for the branches; manually created branches are not auto-promoted unless requested.
_Avoid_: Hidden UI history, duplicate dataset copy, temporary view tab

**Time Attribute**:
An attribute representing time or sequence information on records.
_Avoid_: Timeline, frame, animation

**Observation Recording**:
A time-aware collection of spatial, vision, or sensor observations captured for inspection and analysis.
_Avoid_: Animation, video, project

**Durable Recording**:
A saved observation recording artifact that can be replayed, shared, compared, or inspected later.
_Avoid_: Live stream, export, project file

**Recording Timeline**:
A timeline used to browse logged observations or captured states, not to author VFX-style animation.
_Avoid_: Animation timeline, frame range, playback track

**Filter**:
A rule that selects which records from a spatial dataset remain visible or available downstream.
_Avoid_: Query, mask, selection

**Filtered View**:
A graph-backed logical view of source records selected by filter data, masks, or compatible predicates without materializing a copied dataset by default, preserving record identity for downstream inspection and output.
_Avoid_: Filtered dataset copy, exported subset, duplicated parquet

**Layer Filter Edit**:
A project command made through the layer stack that creates or edits filter graph data behind the layer view, marking affected downstream outputs stale according to the evaluation mode.
_Avoid_: Filter override, display filter, layer-only filter

**Transient Filter**:
A temporary search, highlight, or isolate condition used for immediate viewport exploration without changing the node graph.
_Avoid_: Filter, mask, graph filter

**Style**:
Graph data that defines visual mappings for records or geometry in a viewport, such as color, opacity, width, size, or material.
_Avoid_: Symbology, appearance, render settings

**Default Style Node**:
A visible generated style node created for a promoted layer or starter graph so layer-facing style edits remain graph-backed.
_Avoid_: Layer-only style, hidden renderer setting, implicit symbology

**Viewport State**:
Application- or viewer-owned viewing context, such as camera position, hover targets, transient selections, and saved view configuration, that does not change the node graph.
_Avoid_: Style, layer state, graph state, camera node

**Transient Selection**:
A temporary hover, click, brush, viewport pick, or table selection used for immediate inspection without changing the node graph.
_Avoid_: Selection, mask, picked data

**Selection**:
Graph subset data that identifies records or geometry chosen for durable filtering, styling, analysis, or asset inputs. A selection does not remove records by itself; filtering requires an explicit filter operator or selection-to-filter action.
_Avoid_: Highlight, picked items, selected state

**Selection Commit**:
An explicit graph-authoring action that turns a transient viewport or table selection into visible graph data. A selection commit creates a new selection or mask node by default, and edits an existing node only when the target node is unambiguous and the user chooses an update action.
_Avoid_: Auto-save selection, hidden selection state, silent overwrite

**Committed Selection**:
A transient viewport or table selection that the user explicitly turns into graph-backed selection or mask data by creating or editing a visible selection or mask node. Committed selections refer to records through record identity or derived record identity rather than row position, viewport pick id, or screen position.
_Avoid_: Hidden layer selection, saved UI highlight, selected dataset copy, row-index selection

**Selection Identity Intersection**:
The result of applying a committed selection to a downstream branch by retaining only selected record identities that still exist in that branch. Missing identities are reported for inspection but are not errors unless a required operation depends on them.
_Avoid_: Best-effort selection transfer, row-position selection, silent mismatch

**Mask**:
A typed selection-like subset data kind that marks which records or geometry elements are included by a condition. A mask does not remove records by itself; filtering requires an explicit filter operator or mask-to-filter action.
_Avoid_: Filter, query result, selection state

**Group Alias**:
A Houdini-compatible alias that helps users find selection and mask operators through search, documentation, or help text. Group is not the canonical domain term for durable record subsets because the product also uses grouping for network boxes, manifests, parameters, panels, and general organization.
_Avoid_: Canonical group data kind, network box, node group, layer group

**Layer**:
A user-facing representation of a specific node output used to organize presentation handles such as visibility, order, name, and style binding in the viewport.
_Avoid_: Dataset, file, node

**Layer View**:
A layer-specific visual interpretation of a node output, including its name, visibility, order, and the graph-backed style data used to draw it.
_Avoid_: Duplicate layer, display copy, view layer

**Style Binding**:
The association between a layer view and the style graph data used to draw it.
_Avoid_: Style override, layer style, renderer setting

**Layer Style Edit**:
A project command made through the layer stack that creates or edits style graph data behind the layer view, marking affected downstream viewer-ready outputs stale according to the evaluation mode.
_Avoid_: Style override, display setting, layer-only style

**Generated Node**:
A real graph node created by a higher-level UI action, such as a layer filter edit, layer style edit, import action, or asset authoring action. Generated nodes are inspectable and editable; generation records provenance and initial organization rather than making the node hidden or untouchable.
_Avoid_: Hidden node, implicit node, auto node, internal node

**Managed Generated Node**:
A generated node that remains visibly bound to a layer-facing control or higher-level UI action, allowing safe two-way edits while preserving graph inspectability.
_Avoid_: Hidden managed node, internal state, layer-only behavior

**Generated Region**:
An organized area or grouping in the node graph where generated nodes are placed for readability, such as nodes created together by a manifest import or shelf tool.
_Avoid_: Hidden group, auto layout, clutter area

**Layer Binding**:
The visible association between a layer-facing control and graph-backed nodes or outputs that lets safe UI edits update graph data without creating a hidden second model.
_Avoid_: UI binding, layer state, implicit graph state

**Unbound Layer Binding**:
A layer binding that has broken because the related node, output, or data kind no longer satisfies the layer-facing contract.
_Avoid_: Hidden fallback, silently detached layer, viewer-only layer

**Binding-Preserving Edit**:
A user change to a managed generated node that preserves its layer binding because the edit is organizational or keeps the node output compatible, such as moving the node, changing node color, adding notes or network boxes, renaming display labels, editing compatible filter or style parameters, or temporarily bypassing while compatible input data passes through.
_Avoid_: Structural edit, ownership transfer, unbind

**Structural Edit**:
A user change to a managed generated node that can transfer ownership or break layer compatibility, such as rewiring inputs, changing the operator family, replacing outputs, changing data kind, deleting a managed node, inserting nodes into its managed chain, editing generated provenance, or changing parameters that no longer map cleanly to a layer-facing control.
_Avoid_: Binding-preserving edit, harmless edit, layout edit

**Adopted Node**:
A previously generated node that the user has explicitly taken over as ordinary graph material through structural edits.
_Avoid_: Detached generated node, converted node, user copy

**Graph Layout Item**:
A graph-canvas object that can be positioned, selected, moved, or visually organized, such as a node, graph note, connection routing dot, pinned connection routing dot, or other non-semantic canvas affordance.
_Avoid_: Data kind, graph output, evaluation unit

**Network Box**:
A named, optionally colored visual grouping around graph layout items used to organize the graph canvas without changing data flow. Removing a network box removes only the visual grouping unless the user explicitly chooses to delete its contents; network boxes can help users select nodes for explicit subgraph or procedural asset creation, but they do not automatically become graph containers or assets.
_Avoid_: Group, subgraph, graph container

**Network Box Label**:
A short user-authored title or comment shown on a network box to annotate the box's contents. Network box labels are graph organization metadata; longer explanations should use graph notes or node comments.
_Avoid_: Graph note, node comment, asset documentation, hidden behavior

**Network Box Membership**:
The visual inclusion of graph layout items inside a network box for graph organization. Adding or removing items from a network box does not rewire nodes, affect evaluation, change layer bindings, change asset membership, or alter output participation.
_Avoid_: Subgraph membership, asset membership, evaluation scope

**Network Box Move**:
A graph layout edit that moves a network box and its member graph layout items together without changing graph semantics.
_Avoid_: Move into subgraph, asset relocation, evaluation change

**Network Box Deletion**:
A graph layout edit that removes the network box only and leaves its member graph layout items in the graph unless the user explicitly chooses to delete contents.
_Avoid_: Delete contents, delete nodes, destructive graph edit

**Network Box Contents Deletion**:
An explicit destructive project command that deletes a network box and its selected contents together after the user chooses that behavior.
_Avoid_: Default network box deletion, implicit cleanup, visual-only removal

**Collapsed Network Box**:
A network box shown in a compact visual state to reduce canvas clutter while preserving the underlying nodes, connections, flags, badges, and graph evaluation semantics. Collapsing a network box is not the same as creating a subgraph, asset, or evaluation boundary.
_Avoid_: Subgraph collapse, asset node, hidden evaluation boundary

**Connection Routing Dot**:
A visual organization point placed on one or more connections to route, gather, or tidy connection paths on the graph canvas. A connection routing dot is not a node or operator, does not change data kind, does not evaluate, and does not have node info beyond graph layout metadata.
_Avoid_: Junction node, merge node, conversion node, hidden operator

**Pinned Connection Routing Dot**:
A connection routing dot that persists as graph layout scaffolding even when no active connection currently uses it. Pinned routing dots remain layout-only and can later be reused by connections without becoming data-flow objects.
_Avoid_: Dormant node, placeholder operator, disconnected junction

**Graph Note**:
A Houdini-like plain-text sticky-note annotation placed on the graph canvas to explain intent, assumptions, or workflow context without affecting graph evaluation. Graph notes are graph layout items that can be moved, resized, colored, minimized if useful, and included in network boxes; they remain distinct from asset documentation unless explicitly used during asset authoring.
_Avoid_: Node comment, network box label, rich text document, hidden behavior

**Node Comment**:
A project-saved plain-text user-authored note attached to a specific node and available through node info, graph search, and optional on-canvas display. Node comments package with projects and procedural assets but do not affect graph evaluation and remain distinct from asset documentation unless explicitly used during asset authoring.
_Avoid_: Graph note, node label, hidden annotation, rich text document, evaluation hint

**Node Comment Visibility Toggle**:
A project-saved per-node setting that marks a node comment for on-canvas display when node comment display mode is set to show only manually enabled comments.
_Avoid_: User preference, tooltip state, graph note visibility

**Node Comment Display Mode**:
The user or viewport preference for node comments: show comments for all commented nodes, or show only comments enabled by each node's project-saved node comment visibility toggle.
_Avoid_: Tooltip-only comment, hidden metadata, graph note mode

**Graph Search**:
A graph metadata navigation affordance that searches node names, operator names, parameter names or values, node comments, graph notes, and network box labels. Graph search results navigate to graph items or nodes; they do not filter datasets, search record attribute values, or create graph selections.
_Avoid_: Full-text search engine, dataset search, attribute search, operator browser, table filter

**Layer Visibility**:
An undoable project-saved presentation state that determines whether a layer is included in a composed viewer output without changing graph data meaning.
_Avoid_: Filter, display flag, render flag, node visibility

**Layer Order**:
An undoable project-saved presentation order of visible layers in a composed viewer output, without changing graph data meaning.
_Avoid_: Draw order, z-order, node order, composition operator

**Layer Promotion**:
An undoable project command that makes a compatible node output visible in the layer stack.
_Avoid_: Add layer, publish, expose

**Layer Removal**:
An undoable project command that removes a layer from the layer stack by unpromoting its node output without deleting the underlying graph nodes by default.
_Avoid_: Delete node, delete dataset, destructive layer delete

**Unavailable Layer**:
A layer whose promoted node output no longer exists or cannot currently evaluate because an upstream graph dependency was deleted, disabled, or failed.
_Avoid_: Hidden layer, silent removal, stale display copy

**Generated Node Cleanup**:
An explicit cleanup action that deletes unused generated nodes after the user confirms they are no longer needed.
_Avoid_: Automatic layer delete cleanup, hidden garbage collection, destructive unpromotion

**Destructive Dependency Warning**:
A warning shown before a graph edit deletes, reconnects around, or invalidates nodes with promoted layers, output-targeted dependents, materialized outputs, or other visible downstream consequences.
_Avoid_: Silent graph mutation, post-hoc error only, hidden dependent cleanup

**Layer Stack**:
The ordered collection of layers shown to users who prefer direct spatial exploration over editing the node graph. Durable filtering, styling, selections, edits, and transformations made through the layer stack are graph-backed project commands rather than layer-only behavior; visibility and order remain undoable presentation state.
_Avoid_: Table of contents, legend, scene hierarchy

**Manual Edit**:
A direct user change to geometry, attributes, or parameters that seeds or corrects a procedural workflow.
_Avoid_: Drawing, modeling, sculpting

**Manual Edit Node**:
A graph node that records durable user-authored edits to geometry, attributes, or parameters.
_Avoid_: Edit layer, drawing operation, modeling command

**Source Edit**:
A controlled change to source-like project data created inside the workspace rather than an external dataset.
_Avoid_: File edit, destructive edit, raw data change

**Node Graph**:
A named directed procedural model inside a project that describes how spatial datasets are produced from source data and transformations.
_Avoid_: Workflow, pipeline, flowchart

**Exploratory Node**:
A node placed for experimentation, comparison, or temporary work without needing to be part of a tidy connected flow.
_Avoid_: Random node, scratch node, orphan node

**Null Operator**:
A Houdini-like typed pass-through operator that returns its compatible input unchanged and is commonly used as a named, stable graph anchor for readable downstream references.
_Avoid_: Empty node, hidden alias, output target

**Null Anchor Naming Convention**:
A Houdini-like user-facing convention where `OUT_*` null operators mark stable consumable outputs and `IN_*` null operators mark readable internal inputs. Reference pickers, graph search, and asset authoring flows may prioritize these anchors, but data kinds and asset interfaces determine compatibility.
_Avoid_: Magic prefix, mandatory syntax, hidden type rule

**Reference Input Operator**:
A visible typed operator that imports or references one or more named node outputs elsewhere in the graph model as live one-way dependencies, commonly from null operators, while preserving stable-ID-backed references, connection compatibility, acyclic data flow, coordinate semantics, and procedural asset boundaries. It does not copy source data, apply hidden coordinate transforms, or allow downstream edits to mutate the referenced source.
_Avoid_: Hidden global variable, magic cross-graph wire, untyped path reference

**Reference Coordinate Semantics**:
The rule that reference input operators import graph outputs in their existing graph or substrate coordinate space. Reprojection, substrate conversion, and coordinate transformation require explicit visible projection or conversion operators.
_Avoid_: Hidden Object Merge transform, viewer transform, implicit projection

**Reference Coordinate Compatibility**:
The rule that enabled targets in a reference target set must share compatible graph or substrate coordinate semantics. Targets from incompatible substrate coordinate contracts remain diagnostic or disabled until an explicit projection or conversion operator makes them compatible, optionally through an assisted projection workflow.
_Avoid_: Best-effort overlay, automatic projection, hidden transform

**Reference Target Set**:
The visible collection of compatible node outputs imported by a multi-source reference input operator. All enabled targets in the set must satisfy the same typed output contract and reference coordinate compatibility, preserve source provenance for each imported target, expose target enablement state, and more complex merge, union, comparison, or conflict behavior belongs in explicit downstream operators.
_Avoid_: Hidden merge, untyped source list, implicit union

**Reference Target Enablement**:
A Houdini-like per-target state inside a reference target set that includes or excludes a referenced source from the reference input operator's evaluated output without deleting the target reference or its provenance.
_Avoid_: Deleted target, hidden filter, layer visibility

**Reference Target Identity**:
The stable node and node-output identity used by a reference input operator to preserve a binding across allowed node renames, moves, and readable path changes.
_Avoid_: Name match, path string only, same-named replacement

**Missing Reference**:
A reference input operator state where its referenced node, node output, or allowed boundary target no longer resolves. The node remains visible with diagnostics and its output is stale or unavailable until the user retargets or repairs it.
_Avoid_: Automatic rebinding, silent replacement, deleted dependency

**Live Reference Dependency**:
A one-way dependency created by a reference input operator where upstream changes make the referencing output stale and eligible for normal graph evaluation, while downstream edits remain local to the downstream graph path.
_Avoid_: Two-way binding, copied dataset, source mutation

**Reference Consumer**:
A graph element, usually a reference input operator, that depends on a referenced node output and should be reported when inspecting or deleting that referenced output.
_Avoid_: Hidden user, implicit dependency, name match

**Reference Navigation**:
Houdini-like affordances for finding and moving between reference input operators, referenced outputs, and reference consumers through readable paths, badges, node info, search, and jump actions instead of permanent cross-boundary wires by default.
_Avoid_: Hidden dependency, always-on long wire, global reference list

**Provenance Attribute Projection**:
An explicit projection of source provenance into inspectable, sortable, filterable, or exportable attributes such as source path, source node, source output, source dataset, or source layer.
_Avoid_: Mandatory source columns, hidden materialization, provenance-only copy

**Subgraph**:
A subnet-like node graph contained inside another graph and navigated as part of the graph editing workflow. A subgraph becomes a procedural asset only through explicit asset authoring.
_Avoid_: Folder, group, hidden graph

**Graph Container**:
A node that represents a subgraph from the outside while allowing users to enter and edit its internal graph.
_Avoid_: Collapsed group, folder node, black box

**Collapse to Subgraph**:
A Houdini-like graph operation that wraps selected nodes in a graph container, moves them into an internal subgraph, and rewires compatible external inputs and outputs through a typed boundary.
_Avoid_: Network box, visual collapse, asset creation

**Typed Boundary**:
An explicit set of typed inputs and outputs used when a graph container, procedural asset, plugin operator, or layer promotion exposes data beyond its local editing context.
_Avoid_: Strict graph, required interface, formal schema

**Graph Path**:
The Houdini-like readable navigational location of the current graph or subgraph within a project. Graph paths use user-facing names for navigation while durable references are backed by stable internal IDs.
_Avoid_: Stable identifier, backend path, file path

**Node Path**:
A Houdini-like readable path to a node within a graph or subgraph, using graph and node names for navigation, search, and user-facing references. Node paths are updated or resolved through stable IDs when names change.
_Avoid_: Node ID, durable reference, serialized foreign key

**Serialized Graph Model**:
A versioned persisted representation of a node graph that is independent of the frontend editor and operator runtimes.
_Avoid_: Frontend state, backend graph, save file

**Graph Evaluation**:
The process of computing requested node outputs from their dependencies.
_Avoid_: Run, cook, execute

**Acyclic Data Flow**:
A graph evaluation constraint where data-flow connections cannot form cycles in v1. Normal graph editing blocks cyclic connection attempts while visual organization remains free-form.
_Avoid_: Feedback loop, implicit iteration, recursive graph

**Invalid Connection**:
A persisted connection that cannot participate in graph evaluation because it violates data kind compatibility, acyclic data flow, or a missing endpoint. Invalid connections may be preserved when loading, migrating, or externally generating graph data, but they remain visibly diagnostic until repaired.
_Avoid_: Broken wire, deleted edge, silent repair

**Demand-Driven Evaluation**:
Graph evaluation that computes only the node outputs needed for visible layers, node info, exports, or downstream dependencies.
_Avoid_: Eager evaluation, full recompute, auto-run

**Evaluation Request**:
A user or system request that makes a node output eligible for graph evaluation, such as inspecting node info, promoting a layer, exporting, or explicitly running work.
_Avoid_: Run request, cook request, trigger

**Evaluation Fingerprint**:
The identity of a specific graph evaluation request, including the node output, relevant inputs, parameters, operator versions, and source artifact identities needed to decide whether a cached or completed result matches the current graph request.
_Avoid_: Hash only, cache key, current node state, hidden version

**Evaluation Mode**:
The project or graph policy that determines when requested graph evaluation starts. The v1 modes are automatic evaluation, on-interaction-complete evaluation, and manual evaluation, with on-interaction-complete evaluation as the default.
_Avoid_: Run mode, cook mode, execution setting

**Stale Output**:
A node output whose inputs, parameters, or operator version have changed since it was last computed.
_Avoid_: Dirty result, invalid cache, outdated output

**Last Successful Output**:
The most recent compatible evaluated result for a node output, which may remain visible while the current output is stale, waiting, or failed, as long as stale or error status remains explicit.
_Avoid_: Fresh output, current result, hidden display copy

**Manual Evaluation**:
Graph evaluation that starts only when the user explicitly triggers it.
_Avoid_: Manual cook, run button, paused graph

**On-Interaction-Complete Evaluation**:
Graph evaluation that waits until a high-frequency user interaction is complete, such as mouse-up after dragging or slider release after scrubbing, before evaluating requested stale outputs.
_Avoid_: On mouse up, delayed auto-run, debounce mode

**Automatic Evaluation**:
Graph evaluation that starts automatically when requested outputs become stale and are inexpensive enough to recompute.
_Avoid_: Live mode, auto-cook, reactive execution

**Cached Output**:
An evaluation result stored for reuse when its evaluation fingerprint matches a requested output, without becoming durable project graph data.
_Avoid_: Cache, temp result, saved output, locked output, embedded output snapshot

**Unloaded Node**:
A Houdini-like node cache policy where a node's cached output may be released when not needed, so memory is preferred over faster recook. Unloading affects runtime retention only; it does not change node values, layer bindings, output participation, project durability, or output-target contracts.
_Avoid_: Disabled node, bypassed node, deleted node, locked output, embedded output snapshot

**Runtime Evaluation State**:
Transient or cached execution state produced while evaluating graph outputs, such as running work, cached outputs, completed work item state, canceled work item state, superseded work item state, or failed work item state. Runtime evaluation state is not part of ordinary undo and redo history.
_Avoid_: Project command, graph edit, durable project state

**Materialized Output**:
An explicitly written or cached copy of graph output data created for export, packaging, durable artifact creation, frozen reference copies, performance, or interoperability, with visible location and expected size.
_Avoid_: Default filter result, hidden dataset copy, accidental duplicate file

**Work Item**:
A visible unit of long-running graph work that can be queued, run, monitored, retried, or parallelized.
_Avoid_: PDG item, job, task

**Work Item Retry**:
An evaluation action that requests work for the current graph request after a work item has failed, been canceled, or become stale. Retry targets the current evaluation fingerprint by default, not an obsolete historical fingerprint.
_Avoid_: Historical rerun, replay, cache reuse

**Historical Work Rerun**:
An explicit diagnostic action that reruns a previous work item's historical evaluation fingerprint. It must not update current graph outputs unless that historical fingerprint still matches the current graph request.
_Avoid_: Retry, restore, rollback, current evaluation

**Work Item State**:
The current status of a work item, such as waiting, running, cached, canceled, superseded, failed, or complete.
_Avoid_: Job status, task state, progress dot

**Canceled Work Item**:
A work item stopped before completion by a user or runtime decision. Its requested output remains stale; a compatible last successful output may remain visible with explicit canceled or stale status.
_Avoid_: Failed work item, completed work item, disabled node

**Superseded Work Item**:
A work item whose evaluation fingerprint no longer matches the current project state because an upstream input, parameter, operator version, source artifact identity, or requested output changed before completion. Its result must not update the current graph output even if the obsolete work later completes, though it may remain as diagnostic history or cache data for the exact old fingerprint.
_Avoid_: Failed work item, canceled work item, completed work item, hidden output update

**Execution Panel**:
A panel that shows work items for requested graph evaluation, including queued, running, cached, canceled, superseded, failed, and completed work.
_Avoid_: Task graph, PDG view, job monitor

**Node**:
A single operation in a node graph that receives inputs, applies a defined transformation or analysis, and produces outputs.
_Avoid_: Block, step, operator

**Node ID**:
A stable internal identifier for a node used by durable graph references, connections, layer bindings, asset internals, output operators, diagnostics, and serialized graph data. Node IDs are not the user-facing way people organize or read the graph.
_Avoid_: Node name, display label, graph path

**Node Name**:
A user-editable Houdini-like name shown on a node for readability, graph search, and navigation. Node names are unique within their parent node graph or subgraph, and renaming a node does not break durable graph references because those references use node IDs.
_Avoid_: Stable identifier, operator type, asset id

**Node Rename**:
An undoable project command that changes a node name while preserving the node ID and all durable references to the node. Duplicate names in the same parent graph or subgraph are rejected or resolved through a predictable suffix.
_Avoid_: Recreate node, break references, change operator

**Node Name Collision**:
A rename or creation attempt that would give two nodes the same name within one parent node graph or subgraph. Node name collisions are handled by rejecting the name or applying a predictable unique suffix without changing node IDs.
_Avoid_: Duplicate node name, broken graph path, hidden alias

**Node Copy**:
A graph-editing action that copies selected editable graph material for later paste or duplication without reusing node IDs.
_Avoid_: Linked duplicate, asset instance, hidden reference

**Copied Graph Material**:
The editable project graph data preserved by copy, paste, or duplicate, such as selected nodes, parameters, node comments, node colors, graph layout positions, selected network boxes, graph notes, routing dots, and connections internal to the copied selection.
_Avoid_: Runtime state, cache state, output participation, external reconnect

**Non-Copied Graph State**:
Graph editor, runtime, or system state intentionally excluded from ordinary paste or duplicate operations, such as cached outputs, running work, current node, selected node set, node display flags, node template flags, layer promotion, and output participation.
_Avoid_: Copied graph material, duplicated cache, implicit publication

**Presentation-Aware Duplication**:
An explicit higher-level project command that duplicates selected graph material together with chosen presentation or publication state, such as layer promotions, style bindings, comparison branch setup, or output participation. It is separate from ordinary graph copy, paste, or duplicate.
_Avoid_: Ordinary paste, implicit layer copy, hidden output duplication

**Pasted Node**:
A new node created by paste or duplicate from copied node data. A pasted node receives a new node ID and a graph-local unique node name while preserving copied editable settings and layout metadata where applicable.
_Avoid_: Alias, reference copy, same node id

**Pasted Subgraph**:
A pasted set of copied graph material that preserves connections internal to the copied selection by default. Connections to nodes outside the copied selection remain disconnected with visible diagnostics or reconnect only through an explicit paste option.
_Avoid_: Hidden external reconnect, copied asset, implicit branch

**User Graph Styling**:
User-owned visual styling used to organize the graph canvas, such as node colors, network box colors, network box labels, graph notes, and connection routing dots. User graph styling does not communicate system status or graph evaluation state.
_Avoid_: Status styling, diagnostic styling, evaluation state

**Node Color**:
A user-defined visual color applied to a node for graph organization. Node color must not carry system status meaning or obscure badges, flags, selection, current-node state, or diagnostics.
_Avoid_: Status color, state color, badge

**Node Badge**:
A system-owned visual indicator for node state, such as stale, running, cached, warning, error, current, selected, layer-promoted, output-targeted, managed, adopted, unbound, or commented. Node badges live in a system status lane that remains legible over user graph styling.
_Avoid_: Label, custom color, tag

**Node Flag**:
A graph-visible marker that communicates or changes a node's role in inspection, evaluation, pass-through behavior, layer promotion, or output targeting.
_Avoid_: Houdini display flag, render flag, template flag, layer visibility

**Node Display Flag**:
A graph-visible inspection flag that asks a graph display context to preview a specific compatible node output, usually the primary output. It may create an evaluation request, but it does not create a durable layer, change layer order, or participate in output operators unless the output is separately promoted or connected.
_Avoid_: Layer promotion, layer visibility, output target, render flag

**Node Template Flag**:
A graph-visible read-only inspection flag that asks a graph display context to show a compatible node output as a reference overlay alongside the node display flag output. Multiple node template flags may be enabled for comparison or upstream reference; they do not create durable layers, change layer order, provide editable or selectable graph data, or participate in output operators unless separately promoted or connected.
_Avoid_: Reference layer, ghost layer, output target, layer visibility, selectable template

**Graph Display Context**:
The shared graph-editing inspection context for a node graph or subgraph. It owns one primary node display flag and any node template flags for that graph context, answering which node output is being inspected separate from layer-stack composition and output-target publication.
_Avoid_: Viewport layer state, render target, camera view, export state

**Bypassed Node**:
A Houdini-like node state whose evaluation is intentionally skipped so compatible input data passes through and downstream work can continue without the node's operation. Bypassing is an undoable project command and should preserve layer bindings when the pass-through output remains compatible.
_Avoid_: Hidden node, disabled node, deleted node

**Disabled Node**:
A product-specific inactive node state that produces no usable output and does not pass compatible input through. Downstream graph elements should show unavailable, stale, or error state, and managed layer bindings remain inactive rather than becoming adopted.
_Avoid_: Bypassed node, hidden node, deleted node

**Data Kind**:
The declared category of data carried by a node input, node output, attribute, parameter, or connection.
_Avoid_: Type, schema, format

**Operator**:
A reusable node type that defines a specific transformation, source, filter, styling operation, or analysis operation.
_Avoid_: Node template, tool, command

**Operator Type**:
The stable operator identity used to describe what operation a node performs, distinct from the user-editable node name. Operator type remains visible in node info and, when useful, as a secondary node label.
_Avoid_: Node name, display label, node id

**Plugin**:
An installable extension that can provide code-backed operators, procedural assets, or both.
_Avoid_: Add-on, extension, package

**Plugin Capability Tier**:
The execution boundary that defines what a plugin-provided capability is allowed to do and where it is allowed to run.
_Avoid_: Plugin type, permission level, runtime

**Plugin Operator**:
A code-backed operator supplied by a plugin.
_Avoid_: Custom node, extension node, scripted operator

**Operator Runtime**:
The execution environment responsible for evaluating a code-backed operator under the typed graph contract.
_Avoid_: Execution engine, worker, processor

**Frontend Plugin Operator**:
A plugin operator intended for lightweight user-interface or browser-runtime work.
_Avoid_: UI plugin, browser node, web operator

**Native Plugin Operator**:
A trusted plugin operator intended for local high-performance work outside the browser runtime.
_Avoid_: Local plugin, backend node, native extension

**Remote Plugin Operator**:
A plugin operator whose work is performed by an external service.
_Avoid_: Cloud plugin, service node, remote extension

**Python Interface**:
A trusted scripting and automation surface for using Python libraries with projects, node graphs, operators, and assets.
_Avoid_: Console, macro system, script runner

**Python Console**:
An interactive Python surface for inspecting projects, automating workspace tasks, and experimenting with available APIs.
_Avoid_: REPL, terminal, command line

**Python Operator**:
A code-backed operator implemented with Python libraries and evaluated through the typed graph contract.
_Avoid_: Python node, script node, custom script

**Reproducible Python Operator**:
A Python operator whose behavior is defined by declared inputs, outputs, parameters, dependency requirements, runtime requirements, and captured provenance.
_Avoid_: Arbitrary script, hidden side effect, local notebook cell

**Side-Effecting Operator**:
An operator that performs external effects such as writing files, making network calls, mutating source datasets, or changing project state.
_Avoid_: Reproducible operator, pure operator, hidden action

**Python Expression**:
A Python snippet used to compute a parameter value within the typed parameter system.
_Avoid_: Callback, script, formula

**Python Environment**:
The configured Python runtime and dependency set used by Python operators and scripting workflows.
_Avoid_: Virtualenv, interpreter, package folder

**Project Python Environment**:
A Python environment resolved for a project from the requirements of its Python operators, procedural assets, plugins, and automation.
_Avoid_: Global Python, asset environment, system Python

**Python-Capable Runtime**:
A trusted runtime that can resolve the project Python environment and evaluate Python operators, expressions, or Python-backed assets. Browser capability may request work from a trusted Python-capable runtime and inspect its outputs, but the browser or viewer target does not become the Python operator runtime.
_Avoid_: Browser Python, viewer Python, hidden local script

**Python Dependency Requirement**:
A dependency requirement declared by a Python operator, procedural asset, plugin, or project.
_Avoid_: Package requirement, dependency pin, install request

**Python Environment Manifest**:
A project-owned declaration of the Python version and dependencies needed by Python operators, expressions, and automation.
_Avoid_: Requirements file, pyproject, dependency list

**Python Lockfile**:
A resolved dependency record used to recreate a Python environment consistently.
_Avoid_: Frozen requirements, package snapshot, pinned deps

**Python Dependency Status**:
The visible state of a Python environment, including whether dependencies are missing, resolving, installed, stale, or failed.
_Avoid_: Install status, package status, environment health

**Python Toolchain Manager**:
The app-managed tool used to create Python environments, install dependencies, and run Python commands.
_Avoid_: Package manager, uv, pip

**Operator Family**:
A category of operators grouped by the kind of data or workflow they operate on.
_Avoid_: SOP, CHOP, category, namespace

**Source Operator**:
An operator that creates a spatial dataset from an external source or built-in generator.
_Avoid_: Import operator, loader, reader

**Geometry Operator**:
An operator that transforms, creates, or edits geometry.
_Avoid_: Shape operator, modeling operator

**Filter Operator**:
An operator that selects records from a spatial dataset according to attributes, geometry, computed conditions, or explicit selection or mask subset data.
_Avoid_: Query operator, group operator

**Style Operator**:
An operator that applies visual mappings to records or geometry for viewport display, optionally using selection or mask subset data to style only matching records.
_Avoid_: Symbology operator, appearance operator

**Selection/Mask Operator**:
An operator that creates, edits, combines, inverts, inspects, or converts durable selections and masks as subset data. Houdini-style group terminology may be used as a search alias, but selection and mask remain the canonical output kinds and do not implicitly filter records.
_Avoid_: Group operator, hidden selection state, layer-only selection

**Analysis Operator**:
An operator that computes derived measurements, relationships, classifications, or summaries from spatial datasets.
_Avoid_: Metrics operator, statistics operator

**ML Operator Family**:
An operator family for Rerun-native machine-learning inspection and inference workflows, including model import or reference, inference, output inspection, model-output adaptation, and external training recording inspection.
_Avoid_: Generic Python category, black-box ML script, notebook workflow, training platform

**ML Operator**:
An operator in the ML operator family with visible data, model, dependency, runtime, output, and provenance contracts. An ML operator may be Python-backed, but its ML role remains explicit and bounded by Rerun-native ML scope.
_Avoid_: Python script node, hidden model call, generic analysis operator

**Rerun-Native ML Scope**:
The initial ML product boundary that includes only ML and computer-vision workflows Rerun can already log, visualize, inspect, replay, or naturally receive through its SDK and examples, such as segmentation, detections, classifications, tensors, embeddings, depth, keypoints, tracks, images, video, and training-progress recordings from external workflows. SAM-style segmentation is a reference workflow for this boundary rather than a mandate to expand ML operations.
_Avoid_: In-graph training platform, hidden model registry, custom ML viewer stack

**Rerun-Native ML Gate**:
The scope test for core ML features: include only ML outputs or telemetry that Rerun can represent natively or idiomatically without a private visualization schema, training orchestrator, or model registry.
_Avoid_: Custom ML platform gate, private viewer extension, future training system

**External Training Recording**:
A Rerun-native recording or log produced by an external training workflow and inspected or replayed inside the workspace without turning the graph into the training controller.
_Avoid_: Training operator, fine-tuning node, checkpoint manager, training scheduler

**Model Artifact**:
A typed graph value representing a trained, imported, or selected ML model with version, provenance, dependency metadata, runtime requirements, metrics when available, and a storage reference for large weights or checkpoints.
_Avoid_: Hidden checkpoint, model path, downloaded weight file

**Model Artifact Reference**:
The graph-owned reference to a model artifact stored outside the project file, such as in a cache, artifact store, or managed project package.
_Avoid_: Local path string, implicit cache entry, global model setting

**Inference Result**:
A typed output produced by applying a model artifact to spatial, image, video, sensor, or attribute inputs, such as masks, classes, detections, embeddings, scores, or generated geometry.
_Avoid_: Model artifact, viewer overlay, temporary prediction

**ML Output Kind**:
A semantic data kind for a task-specific inference result, such as segmentation mask, detection, classification, embedding, depth map, pose, track, generated geometry, or confidence score table.
_Avoid_: Generic tensor, viewer overlay, untyped prediction

**Core ML Output Kind**:
A first-class ML output kind expected in the initial graph model: segmentation masks, detection boxes, classification labels and scores, embeddings or tensors, depth maps, points or keypoints, tracks, and generated geometry.
_Avoid_: Exotic model output, custom viewer overlay, untyped result blob

**Inference Adoption**:
An explicit graph operation that turns compatible inference results into durable spatial graph data such as records, masks, selections, curves, polygons, or point datasets while preserving model provenance and confidence.
_Avoid_: Silent prediction conversion, viewer overlay promotion, implicit spatial dataset

**ML-Derived Spatial Data**:
Spatial graph data created through inference adoption that remains filterable, styleable, editable, and reusable while retaining visible model artifact, model version, inference operator, work item, source input, confidence, and adoption provenance.
_Avoid_: Manual source data, anonymous prediction, provenance-free geometry

**Output Operator**:
An explicit graph node that exposes, streams, saves, exports, packages, or publishes node outputs through an output target contract.
_Avoid_: Export operator, sink, writer, hidden viewer side effect

**Generic Output Operator**:
An output operator that expresses semantic payload, output command, and optional preferred output target without exposing target-specific configuration in the graph model.
_Avoid_: Universal viewer node, hidden target adapter, lowest-common-denominator export

**Target-Specialized Output Operator**:
An output operator variant that exposes target-specific options through the target adapter while still communicating through the output target contract.
_Avoid_: Viewer-coupled graph runtime, private target schema, hardwired backend node

**Output Participation**:
The explicit inclusion of a node output in publishing, export, packaging, recording, or viewer-target composition through layer promotion, connection to an output operator, or another graph-owned output command. Display and template flags do not create output participation.
_Avoid_: Render flag, preview output, implicit publish

**Operator Browser**:
A searchable and browsable tree view of available operator families and operators. It can be opened directly for free node creation or in a connection-scoped mode that filters to operators compatible with the source output or target input.
_Avoid_: Node menu, tool palette, shelf

**Node Creation**:
An undoable project command that adds a visible node to a node graph through the operator browser, connection-scoped operator creation, shelf tool, manifest import, asset action, or another graph-backed workflow.
_Avoid_: Hidden workflow step, viewer-only action, implicit operation

**Connection-Scoped Operator Creation**:
A node creation flow started from an existing node input, node output, or connection so the operator browser can prioritize compatible downstream, upstream, insertion, conversion, or merge operators. The created node and any connections remain visible graph data.
_Avoid_: Magic wire, hidden adapter, untyped quick action

**Connection Insertion**:
An undoable graph edit that places a compatible visible node on an existing connection by rewiring source output to inserted node input and inserted node output to target input. It must preserve connection compatibility and acyclic data flow; required conversions are explicit visible conversion operators.
_Avoid_: Silent rewiring, hidden adapter, implicit data-kind change

**Node Deletion**:
An undoable project command that removes one or more nodes from a node graph. Deleting a node may reconnect around it only when the replacement connections are compatible, acyclic, unambiguous, and do not silently damage promoted or output-targeted dependents.
_Avoid_: Hidden cleanup, destructive source edit, silent reconnect

**Reconnect-Around Deletion**:
A Houdini-like node deletion behavior where a simple compatible chain is repaired by connecting downstream inputs to the deleted node's compatible upstream output. If repair is incompatible, cyclic, ambiguous, or destructive to visible dependents, the graph remains visibly unavailable or invalid until the user repairs it.
_Avoid_: Guessing reconnect, hidden conversion, automatic merge

**Shelf**:
A thin action surface for common workflows that creates graph-backed project commands.
_Avoid_: Toolbar, tool palette, command bar

**Shelf Tool**:
A common workspace action exposed on the shelf, such as importing data, creating a filter, styling a layer, promoting an output, creating an asset, packaging a project, or running evaluation. Shelf tools create visible graph-backed nodes, starter graphs, or project commands rather than hidden workflow state.
_Avoid_: Button, shortcut, command

**Custom Shelf Tool**:
A user-defined shelf tool that runs a saved workflow, procedural asset action, or script.
_Avoid_: Macro, custom button, shortcut

**Parameter Panel**:
A panel where users inspect and edit the current node's parameters unless the panel is pinned.
_Avoid_: Properties panel, node settings, inspector

**Pinned Parameter Panel**:
A parameter panel locked to a specific node or asset interface so it does not follow the current node when graph selection changes.
_Avoid_: Detached inspector, duplicate settings panel, hidden selection

**Parameter Edit**:
An undoable project command that changes a node or asset parameter, marks affected downstream outputs stale, and may create an evaluation request according to the evaluation mode. A parameter edit does not silently materialize output data or mutate source artifacts.
_Avoid_: UI tweak, hidden data mutation, source edit

**Promoted Parameter**:
An internal node parameter exposed through a procedural asset's asset interface.
_Avoid_: Exposed setting, public parameter, linked control

**Parameter Group**:
A named grouping of related parameters shown together in a parameter panel or asset interface.
_Avoid_: Tab, section, folder

**Parameter Option**:
A predefined choice available to a parameter that accepts a fixed set of values.
_Avoid_: Menu item, enum value, dropdown choice

**Node Input**:
A named, typed entry point where a node receives data from another node output.
_Avoid_: Input port, socket, plug

**Node Output**:
A named result produced by a node and made available for downstream connections, layers, and inspection.
_Avoid_: Result, port, socket

**Primary Output**:
A node output declared as the default target for quick wiring, default node info, and layer promotion shortcuts. A primary output is a UX convenience; downstream graph meaning still refers to a specific named, typed output.
_Avoid_: Implicit result, whole-node output, semantic shortcut

**Multi-Output Node**:
A node that exposes more than one named, typed node output. Each output is independently inspectable, connectable, promotable where compatible, cacheable, and available to evaluation requests, even when one output is marked as primary.
_Avoid_: Bundled result, vague node output, hidden secondary data

**Compatible Output**:
A node output whose data kind can be connected to a target input or promoted into a layer.
_Avoid_: Valid output, matching port, accepted result

**Connection Compatibility**:
The data-kind rule that determines whether a node output may connect to a node input directly. Incompatible kinds require an explicit conversion operator when a meaningful conversion exists.
_Avoid_: Duck typing, implicit cast, best-effort wiring

**Conversion Operator**:
An explicit operator that converts data from one data kind to another when such a conversion is meaningful.
_Avoid_: Coercion, adapter, cast, hidden reference transform

**Merge Operator**:
An explicit operator that combines compatible inputs into a named typed output when users need merged data for editing, selection, filtering, styling, comparison, promotion, or output targeting.
_Avoid_: Template merge, visual overlay merge, implicit combined selection

**Assisted Conversion**:
An editor action that suggests or inserts a visible conversion operator between two nodes when the user attempts a meaningful incompatible connection. The inserted conversion remains an ordinary node that can be inspected, edited, moved, bypassed, disabled, or deleted.
_Avoid_: Hidden adapter, automatic coercion, magic wire

**Assisted Projection**:
A user-confirmed editor action that creates a visible substrate projection or coordinate conversion operator when a coordinate-incompatible reference target can be made compatible through known substrate coordinate contracts. The created operator remains ordinary graph data that can be inspected, edited, moved, bypassed, disabled, or deleted.
_Avoid_: Automatic projection, hidden Object Merge transform, best-effort overlay

**Node Info**:
A detailed inspection view available for every node or node output, including node name, operator type, data shape, record counts, filtered counts, attributes, geometry kind, bounds, record identity mode, node comments, diagnostics, source provenance, evaluation/cache status, performance summaries, custom operator information, reference target or consumer information, and output-target mapping status.
_Avoid_: Tooltip, debug panel, summary

**Node Diagnostic**:
A node-owned message, warning, or error shown through node badges and node info. Messages are informational, warnings allow graph evaluation to continue, and errors mark unrecoverable problems that block affected downstream evaluation.
_Avoid_: Console log, hidden exception, toast-only error

**Current Node**:
The single node currently focused for parameter editing, inspection, graph navigation, and quick graph actions. A graph may have multiple selected nodes, but at most one current node, and current node state is independent from display and template flags.
_Avoid_: Active node, selected set, display node

**Selected Node**:
A node included in the graph editor's current selection set for actions such as moving, copying, deleting, grouping, coloring, commenting, or creating a network box. Node selection is graph-editor UI state, separate from spatial selections in data.
_Avoid_: Spatial selection, current node, display node

**Connection**:
A directed link that passes data from one node output to another node input while preserving acyclic data flow in v1.
_Avoid_: Edge, wire, link

**Viewport**:
The interactive visual surface where spatial datasets and node outputs are inspected.
_Avoid_: Map, canvas, scene

**Graph-Editing Viewport**:
A viewport mode focused on inspecting the shared graph display context: the node display flag output plus any read-only node template flag overlays. Multiple graph-editing viewports for the same node graph or subgraph follow the same graph display context.
_Avoid_: Composed output view, layer stack preview, render view

**Composed-Output Viewport**:
A viewport mode focused on inspecting graph-owned output participation, such as promoted layers, composed viewer output, or an explicit output operator result.
_Avoid_: Display flag view, template overlay view, graph scratch view

**Primary Viewport**:
The main viewport emphasized by the default workspace layout.
_Avoid_: Main map, main scene, active canvas

**Camera Navigation**:
The viewport controls used to inspect 2D and 3D spatial datasets through pan, zoom, orbit, and related view changes.
_Avoid_: Map navigation, scene controls, view tool

**Panel**:
A movable interface region that contains a focused tool, inspector, browser, viewport, or editor.
_Avoid_: Window, pane, dock

**Workspace Layout**:
The arrangement of panels, including their positions, sizes, grouping, and active views.
_Avoid_: Dock layout, window layout, screen setup

**Workbench Layout Preset**:
A named Houdini-like workspace layout recipe that a user can load for a common mode of work, such as graph editing, inspection, data review, output review, debugging, or asset authoring. Loading a workbench changes panel arrangement and active views only; it does not change graph data, evaluation state, layer bindings, or output participation.
_Avoid_: Hidden workflow mode, graph state preset, custom dock system

**User Preference**:
A personal setting that can override project defaults without changing the shared project model.
_Avoid_: Project setting, local state, option

**Procedural Asset**:
A Houdini-like reusable, parameterized node graph that is explicitly authored from a subgraph or graph container with a typed public interface and can be packaged and used as a higher-level node.
_Avoid_: Houdini Digital Asset, HDA, macro

**Graph Asset**:
A procedural asset that packages graph structure and declarative interface metadata without custom plugin code.
_Avoid_: No-code asset, saved graph, macro

**Python-Backed Asset**:
A procedural asset that includes Python operators or Python expressions and declares the Python dependency requirements and Python-capable runtime needed to evaluate it.
_Avoid_: Hidden Python asset, local script asset, works-on-my-machine asset

**Project Asset**:
A procedural asset stored inside a project and intended for that project's workflows unless explicitly published elsewhere. Newly authored assets are project assets by default.
_Avoid_: Local asset, embedded asset, project HDA

**Shared Asset**:
A procedural asset stored in a shared library and intended for reuse across projects.
_Avoid_: Global asset, installed asset, library HDA

**Asset Library**:
A shared collection of procedural assets available across projects.
_Avoid_: Plugin registry, asset folder, package manager

**Asset Storage Location**:
The project or shared asset library location where a procedural asset definition is saved. Create procedural asset defaults to project asset storage while allowing a shared asset library to be selected when the user intends reuse across projects.
_Avoid_: Hidden install path, global default only, export destination

**Asset Publishing**:
The explicit action of turning a project asset into a shared asset, including its name, version, interface, documentation, dependency requirements, and compatibility checks.
_Avoid_: Export, install, promote

**Asset Definition**:
The reusable saved definition of a procedural asset, including its internal name, version, interface, internal graph, documentation, dependency requirements, and external artifact references.
_Avoid_: Asset instance, copied graph, node template

**Asset Dependency Requirement**:
A dependency declared by an asset definition, such as required Python packages, required operator families, required model artifacts, required source artifact kinds, or required external artifact references.
_Avoid_: Hidden dependency, embedded artifact, global install assumption

**Asset Internal Name**:
The stable identifier of an asset definition, optionally including namespace and version in a Houdini-like form. Asset internal names are chosen during asset creation and should not be silently changed after instances depend on them.
_Avoid_: Display label, node name, mutable title

**Asset Instance**:
A node that references a procedural asset definition and exposes the asset interface while matching that definition by default.
_Avoid_: Copied asset graph, pasted subgraph, embedded tool

**Asset Version Pin**:
The exact asset definition and version used by an asset instance so project behavior remains reproducible until the user explicitly upgrades or changes the definition.
_Avoid_: Floating latest version, automatic asset update, implicit migration

**Asset Upgrade**:
An explicit project command that moves an asset instance from one asset version pin to a newer compatible asset definition or migration path.
_Avoid_: Auto-update, silent replacement, dependency refresh

**Matched Asset Instance**:
An asset instance whose internal graph and interface match its pinned asset definition. New asset instances are matched by default.
_Avoid_: Locked data copy, hidden definition, floating latest

**Allow Editing of Contents**:
A Houdini-like asset instance action that turns a matched asset instance into an unlocked asset instance with local editable graph material, breaking automatic matching to its pinned asset definition until the user saves back or matches the definition again.
_Avoid_: Auto-fork, silent copy, asset upgrade

**Unlocked Asset Instance**:
An asset instance that the user has explicitly turned into editable project graph material, breaking the default link to the shared asset definition.
_Avoid_: Auto-copied asset, silently forked asset, modified shared asset

**Save Asset Definition**:
A Houdini-like asset authoring action that writes compatible local asset edits back to the asset definition so linked matched instances of that definition receive the update.
_Avoid_: Export graph, save project, update unrelated versions

**Match Asset Definition**:
A Houdini-like asset instance action that discards local unlocked edits and relocks an asset instance to its pinned asset definition.
_Avoid_: Asset upgrade, merge local edits, automatic revert

**Asset Interface**:
The typed public surface of a procedural asset, including its inputs, outputs, parameters, metadata, and documentation.
_Avoid_: Public API, exposed settings, type properties

**Asset Boundary Output**:
A typed node output exposed through a procedural asset's asset interface for use by external graph nodes, layers, output operators, or reference input operators.
_Avoid_: Internal null, private asset node, hidden output

**Private Asset Internal**:
Internal graph material inside a matched procedural asset instance that is not exposed through the asset interface and is not a valid external reference target.
_Avoid_: Public node path, asset output, shared dependency

**Compatible Asset Interface Change**:
An edit to an asset definition's typed public interface that linked instances can accept without breaking existing inputs, outputs, promoted parameters, or saved parameter values.
_Avoid_: Silent migration, breaking edit, new version

**Breaking Asset Interface Change**:
An edit to an asset definition's typed public interface that removes, renames, retargets, or changes the data kind or meaning of existing inputs, outputs, promoted parameters, or saved parameter values. Breaking changes require a new asset version, explicit migration, or explicit asset upgrade.
_Avoid_: Compatible edit, silent instance break, hidden type change

**Asset Documentation**:
Documentation exposed through a procedural asset's public interface, distinct from internal node comments or graph notes saved inside the asset definition unless explicitly authored from them.
_Avoid_: Internal node comment, graph note, README

**Asset Authoring**:
The Houdini-like process of creating or editing a procedural asset and its asset interface from the workspace GUI, including promoted parameters and typed inputs or outputs.
_Avoid_: HDA creation, plugin development, scripting

**Create Procedural Asset**:
A Houdini-like asset operation that turns a graph container or subgraph into a reusable procedural asset with a label, stable internal name, version, asset storage location, promoted parameters, typed inputs and outputs, and documentation.
_Avoid_: Collapse to subgraph, export only, hidden macro
