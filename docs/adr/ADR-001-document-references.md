# ADR-001: Document References Strategy

**Status:** Accepted
**Date:** 2026-07-07
**Deciders:** Documentation Engineer, Tech Lead

## Context
Project Aegis builds upon concepts from Agent Zero, whose source code exceeds 150MB. Early discussions considered copying Agent Zero documentation into the project's /a0/docs/ directory. However, this creates duplication, version drift, and violates the single-source-of-truth principle (T2).

## Decision Drivers
- Maintain one source of truth for all reference material
- Minimize repository size and update burden during active development
- Enable offline/CI access when the host filesystem is unavailable
- Keep project documentation self-sufficient for common developer questions

## Considered Options
- **Option A:** Full copy of Agent Zero source into /a0/docs/ — simplest to read, but 150MB+ bloat, immediate version drift, violates T2
- **Option B:** Read-on-demand via REFERENCE_INDEX.md with text_editor_remote access to host filesystem — zero duplication, always fresh, requires host connection
- **Option C:** Git subtree or shallow clone (`--depth 1 --no-checkout`) of only the docs/ subtree — minimal size, syncable, but adds CI complexity
- **Option D:** Symlink to host-mounted Agent Zero repository — auto-updates, but only works when host mount is present

## Decision
Chosen: **Option B — REFERENCE_INDEX.md + read-on-demand**

Create a REFERENCE_INDEX.md at /a0/docs/REFERENCE_INDEX.md that maps every useful Agent Zero document (from `C:\Agent-Zero\agent-zero\`) with a one-line summary and full host path. Developers read specific files via text_editor_remote when needed. For offline/CI scenarios, Option C (`git clone --depth 1 --no-checkout` of docs/ subtree) serves as a documented fallback.

## Consequences
- Positive: Zero duplication, always fresh from source, no sync burden
- Positive: REFERENCE_INDEX.md is easy to maintain and extendable to other non-Agent Zero references
- Positive: Single annotation (`REFERENCE_INDEX.md`) tells the team where to look
- Negative: Requires live host connection for most references; offline fallback adds CI step
- Negative: Slightly higher latency than local copy for repeated lookups of the same file

## Compliance
- All documentation references to Agent Zero material must go through REFERENCE_INDEX.md
- No file from the Agent Zero repo shall be copied verbatim into /a0/docs/ without an explicit ADR exception
- The REFERENCE_INDEX.md must be updated whenever a new reference is needed