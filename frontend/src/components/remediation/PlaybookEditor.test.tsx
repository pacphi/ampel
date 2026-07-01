import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PlaybookEditor } from './PlaybookEditor';
import type { Playbook } from '@/types/playbook';

// Passthrough i18n: t() returns the key so assertions are stable.
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockCreate = vi.fn();
const mockDelete = vi.fn();
const mockPreview = vi.fn();
const mockLoadDefault = vi.fn();
let playbooksData: Playbook[] = [];

vi.mock('@/hooks/usePlaybooks', () => ({
  usePlaybooks: () => ({ data: playbooksData, isLoading: false, isError: false }),
  useCreatePlaybook: () => ({ mutate: mockCreate, isPending: false }),
  useDeletePlaybook: () => ({ mutate: mockDelete, isPending: false }),
  usePreviewPlaybook: () => ({ mutate: mockPreview, isPending: false }),
  useLoadEmbeddedPlaybook: () => ({ mutate: mockLoadDefault, isPending: false }),
}));

const samplePlaybook: Playbook = {
  id: 'pb-1',
  playbookId: 'custom-remediation',
  version: 1,
  source: 'db',
  name: 'Custom Remediation',
  description: null,
  content: 'role: fixer',
  enabled: true,
  scopeType: 'user',
  scopeId: 'user-1',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

beforeEach(() => {
  vi.clearAllMocks();
  playbooksData = [];
});

describe('PlaybookEditor', () => {
  it('should_renderEmptyState_when_noPlaybooks', () => {
    render(<PlaybookEditor />);
    expect(screen.getByText('remediation:playbooks.empty')).toBeInTheDocument();
  });

  it('should_renderPlaybookList_when_playbooksExist', () => {
    playbooksData = [samplePlaybook];
    render(<PlaybookEditor />);
    expect(screen.getByText('Custom Remediation')).toBeInTheDocument();
  });

  it('should_showRenderedPrompt_when_previewSucceeds', async () => {
    playbooksData = [samplePlaybook];
    mockPreview.mockImplementation((_vars, options) => {
      options.onSuccess({
        failureClass: 'build_error',
        role: 'fixer',
        systemInstruction: 'You are a careful build-fixing agent.',
        outputContract: 'unified_diff',
        allowedTools: ['read_file'],
      });
    });
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.preview' }));

    expect(await screen.findByText('You are a careful build-fixing agent.')).toBeInTheDocument();
  });

  it('should_renderCapabilityPills_when_previewSucceeds', async () => {
    playbooksData = [samplePlaybook];
    mockPreview.mockImplementation((_vars, options) => {
      options.onSuccess({
        failureClass: 'build_error',
        role: 'fixer',
        systemInstruction: 'You are a careful build-fixing agent.',
        outputContract: 'unified_diff',
        allowedTools: ['read_file', 'apply_patch'],
      });
    });
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.preview' }));

    // The output-contract value and each allowed tool render as their own pills.
    expect(await screen.findByText(/unified_diff/)).toBeInTheDocument();
    expect(screen.getByText('read_file')).toBeInTheDocument();
    expect(screen.getByText('apply_patch')).toBeInTheDocument();
  });

  it('should_prefillFormWithDefaultYaml_when_loadDefaultClicked', async () => {
    mockLoadDefault.mockImplementation((_vars, options) => {
      options.onSuccess({
        id: '00000000-0000-0000-0000-000000000000',
        playbookId: 'default',
        version: 1,
        source: 'builtin',
        name: 'Default remediation playbook',
        description: 'builtin default',
        content: 'role: You are a remediation engineer.\ntasks: {}',
        enabled: true,
        scopeType: 'global',
        scopeId: null,
        createdAt: '1970-01-01T00:00:00+00:00',
        updatedAt: '1970-01-01T00:00:00+00:00',
      });
    });
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.loadDefault' }));

    // The form opens prefilled with the default YAML but a blank id/name (a
    // sanitized copy that never collides with the built-in).
    const content = await screen.findByLabelText('remediation:playbooks.content');
    expect(content).toHaveValue('role: You are a remediation engineer.\ntasks: {}');
    expect(screen.getByLabelText('remediation:playbooks.playbookId')).toHaveValue('');
    expect(screen.getByLabelText('remediation:playbooks.name')).toHaveValue('');
  });

  it('should_prefillFormFromRow_when_duplicateClicked', async () => {
    playbooksData = [samplePlaybook];
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.duplicate' }));

    expect(await screen.findByLabelText('remediation:playbooks.content')).toHaveValue(
      'role: fixer'
    );
    expect(screen.getByLabelText('remediation:playbooks.playbookId')).toHaveValue(
      'custom-remediation-copy'
    );
    expect(screen.getByLabelText('remediation:playbooks.name')).toHaveValue('Custom Remediation');
  });

  it('should_showError_when_previewFailsWithYamlError', async () => {
    playbooksData = [samplePlaybook];
    mockPreview.mockImplementation((_vars, options) => {
      options.onError({ response: { data: { error: 'invalid playbook YAML: bad indent' } } });
    });
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.preview' }));

    expect(await screen.findByText('invalid playbook YAML: bad indent')).toBeInTheDocument();
  });

  it('should_blockSave_when_requiredFieldsMissing', async () => {
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.create' }));
    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.save' }));

    expect(screen.getByText('remediation:playbooks.errors.playbookIdRequired')).toBeInTheDocument();
    expect(mockCreate).not.toHaveBeenCalled();
  });

  it('should_submitCreate_when_formFilled', async () => {
    const user = userEvent.setup();
    render(<PlaybookEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.create' }));
    await user.type(screen.getByLabelText('remediation:playbooks.playbookId'), 'my-pb');
    await user.type(screen.getByLabelText('remediation:playbooks.name'), 'My Playbook');
    await user.type(screen.getByLabelText('remediation:playbooks.content'), 'role: fixer');
    await user.click(screen.getByRole('button', { name: 'remediation:playbooks.save' }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalled());
    expect(mockCreate).toHaveBeenCalledWith(
      expect.objectContaining({
        playbookId: 'my-pb',
        name: 'My Playbook',
        content: 'role: fixer',
      }),
      expect.anything()
    );
  });
});
