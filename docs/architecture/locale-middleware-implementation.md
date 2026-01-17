# Locale Middleware Implementation Guide

## Quick Reference

**Pattern**: `middleware::from_fn_with_state` (Axum 0.7)
**Files Changed**: 2 files (`middleware/locale.rs`, `routes/mod.rs`)
**Migration Time**: 30 minutes
**Risk Level**: Low

---

## Implementation Pseudocode

### 1. Middleware Signature (locale.rs)

```rust
// BEFORE (broken)
pub async fn locale_detection_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // ❌ Can't access AppState
}

// AFTER (fixed)
pub async fn locale_detection_middleware(
    State(state): State<AppState>,  // ✅ Add this parameter
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // ✅ Now have full state access
}
```

### 2. Detection Logic (locale.rs)

```pseudocode
FUNCTION locale_detection_middleware(state, request, next):
    locale = detect_locale_with_state(request, state)
    request.extensions.insert(DetectedLocale(locale))
    RETURN next.run(request)

FUNCTION detect_locale_with_state(request, state):
    // Priority 1: Query parameter (?lang=fi)
    IF query_param = extract_query_param(request, "lang") THEN
        IF is_supported_locale(query_param) THEN
            RETURN query_param

    // Priority 2: User database preference (requires auth)
    IF user_id = extract_user_from_jwt(request, state) THEN
        IF user = database.find_user(user_id) THEN
            IF user.language IS NOT NULL THEN
                IF is_supported_locale(user.language) THEN
                    RETURN user.language

    // Priority 3: Cookie (lang=fi)
    IF cookie = request.cookies.get("lang") THEN
        IF is_supported_locale(cookie) THEN
            RETURN cookie

    // Priority 4: Accept-Language header
    IF header = request.headers.get("Accept-Language") THEN
        IF locale = parse_accept_language(header) THEN
            RETURN locale

    // Priority 5: Default fallback
    RETURN "en"

FUNCTION extract_user_from_jwt(request, state):
    IF auth_header = request.headers.get("Authorization") THEN
        IF token = auth_header.strip_prefix("Bearer ") THEN
            IF claims = state.auth_service.validate_token(token) THEN
                RETURN claims.user_id
    RETURN NULL
```

### 3. Router Registration (routes/mod.rs)

```rust
// BEFORE (broken)
Router::new()
    .route("/api/endpoint", get(handler))
    .with_state(state)
    .layer(middleware::from_fn(locale_detection_middleware))  // ❌ No state

// AFTER (fixed)
Router::new()
    .route("/api/endpoint", get(handler))
    .layer(middleware::from_fn_with_state(  // ✅ Pass state
        state.clone(),                       // ✅ Cheap Arc clone
        locale_detection_middleware,
    ))
    .with_state(state)
```

---

## Code Changes (Minimal Diff)

### File 1: `crates/ampel-api/src/middleware/locale.rs`

```diff
  pub async fn locale_detection_middleware(
+     State(state): State<AppState>,
      mut req: Request<Body>,
      next: Next,
  ) -> Response {
-     let locale = if let Some(state) = req.extensions().get::<AppState>() {
-         detect_locale_with_state(&req, state).await
-     } else {
-         detect_locale(&req)
-     };
+     let locale = detect_locale_with_state(&req, &state).await;

      req.extensions_mut().insert(DetectedLocale::new(locale));
      next.run(req).await
  }

- // Remove detect_locale function (no longer needed)
- fn detect_locale(req: &Request<Body>) -> String { ... }
```

**Changes**:

- Add `State(state): State<AppState>` parameter (1 line)
- Remove conditional state access (3 lines → 1 line)
- Delete `detect_locale` function (40 lines removed)

### File 2: `crates/ampel-api/src/routes/mod.rs`

```diff
+ use crate::middleware::locale_detection_middleware;

  pub fn create_router(state: AppState) -> Router {
      Router::new()
          // ... all routes ...
-         // TODO: Re-enable after fixing axum 0.7 compatibility
-         // .layer(middleware::from_fn(locale_detection_middleware))
+         .layer(middleware::from_fn_with_state(
+             state.clone(),
+             locale_detection_middleware,
+         ))
          .layer(middleware::from_fn(track_metrics))
          .with_state(state)
  }
```

