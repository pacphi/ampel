# Locale Detection Middleware Architecture Design

**Date**: 2026-01-09
**Status**: PROPOSED
**Axum Version**: 0.7.x
**Context**: Database-aware locale detection middleware needs proper state access pattern

## Problem Statement

The current locale detection middleware (`locale_detection_middleware`) needs database access to read user language preferences, but is using an incompatible pattern for Axum 0.7. The middleware attempts to access `AppState` from request extensions (line 47), which doesn't work with `middleware::from_fn` in Axum 0.7.

```rust
// Current broken pattern
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = if let Some(state) = req.extensions().get::<AppState>() {  // ❌ Returns None
        detect_locale_with_state(&req, state).await
    } else {
        detect_locale(&req)  // Always falls back to this
    };
    // ...
}
```

## Requirements

1. **Database Access**: Must query `user::Entity` to fetch user language preference
2. **JWT Validation**: Must validate JWT tokens using `state.auth_service`
3. **Priority Detection**: Query param → User DB → Cookie → Accept-Language → "en"
4. **Performance**: Minimal overhead, async-friendly
5. **Type Safety**: Compile-time guarantees, no runtime panics
6. **Backward Compatibility**: Minimal changes to existing code
7. **Testing**: Easy to test with mocked state

## Solution Evaluation

### Option A: Request Extensions (Current - BROKEN)

**Pattern**:

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

**Registration**:

```rust
Router::new()
    .with_state(state)
    .layer(middleware::from_fn(locale_detection_middleware))
```

**Issues**:

- ❌ State not available in extensions with `from_fn` in Axum 0.7
- ❌ Always falls back to non-DB detection
- ❌ Unreliable pattern, depends on undocumented behavior

**Verdict**: REJECTED - doesn't work in Axum 0.7

---

### Option B: `from_fn_with_state` (RECOMMENDED)

**Pattern**:

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

**Registration**:

```rust
Router::new()
    .layer(middleware::from_fn_with_state(
        state.clone(),
        locale_detection_middleware,
    ))
    .with_state(state)
```

**Pros**:

- ✅ **Official Axum 0.7 pattern** for middleware with state
- ✅ **Type-safe** - compile error if signature is wrong
- ✅ **Explicit** - state dependency is clear in signature
- ✅ **No fallback needed** - always has state access
- ✅ **Simple migration** - minimal code changes
- ✅ **Testable** - easy to mock `AppState` in tests

**Cons**:

- ⚠️ Requires `state.clone()` (cheap Arc clones)
- ⚠️ Different pattern than `from_fn` (but more explicit)

**Performance**:

- Arc clones are O(1) - just reference counting
- No additional allocations or async overhead
- Same performance as extractors

**Verdict**: RECOMMENDED - official, type-safe, explicit

---

### Option C: Custom Middleware Layer

**Pattern**:

```rust
pub struct LocaleLayer {
    state: AppState,
}

impl<S> Layer<S> for LocaleLayer {
    type Service = LocaleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LocaleService {
            inner,
            state: self.state.clone(),
        }
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
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            let locale = detect_locale_with_state(&req, &state).await;
            req.extensions_mut().insert(DetectedLocale::new(locale));
            future.await
        })
    }
}
```

**Registration**:

```rust
Router::new()
    .layer(LocaleLayer { state: state.clone() })
    .with_state(state)
```

**Pros**:

- ✅ Maximum control over middleware lifecycle
- ✅ Can implement complex coordination logic
- ✅ Follows Tower's Service trait pattern

**Cons**:

- ❌ **80+ lines of boilerplate** vs 5 lines for Option B
- ❌ Complex trait implementations (Service, Layer, Future)
- ❌ Harder to test and maintain
- ❌ More opportunity for bugs (borrow checker, lifetimes)
- ❌ Overkill for simple state access

**Verdict**: REJECTED - unnecessary complexity for this use case

---

### Option D: Extractor Pattern

**Pattern**:

```rust
pub struct DetectedLocale {
    pub code: String,
}

#[async_trait]
impl FromRequestParts<AppState> for DetectedLocale {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let req = Request::from_parts(parts.clone(), Body::empty());
        let locale = detect_locale_with_state(&req, state).await;
        Ok(DetectedLocale::new(locale))
    }
}

// Usage in handlers
pub async fn handler(
    locale: DetectedLocale,  // Extracted automatically
    State(state): State<AppState>,
) -> Response {
    // Use locale.code
}
```

**Pros**:

- ✅ Clean handler signatures
- ✅ Per-handler extraction (only when needed)
- ✅ Type-safe extraction

**Cons**:

- ❌ **Not middleware** - must add to every handler signature
- ❌ Detection happens per-handler, not once per request
- ❌ Can't share detection result across middleware pipeline
- ❌ Breaks existing handlers that don't declare the extractor
- ❌ Requires modifying 40+ handler functions

