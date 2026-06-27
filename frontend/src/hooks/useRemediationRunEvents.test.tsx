import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useRemediationRunEvents } from './useRemediationRunEvents';

const mockFetchSseToken = vi.fn();
vi.mock('@/api/remediation', () => ({
  remediationApi: {
    fetchSseToken: () => mockFetchSseToken(),
  },
  buildRunEventsUrl: (runId: string, token: string) =>
    `/api/remediation/runs/${runId}/events?token=${token}`,
}));

// ---- Minimal EventSource mock ----
class MockEventSource {
  static instances: MockEventSource[] = [];
  url: string;
  readyState = 0;
  onopen: ((e: Event) => void) | null = null;
  onerror: ((e: Event) => void) | null = null;
  closed = false;
  private listeners: Record<string, ((e: Event) => void)[]> = {};

  constructor(url: string) {
    this.url = url;
    MockEventSource.instances.push(this);
  }
  addEventListener(type: string, cb: (e: Event) => void) {
    (this.listeners[type] ??= []).push(cb);
  }
  close() {
    this.closed = true;
    this.readyState = 2;
  }
  open() {
    this.readyState = 1;
    this.onopen?.(new Event('open'));
  }
  emit(type: string, data: unknown) {
    const event = new MessageEvent(type, { data: JSON.stringify(data) });
    for (const cb of this.listeners[type] ?? []) cb(event);
  }
  error() {
    this.onerror?.(new Event('error'));
  }
}

beforeEach(() => {
  vi.clearAllMocks();
  MockEventSource.instances = [];
  vi.stubGlobal('EventSource', MockEventSource as unknown as typeof EventSource);
  mockFetchSseToken.mockResolvedValue({ token: 'tok-123', expiresAt: '2099-01-01T00:00:00Z' });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('useRemediationRunEvents', () => {
  it('should_openEventSourceWithToken_when_runIdProvided', async () => {
    renderHook(() => useRemediationRunEvents({ runId: 'run-1' }));

    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));
    expect(MockEventSource.instances[0].url).toContain('token=tok-123');
    expect(MockEventSource.instances[0].url).toContain('/runs/run-1/events');
  });

  it('should_notOpenStream_when_disabled', async () => {
    renderHook(() => useRemediationRunEvents({ runId: 'run-1', enabled: false }));

    // Give the async path a tick; nothing should connect.
    await Promise.resolve();
    expect(MockEventSource.instances).toHaveLength(0);
    expect(mockFetchSseToken).not.toHaveBeenCalled();
  });

  it('should_dispatchStateChange_when_stateEventReceived', async () => {
    const onStateChanged = vi.fn();
    const { result } = renderHook(() =>
      useRemediationRunEvents({ runId: 'run-1', onStateChanged })
    );

    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));

    act(() => {
      MockEventSource.instances[0].emit('run_state_changed', {
        runId: 'run-1',
        state: 'merging',
        previousState: 'verifying',
        ciStatus: 'success',
        ts: '2026-06-01T00:00:00Z',
      });
    });

    expect(onStateChanged).toHaveBeenCalledOnce();
    expect(result.current.state).toBe('merging');
    expect(result.current.ciStatus).toBe('success');
  });

  it('should_dispatchCiStatusUpdate_when_ciEventReceived', async () => {
    const onCiStatusUpdated = vi.fn();
    const { result } = renderHook(() =>
      useRemediationRunEvents({ runId: 'run-1', onCiStatusUpdated })
    );

    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));

    act(() => {
      MockEventSource.instances[0].emit('ci_status_updated', {
        runId: 'run-1',
        ciStatus: 'failure',
      });
    });

    expect(onCiStatusUpdated).toHaveBeenCalledOnce();
    expect(result.current.ciStatus).toBe('failure');
  });

  it('should_closeStream_when_runFinished', async () => {
    const onFinished = vi.fn();
    const { result } = renderHook(() => useRemediationRunEvents({ runId: 'run-1', onFinished }));

    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));

    act(() => {
      MockEventSource.instances[0].emit('run_finished', {
        runId: 'run-1',
        outcome: 'completed',
        ts: '2026-06-01T00:00:00Z',
      });
    });

    expect(onFinished).toHaveBeenCalledOnce();
    expect(MockEventSource.instances[0].closed).toBe(true);
    expect(result.current.finished).toBe(true);
  });

  it('should_closeStream_when_unmounted', async () => {
    const { unmount } = renderHook(() => useRemediationRunEvents({ runId: 'run-1' }));

    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));

    unmount();

    expect(MockEventSource.instances[0].closed).toBe(true);
  });
});