**Changes**:

- Uncomment and update middleware registration (5 lines)
- Use `from_fn_with_state` instead of `from_fn`
- Pass `state.clone()` (cheap Arc clone)

---

## Testing Implementation

### Unit Tests (Keep Existing)

```rust
#[test]
fn test_normalize_locale() {
    assert_eq!(normalize_locale("en"), "en");
    assert_eq!(normalize_locale("pt"), "pt-BR");
    assert_eq!(normalize_locale("zh"), "zh-CN");
}

#[test]
fn test_is_supported_locale() {
    assert!(is_supported_locale("en"));
    assert!(is_supported_locale("fi"));
    assert!(!is_supported_locale("xx"));
}

#[test]
fn test_parse_accept_language() {
    assert_eq!(
        parse_accept_language("fi;q=0.9,en;q=0.8"),
        Some("fi".to_string())
    );
}
```

### Integration Tests (Add New)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    async fn create_test_state() -> AppState {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        // ... initialize test state
    }

    #[tokio::test]
    async fn test_user_db_preference() {
        let state = create_test_state().await;

        // Create user with language preference
        let user = user::ActiveModel {
            email: Set("test@example.com".to_string()),
            language: Set(Some("fi".to_string())),
            ..Default::default()
        }.insert(&state.db).await.unwrap();

        // Create JWT token
        let token = state.auth_service.create_access_token(user.id).unwrap();

        // Build request
        let req = Request::builder()
            .uri("https://example.com/api")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Test detection
        let locale = detect_locale_with_state(&req, &state).await;
        assert_eq!(locale, "fi");
    }

    #[tokio::test]
    async fn test_priority_order() {
        let state = create_test_state().await;

        // Query param should override everything
        let req = Request::builder()
            .uri("https://example.com/api?lang=de")
            .header(header::COOKIE, "lang=fi")
            .header(header::ACCEPT_LANGUAGE, "fr")
            .body(Body::empty())
            .unwrap();

        let locale = detect_locale_with_state(&req, &state).await;
        assert_eq!(locale, "de");  // Query param wins
    }

    #[tokio::test]
    async fn test_invalid_token_fallback() {
        let state = create_test_state().await;

        // Invalid token should fall back to cookie
        let req = Request::builder()
            .uri("https://example.com/api")
            .header(header::AUTHORIZATION, "Bearer invalid_token")
            .header(header::COOKIE, "lang=fi")
            .body(Body::empty())
            .unwrap();

        let locale = detect_locale_with_state(&req, &state).await;
        assert_eq!(locale, "fi");  // Falls back to cookie
    }
}
```

---

## Migration Checklist

### Pre-Migration

- [ ] Review current middleware implementation
- [ ] Understand Axum 0.7 `from_fn_with_state` pattern
- [ ] Backup current working code (git commit)
- [ ] Ensure tests pass: `make test-backend`

### Migration Steps

- [ ] **Step 1**: Update middleware signature (5 min)
  - Add `State(state): State<AppState>` parameter
  - Remove conditional state access
  - Delete `detect_locale` function

- [ ] **Step 2**: Update router registration (2 min)
  - Use `middleware::from_fn_with_state`
  - Pass `state.clone()`

- [ ] **Step 3**: Add integration tests (10 min)
  - Test user DB preference
  - Test priority order
  - Test fallback behavior

- [ ] **Step 4**: Run tests (5 min)
  - `cargo test --package ampel-api --lib middleware::locale::tests`
  - `make test-backend`

- [ ] **Step 5**: Manual testing (5 min)
  - Start API: `make dev-api`
  - Register user and set language
  - Test with curl/Postman

- [ ] **Step 6**: Update documentation (5 min)
  - Update developer guide
  - Add migration notes

### Post-Migration

- [ ] Commit changes with descriptive message
- [ ] Create pull request
- [ ] Code review
- [ ] Merge to feature branch
- [ ] Deploy to staging
- [ ] Monitor metrics
- [ ] Deploy to production

---

## Rollback Procedure

If issues arise during migration, rollback is simple:

### Option 1: Revert to Non-DB Detection (Fast)

```rust
// middleware/locale.rs
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = detect_locale(&req);  // Simple detection (no DB)
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// routes/mod.rs
Router::new()
    .layer(middleware::from_fn(locale_detection_middleware))
    .with_state(state)
