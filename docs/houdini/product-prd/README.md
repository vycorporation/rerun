# Houdini product PRD source

Date: 2026-06-30

This directory is the fork-local product source for the Houdini graph work.
It replaces the old local-only folder at `/Users/bit/Documents/houdini-clone` so that future PRD, issue, and implementation work has one canonical location inside `vycorporation/rerun`.

## Contents

- `source/CONTEXT.md`: migrated product vocabulary and domain model.
- `source/docs/adr/`: migrated product ADRs.
- `source/docs/spikes/`: migrated spike PRDs.

The files under `source/` are preserved source material.
Current implementation docs in `docs/houdini/` and GitHub issues are the active planning surfaces.

## Deletion rule

After this PR lands, `/Users/bit/Documents/houdini-clone` can be deleted without losing product requirements from that folder.
Do not continue editing the old local folder after deletion; add new PRD material to this fork instead.
