# Locale Detection Middleware Flow Diagrams

## Current Architecture (Broken - Axum 0.7)

```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant Middleware as locale_detection_middleware
    participant Extensions as Request Extensions
    participant Handler

    Client->>Router: HTTP Request
    Router->>Router: with_state(state)
    Router->>Middleware: from_fn(middleware)
    Note over Middleware: No state parameter!

    Middleware->>Extensions: req.extensions().get::<AppState>()
    Extensions-->>Middleware: None ❌
    Note over Middleware: Falls back to detect_locale()<br/>(no DB access)

    Middleware->>Middleware: detect_locale(req)
    Note over Middleware: Only checks:<br/>1. Query param<br/>2. Cookie<br/>3. Accept-Language

    Middleware->>Extensions: insert(DetectedLocale)
    Middleware->>Handler: next.run(req)
    Handler-->>Client: Response

    Note over Handler: User preference ignored!<br/>Always uses cookie/header fallback
```

## Proposed Architecture (Fixed - from_fn_with_state)

```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant Middleware as locale_detection_middleware
    participant DB as PostgreSQL
    participant AuthService
    participant Handler

    Client->>Router: HTTP Request<br/>Authorization: Bearer token
    Router->>Middleware: from_fn_with_state(state, middleware)
    Note over Middleware: State(state): State<AppState> ✅

    Middleware->>Middleware: Extract JWT token from headers
    Middleware->>AuthService: validate_access_token(token)
    AuthService-->>Middleware: Claims { user_id }

    Middleware->>DB: SELECT language FROM users<br/>WHERE id = user_id
    DB-->>Middleware: "fi"

    Note over Middleware: Locale detection priority:<br/>1. Query param (?lang=de)<br/>2. User DB preference (fi) ✅<br/>3. Cookie (lang=en)<br/>4. Accept-Language<br/>5. Fallback (en)

    Middleware->>Middleware: insert(DetectedLocale { code: "fi" })
    Middleware->>Handler: next.run(req)
    Handler->>Handler: Extract locale from extensions
    Handler-->>Client: Response (localized content)

    Note over Client: Content in Finnish (fi) ✅
```

## Component Interaction (C4 - Container Level)

```mermaid
graph TB
    subgraph "Axum Web Server"
        Router[Router<br/>create_router]
        LocaleMiddleware[Locale Detection Middleware<br/>locale_detection_middleware]
        MetricsMiddleware[Metrics Middleware<br/>track_metrics]
        Handlers[API Handlers<br/>accounts, auth, dashboard, etc.]
    end

    subgraph "Application State"
        AppState[AppState<br/>- DatabaseConnection<br/>- AuthService<br/>- Redis<br/>- Config]
        AuthService[AuthService<br/>JWT validation]
    end

    subgraph "Data Stores"
        PostgreSQL[(PostgreSQL<br/>users table)]
        Redis[(Redis Cache<br/>optional)]
    end

    Client[HTTP Client]

    Client -->|1. HTTP Request| Router
    Router -->|2. from_fn_with_state| LocaleMiddleware
    Router -->|3. Inject state| AppState
    LocaleMiddleware -->|4. Validate JWT| AuthService
    LocaleMiddleware -->|5. Query language| PostgreSQL
    LocaleMiddleware -->|6. Optional cache| Redis
    LocaleMiddleware -->|7. Insert DetectedLocale| Handlers
    Router -->|8. Wrap response| MetricsMiddleware
    Handlers -->|9. Response| Client

    style LocaleMiddleware fill:#4CAF50,stroke:#333,stroke-width:3px,color:#fff
    style AppState fill:#2196F3,stroke:#333,stroke-width:2px,color:#fff
    style PostgreSQL fill:#FFA726,stroke:#333,stroke-width:2px,color:#fff
```

## Middleware Execution Order

```mermaid
flowchart LR
    Request[HTTP Request] --> Router

    Router --> LocaleMiddleware
    LocaleMiddleware -->|"✅ Detect locale<br/>Insert DetectedLocale"| MetricsMiddleware

    MetricsMiddleware -->|"✅ Record metrics<br/>Track latency"| Handler

    Handler -->|"✅ Use locale<br/>Extension(DetectedLocale)"| Response

    Response --> Client[HTTP Response]

    style LocaleMiddleware fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
    style MetricsMiddleware fill:#2196F3,stroke:#333,stroke-width:2px,color:#fff
    style Handler fill:#FF9800,stroke:#333,stroke-width:2px,color:#fff
```

**Key Points**:

- Middleware executes in **registration order** (first registered = outermost layer)
- Locale detection runs **before** metrics (so metrics can record locale)
- Both middleware layers wrap handlers (**onion model**)

## Locale Detection Priority Flow

