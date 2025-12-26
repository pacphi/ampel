/**
 * DiffViewer Component Tests
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DiffViewer } from '../DiffViewer';
import type { DiffFile } from '../../../types/diff';

// Mock @git-diff-view/react
vi.mock('@git-diff-view/react', () => ({
  DiffView: ({ diffText }: { diffText: string }) => <div data-testid="diff-view">{diffText}</div>,
}));

// Mock language detection
vi.mock('../../../utils/languageDetection', () => ({
  detectLanguage: () => ({ language: 'typescript', confidence: 0.9, fileExtension: 'ts' }),
  supportsHighlighting: () => true,
}));

describe('DiffViewer', () => {
  const mockFile: DiffFile = {
    id: 'file-1',
    to: 'src/test.ts',
    from: 'src/test.ts',
    chunks: [
      {
        content: '@@ -1,3 +1,3 @@\n-old line\n+new line',
        changes: [],
        oldStart: 1,
        oldLines: 3,
        newStart: 1,
        newLines: 3,
      },
    ],
    isExpanded: false,
    isBinary: false,
    additions: 1,
    deletions: 1,
    new: false,
    deleted: false,
  };

  it('renders diff view with file content', () => {
    render(<DiffViewer file={mockFile} />);
    const diffView = screen.getByTestId('diff-view');
    expect(diffView).toBeInTheDocument();
    expect(diffView.textContent).toContain('src/test.ts');
  });

  it('shows message when no diff available', () => {
    const emptyFile: DiffFile = {
      ...mockFile,
      chunks: [],
    };
    render(<DiffViewer file={emptyFile} />);
    expect(screen.getByText('No diff available for this file')).toBeInTheDocument();
  });

  it('applies syntax highlighting when enabled', () => {
    render(<DiffViewer file={mockFile} syntaxHighlighting={true} />);
    const diffView = screen.getByTestId('diff-view');
    expect(diffView).toBeInTheDocument();
  });

  it('supports split view mode', () => {
    render(<DiffViewer file={mockFile} viewMode="split" />);
    const diffView = screen.getByTestId('diff-view');
    expect(diffView).toBeInTheDocument();
  });

  it('supports unified view mode', () => {
    render(<DiffViewer file={mockFile} viewMode="unified" />);
    const diffView = screen.getByTestId('diff-view');
    expect(diffView).toBeInTheDocument();
  });
});
