# Axum 0.7 Middleware State Access Research

**Date**: 2026-01-09
**Context**: Fixing locale_detection_middleware compilation error in ampel-api
**Axum Version**: 0.7.9
**Axum-Extra Version**: 0.9.6

## Executive Summary

The current implementation has the correct middleware signature but is using the wrong layer attachment method. The middleware function uses `State<AppState>` extractor correctly, but the router is attempting to use `.layer(middleware::from_fn(...))` instead of `.layer(middleware::from_fn_with_state(...))`.

**Root Cause**: Using `from_fn` instead of `from_fn_with_state` when attaching middleware that needs state access.

**Solution**: Replace `middleware::from_fn(locale_detection_middleware)` with `middleware::from_fn_with_state(state.clone(), locale_detection_middleware)`.

---

## Official Documentation

### 1. `from_fn_with_state` Documentation

**Source**: [axum::middleware::from_fn_with_state](https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html)

Creates a middleware from an async function with the given state. The `from_fn` function doesn't support extracting `State` - for that, you need to use `from_fn_with_state`.

**Official Example**:

```rust
use axum::{
    Router,
    http::StatusCode,
    routing::get,
    response::{IntoResponse, Response},
    middleware::{self, Next},
    extract::{Request, State},
};

#[derive(Clone)]
struct AppState { /* ... */ }

async fn my_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // do something with `request`...
    let response = next.run(request).await;
    // do something with `response`...
    response
}

let state = AppState { /* ... */ };
let app = Router::new()
    .route("/", get(|| async { /* ... */ }))
    .layer(middleware::from_fn_with_state(state, my_middleware));
```

### 2. State Extractor Documentation