```mermaid
flowchart TD
    Start([Incoming Request]) --> CheckQuery

    CheckQuery{Query param<br/>?lang=xx}
    CheckQuery -->|Yes + Valid| ReturnQuery[Return Query Locale]
    CheckQuery -->|No / Invalid| CheckAuth

    CheckAuth{Authenticated?<br/>JWT token valid}
    CheckAuth -->|Yes| CheckDB
    CheckAuth -->|No| CheckCookie

    CheckDB{User has<br/>language pref?}
    CheckDB -->|Yes + Valid| ReturnDB[Return DB Locale]
    CheckDB -->|No / Invalid| CheckCookie

    CheckCookie{Cookie<br/>lang=xx}
    CheckCookie -->|Yes + Valid| ReturnCookie[Return Cookie Locale]
    CheckCookie -->|No / Invalid| CheckHeader

    CheckHeader{Accept-Language<br/>header}
    CheckHeader -->|Yes + Valid| ReturnHeader[Return Header Locale]
    CheckHeader -->|No / Invalid| ReturnDefault[Return Default: en]

    ReturnQuery --> Insert[Insert DetectedLocale<br/>into request extensions]
    ReturnDB --> Insert
    ReturnCookie --> Insert
    ReturnHeader --> Insert
    ReturnDefault --> Insert

    Insert --> Next[next.run(req)]
    Next --> End([Response])

    style ReturnQuery fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
    style ReturnDB fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
    style ReturnCookie fill:#FFC107,stroke:#333,stroke-width:2px,color:#333
    style ReturnHeader fill:#FFC107,stroke:#333,stroke-width:2px,color:#333
    style ReturnDefault fill:#F44336,stroke:#333,stroke-width:2px,color:#fff
```

**Priority Levels**:

1. **Query param** (highest) - Explicit user override
2. **User DB preference** - Persistent user choice
3. **Cookie** - Session-based preference
4. **Accept-Language** - Browser default
5. **Default (en)** (lowest) - Fallback

## State Access Pattern Comparison

### ❌ Broken Pattern (Request Extensions)

```mermaid
sequenceDiagram
    participant Router
    participant Middleware
    participant Extensions

    Router->>Router: with_state(state)
    Note over Router: State stored in router

    Router->>Middleware: from_fn(middleware)
    Note over Middleware: No state parameter!

    Middleware->>Extensions: req.extensions().get::<AppState>()
    Note over Extensions: Extensions is empty
    Extensions-->>Middleware: None ❌

    Note over Middleware: Falls back to non-DB logic<br/>User preferences ignored
```

### ✅ Correct Pattern (from_fn_with_state)

```mermaid
sequenceDiagram
    participant Router
    participant Middleware
    participant AppState

    Router->>Router: state.clone()
    Note over Router: Clone Arc (cheap)

    Router->>Middleware: from_fn_with_state(state, middleware)
    Note over Middleware: State(state): State<AppState> ✅

    Middleware->>AppState: Access db, auth_service, etc.
    AppState-->>Middleware: Full state access

    Note over Middleware: Can query database<br/>User preferences respected ✅
```

## Database Query Optimization

```mermaid
flowchart TD
    Request[HTTP Request] --> ExtractToken

    ExtractToken[Extract JWT Token<br/>from Authorization header]
    ExtractToken --> ValidateToken

    ValidateToken{Token valid?}
    ValidateToken -->|Yes| CheckCache
    ValidateToken -->|No| SkipDB[Skip DB query<br/>Fall back to cookie/header]

    CheckCache{Redis cache<br/>available?}
    CheckCache -->|Yes| QueryCache[GET user:uuid:language]
    CheckCache -->|No| QueryDB

    QueryCache --> CacheHit{Cache hit?}
    CacheHit -->|Yes| ReturnCached[Return cached locale]
    CacheHit -->|No| QueryDB

    QueryDB[Query PostgreSQL<br/>SELECT language FROM users<br/>WHERE id = uuid]
    QueryDB --> DBResult{Result found?}

    DBResult -->|Yes| CacheResult[Cache result in Redis<br/>TTL: 1 hour]
    DBResult -->|No| SkipDB

    CacheResult --> ReturnDB[Return DB locale]

    ReturnCached --> End([Locale detected])
    ReturnDB --> End
    SkipDB --> End

    style CheckCache fill:#2196F3,stroke:#333,stroke-width:2px,color:#fff
    style QueryCache fill:#2196F3,stroke:#333,stroke-width:2px,color:#fff
    style QueryDB fill:#FFA726,stroke:#333,stroke-width:2px,color:#fff
```

**Performance Notes**:

- **Redis cache hit**: < 1ms
- **PostgreSQL query**: < 5ms (indexed primary key)
- **Cache miss fallback**: < 20ms total
- **No auth token**: 0ms (skip DB entirely)

## Error Handling Flow

