# Documentation Archives

This directory contains point-in-time documentation that has historical value but is no longer actively maintained. It serves as a record of where we've been—the research, decisions, and implementation work that shaped the current codebase.

---

## Archive Conventions

### Directory Structure

Archives are organized by **date** (YYYY/MM) and **category**:

```
docs/.archives/
├── 2025/
│   └── 12/               # December 2025
│       ├── localization/
│       ├── planning/
│       ├── testing/
│       └── ...
├── 2026/
│   └── 01/               # January 2026
│       ├── localization/
│       ├── testing/
│       └── ...
└── README.md
```

### Filename Guidelines

- **UPPERCASE-WITH-DASHES.md** - Standard for reports, summaries, guides
- **lowercase-with-dashes.md** - Acceptable for implementation notes
- **Include dates in status reports**: `STATUS-REPORT-2026-01-08.md`
- **Prefix with category if ambiguous**: `API-KEY-DIAGNOSTIC.md` vs `TEST-EXECUTION-RESULTS.md`

### What Gets Archived

| Type                     | Example                         | When to Archive                   |
| ------------------------ | ------------------------------- | --------------------------------- |
| Status reports           | `STATUS-REPORT-*.md`            | Immediately after the period ends |
| Implementation summaries | `*-COMPLETE.md`, `*-SUMMARY.md` | When work is merged               |
| Test results             | `TEST-*.md`, `*-RESULTS.md`     | After issues are resolved         |
| Research                 | `*-RESEARCH.md`                 | When decision is made             |
| Migration guides         | `*-MIGRATION.md`                | After migration is complete       |
| Diagnostic notes         | `*-DIAGNOSTIC.md`               | After issue is resolved           |

---

## Project History

A narrative of the major documentation epochs, for anyone curious about how we got here.

### December 2025 - Foundation

**Focus**: Core infrastructure, testing patterns, initial feature planning

- Established SQLite CI testing patterns for faster local development
- Created multi-account PAT support design
- Built observability infrastructure (Prometheus, Grafana)
- Implemented repository visibility filters across all providers

### Late December 2025 - Quality & Performance

**Focus**: Git diff feature, visibility breakdown tiles, performance optimization

- TDD refactoring cycle for git diff integration
- Performance benchmarking and index optimization
- Integration test infrastructure improvements
- E2E testing infrastructure analysis

### January 2026 - Internationalization (i18n)

**Focus**: Full localization support for 27 languages

- Axum 0.7 → 0.8 migration (required for locale middleware)
- Backend translation system with rust-i18n
- Frontend i18next integration with RTL support
- Language switcher UI components
- Extensive test suite updates for i18n compatibility

---

## Archive Organization

### `/2026/01/` - January 2026

**localization/** - i18n implementation artifacts

- API troubleshooting, translation completion reports, quality reviews

**testing/** - i18n test suite work

- Test fix summaries, validation reports, integration results

**migrations/** - Framework upgrades

- Axum 0.7 to 0.8 migration guide

**research/** - Technical investigations

- Axum middleware state access patterns

**i18n/** - Implementation notes

- User language detection enhancement details

### `/2025/12/` - December 2025

**implementation/** (1 file)

- OAuth-based architecture design

**localization/** (27 files) - i18n Phase 0-8 implementation

- PHASE-\* status reports and completion summaries
- FINAL-\* language strategy and integration reports
- Research: Systran API, translation provider evaluation
- Implementation summaries: CLI integration, DeepL client, CI/CD setup
- Analysis: Language code consistency, comparison tables, standardization

**migrations/** (2 files) - Database optimization

- Performance indexes migration summary and test guide

**performance/** - Visibility breakdown performance work

- Implementation summaries, metrics documentation

**planning/** (6 files) - Completed feature implementations

- Multi-account PAT support design
- Merge operations and notifications
- Repository visibility filter implementation
- Visibility breakdown tiles implementation
- Git diff view integration plan

**quality/** (7 files) - Code quality remediation

- Docker consolidation, deployment docs, observability docs
- Implementation review, readme redesign proposal
- Remediation status reports

**research/** (6 files) - Technical investigations

- Provider API visibility research (GitHub, GitLab, Bitbucket)
- Observability research, quality assessment
- SQLite CI testing best practices

**testing/** (16+ files) - Test infrastructure and reports

- SQLite CI testing, workflow guides
- Integration test architecture
- Phase 1 i18n test plans and validation results
- Visibility breakdown test summaries

---

## Finding Current Documentation

**Active documentation** lives in the main `/docs/` directory:

| Location               | Contents                                      |
| ---------------------- | --------------------------------------------- |
| `/docs/`               | Core guides (setup, deployment, contributing) |
| `/docs/architecture/`  | ADRs and design documents                     |
| `/docs/features/`      | Feature specifications                        |
| `/docs/localization/`  | Active i18n guides (developer, user)          |
| `/docs/observability/` | Monitoring and metrics                        |
| `/docs/planning/`      | Product specs and roadmaps                    |
| `/docs/testing/`       | Test guides and patterns                      |

---

## Archive Policy

Documents are archived when:

- They describe a **completed** implementation (task is done)
- They are **superseded** by newer documentation
- They represent **point-in-time** assessments, reports, or research
- They have **historical value** but are no longer actively referenced

When archiving, place documents in the appropriate `YYYY/MM/category/` directory.

---

_Last updated: January 9, 2026_
