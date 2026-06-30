import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { render } from '../../../tests/setup/test-utils';
import Footer from './Footer';

describe('Footer', () => {
  it('renders the build-time app version', () => {
    render(<Footer />);

    // displayVersion() is fed by the Vite `define` constants; under the test
    // runner that yields the DEV pre-release form vX.Y.Z-dev+<sha>.
    expect(screen.getByTestId('app-version').textContent).toMatch(/^v\d+\.\d+\.\d+/);
  });

  it('places the version inside the footer landmark', () => {
    render(<Footer />);

    // <footer> exposes the implicit `contentinfo` landmark role.
    expect(screen.getByRole('contentinfo')).toContainElement(screen.getByTestId('app-version'));
  });
});
