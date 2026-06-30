# Houdini workbench layout presets

Date: 2026-06-30

## Requirement

Add Houdini-style workbench layout presets for the native Rerun product fork.

A workbench preset is a named blueprint layout recipe for a common Houdini graph workflow.
Loading a preset arranges existing Houdini graph views, containers, and viewport surfaces; it does not create graph data, change evaluation state, rewrite layer bindings, or hide semantics in viewer-only state.

## Product intent

The current blueprint-tab integration makes the Network, Parameters, Info, Display, Operators, Find, Layers, Data, Outputs, Project, and Houdini Graph surfaces movable inside the native Rerun viewport system.
Workbench presets build on that by giving users known starting layouts they can load, customize, and return to.

The model should feel like Houdini desktops or workbenches:

- a graph-focused workbench for network editing and node parameters
- an inspection workbench for Parameters, Info, Display, Find, and Layers
- a data workbench for table and attribute inspection
- an output workbench for Rerun output, layer visibility, and recording/export checks
- a debug workbench for evaluation status, diagnostics, and performance instrumentation
- an asset-authoring workbench once graph containers and procedural assets mature

## Layout contract

Workbench presets should use normal Rerun blueprint primitives:

- views
- tab containers
- split containers
- container display names
- view display names
- active tab selection where the underlying blueprint supports it

They should not reintroduce a custom nested dock system inside a single Houdini view.
After loading a preset, users should still be able to move, split, tab, close, and rearrange the surfaces through the same native Rerun tab and blueprint controls.

## Project defaults and user overrides

A project may recommend a default workbench layout.
Users may override that with personal workbench preferences without rewriting the shared project graph or project default.

The durable graph model remains the source of truth for nodes, parameters, geometry, layers, display/template flags, output operators, and asset boundaries.
Workbench layout state is presentation state layered above the graph model.

## Implementation follow-up

Track the implementation in #72.
