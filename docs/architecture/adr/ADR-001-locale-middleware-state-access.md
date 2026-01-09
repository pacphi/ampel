# ADR-001: Locale Middleware State Access Pattern

**Status**: Proposed
**Date**: 2026-01-09
**Deciders**: System Architecture Team, Backend Team
**Technical Story**: [i18n Phase 2] Enable user language preference from database in locale detection middleware

---

## Context

### Problem Statement

The Ampel application needs to detect user locale/language preferences for internationalization (i18n). The current locale detection middleware (`locale_detection_middleware`) attempts to access the database to read user language preferences, but fails because it cannot access `AppState` in Axum 0.7.

**Current Detection Priority**:

1. Query parameter (`?lang=fi`)
2. User database preference ❌ **BROKEN** - no state access
3. Cookie (`lang=fi`)
4. Accept-Language header
5. Default (`en`)

### Technical Context

- **Framework**: Axum 0.7.x
- **Pattern Used**: `middleware::from_fn` (stateless middleware)
- **State Required**: `AppState` containing:
  - `DatabaseConnection` - to query `users.language` column
  - `AuthService` - to validate JWT tokens and extract user ID

### Current Implementation (Broken)

```rust
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = if let Some(state) = req.extensions().get::<AppState>() {
        detect_locale_with_state(&req, state).await
    } else {
        detect_locale(&req)  // Always falls back to this
    };
    // ...
}

// Router registration
Router::new()
    .with_state(state)
    .layer(middleware::from_fn(locale_detection_middleware))  // ❌ No state injection
```

**Why It Fails**:
In Axum 0.7, `middleware::from_fn` does NOT automatically inject `AppState` into request extensions. The `req.extensions().get::<AppState>()` call always returns `None`, so user database preferences are never queried.

### Business Impact

- **User Experience**: Users must manually set language preference via query param or cookie on every device
- **Feature Completeness**: User profile language setting is ignored
- **Competitive Disadvantage**: Modern apps remember user preferences across sessions/devices
- **i18n Adoption**: Reduced adoption of non-English languages (users default to English)

---

## Decision

**We will use Axum's `middleware::from_fn_with_state` pattern to provide explicit state access to the locale detection middleware.**

### Implementation

```rust
// Middleware signature with explicit State parameter
pub async fn locale_detection_middleware(
    State(state): State<AppState>,  // ✅ Injected by Axum
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let locale = detect_locale_with_state(&req, &state).await;
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// Router registration with state
Router::new()
    .layer(middleware::from_fn_with_state(
        state.clone(),                 // ✅ Pass state clone
        locale_detection_middleware,
    ))
    .with_state(state)
```

### Key Changes

1. **Middleware Signature**: Add `State(state): State<AppState>` parameter
2. **Router Registration**: Use `from_fn_with_state` instead of `from_fn`
3. **State Passing**: Pass `state.clone()` (cheap Arc clone)
4. **Fallback Removal**: Remove `detect_locale` function (always have state now)

---

## Alternatives Considered

### Option A: Request Extensions (Status Quo - Rejected)

**Approach**: Continue trying to access state from request extensions

```rust
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = if let Some(state) = req.extensions().get::<AppState>() {
        detect_locale_with_state(&req, state).await
    } else {
        detect_locale(&req)
    };
    // ...
}
```

**Pros**:

- No code changes needed
- Simple middleware signature

**Cons**:

- ❌ **Doesn't work in Axum 0.7** (always returns None)
- ❌ Relies on undocumented behavior
- ❌ Unreliable, breaks silently
- ❌ User preferences never detected

**Verdict**: REJECTED - fundamentally broken in Axum 0.7

---

### Option B: `from_fn_with_state` (SELECTED)

**Approach**: Use Axum's official pattern for stateful middleware

```rust
pub async fn locale_detection_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let locale = detect_locale_with_state(&req, &state).await;
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}
```

**Pros**:

- ✅ **Official Axum 0.7 pattern** (documented, supported)
- ✅ **Type-safe** - compiler enforces state parameter
- ✅ **Explicit** - clear dependency on AppState
- ✅ **Simple** - 5 lines of code change
- ✅ **Testable** - easy to mock state
- ✅ **Performant** - Arc clones are O(1)

**Cons**:

- ⚠️ Requires `state.clone()` (negligible cost)
- ⚠️ Different pattern than `from_fn` (but clearer)

**Verdict**: SELECTED - official, type-safe, minimal changes

---

### Option C: Custom Middleware Layer (Rejected)

**Approach**: Implement Tower's `Layer` and `Service` traits manually