**Source**: [axum::extract::State](https://docs.rs/axum/latest/axum/extract/struct.State.html)

The preferred approach for sharing application state. More type-safe than Extensions. State is global within a router and set using `.with_state()`, but **middleware cannot access state set by `.with_state()`** - you must pass it directly to `from_fn_with_state()`.

---

## Key Differences: `from_fn` vs `from_fn_with_state`

**Source**: [Axum Middleware Documentation](https://docs.rs/axum/latest/axum/middleware/index.html)

| Feature          | `from_fn`                       | `from_fn_with_state`                        |
| ---------------- | ------------------------------- | ------------------------------------------- |
| **State Access** | ❌ Cannot extract `State<T>`    | ✅ Can extract `State<T>`                   |
| **Signature**    | Takes only middleware function  | Takes state first, then middleware function |
| **Use Case**     | Simple middleware without state | Middleware needing database, config, etc.   |
| **Introduced**   | Original API                    | Added in axum 0.6.0                         |

**Important**: The `from_fn` function doesn't support extracting State. For that, you need to use `from_fn_with_state`.

---

## `layer` vs `route_layer`

**Source**: [Axum Router Documentation](https://docs.rs/axum/latest/axum/routing/struct.Router.html)

### `Router::layer`

- Applies middleware to **all routes added before** calling `.layer()`
- Runs **after routing** (cannot rewrite request URI)
- Additional routes added after `.layer()` will **not** have the middleware

### `Router::route_layer`

- Middleware only runs **if the request matches a route**
- Useful for authorization middleware that might return early
- With `route_layer`: 404 responses remain 404 even with invalid auth
- With `layer`: 404 might become 401 if auth fails

**Best Practice**: Use `route_layer` for authorization/authentication middleware. Use `layer` for logging, metrics, CORS.

**Source**: [GitHub Discussion #2878](https://github.com/tokio-rs/axum/discussions/2878)

---

## State vs Extensions Pattern

**Source**: [GitHub Discussion #1830](https://github.com/tokio-rs/axum/discussions/1830)

### State Pattern (Preferred)

✅ **Advantages**:

- Type-safe at compile time
- Better performance
- Explicit in handler signatures

❌ **Disadvantages**:

- Less dynamic
- Must be set before routes
- Cannot be modified per-request

### Extensions Pattern

✅ **Advantages**:

- Request-specific data
- Dynamic insertion in middleware
- Flexible for per-request state

❌ **Disadvantages**:

- Runtime errors if extension missing (500 Internal Server Error)
- Less type-safe
- Easier to forget to add

**Recommendation**:

- Use **State** for global application state (database pools, config, services)
- Use **Extensions** for request-specific data (authenticated user, detected locale)

---

## Database Connection Pool Pattern

**Source**: [GitHub Discussion #2819](https://github.com/tokio-rs/axum/discussions/2819)

### AppState with Database Pool

```rust
#[derive(Clone)]
struct AppState {
    pool: PgPool,  // sqlx::PgPool is cheap to clone (Arc internally)
}

let state = AppState { pool };

let app = Router::new()
    .route("/", get(handler))
    .with_state(state.clone())
    .layer(middleware::from_fn_with_state(state.clone(), my_middleware));
```

**Key Points**:

- Database pools like `sqlx::PgPool` are cheap to clone (internally use `Arc`)
- `AppState` must derive `Clone`
- You can clone state for both `.with_state()` and `from_fn_with_state()`
- No need to wrap the entire `AppState` in `Arc` unless you have non-cloneable fields

**Source**: [Leapcell State Management Guide](https://leapcell.io/blog/robust-state-management-in-actix-web-and-axum-applications)

---

## Common Middleware Errors and Solutions

**Source**: [GitHub Discussion #2664](https://github.com/tokio-rs/axum/discussions/2664), [Discussion #2766](https://github.com/tokio-rs/axum/discussions/2766)

### Error 1: FromFn Trait Not Satisfied

**Cause**: Using `from_fn` when middleware extracts `State<T>`
**Solution**: Use `from_fn_with_state` instead

### Error 2: Body is !Sync

**Cause**: Passing `&Request<Body>` to async helper functions
**Solution**: Pass request by value or extract specific parts before async calls

**From Discussion #2571**: `Request<Body>` is `!Sync` because Body is `!Sync`. That means `&Request<Body>` is `!Send`, making the future returned by middleware also `!Send`.

### Error 3: Request Borrowing Issues

**Cause**: Trying to pass `&Request` references across `.await` points
**Solution**: Use `request.into_parts()` or extract data before async operations

---

## Locale Detection Middleware Examples

### 1. axum_l10n Crate

**Source**: [GitHub - tronicboy1/axum_l10n](https://github.com/tronicboy1/axum_l10n)

Ready-to-use locale detection middleware:

```rust
use axum_l10n::LanguageIdentifierExtractorLayer;
use unic_langid::{langid, LanguageIdentifier};

const ENGLISH: LanguageIdentifier = langid!("en");
const JAPANESE: LanguageIdentifier = langid!("ja");

let router = Router::new()
    .route("/", get(handler))
    .layer(LanguageIdentifierExtractorLayer::new(
        ENGLISH,  // default locale
        vec![ENGLISH, JAPANESE],  // supported locales
        RedirectMode::NoRedirect,
    ));
```

**Detection Order**: Accept-Language header → Language identifier in Extension

### 2. Custom Implementation Pattern

**Source**: [Yieldcode Blog - Web App Localization](https://yieldcode.blog/post/webapp-localization-in-rust/)

```rust
async fn locale_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Accept-Language header
    let locale = request
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| parse_accept_language(s))
        .unwrap_or_else(|| "en".to_string());

    // Store in extensions for handler access
    request.extensions_mut().insert(DetectedLocale { code: locale });

    next.run(request).await
}
```

---

## Recommended Solution for Ampel

### Current Implementation Analysis

**File**: `crates/ampel-api/src/middleware/locale.rs`

✅ **Correct**:

- Middleware signature with `State<AppState>` extractor
- Proper request handling
- Database access for user preferences
- Extension insertion for downstream handlers

❌ **Incorrect** (in `routes/mod.rs`):

```rust
// Line 152 - WRONG:
.layer(middleware::from_fn(locale_detection_middleware))
```

### Fix Required

**File**: `crates/ampel-api/src/routes/mod.rs`

```rust
// Replace this:
.layer(middleware::from_fn(locale_detection_middleware))

// With this:
.layer(middleware::from_fn_with_state(
    state.clone(),
    locale_detection_middleware
))
```

### Complete Working Pattern

```rust
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ... routes ...
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            locale_detection_middleware
        ))
        .layer(middleware::from_fn(track_metrics))
}
```

**Key Points**:

1. Clone `state` for both `.with_state()` and `from_fn_with_state()`
2. `AppState` already derives `Clone`, so this is cheap
3. Database connections (SeaORM) are Arc-based internally
4. Place middleware in correct order (locale detection before metrics)

---

## Best Practices Summary

### ✅ DO:

- Use `from_fn_with_state` for middleware that needs database/state access
- Use `State<T>` extractor in middleware signature
- Clone state when passing to both `.with_state()` and middleware
- Derive `Clone` on `AppState`
- Use `route_layer` for auth middleware
- Use `Extensions` for per-request data
- Validate state before starting the server

### ❌ DON'T:

- Use `from_fn` when middleware extracts `State<T>`
- Pass `&Request<Body>` to async helper functions
- Wrap entire `AppState` in additional `Arc` unless necessary
- Use `.layer()` for authentication middleware (use `.route_layer()`)
- Rely on Extensions for global state (use State instead)
- Add generic type parameters to middleware functions

---

## Migration Checklist

- [x] ✅ Middleware signature uses `State<AppState>` - Already correct
- [x] ✅ `AppState` derives `Clone` - Already correct
- [ ] ⚠️ Router uses `from_fn_with_state` instead of `from_fn` - **NEEDS FIX**
- [x] ✅ State cloned for middleware - **WILL BE FIXED**
- [x] ✅ Extensions used for per-request data (DetectedLocale) - Already correct
- [x] ✅ No `&Request` passed to async functions - Already correct

---

## Axum 0.7 Middleware Architecture

```
Router Creation
    ↓
.route("/api/...", handlers)  ← Define routes
    ↓
.with_state(state.clone())    ← Set global state for handlers
    ↓
.layer(from_fn_with_state(    ← Add state-aware middleware
    state.clone(),
    locale_detection_middleware
))
    ↓
.layer(from_fn(               ← Add stateless middleware
    track_metrics
))
    ↓
Server Start
```

**Execution Order**: Layers run in **reverse** order (bottom to top)

1. Request arrives
2. `track_metrics` runs first (last layer added)
3. `locale_detection_middleware` runs second (first layer added)
4. Route handler runs

---

## References

### Official Documentation

- [axum::middleware::from_fn_with_state](https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html)
- [axum::extract::State](https://docs.rs/axum/latest/axum/extract/struct.State.html)
- [axum::middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
- [axum::Router](https://docs.rs/axum/latest/axum/routing/struct.Router.html)

### GitHub Discussions

- [How do I access state in middleware? #1912](https://github.com/tokio-rs/axum/discussions/1912)
- [Trait bound error with from_fn_with_state #2664](https://github.com/tokio-rs/axum/discussions/2664)
- [State vs Extensions advantages #1830](https://github.com/tokio-rs/axum/discussions/1830)
- [layer vs route_layer difference #2878](https://github.com/tokio-rs/axum/discussions/2878)
- [Database connection pooling #2508](https://github.com/tokio-rs/axum/discussions/2508)

### Community Resources

- [Medium: Axum Middleware State Access](https://medium.com/@mikecode/axum-29-in-middleware-access-to-share-state-fca81a985b4d)
- [Leapcell: State Management in Axum](https://leapcell.io/blog/robust-state-management-in-actix-web-and-axum-applications)
- [Yieldcode: Web App Localization in Rust](https://yieldcode.blog/post/webapp-localization-in-rust/)
- [Shuttle: Axum Production Guide](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust)

### Related Crates

- [axum_l10n](https://github.com/tronicboy1/axum_l10n) - Locale detection utilities for Axum

---

## Conclusion

The locale detection middleware implementation is **architecturally correct** but uses the **wrong layer attachment method**. The fix is straightforward:

1. Change `from_fn(locale_detection_middleware)` to `from_fn_with_state(state.clone(), locale_detection_middleware)` in `routes/mod.rs`
2. Uncomment the middleware layer (line 152)
3. Uncomment the import (line 15)

This follows the official Axum 0.7 pattern for middleware with state access and matches the documented best practices from the Axum maintainers.
