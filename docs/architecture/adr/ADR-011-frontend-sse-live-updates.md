# ADR-011: Frontend Live Update Transport for the Remediation Run Timeline

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: Enable sub-second live progress updates on the Remediation Run detail page as autonomous fleet runs advance through the selecting → consolidating → remediating → verifying → merging → completed lifecycle.

---

## Context

### Problem Statement

The Fleet PR Remediation Loops feature introduces long-running autonomous runs that transition through multiple states over tens of seconds to several minutes. Users navigating to the Remediation Run detail page need to observe progress in near-real-time without manually refreshing — both to gain trust in the system and to act quickly if a run stalls or errors.

The run detail page must display at minimum: the current run state, per-PR CI check results as they arrive, the active agent iteration count, and a terminal signal (completed, failed, or cancelled). Polling the REST API at a coarse interval would introduce visible lag at state transitions and generate unnecessary load; a push-based transport is preferable.

A secondary concern is implementation cost. The codebase already ships a working SSE pattern for the bulk-merge feature (`/api/merges/{id}/events`), including an Axum SSE stream on the backend and a `useMergeRunEvents` hook on the frontend. Any new transport choice must justify its complexity delta against this existing investment.

The fleet overview page is a deliberately separate concern: it aggregates status across many repositories and is updated via TanStack Query polling (`useFleetRemediation` with `refetchInterval`). Sub-second fidelity is not required there, so it is out of scope for this ADR.

### Technical Context

- **Backend framework**: Axum 0.8 + Tokio; SSE is a first-class Axum primitive (`axum::response::sse`).
- **Frontend framework**: React 19 + TanStack Query v5; `EventSource` is a browser built-in with automatic reconnect.
- **Existing SSE pattern**: `GET /api/merges/{id}/events` → Axum SSE stream; `useMergeRunEvents` hook wraps `EventSource`.
- **Auth**: JWT in Authorization header; browser `EventSource` does not support custom headers — the existing pattern uses a short-lived token passed as a query parameter (`?token=`) or relies on the httpOnly refresh-cookie session. This constraint applies equally to any new SSE endpoint.
- **Run states (ordered)**: `selecting` → `consolidating` → `remediating` → `verifying` → `merging` → `completed` (terminal); also `failed` and `cancelled`.
- **Event cardinality**: a single run emits O(tens) of events — far below any backpressure concern.
- **Infrastructure**: no dedicated message broker is in place; Redis is present (used for caching) but no pub/sub consumers exist yet.
- **Deployment**: Fly.io; HTTP/1.1 and HTTP/2 both supported; no WebSocket proxy configuration exists.

---

## Decision

**We will implement live updates for the Remediation Run detail page using Server-Sent Events (SSE) over `GET /api/remediation/runs/{id}/events`, directly mirroring the existing bulk-merge SSE pattern.**

The run detail page is a strictly server-to-client push scenario: the backend owns all state transitions and the client has no need to send messages mid-stream. SSE is the canonical HTTP mechanism for this pattern. Reusing the established bulk-merge implementation minimises new surface area, keeps the frontend hook API consistent, and avoids introducing WebSocket infrastructure or a polling fallback that would degrade the user experience.

### Implementation Notes

**Backend — event types**

Define a `RemediationRunEvent` enum in `ampel-worker` (or `ampel-core`) with the following variants, serialised as `event: <type>\ndata: <json>\n\n`:

| Event type | Payload fields | Trigger |
|---|---|---|
| `RunStateChanged` | `run_id`, `new_state`, `previous_state`, `timestamp` | Worker transitions the run FSM |
| `CiCheckUpdated` | `run_id`, `pr_id`, `check_name`, `status`, `url` | CI poller receives a webhook or re-polls |
| `AgentIterationCompleted` | `run_id`, `iteration`, `prs_affected`, `action_summary` | Remediation agent finishes one loop |
| `RunFinished` | `run_id`, `outcome` (`completed`/`failed`/`cancelled`), `summary`, `timestamp` | Run reaches a terminal state |

**Backend — Axum handler sketch**

```rust
// crates/ampel-api/src/handlers/remediation_runs.rs

pub async fn remediation_run_events(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.run_event_bus.subscribe(run_id);
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(evt) => {
                let event_type = evt.event_type(); // &'static str
                let data = serde_json::to_string(&evt).ok()?;
                Some(Ok(Event::default().event(event_type).data(data)))
            }
            Err(_) => None,
        }
    });
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
```