```rust
pub struct LocaleLayer {
    state: AppState,
}

impl<S> Layer<S> for LocaleLayer {
    type Service = LocaleService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        LocaleService { inner, state: self.state.clone() }
    }
}

pub struct LocaleService<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request<Body>> for LocaleService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    // ... 60+ lines of trait implementations
}
```

**Pros**:

- ✅ Maximum control over middleware lifecycle
- ✅ Can implement complex coordination logic
- ✅ Follows Tower's service pattern

**Cons**:

- ❌ **80+ lines of boilerplate** vs 5 lines for Option B
- ❌ Complex trait implementations (Service, Layer, Future, Pin)
- ❌ Hard to understand and maintain
- ❌ More opportunity for bugs (lifetimes, borrow checker)
- ❌ Overkill for simple state access

**Verdict**: REJECTED - unnecessary complexity

---

### Option D: Extractor Pattern (Rejected)

**Approach**: Use `FromRequestParts` trait to extract locale in handlers

```rust
#[async_trait]
impl FromRequestParts<AppState> for DetectedLocale {
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let locale = detect_locale_with_state(parts, state).await;
        Ok(DetectedLocale::new(locale))
    }
}

// Usage in every handler
pub async fn handler(
    locale: DetectedLocale,  // Extracted automatically
    State(state): State<AppState>,
) -> Response {
    // ...
}
```

**Pros**:

- ✅ Clean handler signatures
- ✅ Type-safe extraction
- ✅ Per-handler control

**Cons**:

- ❌ **Not middleware** - must add to every handler
- ❌ Detection happens per-handler (not once per request)
- ❌ Can't share detection result across middleware pipeline
- ❌ Requires modifying 40+ handler signatures
- ❌ Breaks DRY principle

**Verdict**: REJECTED - not a middleware pattern, too invasive

---

## Trade-off Analysis

| Aspect                 | Option A (Extensions)    | Option B (from_fn_with_state) ⭐ | Option C (Custom Layer) | Option D (Extractor) |
| ---------------------- | ------------------------ | -------------------------------- | ----------------------- | -------------------- |
| **Complexity**         | Low (but broken)         | ⭐ Low (5 lines)                 | High (80+ lines)        | Medium (per-handler) |
| **Type Safety**        | ❌ None (runtime fail)   | ⭐ Compile-time                  | Compile-time            | Compile-time         |
| **Testability**        | Hard (needs real router) | ⭐ Easy (mock state)             | Hard (Tower traits)     | Easy (mock state)    |
| **Maintainability**    | ❌ Unreliable            | ⭐ Clear pattern                 | ❌ Complex traits       | ❌ Repetitive        |
| **Performance**        | N/A (broken)             | ⭐ Excellent (O(1))              | Excellent               | Good (per-handler)   |
| **Axum Compatibility** | ❌ Broken in 0.7         | ⭐ Official pattern              | Works                   | Works                |
| **Migration Effort**   | N/A (broken)             | ⭐ 5 minutes                     | 2 hours                 | 4+ hours             |
| **LOC Changed**        | 0 (but broken)           | ⭐ ~15 lines                     | ~100 lines              | ~200 lines           |

**Winner**: Option B (`from_fn_with_state`) - official, simple, type-safe, easy to migrate

---

## Rationale

### Why `from_fn_with_state` is the Right Choice

1. **Official Pattern**: Documented in Axum 0.7 guide as the standard way to write stateful middleware
2. **Type Safety**: Compiler enforces state parameter - can't forget to pass it
3. **Simplicity**: Minimal code changes (2 files, ~15 lines total)
4. **Explicit Dependencies**: Signature clearly shows middleware needs AppState
5. **Testability**: Easy to create mock AppState for unit tests
6. **Performance**: Arc clones are O(1) atomic operations (nanoseconds)
7. **Future-Proof**: Follows Tower/Axum idioms, likely to be stable

### Why Other Options Don't Work

- **Option A**: Fundamentally broken in Axum 0.7 (not a real option)
- **Option C**: 16x more code for same functionality (poor engineering)
- **Option D**: Not middleware, breaks existing handlers (wrong abstraction)

### Alignment with Project Principles

From `CLAUDE.md`:

- **Clean Architecture**: Separate concerns (middleware vs handlers) ✅
- **Test-First**: Easy to test with mock state ✅
- **Modular Design**: Files under 500 lines ✅
- **Best Practices**: Follow framework conventions ✅

---

## Consequences

### Positive