```mermaid
flowchart TD
    Start([Detect Locale]) --> TryQuery

    TryQuery[Try Query Param] --> QueryErr{Error?}
    QueryErr -->|No error| ValidQuery{Valid locale?}
    QueryErr -->|Parse error| TryDB
    ValidQuery -->|Yes| Success[✅ Return locale]
    ValidQuery -->|No| TryDB

    TryDB[Try User DB] --> DBErr{Error?}
    DBErr -->|No error| ValidDB{Valid locale?}
    DBErr -->|DB/Auth error| TryCookie
    ValidDB -->|Yes| Success
    ValidDB -->|No| TryCookie

    TryCookie[Try Cookie] --> CookieErr{Error?}
    CookieErr -->|No error| ValidCookie{Valid locale?}
    CookieErr -->|Parse error| TryHeader
    ValidCookie -->|Yes| Success
    ValidCookie -->|No| TryHeader

    TryHeader[Try Accept-Language] --> HeaderErr{Error?}
    HeaderErr -->|No error| ValidHeader{Valid locale?}
    HeaderErr -->|Parse error| Fallback
    ValidHeader -->|Yes| Success
    ValidHeader -->|No| Fallback

    Fallback[Fallback: en] --> Success

    Success --> End([Continue request])

    style Success fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
    style Fallback fill:#FFC107,stroke:#333,stroke-width:2px,color:#333
```

**Error Philosophy**:

- **No panics** - all errors are caught and handled
- **Graceful degradation** - fall back to next source
- **Always succeed** - worst case returns "en"
- **No user impact** - errors are transparent

---

## Testing Strategy Diagram

```mermaid
graph TB
    subgraph "Unit Tests"
        UT1[test_normalize_locale<br/>Case normalization]
        UT2[test_is_supported_locale<br/>Validation logic]
        UT3[test_parse_accept_language<br/>Header parsing]
        UT4[test_extract_query_param<br/>Query string parsing]
    end

    subgraph "Integration Tests"
        IT1[test_middleware_with_authenticated_user<br/>Full DB flow]
        IT2[test_middleware_with_invalid_token<br/>Fallback handling]
        IT3[test_middleware_priority_order<br/>Detection precedence]
        IT4[test_middleware_performance<br/>Latency < 5ms]
    end

    subgraph "E2E Tests"
        E2E1[Manual: Register user + set preference]
        E2E2[Manual: Test with curl/Postman]
        E2E3[Automated: Frontend integration tests]
    end

    UT1 --> IT1
    UT2 --> IT1
    UT3 --> IT1
    UT4 --> IT1

    IT1 --> E2E1
    IT2 --> E2E2
    IT3 --> E2E2
    IT4 --> E2E3

    style IT1 fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
    style IT4 fill:#2196F3,stroke:#333,stroke-width:2px,color:#fff
```

---

## Migration Path

```mermaid
flowchart LR
    Current[Current State<br/>❌ Broken middleware<br/>No DB access]

    Current -->|Step 1| UpdateSig[Update Middleware Signature<br/>Add State parameter<br/>5 min]

    UpdateSig -->|Step 2| UpdateRouter[Update Router Registration<br/>from_fn_with_state<br/>2 min]

    UpdateRouter -->|Step 3| AddTests[Add Integration Tests<br/>Test with mock DB<br/>10 min]

    AddTests -->|Step 4| RunTests[Run Test Suite<br/>make test-backend<br/>5 min]

    RunTests -->|Step 5| UpdateDocs[Update Documentation<br/>Developer guide<br/>5 min]

    UpdateDocs --> Complete[✅ Complete<br/>DB-aware locale detection<br/>Total: 30 min]

    style Current fill:#F44336,stroke:#333,stroke-width:2px,color:#fff
    style Complete fill:#4CAF50,stroke:#333,stroke-width:2px,color:#fff
```

**Rollback Plan**: If issues arise, revert to simple detection (no DB) in < 5 minutes.

---

## Architecture Principles

### 1. Fail-Safe Design

- **Multiple fallbacks**: Query → DB → Cookie → Header → Default
- **No panics**: All errors caught and handled gracefully
- **Always succeeds**: Worst case returns "en"

### 2. Performance First

- **Minimal overhead**: Single DB query for authenticated users
- **Optional caching**: Redis cache for frequent lookups
- **Early exits**: Skip DB if not authenticated

### 3. Type Safety

- **Compile-time guarantees**: State parameter enforced by compiler
- **No runtime surprises**: Can't forget to pass state
- **Clear dependencies**: Signature shows what middleware needs

### 4. Testability

- **Unit tests**: Test helper functions in isolation
- **Integration tests**: Test with mock database
- **E2E tests**: Test with real API calls

### 5. Maintainability

- **Clear code**: Simple, linear flow
- **Good docs**: Architecture decision records
- **Easy rollback**: Can revert in < 5 minutes

---

## References

- Main Architecture Document: [LOCALE_MIDDLEWARE_DESIGN.md](./LOCALE_MIDDLEWARE_DESIGN.md)
- Localization Spec: [docs/localization/SPECIFICATION.md](../localization/SPECIFICATION.md)
- Axum Middleware Guide: https://docs.rs/axum/0.7/axum/middleware/
- Tower Service Trait: https://docs.rs/tower/latest/tower/trait.Service.html