The `run_event_bus` is a `tokio::sync::broadcast::Sender<RemediationRunEvent>` stored in `AppState`, keyed by `run_id`. The worker broadcasts to it on every state transition; the handler subscribes and forwards to the SSE stream. When the run reaches a terminal state the worker broadcasts `RunFinished` and the stream ends naturally.

**Frontend — hook**

```typescript
// frontend/src/hooks/useRemediationRunEvents.ts

export function useRemediationRunEvents(
  runId: string,
  handlers: {
    onStateChanged?: (e: RunStateChangedEvent) => void;
    onCiCheckUpdated?: (e: CiCheckUpdatedEvent) => void;
    onIterationCompleted?: (e: AgentIterationCompletedEvent) => void;
    onFinished?: (e: RunFinishedEvent) => void;
  }
) {
  useEffect(() => {
    const token = getShortLivedToken(); // existing util, mirrors bulk-merge pattern
    const url = `/api/remediation/runs/${runId}/events?token=${token}`;
    const es = new EventSource(url);

    es.addEventListener('RunStateChanged', (e) =>
      handlers.onStateChanged?.(JSON.parse(e.data)));
    es.addEventListener('CiCheckUpdated', (e) =>
      handlers.onCiCheckUpdated?.(JSON.parse(e.data)));
    es.addEventListener('AgentIterationCompleted', (e) =>
      handlers.onIterationCompleted?.(JSON.parse(e.data)));
    es.addEventListener('RunFinished', (e) => {
      handlers.onFinished?.(JSON.parse(e.data));
      es.close();
    });

    return () => es.close();
  }, [runId]);
}
```

The hook mirrors `useMergeRunEvents` in structure; callers are responsible for updating local React state in the provided callbacks. TanStack Query is used for the initial run load (`useRemediationRun`); SSE events then patch that state incrementally without triggering a full refetch.

**Auth note**: the existing short-lived token approach (a one-time-use token issued by a dedicated `/api/auth/sse-token` endpoint, valid for 30 seconds) is reused unchanged. No new auth mechanism is introduced.

**Keep-alive**: a 15-second SSE keep-alive comment prevents proxy and load-balancer idle-connection teardowns, consistent with the bulk-merge endpoint.

---

## Alternatives Considered

### Option A: SSE (Accepted)

**Pros**:
- Direct reuse of the bulk-merge SSE pattern — backend handler, frontend hook, auth token strategy, and keep-alive handling are all established.
- Server-to-client only; bidirectional capability is not needed.
- `EventSource` provides automatic reconnect with exponential back-off at zero cost.
- HTTP/1.1 compatible; no proxy reconfiguration required on Fly.io.
- No new infrastructure dependencies.

**Cons**:
- Browser `EventSource` does not support custom request headers; the short-lived token workaround adds a small auth complexity (already accepted for bulk-merge).
- HTTP/1.1 limits concurrent SSE connections per domain to 6 (browser limit); opening multiple run detail tabs simultaneously approaches this ceiling. HTTP/2 (enabled on Fly.io) multiplexes streams and resolves this.

**Verdict**: Accepted. The pattern is proven in the codebase, the constraints are known and already mitigated.

---

### Option B: WebSocket (Rejected)

**Pros**:
- Full-duplex; could support future client-to-server commands (e.g., cancel a run mid-stream).
- Single persistent connection; no per-event overhead.

**Cons**:
- Bidirectional capability is unused for the run timeline; the complexity is not justified.
- Requires `ws://` / `wss://` protocol handling in the frontend, a separate Axum upgrade handler, and explicit proxy/load-balancer configuration on Fly.io.
- No existing WebSocket infrastructure in the codebase; would be net-new rather than net-reuse.
- Cancellation (the only plausible client-to-server use case) is already handled via a separate REST endpoint `DELETE /api/remediation/runs/{id}`.

**Verdict**: Rejected. Overkill for a unidirectional progress stream; introduces protocol and infrastructure complexity with no functional benefit over SSE in this context.

---

### Option C: Long Polling (Rejected)

**Pros**:
- Universally compatible; works behind any HTTP proxy without configuration.
- No persistent connection held open on the server.