1. **User Experience**: User language preferences are now respected across all devices/sessions
2. **Feature Completeness**: All 5 locale detection sources work (query, DB, cookie, header, default)
3. **Type Safety**: Compile-time guarantee that state is available
4. **Code Quality**: Simpler code (remove conditional fallback logic)
5. **Testability**: Easy to write integration tests with mock database
6. **Maintainability**: Clear, explicit dependencies in middleware signature
7. **Performance**: No overhead vs manual Layer implementation

### Negative

1. **State Clone Required**: Must call `state.clone()` in router (Arc clone is cheap but not free)
2. **Pattern Difference**: Different from `from_fn` (but more explicit)
3. **Migration Effort**: Requires updating 2 files and adding tests (~30 minutes)

### Neutral

1. **Learning Curve**: Team must understand `from_fn_with_state` pattern
2. **Documentation**: Must update developer guide with new pattern

---

## Implementation Plan

### Phase 1: Core Changes (10 minutes)

1. Update middleware signature in `middleware/locale.rs`
2. Update router registration in `routes/mod.rs`
3. Remove `detect_locale` fallback function

### Phase 2: Testing (15 minutes)

4. Add integration tests with mock database
5. Test user preference detection with JWT
6. Test priority order (query > DB > cookie > header > default)
7. Test fallback when DB fails

### Phase 3: Documentation (5 minutes)

8. Update developer guide with new pattern
9. Add migration notes for future reference

**Total Time**: 30 minutes

### Rollback Strategy

If issues arise, revert to simple detection (no DB access):

```rust
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = detect_locale(&req);  // No DB access
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}
```

**Rollback Time**: < 5 minutes

---

## Performance Implications

### Latency Impact

| Scenario               | Before | After  | Delta               |
| ---------------------- | ------ | ------ | ------------------- |
| No auth                | < 1ms  | < 1ms  | 0ms (no DB query)   |
| Authenticated (cached) | < 1ms  | < 2ms  | +1ms (Redis lookup) |
| Authenticated (DB hit) | < 1ms  | < 10ms | +9ms (DB query)     |

### Optimization Opportunities

1. **Redis Caching** (Optional):
   - Cache `user:{uuid}:language` in Redis
   - TTL: 1 hour
   - Reduces DB queries by 95%+

2. **In-Memory LRU Cache** (Simpler):
   - 1000-entry LRU cache (~100KB memory)
   - Invalidate on language update
   - No external dependencies

**Recommendation**: Start without caching, add if profiling shows bottleneck.

### Database Query Optimization

- ✅ Primary key lookup (indexed)
- ✅ Single column SELECT (minimal data transfer)
- ✅ Connection pooling (already configured)
- ✅ Async/await (non-blocking)

**Expected P99 Latency**: < 20ms (well within acceptable range)

---

## Risk Assessment

### Risk Level: LOW

**Justification**:

- Official Axum pattern (well-tested, documented)
- Minimal code changes (2 files, 15 lines)
- Easy rollback (< 5 minutes)
- Graceful degradation (fallback to cookie/header if DB fails)
- No breaking changes to API
- Extensive test coverage planned

### Mitigation Strategies

| Risk                            | Likelihood | Impact | Mitigation                              |
| ------------------------------- | ---------- | ------ | --------------------------------------- |
| Performance regression          | Low        | Medium | Add metrics, benchmark before/after     |
| Database connection issues      | Low        | Medium | Graceful fallback to cookie/header      |
| Breaking changes to handlers    | Very Low   | High   | No changes needed (backward compatible) |
| Test failures                   | Medium     | Low    | Write comprehensive integration tests   |
| Team unfamiliarity with pattern | Low        | Low    | Document in developer guide             |

---

## Success Metrics

### Functional Requirements

- [ ] Middleware compiles without errors
- [ ] Query param detection works
- [ ] User DB preference detection works (with JWT)
- [ ] Cookie detection works
- [ ] Accept-Language header detection works
- [ ] Default fallback works
- [ ] Priority order is correct

### Non-Functional Requirements

- [ ] All tests pass (unit + integration)
- [ ] No performance regression (< 10ms added latency)
- [ ] Code coverage > 80%
- [ ] Documentation complete
- [ ] No breaking changes to API

### Business Metrics

- [ ] User language preference adoption > 50% (within 1 month)
- [ ] Reduction in manual language switching (cookie/query param)
- [ ] Improved user satisfaction scores for non-English users

---

## Monitoring & Observability

### Metrics to Track

1. **Locale Detection Source Distribution**:
   - `locale_detection_source{source="query"}` - count
   - `locale_detection_source{source="database"}` - count
   - `locale_detection_source{source="cookie"}` - count
   - `locale_detection_source{source="header"}` - count
   - `locale_detection_source{source="default"}` - count

