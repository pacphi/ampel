/**
 * BDD Test Helpers
 *
 * Provides Given/When/Then structure for behavior-driven test organization.
 * Makes tests more readable and aligns with user story acceptance criteria.
 *
 * NOTE: Function names are capitalized to avoid conflicts with Promise.prototype.then
 *
 * @example
 * ```typescript
 * import { Feature, Scenario, Given, When, Then } from '@/tests/setup/bdd-helpers';
 *
 * Feature('Dashboard', () => {
 *   Scenario('User views repository status', () => {
 *     Given('user is authenticated', async () => {
 *       setupAuthenticatedUser();
 *     });
 *
 *     When('dashboard loads', async () => {
 *       render(<Dashboard />);
 *       await waitForLoadingToFinish();
 *     });
 *
 *     Then('repositories are displayed with status indicators', () => {
 *       expect(screen.getByText('ampel-frontend')).toBeInTheDocument();
 *       expect(screen.getByTestId('status-green')).toBeInTheDocument();
 *     });
 *   });
 * });
 * ```
 */

// ============================================================================
// Types
// ============================================================================

/**
 * Step function that can be sync or async
 */
type StepFn = () => void | Promise<void>;

/**
 * Test context shared between steps
 */
export interface TestContext {
  [key: string]: unknown;
}

// ============================================================================
// Global accessor helpers
// ============================================================================

// Access vitest globals at runtime through globalThis
// These are only available when tests are actually running
interface VitestGlobals {
  describe: (name: string, fn: () => void) => void;
  it: ((name: string, fn: () => void | Promise<void>) => void) & {
    skip: (name: string, fn: () => void | Promise<void>) => void;
    only: (name: string, fn: () => void | Promise<void>) => void;
  };
  beforeEach: (fn: () => void | Promise<void>) => void;
}

function getGlobals(): VitestGlobals {
  const g = globalThis as unknown as VitestGlobals;
  if (!g.describe || !g.it || !g.beforeEach) {
    throw new Error(
      'BDD helpers must be used within a vitest test context. ' +
        'Make sure globals: true is set in vitest.config.ts'
    );
  }
  return g;
}

// ============================================================================
// Core BDD Functions
// ============================================================================

/**
 * Define a feature (test suite).
 * Wraps `describe` with feature-focused naming.
 *
 * @example
 * ```typescript
 * Feature('Authentication', () => {
 *   // scenarios...
 * });
 * ```
 */
export function Feature(name: string, fn: () => void): void {
  getGlobals().describe(`Feature: ${name}`, fn);
}

/**
 * Define a scenario (test case).
 * Wraps `it` with scenario-focused naming.
 *
 * @example
 * ```typescript
 * Scenario('User logs in with valid credentials', () => {
 *   // Given, When, Then...
 * });
 * ```
 */
export function Scenario(name: string, fn: () => void | Promise<void>): void {
  getGlobals().it(`Scenario: ${name}`, fn);
}

/**
 * Define a scenario that should be skipped.
 */
export function xScenario(name: string, fn: () => void | Promise<void>): void {
  getGlobals().it.skip(`Scenario: ${name}`, fn);
}

/**
 * Define a focused scenario (only this will run).
 */
export function fScenario(name: string, fn: () => void | Promise<void>): void {
  getGlobals().it.only(`Scenario: ${name}`, fn);
}

/**
 * Given step - establishes the initial context.
 * Use for setup, authentication state, initial data.
 *
 * @example
 * ```typescript
 * await Given('user is logged in', async () => {
 *   setupAuthenticatedUser();
 *   render(<App />);
 * });
 * ```
 */
export async function Given(description: string, fn: StepFn): Promise<void> {
  // Log step for debugging
  if (process.env.DEBUG_BDD) {
    console.log(`  Given ${description}`);
  }
  await fn();
}

/**
 * When step - describes the action being tested.
 * Use for user interactions, API calls, events.
 *
 * @example
 * ```typescript
 * await When('user clicks login button', async () => {
 *   await user.click(screen.getByRole('button', { name: /login/i }));
 * });
 * ```
 */
export async function When(description: string, fn: StepFn): Promise<void> {
  if (process.env.DEBUG_BDD) {
    console.log(`  When ${description}`);
  }
  await fn();
}

/**
 * Then step - describes the expected outcome.
 * Use for assertions, verification.
 *
 * @example
 * ```typescript
 * await Then('dashboard is displayed', () => {
 *   expect(screen.getByText('Dashboard')).toBeInTheDocument();
 * });
 * ```
 */
export async function Then(description: string, fn: StepFn): Promise<void> {
  if (process.env.DEBUG_BDD) {
    console.log(`  Then ${description}`);
  }
  await fn();
}

/**
 * And step - additional step that continues the previous type.
 * Use for additional setup, actions, or assertions.
 *
 * @example
 * ```typescript
 * await Given('user is authenticated', () => {...});
 * await And('has repositories configured', () => {...});
 * ```
 */
export async function And(description: string, fn: StepFn): Promise<void> {
  if (process.env.DEBUG_BDD) {
    console.log(`  And ${description}`);
  }
  await fn();
}

/**
 * But step - describes a contrasting condition.
 * Use for exceptions, negative cases.
 *
 * @example
 * ```typescript
 * await Then('form is valid', () => {...});
 * await But('submit button is disabled until all required fields are filled', () => {...});
 * ```
 */
export async function But(description: string, fn: StepFn): Promise<void> {
  if (process.env.DEBUG_BDD) {
    console.log(`  But ${description}`);
  }
  await fn();
}

