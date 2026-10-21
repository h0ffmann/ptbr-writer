import { chromium, expect, test, type BrowserContext, type Page } from '@playwright/test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { ProviderId, Settings } from '../src/types';
import { startFakeServer, type FakeServer } from './fake-server';

const EXTENSION = join(dirname(fileURLToPath(import.meta.url)), '..', 'dist-test');
const TYPED = 'Foi decidido que o deploy vai ser na sexta.';
const FIXED = 'A equipe decidiu que o deploy vai ser na sexta.';
// How long the pipeline (debounce, service worker, fetch, render) may take before a test fails. The
// wait ends as soon as the underline shows, so a fast run stays fast; a loaded machine (CI, parallel
// builds) can take several seconds, and 5 s made the first test flaky.
const UI_TIMEOUT = 20_000;

let server: FakeServer;
// Module state, one test at a time: playwright.config.ts sets workers: 1.
let context: BrowserContext | undefined;
let profile: string | undefined;

test.beforeAll(async () => {
  server = await startFakeServer();
});
test.afterAll(async () => {
  await server.close();
});
test.afterEach(async () => {
  await context?.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
  context = undefined;
  profile = undefined;
});

// A fresh profile with the extension, and its settings written before any page loads:
// the content script reads them at load (and again when they change).
async function launch(provider: ProviderId): Promise<BrowserContext> {
  profile = mkdtempSync(join(tmpdir(), 'ptbr-e2e-'));
  context = await chromium.launchPersistentContext(profile, {
    channel: 'chromium',
    headless: true,
    args: [`--disable-extensions-except=${EXTENSION}`, `--load-extension=${EXTENSION}`],
  });
  const worker = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
  const base = `http://127.0.0.1:${server.port}`;
  const settings: Partial<Settings> = {
    enabled: true,
    provider,
    keys: { claude: 'sk-test', deepseek: 'sk-test' },
    apiBase: { claude: base, deepseek: base },
    sites: { '127.0.0.1': { enabled: true, level: 'descritivo' } },
  };
  await worker.evaluate((s) => chrome.storage.local.set({ settings: s }), settings);
  return context;
}

// Opens a page and waits until the content script has read its settings: the script reads them
// first, then listens for focus, so typing earlier would go unseen. The wait runs in the script's
// own world (found through CDP) and queues a storage read behind the script's own.
async function open(host: string, page: string): Promise<Page> {
  if (!context) throw new Error('launch() first');
  const p = await context.newPage();
  const cdp = await context.newCDPSession(p);
  const world = new Promise<number>((ok) =>
    cdp.on('Runtime.executionContextCreated', ({ context: c }) => {
      if (c.name === 'ptbr-writer') ok(c.id);
    }),
  );
  await cdp.send('Runtime.enable');
  await p.goto(`http://${host}:${server.port}/${page}`);
  await cdp.send('Runtime.evaluate', {
    contextId: await world,
    expression: "chrome.storage.local.get('settings').then(() => true)",
    awaitPromise: true,
  });
  await cdp.detach();
  return p;
}

async function accept(page: Page) {
  const underline = page.locator('.ptbr-underline-erro');
  await expect(underline).toHaveCount(1, { timeout: UI_TIMEOUT });
  await underline.locator('.ptbr-hit').first().click();
  await page.locator('.ptbr-card [data-action=accept]').click();
}

for (const provider of ['claude', 'deepseek'] as const) {
  test(`${provider}: textarea, sublinha, aceita`, async () => {
    await launch(provider);
    const page = await open('127.0.0.1', 'textarea.html');
    await page.locator('#field').click();
    await page.keyboard.type(TYPED);
    await accept(page);
    await expect(page.locator('#field')).toHaveValue(FIXED);
    const path = provider === 'claude' ? '/v1/messages' : '/chat/completions';
    expect(server.calls.some((c) => c.path === path)).toBe(true);
  });
}

test('contenteditable estilo Gmail', async () => {
  await launch('claude');
  const page = await open('127.0.0.1', 'editable.html');
  await page.locator('#reply').click();
  await page.keyboard.type(TYPED);
  await accept(page);
  await expect(page.locator('#reply')).toHaveText(FIXED);
  await expect(page.locator('#quote')).toHaveText('Foi decidido que a reunião é na segunda.');
});

test('site não liberado não chama a API', async () => {
  await launch('claude');
  const before = server.calls.length;
  const page = await open('localhost', 'textarea.html');
  await page.locator('#field').click();
  await page.keyboard.type(TYPED);
  await page.waitForTimeout(2_000);
  await expect(page.locator('.ptbr-underline-erro')).toHaveCount(0);
  expect(server.calls.filter((c) => c.host.startsWith('localhost'))).toEqual([]);
  expect(server.calls.length).toBe(before);
});

test('desligar o site numa aba aberta para as chamadas', async () => {
  await launch('claude');
  const page = await open('127.0.0.1', 'textarea.html');
  await page.locator('#field').click();
  await page.keyboard.type(TYPED);
  await expect(page.locator('.ptbr-underline-erro')).toHaveCount(1, { timeout: UI_TIMEOUT });
  const before = server.calls.length;
  expect(before).toBeGreaterThan(0);
  const worker = context!.serviceWorkers()[0];
  await worker.evaluate(async () => {
    const { settings } = (await chrome.storage.local.get('settings')) as { settings: object };
    await chrome.storage.local.set({
      settings: { ...settings, sites: { '127.0.0.1': { enabled: false, level: 'descritivo' } } },
    });
  });
  await page.keyboard.type(' E mais uma frase foi escrita.');
  await page.waitForTimeout(2_000);
  expect(server.calls.length).toBe(before);
  await expect(page.locator('.ptbr-underline-erro')).toHaveCount(0);
});

test('passar o mouse na palavra abre o cartão', async () => {
  await launch('claude');
  const page = await open('127.0.0.1', 'textarea.html');
  await page.locator('#field').click();
  await page.keyboard.type(TYPED);
  const seg = page.locator('.ptbr-underline-erro .ptbr-seg').first();
  await expect(seg).toHaveCount(1, { timeout: UI_TIMEOUT });
  const box = (await seg.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.locator('.ptbr-card [data-action=accept]').click();
  await expect(page.locator('#field')).toHaveValue(FIXED);
});
