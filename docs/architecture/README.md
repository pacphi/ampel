# Architecture Documentation

This directory contains architecture decision records (ADRs) and design documents for the Ampel project.

---

## 📁 Directory Structure

```
docs/architecture/
├── README.md                                    # This file
├── QUICK_REFERENCE.md                           # Quick reference card (2 pages)
├── LOCALE_MIDDLEWARE_SUMMARY.md                 # Executive summary (8 pages)
├── LOCALE_MIDDLEWARE_DESIGN.md                  # Comprehensive design (15 pages)
├── locale-middleware-flow.md                    # Visual diagrams (10 pages)
├── locale-middleware-implementation.md          # Implementation guide (12 pages)
└── adr/
    └── ADR-001-locale-middleware-state-access.md  # Architecture Decision Record (18 pages)
```

---

## 🎯 Locale Middleware Architecture (2026-01-09)

### Quick Links

- **[Quick Reference](./QUICK_REFERENCE.md)** ⭐ START HERE - 2-page implementation guide
- **[Summary](./LOCALE_MIDDLEWARE_SUMMARY.md)** - Executive summary for stakeholders
- **[Design](./LOCALE_MIDDLEWARE_DESIGN.md)** - Comprehensive architecture analysis
- **[Flow Diagrams](./locale-middleware-flow.md)** - Visual architecture diagrams
- **[Implementation](./locale-middleware-implementation.md)** - Step-by-step guide
- **[ADR-001](./adr/ADR-001-locale-middleware-state-access.md)** - Official decision record

### Document Purpose

| Document            | Audience        | Purpose                                 | Pages |
| ------------------- | --------------- | --------------------------------------- | ----- |
| **Quick Reference** | Developers      | Fast implementation reference           | 2     |
| **Summary**         | Team leads, PMs | Executive overview and next steps       | 8     |
| **Design**          | Architects      | Detailed design analysis and trade-offs | 15    |
| **Flow Diagrams**   | Visual learners | Architecture visualizations             | 10    |
| **Implementation**  | Developers      | Step-by-step implementation guide       | 12    |
| **ADR-001**         | Decision makers | Official architectural decision         | 18    |

### Reading Guide

**For Developers** (Implementation):

1. Start with [Quick Reference](./QUICK_REFERENCE.md) (5 minutes)
2. Read [Implementation Guide](./locale-middleware-implementation.md) if needed (15 minutes)
3. Refer to [Flow Diagrams](./locale-middleware-flow.md) for visual understanding

**For Architects** (Review):

1. Read [Summary](./LOCALE_MIDDLEWARE_SUMMARY.md) (10 minutes)
2. Review [Design Document](./LOCALE_MIDDLEWARE_DESIGN.md) (30 minutes)
3. Check [ADR-001](./adr/ADR-001-locale-middleware-state-access.md) for decision rationale

**For Stakeholders** (Overview):

1. Read [Summary](./LOCALE_MIDDLEWARE_SUMMARY.md) only (10 minutes)

---

## 📝 Architecture Decision Records (ADRs)

### ADR Index

| ADR                                                        | Title                                  | Status   | Date       |
| ---------------------------------------------------------- | -------------------------------------- | -------- | ---------- |
| [ADR-001](./adr/ADR-001-locale-middleware-state-access.md) | Locale Middleware State Access Pattern | Proposed | 2026-01-09 |

### ADR Template

When creating new ADRs, follow this structure:

```markdown
# ADR-NNN: Title

**Status**: Proposed | Accepted | Deprecated | Superseded
**Date**: YYYY-MM-DD
**Deciders**: [List]
**Technical Story**: [Brief context]

## Context

[Problem statement and background]

## Decision

[The change we're proposing]

## Alternatives Considered

[Other options and why they were rejected]

## Rationale

[Why this decision makes sense]

## Consequences

[Positive and negative outcomes]

## References

[Links to related docs]
```

---

## 🎨 Design Principles

### Ampel Architecture Principles

1. **Clean Architecture**: Separate concerns (layers: API, Core, DB, Providers)
2. **Type Safety**: Compile-time guarantees, avoid runtime panics
3. **Test-First**: Write tests before implementation (TDD)
4. **Explicit Dependencies**: Clear dependency injection, no hidden globals
5. **Performance**: Async/await, connection pooling, caching where needed
6. **Fail-Safe**: Graceful degradation, always have fallback
7. **Simplicity**: Minimal code for maximum value (avoid over-engineering)
8. **Documentation**: Keep ADRs and design docs up-to-date

### Technology Stack

- **Backend**: Rust 1.91+ (Axum 0.7, SeaORM, Apalis)
- **Frontend**: React 19 + TypeScript (Vite, TanStack Query)
- **Database**: PostgreSQL 16, Redis 7
- **Testing**: Vitest, cargo test, integration tests

---

## 🔄 Architecture Review Process

### Before Implementation

1. **Research**: Understand problem and constraints
2. **Design**: Create comprehensive design document
3. **ADR**: Write Architecture Decision Record
4. **Review**: Team review of design and ADR
5. **Approval**: Get sign-off from tech leads

### During Implementation

6. **Implement**: Follow implementation guide
7. **Test**: Write comprehensive tests (unit + integration)
8. **Document**: Update developer guides
9. **Review**: Code review with focus on architecture adherence

### After Deployment

10. **Monitor**: Track metrics and performance
11. **Learn**: Document lessons learned
12. **Iterate**: Refine architecture based on real-world usage

---

## 📊 Diagrams and Visualizations

### Diagram Types Used

- **Sequence Diagrams** (Mermaid) - Show request/response flow
- **Component Diagrams** (C4 model) - Show system structure
- **Flowcharts** (Mermaid) - Show decision logic and priority
- **Comparison Tables** - Show trade-off analysis
- **Architecture Diagrams** (Mermaid) - Show component interactions

