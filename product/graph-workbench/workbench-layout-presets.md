# Houdini workbench layout presets

Date: 2026-06-30

## Requirement

Add Houdini-style workbench layout presets for the native Rerun product fork.

A workbench preset is a named blueprint layout recipe for a common Houdini graph workflow.
Loading a preset arranges existing Houdini graph views, containers, and viewport surfaces; it does not create graph data, change evaluation state, rewrite layer bindings, or hide semantics in viewer-only state.

Rerun already has `.rbl` blueprint save/load machinery.
The product feature is the Houdini-facing layout manager on top of that substrate: users can load a workbench, duplicate it, edit the layout through native Rerun blueprint tabs, save the result under a new name, and return to it later.

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

## Load, duplicate, edit, and save

The workbench UI should expose four user-facing flows:

- Load: choose a bundled, project, or personal workbench layout and apply it to the current blueprint.
- Duplicate: copy an existing workbench layout into a personal editable layout.
- Edit: rearrange panels through normal Rerun blueprint tabs, splits, and containers.
- Save: persist the edited layout as a named personal or project workbench.

Personal saved workbenches should not rewrite project defaults.
Project workbenches should be explicit shared project data or checked-in product presets.
Saving a workbench should preserve named container/view identity so reloaded layouts do not fall back to placeholder `/` titles.

If possible, saved workbenches should reuse Rerun `.rbl` blueprint serialization.
Any Houdini-specific wrapper should store only metadata needed for naming, categorization, default selection, and compatibility.

## Implementation follow-up

Track the bundled preset implementation in #72.
Track the richer named personal/project workbench browser in #74.

Initial implementation slice: bundled workbench layout loading is available from the viewport
Workbench menu, using native Rerun blueprint views, splits, tab containers, and named containers.
Bundled presets currently cover Network + Inspector, Houdini Default, Graph Review, Data
Inspection, and Output / Debug workbenches.
The Workbench toolbar also exposes `.rbl`-backed open/save-as actions so users can load a saved
layout file or duplicate the current edited layout without leaving the graph workspace.

Follow-up work remains for a dedicated personal/project workbench browser with named layout
metadata, categories, default-selection controls, and a clearer distinction between "save current
blueprint for this app" and "save this workbench as a reusable named layout."

Issue #74 first browser slice: the Workbench menu now groups bundled presets with saved personal
and project workbench metadata. Saved entries are named JSON metadata wrappers that point at `.rbl`
blueprint payloads and track separate personal/project default flags. Bundled presets can be
duplicated into personal metadata and saved directly to the registered `.rbl` payload path.
