import { defineConfig } from '@playwright/test';

// The extension is loaded from dist-test (`cargo xtask build --test`); each test launches its own Chromium.
export default defineConfig({
  testDir: 'e2e',
  globalSetup: './e2e/global-setup.ts',
  timeout: 60_000,
  workers: 1,
  retries: 0,
  reporter: process.env.CI ? 'github' : 'list',
});