**Verdict**: REJECTED - not appropriate for cross-cutting concerns

---

## Recommended Solution: Option B (`from_fn_with_state`)

### Implementation Plan

#### 1. Update Middleware Signature

**File**: `crates/ampel-api/src/middleware/locale.rs`

```rust
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

/// Middleware to detect and set locale with database access
///
/// Detection order:
/// 1. Query parameter (?lang=fi)
/// 2. User database preference (if authenticated)
/// 3. Cookie (lang=fi)
/// 4. Accept-Language header
/// 5. Fallback to "en"
pub async fn locale_detection_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let locale = detect_locale_with_state(&req, &state).await;
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// Remove the old locale_detection_middleware function
// Keep detect_locale_with_state as-is (already correct)
// Remove detect_locale (no longer needed - we always have state)
```

**Changes**:

- Add `State(state): State<AppState>` parameter
- Remove conditional fallback logic (always use DB-aware detection)
- Remove `detect_locale` function (no longer needed)

#### 2. Update Router Registration

**File**: `crates/ampel-api/src/routes/mod.rs`

```rust
use crate::middleware::locale_detection_middleware;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ... all routes ...
        .layer(middleware::from_fn_with_state(
            state.clone(),
            locale_detection_middleware,
        ))
        .layer(middleware::from_fn(track_metrics))
        .with_state(state)
}
```

**Key Points**:

- `from_fn_with_state` requires `state.clone()` (cheap Arc clone)
- Place **before** `.with_state(state)` to consume state once
- Order: locale → metrics (locale runs first, metrics wraps it)

#### 3. Update Tests

**File**: `crates/ampel-api/src/middleware/locale.rs`

Add integration tests with state:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::services::AuthService;
    use ampel_db::encryption::EncryptionService;
    use sea_orm::Database;
    use std::sync::Arc;

    async fn create_test_state() -> AppState {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let auth_service = AuthService::new("test_secret".to_string());
        let encryption_service = EncryptionService::new(vec![0u8; 32]);

        AppState {
            db,
            redis: None,
            auth_service: Arc::new(auth_service),
            encryption_service: Arc::new(encryption_service),
            provider_factory: Arc::new(ProviderFactory::new()),
            config: Arc::new(Config::default()),
            metrics_handle: PrometheusHandle::default(),
        }
    }

    #[tokio::test]
    async fn test_locale_detection_with_user_preference() {
        let state = create_test_state().await;

        // Create test user with language preference
        let user = user::ActiveModel {
            email: Set("test@example.com".to_string()),
            language: Set(Some("fi".to_string())),
            ..Default::default()
        };
        let user = user.insert(&state.db).await.unwrap();

        // Create JWT token
        let token = state.auth_service.create_access_token(user.id).unwrap();

        // Build request with Authorization header
        let req = Request::builder()
            .uri("https://example.com/api")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let locale = detect_locale_with_state(&req, &state).await;
        assert_eq!(locale, "fi");
    }

    #[tokio::test]
    async fn test_locale_detection_priority() {
        let state = create_test_state().await;

        // Query param should override everything
        let req = Request::builder()
            .uri("https://example.com/api?lang=de")
            .header(header::COOKIE, "lang=fi")
            .header(header::ACCEPT_LANGUAGE, "fr")
            .body(Body::empty())
            .unwrap();

        let locale = detect_locale_with_state(&req, &state).await;
        assert_eq!(locale, "de");
    }

    // Keep existing unit tests for helper functions
    // (normalize_locale, is_supported_locale, parse_accept_language, etc.)
}
```

#### 4. Documentation Updates

**File**: `docs/localization/DEVELOPER-GUIDE.md`

Add section on middleware architecture:

````markdown
## Locale Detection Middleware

### Architecture

The locale detection middleware uses Axum 0.7's `from_fn_with_state` pattern
to access the database and validate user preferences.

**Detection Priority**:

1. Query parameter: `?lang=fi`
2. User database preference (requires authentication)
3. Cookie: `lang=fi`
4. Accept-Language header
5. Fallback: `en`

### Adding Locale-Aware Handlers

```rust
use crate::middleware::DetectedLocale;

pub async fn my_handler(
    Extension(locale): Extension<DetectedLocale>,
    State(state): State<AppState>,
) -> Response {
    // Use locale.code for language-specific logic
}
```
````

### Testing

```bash
# Test with query parameter
curl "http://localhost:8080/api/endpoint?lang=fi"

# Test with cookie
curl -H "Cookie: lang=de" http://localhost:8080/api/endpoint

# Test with Accept-Language
curl -H "Accept-Language: fr-FR,fr;q=0.9" http://localhost:8080/api/endpoint