```

**Impact**: Disables user preference detection, but keeps query/cookie/header working.

### Option 2: Git Revert (Safest)

```bash
# Revert to previous commit
git revert HEAD

# Or reset to specific commit
git reset --hard <commit-hash>
```

**Impact**: Complete rollback to previous working state.

---

## Performance Benchmarks

### Expected Latency

| Scenario                   | Latency | Notes                      |
| -------------------------- | ------- | -------------------------- |
| No auth token              | < 1ms   | Skip DB, use cookie/header |
| Authenticated (Redis hit)  | < 2ms   | Redis cache lookup         |
| Authenticated (DB hit)     | < 10ms  | PostgreSQL query (indexed) |
| Authenticated (cold start) | < 20ms  | DB query + cache write     |

### Optimization Targets

- [ ] Add Redis caching for user language (if needed)
- [ ] Add connection pool monitoring
- [ ] Add query performance metrics
- [ ] Set up alerting for slow queries (> 50ms)

---

## Common Issues & Solutions

### Issue 1: Compilation Error - "State not in scope"

```
error[E0433]: failed to resolve: use of undeclared type `State`
```

**Solution**: Add import

```rust
use axum::extract::State;
```

### Issue 2: Middleware Not Applying

```rust
// ❌ Wrong order
Router::new()
    .with_state(state)
    .layer(middleware::from_fn_with_state(...))  // Too late!

// ✅ Correct order
Router::new()
    .layer(middleware::from_fn_with_state(...))
    .with_state(state)
```

### Issue 3: State Moved Error

```
error[E0382]: use of moved value: `state`
```

**Solution**: Clone state (Arc clone is cheap)

```rust
.layer(middleware::from_fn_with_state(state.clone(), ...))
.with_state(state)  // Original state still available
```

### Issue 4: Tests Failing with "Database Not Found"

**Solution**: Use in-memory SQLite for tests

```rust
let db = Database::connect("sqlite::memory:").await.unwrap();
```

---

## Success Criteria

### Functional Requirements

- [ ] Middleware compiles without errors
- [ ] Query param detection works (`?lang=fi`)
- [ ] User DB preference detection works (with JWT)
- [ ] Cookie detection works (`lang=fi`)
- [ ] Accept-Language header detection works
- [ ] Default fallback works (`en`)
- [ ] Priority order is correct (query > DB > cookie > header > default)

### Non-Functional Requirements

- [ ] All tests pass (unit + integration)
- [ ] No performance regression (< 10ms added latency)
- [ ] No memory leaks (valgrind/sanitizers)
- [ ] Code coverage maintained (> 80%)
- [ ] Documentation updated
- [ ] No breaking changes to API

---

## Next Steps

1. **Review**: Team review of architecture and implementation
2. **Implement**: Follow migration checklist (30 minutes)
3. **Test**: Run full test suite + manual testing
4. **Deploy**: Merge to feature branch → staging → production
5. **Monitor**: Watch metrics for 24 hours
6. **Document**: Update developer guide with lessons learned

---

## Additional Resources

- [Main Architecture Document](./LOCALE_MIDDLEWARE_DESIGN.md)
- [Flow Diagrams](./locale-middleware-flow.md)
- [Axum Middleware Guide](https://docs.rs/axum/0.7/axum/middleware/)
- [Tower Service Trait](https://docs.rs/tower/latest/tower/trait.Service.html)
- [Ampel Localization Spec](../localization/SPECIFICATION.md)

---

**Author**: System Architecture Designer
**Last Updated**: 2026-01-09
**Status**: Ready for Implementation