// ============================================================================
// Lowercase aliases (use with caution - 'then' may conflict with Promise.then)
// ============================================================================

// Export lowercase aliases for convenience
// Note: 'then' is intentionally NOT exported as lowercase to avoid Promise conflicts
export { Given as given, When as when, And as and, But as but };

// ============================================================================
// Background Support
// ============================================================================

/**
 * Define background steps that run before each scenario.
 * Use for common setup across multiple scenarios.
 *
 * @example
 * ```typescript
 * Feature('Dashboard', () => {
 *   Background(() => {
 *     Given('user is authenticated', setupAuthenticatedUser);
 *   });
 *
 *   Scenario('View repositories', () => {...});
 *   Scenario('Filter by status', () => {...});
 * });
 * ```
 */
export function Background(fn: () => void | Promise<void>): void {
  getGlobals().beforeEach(async () => {
    if (process.env.DEBUG_BDD) {
      console.log('  Background:');
    }
    await fn();
  });
}

// ============================================================================
// Scenario Outline Support
// ============================================================================

/**
 * Define a parameterized scenario outline.
 * Runs the same scenario with different data sets.
 *
 * @example
 * ```typescript
 * ScenarioOutline(
 *   'User filters PRs by status',
 *   [
 *     { filter: 'green', expectedCount: 2 },
 *     { filter: 'yellow', expectedCount: 3 },
 *     { filter: 'red', expectedCount: 1 },
 *   ],
 *   async ({ filter, expectedCount }) => {
 *     await When(`user selects ${filter} filter`, async () => {
 *       await user.click(screen.getByRole('button', { name: filter }));
 *     });
 *
 *     await Then(`${expectedCount} PRs are displayed`, () => {
 *       expect(screen.getAllByTestId('pr-card')).toHaveLength(expectedCount);
 *     });
 *   }
 * );
 * ```
 */
export function ScenarioOutline<T extends Record<string, unknown>>(
  name: string,
  examples: T[],
  fn: (example: T) => void | Promise<void>
): void {
  const { describe: describeFunc, it: itFunc } = getGlobals();

  describeFunc(`Scenario Outline: ${name}`, () => {
    examples.forEach((example, index) => {
      const exampleName = Object.entries(example)
        .map(([key, value]) => `${key}=${value}`)
        .join(', ');

      itFunc(`Example ${index + 1}: ${exampleName}`, async () => {
        await fn(example);
      });
    });
  });
}

// ============================================================================
// Test Data Helpers
// ============================================================================

/**
 * Create a test data table for scenario outlines.
 *
 * @example
 * ```typescript
 * const loginCases = dataTable([
 *   ['email', 'password', 'expectedResult'],
 *   ['valid@test.com', 'password123', 'success'],
 *   ['invalid@test.com', 'wrong', 'error'],
 *   ['', 'password', 'validation_error'],
 * ]);
 *
 * ScenarioOutline('Login attempts', loginCases, async ({ email, password, expectedResult }) => {
 *   // test implementation
 * });
 * ```
 */
export function dataTable<T extends string>(rows: [T[], ...unknown[][]]): Record<T, unknown>[] {
  const [headers, ...data] = rows;

  return data.map((row) =>
    headers.reduce(
      (acc, header, index) => ({
        ...acc,
        [header]: row[index],
      }),
      {} as Record<T, unknown>
    )
  );
}

// ============================================================================
// Step Definitions (Reusable)
// ============================================================================

/**
 * Registry for reusable step definitions.
 * Allows defining steps once and reusing across tests.
 */
class StepRegistry {
  private steps: Map<string, StepFn> = new Map();

  /**
   * Define a reusable step
   */
  define(pattern: string, fn: StepFn): void {
    this.steps.set(pattern, fn);
  }

  /**
   * Get a defined step
   */
  get(pattern: string): StepFn | undefined {
    return this.steps.get(pattern);
  }

  /**
   * Clear all defined steps
   */
  clear(): void {
    this.steps.clear();
  }
}

export const stepRegistry = new StepRegistry();

/**
 * Define a reusable Given step
 */
export function defineGiven(pattern: string, fn: StepFn): void {
  stepRegistry.define(`given:${pattern}`, fn);
}

/**
 * Define a reusable When step
 */
export function defineWhen(pattern: string, fn: StepFn): void {
  stepRegistry.define(`when:${pattern}`, fn);
}

/**
 * Define a reusable Then step
 */
export function defineThen(pattern: string, fn: StepFn): void {
  stepRegistry.define(`then:${pattern}`, fn);
}

/**
 * Use a predefined Given step
 */
export async function useGiven(pattern: string): Promise<void> {
  const step = stepRegistry.get(`given:${pattern}`);
  if (!step) {
    throw new Error(`No step defined for: Given ${pattern}`);
  }
  await Given(pattern, step);
}

/**
 * Use a predefined When step
 */
export async function useWhen(pattern: string): Promise<void> {
  const step = stepRegistry.get(`when:${pattern}`);
  if (!step) {
    throw new Error(`No step defined for: When ${pattern}`);
  }
  await When(pattern, step);
}

/**
 * Use a predefined Then step
 */
export async function useThen(pattern: string): Promise<void> {
  const step = stepRegistry.get(`then:${pattern}`);
  if (!step) {
    throw new Error(`No step defined for: Then ${pattern}`);
  }
  await Then(pattern, step);
}

// ============================================================================
// Cleanup
// ============================================================================

/**
 * Reset all BDD state between tests
 */
export function resetBDDState(): void {
  stepRegistry.clear();
}