# Test with JWT (requires valid token)
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/endpoint
```

````

### Migration Steps

#### Step 1: Update Middleware (5 min)

```bash
# Edit locale.rs middleware signature
vim crates/ampel-api/src/middleware/locale.rs
````

Changes:

- Add `State(state): State<AppState>` parameter
- Remove conditional state access logic
- Remove `detect_locale` function

#### Step 2: Update Router (2 min)

```bash
# Edit routes/mod.rs
vim crates/ampel-api/src/routes/mod.rs
```

Changes:

- Replace `middleware::from_fn` with `middleware::from_fn_with_state`
- Pass `state.clone()` to middleware

#### Step 3: Update Tests (10 min)

```bash
# Add integration tests with state
vim crates/ampel-api/src/middleware/locale.rs
```

Add:

- `create_test_state()` helper
- `test_locale_detection_with_user_preference()`
- `test_locale_detection_priority()`

#### Step 4: Run Tests (5 min)

```bash
# Test backend
make test-backend

# Test specific middleware
cargo test --package ampel-api --lib middleware::locale::tests
```

#### Step 5: Update Documentation (5 min)

```bash
# Document architecture and usage
vim docs/localization/DEVELOPER-GUIDE.md
```

**Total Migration Time**: ~30 minutes

### Rollback Plan

If issues arise, rollback is simple:

```rust
// Revert to non-DB detection (temporary)
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = detect_locale(&req);  // No DB access
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// Revert router registration
Router::new()
    .layer(middleware::from_fn(locale_detection_middleware))
    .with_state(state)
```

This disables user preference detection but keeps query/cookie/header detection working.

---

## Testing Strategy

### Unit Tests (Existing - Keep)

- `test_normalize_locale()` - locale code normalization
- `test_is_supported_locale()` - locale validation
- `test_parse_accept_language()` - header parsing
- `test_extract_query_param()` - query string parsing
- `test_locale_detection_*` - detection priority (without state)

### Integration Tests (New - Add)

```rust
#[tokio::test]
async fn test_middleware_with_authenticated_user() {
    // Create test state with in-memory database
    // Insert user with language preference
    // Create valid JWT token
    // Call middleware
    // Assert locale matches user preference
}

#[tokio::test]
async fn test_middleware_with_invalid_token() {
    // Create test state
    // Call middleware with invalid token
    // Assert falls back to cookie/header/default
}

#[tokio::test]
async fn test_middleware_priority_order() {
    // Test all detection sources simultaneously
    // Assert query param wins
}

#[tokio::test]
async fn test_middleware_performance() {
    // Measure detection time with DB access
    // Assert < 5ms for cached user lookup
}
```

### End-to-End Tests (Manual)

```bash
# 1. Start API server
make dev-api

# 2. Register user and set language preference
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}'

curl -X PUT http://localhost:8080/api/v1/user/preferences/language \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"language":"fi"}'

# 3. Test locale detection
curl -v http://localhost:8080/api/dashboard/summary \
  -H "Authorization: Bearer $TOKEN"

# 4. Check response headers or logs for locale
# Should see: DetectedLocale { code: "fi" }
```

---

## Performance Considerations

### Database Query Impact

- **User lookup**: O(1) with primary key index
- **Caching**: Redis cache for user language (if available)
- **Fallback**: No DB query if unauthenticated
- **Expected latency**: < 5ms for cached, < 20ms for DB hit

### Optimization Opportunities

1. **Redis Cache** (Optional - Future Enhancement):

```rust
async fn detect_locale_with_state(req: &Request<Body>, state: &AppState) -> String {
    // Try Redis cache first
    if let Some(redis) = &state.redis {
        if let Some(user_id) = try_extract_user_from_jwt(req, state) {
            let cache_key = format!("user:{}:language", user_id);
            if let Ok(Some(lang)) = redis.get::<String>(&cache_key).await {
                return lang;
            }
        }
    }

    // Fall back to DB query...
}
```

2. **In-Memory Cache** (Simpler Alternative):

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

pub struct LanguageCache {
    cache: Arc<RwLock<LruCache<Uuid, String>>>,
}

