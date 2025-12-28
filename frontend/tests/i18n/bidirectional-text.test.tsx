/**
 * Bidirectional Text (BiDi) Tests
 *
 * Tests handling of mixed LTR and RTL content:
 * - Mixed text rendering
 * - Numbers in RTL text
 * - URLs in RTL text
 * - Punctuation handling
 * - Unicode bidirectional algorithm compliance
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '@/i18n';
import RTLProvider from '@/components/RTLProvider';

function TestWrapper({ children }: { children: React.ReactNode }) {
  return (
    <I18nextProvider i18n={i18n}>
      <RTLProvider>{children}</RTLProvider>
    </I18nextProvider>
  );
}

describe('Bidirectional Text (BiDi) Tests', () => {
  beforeEach(async () => {
    localStorage.clear();
  });

  describe('Mixed LTR and RTL Text', () => {
    it('should render Arabic text with English names correctly', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="mixed">مرحبا بك في Ampel Dashboard</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('mixed');
      expect(text.textContent).toBe('مرحبا بك في Ampel Dashboard');
    });

    it('should render Hebrew text with English terms correctly', async () => {
      await i18n.changeLanguage('he');

      render(
        <TestWrapper>
          <p data-testid="mixed">ברוכים הבאים ל-Ampel Dashboard</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('mixed');
      expect(text.textContent).toBe('ברוכים הבאים ל-Ampel Dashboard');
    });

    it('should handle multiple language switches in same text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="multi">مستودع GitHub repository مع pull requests من developers</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('multi');
      expect(text).toBeInTheDocument();
      expect(text.textContent).toContain('GitHub');
      expect(text.textContent).toContain('مستودع');
    });
  });

  describe('Numbers in RTL Text', () => {
    it('should render Western Arabic numerals in Arabic text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="numbers">لديك 42 طلب سحب مفتوح</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('numbers');
      expect(text.textContent).toContain('42');
    });

    it('should render numbers in Hebrew text', async () => {
      await i18n.changeLanguage('he');

      render(
        <TestWrapper>
          <p data-testid="numbers">יש לך 99 בקשות משיכה פתוחות</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('numbers');
      expect(text.textContent).toContain('99');
    });

    it('should handle decimal numbers in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="decimal">النتيجة: 3.14159</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('decimal');
      expect(text.textContent).toContain('3.14159');
    });

    it('should handle percentages in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="percent">التقدم: 85%</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('percent');
      expect(text.textContent).toContain('85%');
    });

    it('should handle dates in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="date">التاريخ: 2024-12-27</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('date');
      expect(text.textContent).toContain('2024-12-27');
    });
  });

  describe('URLs and Code in RTL Text', () => {
    it('should render URLs correctly in Arabic text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="url">زيارة https://github.com/ampel/ampel للمزيد من المعلومات</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('url');
      expect(text.textContent).toContain('https://github.com/ampel/ampel');
    });

    it('should render URLs correctly in Hebrew text', async () => {
      await i18n.changeLanguage('he');

      render(
        <TestWrapper>
          <p data-testid="url">בקר באתר https://example.com למידע נוסף</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('url');
      expect(text.textContent).toContain('https://example.com');
    });

    it('should render email addresses in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="email">البريد الإلكتروني: user@example.com</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('email');
      expect(text.textContent).toContain('user@example.com');
    });

    it('should render code snippets in RTL text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="code">
            استخدم الأمر <code>git pull</code> لتحديث المستودع
          </p>
        </TestWrapper>
      );

      const text = screen.getByTestId('code');
      expect(text.textContent).toContain('git pull');
    });

    it('should render file paths in RTL text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="path">الملف: /src/components/Dashboard.tsx</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('path');
      expect(text.textContent).toContain('/src/components/Dashboard.tsx');
    });
  });

  describe('Punctuation in RTL', () => {
    it('should handle question marks in Arabic', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="question">هل أنت متأكد؟</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('question');
      expect(text.textContent).toContain('؟');
    });

    it('should handle exclamation marks in Hebrew', async () => {
      await i18n.changeLanguage('he');

      render(
        <TestWrapper>
          <p data-testid="exclaim">תודה רבה!</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('exclaim');
      expect(text.textContent).toContain('!');
    });

    it('should handle parentheses in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="parens">النتيجة (42) من 100</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('parens');
      expect(text.textContent).toContain('(42)');
    });

    it('should handle quotes in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="quotes">قال "مرحبا" وذهب</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('quotes');
      expect(text.textContent).toContain('"مرحبا"');
    });
  });

  describe('Complex BiDi Scenarios', () => {
    it('should handle nested LTR in RTL', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="nested">
            المستخدم <strong>John Doe</strong> قام بإنشاء pull request في{' '}
            <code>feature/rtl-support</code>
          </p>
        </TestWrapper>
      );

      const text = screen.getByTestId('nested');
      expect(text.textContent).toContain('John Doe');
      expect(text.textContent).toContain('feature/rtl-support');
    });

    it('should handle lists with mixed content', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <ul data-testid="list">
            <li>Element 1: قيمة نصية</li>
            <li>Element 2: https://example.com</li>
            <li>Element 3: 42 items</li>
          </ul>
        </TestWrapper>
      );

      const list = screen.getByTestId('list');
      expect(list).toBeInTheDocument();
    });

    it('should handle tables with RTL headers', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <table data-testid="table">
            <thead>
              <tr>
                <th>الاسم</th>
                <th>العدد</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Item 1</td>
                <td>42</td>
              </tr>
            </tbody>
          </table>
        </TestWrapper>
      );

      const table = screen.getByTestId('table');
      expect(table).toBeInTheDocument();
    });
  });

  describe('Unicode BiDi Control Characters', () => {
    it('should handle Left-to-Right Mark (LRM)', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="lrm">النص‎ العربي</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('lrm');
      expect(text).toBeInTheDocument();
    });

    it('should handle Right-to-Left Mark (RLM)', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <p data-testid="rlm">Text‏ with RLM</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('rlm');
      expect(text).toBeInTheDocument();
    });

    it('should handle Left-to-Right Embedding (LRE)', async () => {
      await i18n.changeLanguage('ar');

      // LRE is used to force LTR direction for embedded text
      render(
        <TestWrapper>
          <p data-testid="lre">النص مع embedded text النص</p>
        </TestWrapper>
      );

      const text = screen.getByTestId('lre');
      expect(text).toBeInTheDocument();
    });
  });

  describe('Form Inputs BiDi', () => {
    it('should handle RTL input placeholder', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <input data-testid="input" placeholder="أدخل النص هنا" />
        </TestWrapper>
      );

      const input = screen.getByTestId('input');
      expect(input).toHaveAttribute('placeholder', 'أدخل النص هنا');
    });

    it('should handle mixed content in input values', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <input data-testid="input" defaultValue="النص مع Email: user@example.com" />
        </TestWrapper>
      );

      const input = screen.getByTestId('input') as HTMLInputElement;
      expect(input.value).toContain('user@example.com');
    });

    it('should handle textarea with mixed content', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestWrapper>
          <textarea
            data-testid="textarea"
            defaultValue="السطر الأول مع URL: https://example.com&#10;السطر الثاني مع رقم: 42"
          />
        </TestWrapper>
      );

      const textarea = screen.getByTestId('textarea') as HTMLTextAreaElement;
      expect(textarea.value).toContain('https://example.com');
      expect(textarea.value).toContain('42');
    });
  });
});
