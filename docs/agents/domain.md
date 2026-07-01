# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

This is a multi-context repo.
Read `CONTEXT-MAP.md` at the repo root first.

## Before exploring graph workbench product-fork work

For Houdini-like graph UI, node network, workbench layout, Python operator, procedural asset, output target, or product-fork issues, read:

- `product/graph-workbench/product-prd/source/CONTEXT.md`
- relevant ADRs in `product/graph-workbench/product-prd/source/docs/adr/`
- current planning docs directly under `product/graph-workbench/`

## Before exploring general Rerun platform work

Use the existing root engineering docs:

- `AGENTS.md`
- `CLAUDE.md`
- `ARCHITECTURE.md`
- `DESIGN.md`
- `docs/README.md`

## Use the glossary's vocabulary

When your output names a domain concept in an issue title, refactor proposal, hypothesis, or test name, use the term as defined in the relevant `CONTEXT.md`.
Do not drift to synonyms the glossary explicitly avoids.

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding it.