impl LanguageCache {
    pub async fn get_or_fetch(&self, user_id: Uuid, db: &DatabaseConnection) -> Option<String> {
        // Check cache
        if let Some(lang) = self.cache.read().await.peek(&user_id) {
            return Some(lang.clone());
        }

        // Fetch from DB
        if let Ok(Some(user)) = user::Entity::find_by_id(user_id).one(db).await {
            if let Some(lang) = user.language {
                self.cache.write().await.put(user_id, lang.clone());
                return Some(lang);
            }
        }

        None
    }
}
```

**Recommendation**: Start without caching. Add Redis cache if profiling shows DB as bottleneck (unlikely for language preference lookups).

---

## Trade-offs Summary

| Aspect                 | Option A (Extensions)    | Option B (from_fn_with_state) | Option C (Custom Layer) | Option D (Extractor) |
| ---------------------- | ------------------------ | ----------------------------- | ----------------------- | -------------------- |
| **Complexity**         | Low (but broken)         | Low                           | High                    | Medium               |
| **Boilerplate**        | 5 lines                  | 5 lines                       | 80+ lines               | 10 lines per handler |
| **Type Safety**        | ❌ Runtime failure       | ✅ Compile-time               | ✅ Compile-time         | ✅ Compile-time      |
| **Testability**        | Hard (needs real router) | Easy (mock state)             | Hard (Tower traits)     | Easy (mock state)    |
| **Maintainability**    | ❌ Unreliable            | ✅ Clear pattern              | ❌ Complex traits       | ❌ Repetitive        |
| **Performance**        | N/A (broken)             | Excellent                     | Excellent               | Good (per-handler)   |
| **Axum Compatibility** | ❌ Broken in 0.7         | ✅ Official pattern           | ✅ Works                | ✅ Works             |
| **Migration Effort**   | N/A                      | 5 minutes                     | 2 hours                 | 4+ hours             |

**Winner**: Option B (`from_fn_with_state`) - official, simple, type-safe, easy to migrate

---

## Architecture Decision Record (ADR)

### ADR-001: Use `from_fn_with_state` for Locale Detection Middleware

**Status**: Proposed
**Date**: 2026-01-09
**Deciders**: System Architecture Team

#### Context

Locale detection middleware needs database access to read user language preferences. Axum 0.7 requires explicit state passing for middleware.

#### Decision

Use `middleware::from_fn_with_state` with explicit `State<AppState>` parameter.

#### Rationale

- **Official Pattern**: Recommended by Axum documentation
- **Type Safety**: Compile-time guarantee of state availability
- **Simplicity**: Minimal code changes (5 lines)
- **Performance**: Arc clones are O(1), no overhead
- **Testability**: Easy to mock state in unit tests

#### Consequences

**Positive**:

- Type-safe state access in middleware
- No runtime surprises (compile errors if wrong)
- Clear dependency on AppState in signature
- Easy to test with mock state

**Negative**:

- Requires `state.clone()` in router (cheap Arc clone)
- Different pattern than `from_fn` (but more explicit)

#### Alternatives Considered

- **Request Extensions**: Broken in Axum 0.7
- **Custom Layer**: Overkill (80+ lines of boilerplate)
- **Extractor Pattern**: Not middleware, requires modifying all handlers

---

## Appendix: Code Comparison

### Before (Broken)

```rust
// middleware/locale.rs
pub async fn locale_detection_middleware(mut req: Request<Body>, next: Next) -> Response {
    let locale = if let Some(state) = req.extensions().get::<AppState>() {
        detect_locale_with_state(&req, state).await
    } else {
        detect_locale(&req)  // Always falls back to this
    };
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// routes/mod.rs
Router::new()
    .with_state(state)
    .layer(middleware::from_fn(locale_detection_middleware))  // ❌ No state access
```

### After (Fixed)

```rust
// middleware/locale.rs
pub async fn locale_detection_middleware(
    State(state): State<AppState>,  // ✅ Explicit state parameter
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let locale = detect_locale_with_state(&req, &state).await;  // ✅ Always has state
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

// routes/mod.rs
Router::new()
    .layer(middleware::from_fn_with_state(
        state.clone(),  // ✅ Pass state to middleware
        locale_detection_middleware,
    ))
    .with_state(state)
```

**Key Differences**:

- Add `State(state): State<AppState>` parameter
- Remove conditional state access (always available)
- Use `from_fn_with_state` in router registration
- Pass `state.clone()` (cheap Arc clone)

---

## References

- [Axum 0.7 Middleware Guide](https://docs.rs/axum/0.7/axum/middleware/index.html)
- [Axum State Sharing](https://docs.rs/axum/0.7/axum/extract/struct.State.html)
- [Tower Middleware](https://docs.rs/tower/latest/tower/trait.Service.html)
- [Ampel Localization Specification](./localization/SPECIFICATION.md)

---

## Next Steps

1. **Review**: Team review of this architecture document
2. **Implement**: Apply changes per migration plan (30 minutes)
3. **Test**: Run integration tests with real database
4. **Deploy**: Merge to feature branch, test in staging
5. **Monitor**: Check performance metrics after deployment
6. **Document**: Update developer guide with new pattern

---

**Author**: System Architecture Designer (Claude Code Agent)
**Review Required**: Yes
**Estimated Implementation Time**: 30 minutes
**Risk Level**: Low (simple, well-tested pattern)
