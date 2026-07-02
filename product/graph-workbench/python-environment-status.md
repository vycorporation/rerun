# Houdini Python environment status

Date: 2026-06-28

## Decision

Use app-managed, project-specific uv environments for Houdini Python operators.
Do not use global system Python as the default execution path.
Do not implement environment bootstrapping until the Python operator declaration and graph-node surfaces are approved.

The environment model exists to make Python operators reproducible, inspectable, and repairable.
It is project state, not viewer state.

## Project environment record

A Houdini project should record a Python environment descriptor alongside the graph sidecar or project model.
The descriptor should include a project environment id, Python version requirement, requirements source, lock status, lock digest, environment path, resolver tool, resolver version, last health check, and last failure summary.

The requirements source should start as project-local data.
It may be stored as a `pyproject.toml` fragment, a generated requirements list, or a product-specific Python environment section.
The model should preserve enough structure to explain which graph operators contributed each dependency.

The environment path should be inside an app-managed project area by default.
It should not point at `/usr/bin/python`, Homebrew Python, Conda base, pyenv global, or any other global interpreter unless the user explicitly opts into an advanced override.

## Lock status

The environment has an explicit lock status.
The minimum states are missing, unlocked, resolving, locked, ready, stale, failed, and disabled.

Missing means the project has Python operators but no recorded environment.
Unlocked means dependency requirements exist but no lock digest has been produced.
Resolving means uv is actively creating or updating the lock.
Locked means a lock digest exists but the runtime environment has not been verified.
Ready means the lock, interpreter, installed packages, and declared operator requirements are healthy.
Stale means operator requirements, Python version, platform, or lock inputs changed after the last ready state.
Failed means resolution, installation, interpreter validation, or import validation failed.
Disabled means the user intentionally turned off project Python execution.

## Dependency health

Dependency health should be computed from declared operator requirements and the resolved project environment.
The status should report missing packages, version conflicts, incompatible Python version, platform marker mismatch, failed imports, stale lock digest, and resolver errors.

Health is separate from trust.
A project can have a healthy environment but still require user trust before first execution.
A trusted project can have an unhealthy environment and remain blocked until repaired.

## UI/status surface

The graph panel should show a compact project Python status.
The node inspector should show operator-specific dependency status for Python operator nodes.

The project status should include missing, resolving, ready, stale, failed, and disabled states.
Missing should offer setup or select-project-environment actions.
Resolving should show progress, current resolver step, and a cancel action.
Ready should show Python version, package count, lock digest, environment path, and last health check time.
Stale should show what changed and offer resolve.
Failed should show a concise error summary, stderr tail, and repair actions.
Disabled should show that Python execution is intentionally unavailable.

The status surface should not run Python implicitly when a project opens.
It can inspect recorded metadata immediately.
It should require an explicit resolve, repair, trust, or run action before doing work that mutates the environment or executes project code.

## Resolving operator dependencies

Each Python operator declaration contributes dependency requirements to the project environment.
The resolver input is the union of enabled operator requirements plus project-level requirements.
Disabled or inert operator declarations should not force environment resolution unless the user asks for full-project validation.

Dependency conflicts should be reported against the contributing operators.
The UI should be able to say which node or operator declaration introduced a package requirement.

The resolved lock digest becomes part of Python operator cache keys.
If the lock digest changes, dependent Python operator nodes become stale.

## Execution handoff

Python execution should receive an explicit interpreter path from the project environment record.
It should not discover Python from `PATH` by default.
It should not fall back to global system Python if the project environment is missing or failed.

The process-boundary execution prototype in #40 should treat a missing or unhealthy environment as a node failure or blocked evaluation state.
It should report that status through the Python operator node surface in #41.

## Security and trust

Environment resolution can download and install code.
Project trust should be explicit before resolving dependencies or executing Python.

The v1 model can use coarse trusted desktop permissions.
The descriptor should still record capability-relevant settings such as network access during resolve, package index sources, local file roots, GPU availability, and subprocess permission.

The product should not promise sandboxing in v1.
It should instead clearly communicate trusted local execution and avoid automatic execution on project open.

## Non-goals

Do not implement uv bootstrapping in this issue.
Do not execute Python in this issue.
Do not support remote execution in this issue.
Do not support global system Python as the default.
Do not build a package manager UI beyond the status and repair surface needed for Houdini Python operators.

## Implementation follow-ups

1. #45 - Add a serializable project Python environment descriptor.
2. #46 - Add graph panel and node inspector status surfaces for environment health.
3. #43 - Prototype uv lock/resolve as an explicit trusted project action.
4. #44 - Connect ready/stale/failed environment state to Python operator evaluation blocking.
5. #282 - Complete: record explicit project Python trust in the environment
   descriptor and confirm trust before starting the existing uv resolve
   lifecycle from the workbench; untrusted configured environments block Python
   operator readiness until explicitly trusted.
