// Web Vitals monitoring
export interface WebVitalsMetric {
  name: string;
  value: number;
  rating: 'good' | 'needs-improvement' | 'poor';
  delta: number;
  id: string;
}

// Track Web Vitals (CLS, FID, LCP, FCP, TTFB)
export const reportWebVitals = (metric: WebVitalsMetric): void => {
  // Log to console in development
  if (import.meta.env.DEV) {
    console.log(`[Web Vitals] ${metric.name}:`, {
      value: metric.value,
      rating: metric.rating,
    });
  }

  // Send to analytics endpoint
  if (import.meta.env.PROD) {
    const body = JSON.stringify({
      metric: metric.name,
      value: metric.value,
      rating: metric.rating,
      url: window.location.href,
      timestamp: Date.now(),
    });

    // Use sendBeacon if available (non-blocking)
    if (navigator.sendBeacon) {
      navigator.sendBeacon('/api/analytics/web-vitals', body);
    } else {
      fetch('/api/analytics/web-vitals', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body,
        keepalive: true,
      }).catch((err) => {
        console.error('Failed to report web vitals:', err);
      });
    }
  }
};

// Track custom performance marks
export const trackPerformance = (name: string, metadata?: Record<string, unknown>): void => {
  const mark = `custom:${name}`;
  performance.mark(mark);

  if (import.meta.env.DEV) {
    console.log(`[Performance] ${name}`, metadata);
  }

  // Send custom performance metrics
  if (import.meta.env.PROD && metadata) {
    fetch('/api/analytics/performance', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name,
        timestamp: Date.now(),
        ...metadata,
      }),
    }).catch((err) => {
      console.error('Failed to track performance:', err);
    });
  }
};

// Track user actions for analytics
export const trackEvent = (
  category: string,
  action: string,
  label?: string,
  value?: number
): void => {
  if (import.meta.env.DEV) {
    console.log(`[Event] ${category}:${action}`, { label, value });
  }

  if (import.meta.env.PROD) {
    fetch('/api/analytics/events', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        category,
        action,
        label,
        value,
        timestamp: Date.now(),
        url: window.location.href,
      }),
    }).catch((err) => {
      console.error('Failed to track event:', err);
    });
  }
};

// Initialize monitoring
export const initMonitoring = (): void => {
  // Load web-vitals library dynamically to avoid blocking
  import('web-vitals')
    .then(({ onCLS, onINP, onLCP, onFCP, onTTFB }) => {
      onCLS(reportWebVitals);
      onINP(reportWebVitals);
      onLCP(reportWebVitals);
      onFCP(reportWebVitals);
      onTTFB(reportWebVitals);
    })
    .catch((err) => {
      console.error('Failed to load web-vitals:', err);
    });

  // Track page visibility changes
  document.addEventListener('visibilitychange', () => {
    trackEvent('page', 'visibility', document.hidden ? 'hidden' : 'visible');
  });

  // Track unhandled errors
  window.addEventListener('error', (event) => {
    console.error('Unhandled error:', event.error);
    fetch('/api/errors', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        error: {
          message: event.message,
          filename: event.filename,
          lineno: event.lineno,
          colno: event.colno,
          stack: event.error?.stack,
        },
        timestamp: Date.now(),
        url: window.location.href,
      }),
    }).catch((err) => {
      console.error('Failed to report error:', err);
    });
  });

  // Track unhandled promise rejections
  window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
    fetch('/api/errors', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        error: {
          message: 'Unhandled Promise Rejection',
          reason: String(event.reason),
        },
        timestamp: Date.now(),
        url: window.location.href,
      }),
    }).catch((err) => {
      console.error('Failed to report promise rejection:', err);
    });
  });
};
