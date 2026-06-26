# Ampel Upgrade Intelligence — Documentation

> A self-learning, multi-provider polyglot repository upgrade orchestration system built on Ampel + RuVector + SONA.

## Documents

| Document | Description |
|----------|-------------|
| [PRD](prd.md) | Product Requirements Document — goals, personas, requirements, success metrics |
| [Technical Plan](technical-plan.md) | Phased implementation plan with grouped task sets and milestones |
| [Use Cases](use-cases.md) | Detailed use cases covering solo developer through enterprise fleet |
| [User Journey](user-journey.md) | Step-by-step user journeys for key workflows |
| [Architecture](architecture.md) | Technical architecture, component design, and integration points |
| [Citations](citations.md) | Full bibliography of research papers, tools, and references |

## Quick Context

This product extends [pacphi/ampel](https://github.com/pacphi/ampel) — a Rust-based multi-provider PR management dashboard — into a **self-learning polyglot repository upgrade orchestration engine**. It integrates:

- **[ruvnet/ruvector](https://github.com/ruvnet/ruvector)** — self-learning vector database with SONA adaptive engine and MCP Brain Server
- **ruvllm** — local LLM inference with MicroLoRA per-request adaptation (crate within ruvector)
- **SONA** — Self-Optimizing Neural Architecture for three-tier adaptive learning
- Polyglot ecosystem support via tree-sitter (100+ languages)
- Multi-provider Git support (GitHub, GitLab, Bitbucket, Azure DevOps, Gitea/Forgejo)

Every merged or failed upgrade PR is a training event. The system gets measurably better with every outcome — from the first deployment through thousands of repos at scale.