### Tools

- **Mermaid**: Embedded in Markdown for easy versioning
- **ASCII Art**: For simple diagrams in code comments
- **PlantUML**: For complex UML diagrams (if needed)

---

## 🧪 Testing Strategy

### Architecture Testing Levels

1. **Unit Tests**: Test individual functions (helper functions, utilities)
2. **Integration Tests**: Test with mock database (middleware with state)
3. **E2E Tests**: Test full request/response cycle (with real API)
4. **Performance Tests**: Benchmark latency and throughput
5. **Load Tests**: Test under high concurrency

### Test Pyramid

```
       /\
      /E2E\       ← Few (critical paths only)
     /------\
    /  Integ \    ← Some (key integrations)
   /----------\
  /    Unit    \  ← Many (all logic paths)
 /--------------\
```

---

## 📚 Related Documentation

### Project Documentation

- [Main README](../../README.md) - Project overview
- [Testing Guide](../TESTING.md) - Comprehensive testing documentation
- [Localization Spec](../localization/SPECIFICATION.md) - i18n requirements
- [Developer Guide](../localization/DEVELOPER-GUIDE.md) - i18n developer guide

### External Resources

- [Axum Documentation](https://docs.rs/axum/0.7/)
- [Tower Middleware Guide](https://docs.rs/tower/latest/tower/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

---

## 🤝 Contributing

### Adding New Architecture Documents

1. **Create Design Document**: Follow template in this README
2. **Add Visual Diagrams**: Use Mermaid for consistency
3. **Write ADR**: Document decision with rationale
4. **Update Index**: Add links to this README
5. **Get Review**: Architecture review before merge

### Document Standards

- **Format**: Markdown with Mermaid diagrams
- **Naming**: `lowercase-with-dashes.md`
- **ADR Naming**: `ADR-NNN-short-title.md` (zero-padded number)
- **Max Width**: 100 characters per line (for readability)
- **Code Blocks**: Always specify language for syntax highlighting
- **Tables**: Use for comparison, not large data dumps

### Review Checklist

- [ ] Clear problem statement
- [ ] Multiple alternatives considered
- [ ] Trade-off analysis included
- [ ] Decision rationale documented
- [ ] Consequences (positive and negative) listed
- [ ] References to related docs
- [ ] Code examples are correct and tested
- [ ] Diagrams are clear and accurate
- [ ] Success criteria defined
- [ ] Rollback plan documented

---

## 📅 Document Maintenance

### When to Update

- **ADR Status Change**: When decision is accepted/rejected/superseded
- **Implementation Complete**: Add lessons learned
- **Architecture Change**: Create new ADR, mark old as superseded
- **New Features**: Add design docs before implementation
- **Performance Issues**: Document optimization decisions

### Versioning

- **ADRs**: Never modified after acceptance (create new ADR if needed)
- **Design Docs**: Updated as design evolves (track in git history)
- **Diagrams**: Keep in sync with code (review in PRs)

---

## 🔍 Finding Information

### Quick Search

```bash
# Find all ADRs
ls docs/architecture/adr/

# Search for pattern in architecture docs
grep -r "middleware" docs/architecture/

# Find recent architecture changes
git log --all -- docs/architecture/
```

### Common Queries

- **"How do I implement middleware?"** → [Implementation Guide](./locale-middleware-implementation.md)
- **"Why did we choose this pattern?"** → [ADR-001](./adr/ADR-001-locale-middleware-state-access.md)
- **"What does the flow look like?"** → [Flow Diagrams](./locale-middleware-flow.md)
- **"Quick reference?"** → [Quick Reference](./QUICK_REFERENCE.md)

---

## 📞 Contact

**Architecture Questions**: Review relevant ADR or design document first
**Implementation Help**: Check implementation guide and code examples
**Process Questions**: See [Contributing](#-contributing) section

**Team**:

- Architecture Team: Reviews and approves ADRs
- Backend Team: Implements and maintains backend architecture
- Frontend Team: Implements and maintains frontend architecture

---

## 🎯 Future Work

### Planned Architecture Improvements

- [ ] Add caching layer architecture (Redis patterns)
- [ ] Document provider abstraction architecture
- [ ] Create background job architecture guide
- [ ] Add authentication/authorization flow diagrams
- [ ] Document API versioning strategy

### Template Ideas

- ADR template (done - see above)
- Design document template
- Implementation guide template
- Performance analysis template
- Migration guide template

---

## 📊 Metrics

### Architecture Health Indicators

- **ADR Coverage**: Major decisions should have ADRs
- **Documentation Lag**: Docs should be updated within 1 week of code changes
- **Review Velocity**: Architecture reviews should happen within 2 days
- **Implementation Accuracy**: Code should match design (measured in code reviews)

### Current Status (as of 2026-01-09)

- **ADRs**: 1 (Locale Middleware)
- **Design Docs**: 4 (Locale Middleware series)
- **Diagrams**: 10+ (Mermaid sequence, flow, component)
- **Last Review**: 2026-01-09

---

## 🚀 Getting Started

**New to the project?**

1. Read [Main README](../../README.md)
2. Review [Architecture Principles](#ampel-architecture-principles)
3. Read existing ADRs to understand past decisions
4. Ask questions in team channels

**Implementing a feature?**

1. Check if design exists (search this directory)
2. If not, create design document and ADR
3. Get architecture review
4. Follow implementation guide
5. Update docs after completion

**Making an architectural change?**

1. Research alternatives
2. Write design document
3. Create ADR with rationale
4. Get team review and approval
5. Implement and document lessons learned

---

**Last Updated**: 2026-01-09
**Maintainer**: Architecture Team
**Status**: Active Development
