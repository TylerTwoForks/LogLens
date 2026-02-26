# CLAUDE.md — LogLens Orchestration Rules

This repository uses a portable Claude setup that mirrors Cursor rules as standard Markdown files.

## Primary Rule Loading Order

1. `.claude/skills/rules/loglens-current-phase.md` (active execution pointer)
2. The active phase file referenced by `phase_rule_file`
3. `.claude/skills/rules/loglens-product-vision.md`
4. Stack-specific guidance as needed:
   - Next.js: `.claude/skills/rules/next.js/next.js.md`
   - Python/data workflows: `.claude/skills/rules/python.md`
   - Rust projects: `.claude/skills/rules/rust/main.md`

## Phase Guardrails (Current)

- Active phase: `phase-5-security-privacy-hardening`
- Keep changes minimal, phase-aligned, and incrementally testable.
- If work requests jump ahead of current phase scope, call it out and confirm before implementing.
- Do not add Redis or distributed workers before Phase 7 unless explicitly requested.

## Product Non-Negotiables

- Privacy first: do not persist raw logs as durable application data.
- Multi-tenant safety: strict organization boundary enforcement.
- Explainability: chart outputs must map to parsed events.
- Fast feedback: users should see progress and results quickly.

## Directory Layout

- `.claude/skills/rules/` — converted rule set from `.cursor/rules/` (`.md` format)
- `.claude/skills/README.md` — inventory of converted rule files
- `.claude/agents/rust-development-agents.md` — Rust role orchestration guidance
- `.claude/agents/README.md` — agent index

## Maintenance Workflow

When `.cursor/rules` changes, regenerate the mirrored `.claude/skills/rules` files so both environments stay aligned.
