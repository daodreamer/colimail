/**
 * Vitest setup file
 * Configure global test environment and mocks
 */

import { vi } from 'vitest';

// Mock Tauri API
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Export for use in tests
export { mockInvoke };
