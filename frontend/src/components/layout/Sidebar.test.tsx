import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { render } from '../../../tests/setup/test-utils';
import Sidebar from './Sidebar';

describe('Sidebar', () => {
  it('renders the build-time app version beside the brand title', () => {
    render(<Sidebar />);

    // displayVersion() is fed by the Vite `define` constants; under the test
    // runner that yields the DEV pre-release form vX.Y.Z-dev+<sha>.
    expect(screen.getByTestId('app-version').textContent).toMatch(/^v\d+\.\d+\.\d+/);
  });

  it('places the version next to the app name in the brand header', () => {
    render(<Sidebar />);

    // The version sits in the same baseline-aligned group as the "Ampel" title.
    const version = screen.getByTestId('app-version');
    const brand = screen.getByText('Ampel').parentElement;
    expect(brand).toContainElement(version);
  });
});
