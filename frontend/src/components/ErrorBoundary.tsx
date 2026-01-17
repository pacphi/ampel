import { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { i18n } from '@/i18n';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('ErrorBoundary caught an error:', error, errorInfo);

    // Call custom error handler if provided
    this.props.onError?.(error, errorInfo);

    // Log to monitoring service (e.g., Sentry)
    if (import.meta.env.PROD) {
      this.logErrorToService(error, errorInfo);
    }

    this.setState({ error, errorInfo });
  }

  logErrorToService(error: Error, errorInfo: ErrorInfo): void {
    // Send error to monitoring service
    // Example with fetch to a backend endpoint
    fetch('/api/errors', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        error: {
          name: error.name,
          message: error.message,
          stack: error.stack,
        },
        errorInfo: {
          componentStack: errorInfo.componentStack,
        },
        userAgent: navigator.userAgent,
        timestamp: new Date().toISOString(),
        url: window.location.href,
      }),
    }).catch((err) => {
      console.error('Failed to log error:', err);
    });
  }

  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex items-center justify-center min-h-screen bg-gray-50 p-4">
          <Card className="w-full max-w-2xl">
            <CardHeader>
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-6 w-6 text-red-500" />
                <CardTitle>{i18n.t('errors:boundary.title')}</CardTitle>
              </div>
              <CardDescription>{i18n.t('errors:boundary.description')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {import.meta.env.DEV && this.state.error && (
                <div className="bg-gray-100 p-4 rounded-md overflow-auto">
                  <h3 className="font-semibold text-sm mb-2">
                    {i18n.t('errors:boundary.details')}
                  </h3>
                  <pre className="text-xs text-red-600">{this.state.error.toString()}</pre>
                  {this.state.errorInfo && (
                    <pre className="text-xs mt-2 text-gray-600">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  )}
                </div>
              )}
              <div className="flex gap-2">
                <Button onClick={this.handleReset} variant="outline">
                  {i18n.t('errors:boundary.tryAgain')}
                </Button>
                <Button onClick={this.handleReload}>{i18n.t('errors:boundary.reload')}</Button>
              </div>
            </CardContent>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
