# Design Source Methodology

A structured, technology-agnostic framework for designing and building software with AI assistance.

## Philosophy

Software development with AI assistants works best when both human and AI operate within a shared, explicit process. Ad-hoc prompting produces inconsistent results. A methodology provides:

- **Predictability** — Both parties know what phase they're in and what's expected
- **Quality** — Defined gates prevent shipping incomplete or untested work
- **Continuity** — Session boundaries don't erase context; decisions are recorded
- **Scalability** — The same process works for a CLI tool, a web app, or a distributed system

## Core Principles

1. **Specification before implementation** — Write down what you're building before writing code
2. **Incremental delivery** — Ship working software in small, verifiable iterations
3. **Decisions are permanent artifacts** — Record architectural choices with rationale
4. **The Project Definition is the source of truth** — All technology-specific details live in one place
5. **Quality gates are non-negotiable** — Every iteration passes the project's defined checks
6. **AI is a collaborator, not an oracle** — The human owns decisions; the AI accelerates execution

## The Iteration Cycle

Every unit of work follows 5 phases:

```
┌─────────────────────────────────────────┐
│              1. ANALYZE                  │
│  Understand requirements, break down    │
│  complexity, identify unknowns          │
└────────────────────┬────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────┐
│               2. PLAN                    │
│  Define architecture, structure,        │
│  milestones, and blockers               │
└────────────────────┬────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────┐
│              3. SPECIFY                  │
│  Write technical refinement docs for    │
│  each unit of work                      │
└────────────────────┬────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────┐
│             4. IMPLEMENT                 │
│  Build, test, integrate following       │
│  the specification and plan             │
└────────────────────┬────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────┐
│              5. VERIFY                   │
│  Run quality gates, update docs,        │
│  prepare for delivery                   │
└─────────────────────────────────────────┘

         Repeat for each work unit.
```

## Directory Structure

```
impl/
├── memory.md              # Current project state, active work
├── project-definition.md  # Technology stack, conventions, quality gates
├── resume-session.md      # Prompt to resume work
├── methodology/           # This folder — process documentation
│   ├── overview.md        # This file
│   ├── phases/            # Detailed guide for each phase
│   ├── roles.md           # AI assistant role and expectations
│   └── decision-framework.md
└── history/               # Iteration tracking files
```

## Getting Started

1. Read the [phase guides](phases/) to understand the workflow
2. Use `resume-session.md` to resume work with an AI assistant
3. Follow the iteration cycle for each unit of work, tracking progress in `history/`