**Cons**:
- Higher latency per event (round-trip per poll cycle vs. immediate push).
- More requests; each poll cycle opens a new connection and incurs HTTP overhead.
- Requires client-side sequencing (cursor or `since` timestamp) to avoid replaying events.
- No meaningful advantage over SSE for this use case; the Fly.io environment already supports persistent connections.

**Verdict**: Rejected. Strictly inferior to SSE for a scenario where the infrastructure supports it.

---

## Trade-off Analysis

| Aspect | Option A: SSE | Option B: WebSocket | Option C: Long Polling |
|---|---|---|---|
| Implementation effort | Low — reuse existing pattern | High — new protocol, new infra | Medium — cursor logic, no reuse |
| Latency | Sub-second push | Sub-second push | 1–3 s typical |
| Infrastructure changes | None | Proxy config on Fly.io | None |
| Reconnect handling | Built-in (EventSource) | Manual | Inherent (per-request) |
| Bidirectional | No (not needed) | Yes (not needed) | No |
| Auth mechanism | Short-lived token (existing) | Cookie or header | Standard JWT |
| HTTP/1.1 connection limit | 6 concurrent (browser) | 1 per tab (separate) | Many (short-lived) |
| HTTP/2 multiplexed | Yes | N/A | Yes |
| Code reuse | High (mirrors bulk-merge) | None | Low |

---

## Consequences

### Positive

- The run detail page reflects state changes within milliseconds of the worker broadcasting them, giving users immediate feedback during autonomous runs.
- The frontend hook API is consistent with the existing `useMergeRunEvents` pattern; developers familiar with bulk-merge SSE can contribute to run timeline code without a learning curve.
- No new backend services, message brokers, or proxy rules are required.
- `EventSource` automatic reconnect means brief network interruptions do not permanently break the live view.

### Negative

- The short-lived SSE token mechanism adds a small auth round-trip before the `EventSource` connection opens; this is an existing accepted trade-off from the bulk-merge feature, not a new one.
- HTTP/1.1 clients opening more than 6 simultaneous SSE connections from the same domain (e.g., multiple run detail tabs) will queue. This is a browser constraint, not an Axum constraint, and is resolved by HTTP/2 multiplexing which Fly.io enables.
- The backend `broadcast::Sender` channel drops events if no receivers are subscribed at the moment of broadcast (late-joining clients miss historical events). The run detail page mitigates this by loading the current run snapshot via the initial REST call before opening the SSE stream; missed intermediate events are acceptable because the snapshot always reflects the latest state.

### Neutral

- The fleet overview page continues to use TanStack Query polling (`refetchInterval: 30_000`); this ADR does not change that behaviour.
- Future runs that need client-to-server messages mid-stream (not currently planned) would require upgrading to WebSocket or adding a parallel REST endpoint; SSE does not preclude either.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Proxy/load-balancer closes idle SSE connections | Medium | 15-second SSE keep-alive comment (mirrors bulk-merge endpoint) |
| Late-joining client misses state events | Low | Initial REST snapshot loaded before SSE stream opens; snapshot always reflects current state |
| Worker crashes without emitting `RunFinished` | Medium | Frontend detects `EventSource` `error` event after reconnect attempts exhaust; falls back to polling the REST snapshot endpoint every 10 s for up to 5 minutes |
| HTTP/1.1 browser 6-connection limit | Low | HTTP/2 multiplexing on Fly.io; documented in operator runbook |
| Short-lived token expiry before stream opens | Low | Token TTL is 30 s; stream must be opened within that window (existing constraint from bulk-merge) |
| `broadcast::Sender` channel lag on burst | Low | Channel capacity set to 256 events; remediation runs emit O(tens) of events over their lifetime |

---

## Related ADRs

- ADR-001: Locale Middleware State Access Pattern — establishes the `from_fn_with_state` middleware pattern used in the Axum handler for this endpoint.
- ADR-008 (planned): Remediation Playbook Storage and Execution Model — defines the run FSM whose state transitions are the primary source of SSE events.
- ADR-009 (planned): Sandbox Isolation for Autonomous Remediation Agents — defines the worker process that broadcasts events to the bus consumed by this endpoint.
- ADR-010 (planned): AI Model Provider Abstraction for Remediation Inference — defines the agent iteration lifecycle that emits `AgentIterationCompleted` events.
