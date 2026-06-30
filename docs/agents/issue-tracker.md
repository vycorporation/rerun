# Issue tracker: GitHub

Issues and PRDs for this repo live in GitHub Issues on `vycorporation/rerun`.
Use the `gh` CLI for issue operations.
Do not create product-fork issues in `rerun-io/rerun`.

## Conventions

- Create an issue: `gh issue create --repo vycorporation/rerun --title "..." --body-file ...`.
- Read an issue: `gh issue view <number> --repo vycorporation/rerun --comments`.
- List issues: `gh issue list --repo vycorporation/rerun --state open --json number,title,body,labels,comments`.
- Comment on an issue: `gh issue comment <number> --repo vycorporation/rerun --body "..."`.
- Apply or remove labels: `gh issue edit <number> --repo vycorporation/rerun --add-label "..."` / `--remove-label "..."`.
- Close an issue: `gh issue close <number> --repo vycorporation/rerun --comment "..."`.

## Pull requests as a triage surface

PRs as a request surface: no.

Collaborator PRs can still be reviewed directly when requested, but `/triage` should not treat external PRs as incoming feature requests by default.

## When a skill says "publish to the issue tracker"

Create a GitHub issue in `vycorporation/rerun`.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --repo vycorporation/rerun --comments`.