2. **Performance Metrics**:
   - `locale_detection_duration_seconds` - histogram
   - `locale_db_query_duration_seconds` - histogram
   - `locale_db_query_errors_total` - counter

3. **Database Metrics**:
   - `user_language_cache_hits_total` - counter (if caching)
   - `user_language_cache_misses_total` - counter (if caching)

### Alerts

- **High Latency**: Alert if P99 > 50ms (indicates DB issues)
- **High Error Rate**: Alert if DB errors > 1% (indicates connection problems)
- **Low Adoption**: Alert if DB source < 10% after 1 week (indicates low feature adoption)

---

## Documentation Updates

### Files to Update

1. **Developer Guide** (`docs/localization/DEVELOPER-GUIDE.md`):
   - Add section on middleware architecture
   - Document `from_fn_with_state` pattern
   - Add usage examples for locale-aware handlers

2. **Architecture Docs** (new):
   - `docs/architecture/LOCALE_MIDDLEWARE_DESIGN.md` - comprehensive design
   - `docs/architecture/locale-middleware-flow.md` - visual diagrams
   - `docs/architecture/locale-middleware-implementation.md` - implementation guide
   - `docs/architecture/ADR-001-locale-middleware-state-access.md` - this ADR

3. **Testing Guide** (`docs/TESTING.md`):
   - Add middleware testing examples
   - Document mock state creation

---

## Related Decisions

- **ADR-000**: Use Axum 0.7 as web framework (implicit, from project setup)
- **ADR-TBD**: i18n architecture and locale detection strategy

---

## References

### External Documentation

- [Axum 0.7 Middleware Guide](https://docs.rs/axum/0.7/axum/middleware/index.html)
- [Axum State Sharing](https://docs.rs/axum/0.7/axum/extract/struct.State.html)
- [Tower Middleware Architecture](https://docs.rs/tower/latest/tower/trait.Service.html)

### Internal Documentation

- [Localization Specification](../../localization/SPECIFICATION.md)
- [Localization Architecture](../../localization/ARCHITECTURE.md)
- [Testing Documentation](../../TESTING.md)

### Code References

- `crates/ampel-api/src/middleware/locale.rs` - current implementation
- `crates/ampel-api/src/routes/mod.rs` - router configuration
- `crates/ampel-api/src/state.rs` - AppState definition

---

## Decision Log

| Date       | Event          | Decision Maker               |
| ---------- | -------------- | ---------------------------- |
| 2026-01-09 | ADR created    | System Architecture Designer |
| TBD        | Team review    | Backend Team                 |
| TBD        | Approval       | Tech Lead                    |
| TBD        | Implementation | Backend Developer            |
| TBD        | Deployment     | DevOps                       |

---

## Appendix: Code Comparison

### Before (Broken)

```rust
// middleware/locale.rs
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = if let Some(state) = req.extensions().get::<AppState>() {
        detect_locale_with_state(&req, state).await  // Never executes
    } else {
        detect_locale(&req)  // Always executes
    };
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

fn detect_locale(req: &Request<Body>) -> String {
    // Check query param, cookie, header
    // NO DATABASE ACCESS
}

// routes/mod.rs
Router::new()
    .with_state(state)
    .layer(middleware::from_fn(locale_detection_middleware))  // ❌
```

### After (Fixed)

```rust
// middleware/locale.rs
pub async fn locale_detection_middleware(
    State(state): State<AppState>,  // ✅ Injected by Axum
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let locale = detect_locale_with_state(&req, &state).await;  // ✅ Always has state
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// detect_locale function removed (no longer needed)

// routes/mod.rs
Router::new()
    .layer(middleware::from_fn_with_state(  // ✅ With state
        state.clone(),
        locale_detection_middleware,
    ))
    .with_state(state)
```

**Key Difference**: State is explicitly passed and injected by Axum, not retrieved from extensions.

---

## Approval

**Status**: ⏳ Pending Review

**Reviewers**:

- [ ] Tech Lead (Architecture approval)
- [ ] Backend Lead (Implementation review)
- [ ] QE Lead (Testing strategy review)

**Next Steps**:

1. Schedule architecture review meeting
2. Present ADR to team
3. Address feedback and concerns
4. Update ADR based on review
5. Get final approval
6. Proceed with implementation

---

**Author**: System Architecture Designer (Claude Code Agent)
**Created**: 2026-01-09
**Last Updated**: 2026-01-09
**Status**: Proposed → Pending Review → Approved (TBD)
