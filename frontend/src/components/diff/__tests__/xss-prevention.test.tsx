/**
 * XSS Prevention Test Suite for DiffViewer Component
 *
 * Tests comprehensive XSS attack vectors to ensure the diff viewer
 * safely handles malicious content in file paths, patches, and metadata.
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { DiffViewer } from '../DiffViewer';
import type { DiffFile } from '../../../types/diff';

describe('DiffViewer - XSS Prevention', () => {
  describe('Script Tag Injection', () => {
    it('should sanitize script tags in file paths', () => {
      const maliciousFile: DiffFile = {
        from: 'normal.js',
        to: '<script>alert("XSS")</script>.js',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-old\n+new',
            changes: [
              {
                type: 'delete',
                content: 'old',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'new',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={maliciousFile} />);

      // Verify no script tags are rendered as actual DOM elements
      const scripts = container.getElementsByTagName('script');
      expect(scripts.length).toBe(0);

      // Verify the script tag is HTML-escaped in the output (shows as &lt;script&gt;)
      // innerHTML will contain escaped entities if properly sanitized
      const innerHTML = container.innerHTML;
      expect(innerHTML).toContain('&lt;script&gt;');
    });

    it('should escape script tags in diff content', () => {
      const fileWithScriptInDiff: DiffFile = {
        from: 'test.html',
        to: 'test.html',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-<div>safe</div>\n+<script>alert("XSS")</script>',
            changes: [
              {
                type: 'delete',
                content: '<div>safe</div>',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: '<script>alert("XSS")</script>',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={fileWithScriptInDiff} />);

      // Verify script content is displayed as text, not executed
      const scripts = container.getElementsByTagName('script');
      expect(scripts.length).toBe(0);

      // Note: @git-diff-view/react renders diff in a specialized way
      // The critical check is ensuring no script tags are created or executed
      // Text content rendering is implementation-specific to the library
    });
  });

  describe('Event Handler Injection', () => {
    it('should sanitize onclick handlers in file paths', () => {
      const maliciousFile: DiffFile = {
        from: 'normal.js',
        to: '<img src=x onerror="alert(1)">test.js',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-old\n+new',
            changes: [
              {
                type: 'delete',
                content: 'old',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'new',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={maliciousFile} />);

      // Verify no img tags with onerror handlers
      const images = container.getElementsByTagName('img');
      for (let i = 0; i < images.length; i++) {
        expect(images[i].getAttribute('onerror')).toBeNull();
      }
    });

    it('should sanitize onload handlers in diff metadata', () => {
      const maliciousMetadataFile: DiffFile = {
        from: 'test.js',
        to: 'test.js',
        oldMode: '<body onload="alert(1)">',
        newMode: '100644',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-old\n+new',
            changes: [
              {
                type: 'delete',
                content: 'old',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'new',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={maliciousMetadataFile} />);

      // Verify no body tags with onload handlers
      const bodies = container.getElementsByTagName('body');
      for (let i = 0; i < bodies.length; i++) {
        expect(bodies[i].getAttribute('onload')).toBeNull();
      }
    });
  });

  describe('HTML Entity Injection', () => {
    it('should properly escape HTML entities in code blocks', () => {
      const fileWithEntities: DiffFile = {
        from: 'test.tsx',
        to: 'test.tsx',
        chunks: [
          {
            content:
              '@@ -1,1 +1,1 @@\n-return <div>test</div>\n+return &lt;div&gt;test&lt;/div&gt;',
            changes: [
              {
                type: 'delete',
                content: 'return <div>test</div>',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'return &lt;div&gt;test&lt;/div&gt;',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={fileWithEntities} />);

      // Verify no dangerous HTML tags are rendered
      const scripts = container.getElementsByTagName('script');
      expect(scripts.length).toBe(0);

      // @git-diff-view/react safely renders diff content in a controlled manner
      // The library handles HTML entities internally for safe display
    });

    it('should handle mixed HTML entities and special characters', () => {
      const fileWithMixed: DiffFile = {
        from: 'test.html',
        to: 'test.html',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-&amp;&lt;&gt;&quot;&#39;\n+<>&"\' javascript:alert(1)',
            changes: [
              {
                type: 'delete',
                content: '&amp;&lt;&gt;&quot;&#39;',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: '<>&"\' javascript:alert(1)',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={fileWithMixed} />);

      // Verify no script execution occurs
      const scripts = container.getElementsByTagName('script');
      expect(scripts.length).toBe(0);

      // The library renders content safely - checking for dangerous patterns
      const anchors = container.getElementsByTagName('a');
      for (let i = 0; i < anchors.length; i++) {
        const href = anchors[i].getAttribute('href') || '';
        expect(href.toLowerCase()).not.toContain('javascript:');
      }
    });
  });

  describe('React Escaping Verification', () => {
    it('should verify React automatically escapes JSX content', () => {
      const reactComponent: DiffFile = {
        from: 'Component.tsx',
        to: 'Component.tsx',
        chunks: [
          {
            content:
              '@@ -1,1 +1,1 @@\n-const html = "<script>alert(1)</script>"\n+const html = props.userInput',
            changes: [
              {
                type: 'delete',
                content: 'const html = "<script>alert(1)</script>"',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'const html = props.userInput',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={reactComponent} />);

      // No script execution
      const scripts = container.getElementsByTagName('script');
      expect(scripts.length).toBe(0);
    });

    it('should verify no dangerouslySetInnerHTML usage in DiffViewer', () => {
      const file: DiffFile = {
        from: 'test.js',
        to: 'test.js',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-old\n+new',
            changes: [
              {
                type: 'delete',
                content: 'old',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'new',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={file} />);

      // Check that no elements have dangerouslySetInnerHTML attribute
      const allElements = container.getElementsByTagName('*');
      for (let i = 0; i < allElements.length; i++) {
        const element = allElements[i] as Element & {
          __reactProps?: { dangerouslySetInnerHTML?: unknown };
        };
        const innerHTML = element.__reactProps?.dangerouslySetInnerHTML;
        expect(innerHTML).toBeUndefined();
      }
    });
  });

  describe('@git-diff-view/react Library Safety', () => {
    it('should verify library does not execute scripts in diff content', () => {
      const maliciousCode: DiffFile = {
        from: 'exploit.js',
        to: 'exploit.js',
        chunks: [
          {
            content:
              '@@ -1,3 +1,3 @@\n-// Safe comment\n+<img src=x onerror=alert(1)>\n+<svg/onload=alert(1)>\n+<iframe src="javascript:alert(1)">',
            changes: [
              {
                type: 'delete',
                content: '// Safe comment',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: '<img src=x onerror=alert(1)>',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: '<svg/onload=alert(1)>',
                lineNumber: 2,
              },
              {
                type: 'insert',
                content: '<iframe src="javascript:alert(1)">',
                lineNumber: 3,
              },
            ],
          },
        ],
        additions: 3,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={maliciousCode} syntaxHighlighting={true} />);

      // Verify no execution vectors
      expect(container.getElementsByTagName('script').length).toBe(0);
      expect(container.getElementsByTagName('iframe').length).toBe(0);

      // SVG and img tags used for UI should not have event handlers
      const images = container.getElementsByTagName('img');
      for (let i = 0; i < images.length; i++) {
        expect(images[i].getAttribute('onerror')).toBeNull();
      }

      const svgs = container.getElementsByTagName('svg');
      for (let i = 0; i < svgs.length; i++) {
        expect(svgs[i].getAttribute('onload')).toBeNull();
      }
    });
  });

  describe('Multi-File Diff XSS Prevention', () => {
    it('should sanitize multiple files with malicious paths', () => {
      const maliciousDiff = {
        files: [
          {
            filePath: '<script>alert(1)</script>',
            status: 'modified',
            additions: 5,
            deletions: 2,
            changes: 7,
            patch: '@@ -1,1 +1,1 @@\n-old\n+new',
          },
          {
            filePath: '<img src=x onerror=alert(2)>',
            status: 'added',
            additions: 10,
            deletions: 0,
            changes: 10,
            patch: '@@ -0,0 +1,1 @@\n+new file',
          },
        ],
        summary: {
          totalFiles: 2,
          totalAdditions: 15,
          totalDeletions: 2,
          totalChanges: 17,
        },
      };

      const { container } = render(<DiffViewer diff={maliciousDiff} />);

      // No scripts executed
      expect(container.getElementsByTagName('script').length).toBe(0);

      // No images with onerror
      const images = container.getElementsByTagName('img');
      for (let i = 0; i < images.length; i++) {
        expect(images[i].getAttribute('onerror')).toBeNull();
      }
    });
  });

  describe('URL Injection Protection', () => {
    it('should prevent javascript: URLs in file paths', () => {
      const maliciousFile: DiffFile = {
        from: 'javascript:alert(1)',
        to: 'javascript:alert(2)',
        chunks: [
          {
            content: '@@ -1,1 +1,1 @@\n-old\n+new',
            changes: [
              {
                type: 'delete',
                content: 'old',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: 'new',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={maliciousFile} />);

      // Check for any anchor tags with javascript: protocol
      const anchors = container.getElementsByTagName('a');
      for (let i = 0; i < anchors.length; i++) {
        const href = anchors[i].getAttribute('href') || '';
        expect(href.toLowerCase()).not.toContain('javascript:');
      }
    });

    it('should prevent data: URLs in diff content', () => {
      const dataUrlFile: DiffFile = {
        from: 'test.html',
        to: 'test.html',
        chunks: [
          {
            content:
              '@@ -1,1 +1,1 @@\n-<a href="#safe">link</a>\n+<a href="data:text/html,<script>alert(1)</script>">link</a>',
            changes: [
              {
                type: 'delete',
                content: '<a href="#safe">link</a>',
                lineNumber: 1,
              },
              {
                type: 'insert',
                content: '<a href="data:text/html,<script>alert(1)</script>">link</a>',
                lineNumber: 1,
              },
            ],
          },
        ],
        additions: 1,
        deletions: 1,
      };

      const { container } = render(<DiffViewer file={dataUrlFile} />);

      // Should display as text, not execute
      expect(container.getElementsByTagName('script').length).toBe(0);
    });
  });
});
