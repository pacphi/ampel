import { useEffect, useRef, useState } from 'react';
import { buildRunEventsUrl, remediationApi } from '@/api/remediation';
import type {
  CiStatus,
  CiStatusUpdatedEvent,
  RunFinishedEvent,
  RunState,
  RunStateChangedEvent,
} from '@/types/remediation';

export interface UseRemediationRunEventsOptions {
  /** Run to subscribe to. When falsy the hook stays idle. */
  runId: string;
  /** Master switch — set false to keep the stream closed (e.g. terminal run). */
  enabled?: boolean;
  /** Maximum reconnect attempts before giving up. */
  maxRetries?: number;
  /** Backoff base (ms) between reconnect attempts. */
  retryDelayMs?: number;
  onStateChanged?: (event: RunStateChangedEvent) => void;
  onCiStatusUpdated?: (event: CiStatusUpdatedEvent) => void;
  onFinished?: (event: RunFinishedEvent) => void;
}

export interface RemediationRunEventsState {
  /** Latest state observed over the stream, or null before the first event. */
  state: RunState | null;
  /** Latest CI status observed over the stream, or null before the first event. */
  ciStatus: CiStatus | null;
  /** True once a `run_finished` event has arrived. */
  finished: boolean;
  /** True while an EventSource connection is open. */
  connected: boolean;
}

const DEFAULT_MAX_RETRIES = 3;
const DEFAULT_RETRY_DELAY_MS = 2000;

/**
 * Live remediation-run event subscription over SSE.
 *
 * Auth: `EventSource` cannot send headers, so the hook first POSTs for a
 * short-lived token (`fetchSseToken`) and opens the stream with `?token=`.
 * It wires the three named events to callbacks and tracks live run state.
 *
 * Lifecycle: closes on `run_finished` and on unmount. On a transport error it
 * reconnects with a fixed backoff up to `maxRetries`, then stops (fallback to
 * the polled run-detail query).
 */
export function useRemediationRunEvents({
  runId,
  enabled = true,
  maxRetries = DEFAULT_MAX_RETRIES,
  retryDelayMs = DEFAULT_RETRY_DELAY_MS,
  onStateChanged,
  onCiStatusUpdated,
  onFinished,
}: UseRemediationRunEventsOptions): RemediationRunEventsState {
  const [state, setState] = useState<RunState | null>(null);
  const [ciStatus, setCiStatus] = useState<CiStatus | null>(null);
  const [finished, setFinished] = useState(false);
  const [connected, setConnected] = useState(false);

  // Keep latest callbacks without re-opening the stream on every render.
  const cbRef = useRef({ onStateChanged, onCiStatusUpdated, onFinished });
  useEffect(() => {
    cbRef.current = { onStateChanged, onCiStatusUpdated, onFinished };
  }, [onStateChanged, onCiStatusUpdated, onFinished]);

  useEffect(() => {
    if (!runId || !enabled) {
      return;
    }

    let cancelled = false;
    let source: EventSource | null = null;
    let retries = 0;
    let retryTimer: ReturnType<typeof setTimeout> | undefined;

    const close = () => {
      if (source) {
        source.close();
        source = null;
      }
      setConnected(false);
    };

    const connect = async () => {
      if (cancelled) return;
      let token: string;
      try {
        const sse = await remediationApi.fetchSseToken();
        token = sse.token;
      } catch {
        scheduleRetry();
        return;
      }
      if (cancelled) return;

      source = new EventSource(buildRunEventsUrl(runId, token));

      source.onopen = () => {
        if (cancelled) return;
        retries = 0;
        setConnected(true);
      };

      source.addEventListener('run_state_changed', (e) => {
        const data = parse<RunStateChangedEvent>(e);
        if (!data) return;
        setState(data.state);
        setCiStatus(data.ciStatus);
        cbRef.current.onStateChanged?.(data);
      });

      source.addEventListener('ci_status_updated', (e) => {
        const data = parse<CiStatusUpdatedEvent>(e);
        if (!data) return;
        setCiStatus(data.ciStatus);
        cbRef.current.onCiStatusUpdated?.(data);
      });

      source.addEventListener('run_finished', (e) => {
        const data = parse<RunFinishedEvent>(e);
        setFinished(true);
        if (data) cbRef.current.onFinished?.(data);
        close();
      });

      source.onerror = () => {
        if (cancelled) return;
        close();
        scheduleRetry();
      };
    };

    const scheduleRetry = () => {
      if (cancelled || retries >= maxRetries) return;
      retries += 1;
      retryTimer = setTimeout(connect, retryDelayMs * retries);
    };

    void connect();

    return () => {
      cancelled = true;
      if (retryTimer) clearTimeout(retryTimer);
      close();
    };
  }, [runId, enabled, maxRetries, retryDelayMs]);

  return { state, ciStatus, finished, connected };
}

function parse<T>(event: Event): T | null {
  const msg = event as MessageEvent;
  if (typeof msg.data !== 'string') return null;
  try {
    return JSON.parse(msg.data) as T;
  } catch {
    return null;
  }
}
