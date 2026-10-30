import { chromium, expect, test as base, type BrowserContext, type Page, type Worker } from '@playwright/test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { startFakeServer, type FakeServer } from './fake-server';

export { expect };

const EXTENSION = join(dirname(fileURLToPath(import.meta.url)), '..', 'dist-test');
export const TYPED = 'Foi decidido que o deploy vai ser na sexta.';
export const FIXED = 'A equipe decidiu que o deploy vai ser na sexta.';
// How long the pipeline (debounce, service worker, fetch, render) may take before a test fails. The
// wait ends as soon as the underline shows, so a fast run stays fast; a loaded machine (CI, parallel
// builds) can take several seconds, and 5 s made the first test flaky.
export const UI_TIMEOUT = 20_000;
// Longer than the watcher's debounce (1 s): after it, a check that was going to happen has happened.
export const SETTLE = 2_000;

export type ProviderId = 'claude' | 'deepseek' | 'ollama' | 'ollaya';
export type Level = 'estrito' | 'descritivo' | 'leve' | 'científico';
export interface SiteConfig {
  enabled: boolean;
  level: Level;
}
/** The stored shape of the settings (settings.rs, `Settings`), every field optional. */
export interface Settings {
  enabled: boolean;
  provider: ProviderId;
  keys: Record<ProviderId, string>;
  models: Partial<Record<ProviderId, string>>;
  ollayaStyle: boolean;
  sites: Record<string, SiteConfig>;
  tone: 'informal' | 'neutro' | 'formal';
  rewrite: boolean;
  neverFix: string[];
  delayMs: 1000 | 2000 | 3000;
  apiBase: Partial<Record<ProviderId, string>>;
}

/** A Chromium with the extension, on a fresh profile; its settings written before any page loads. */
export class Ext {
  context!: BrowserContext;
  worker!: Worker;
  id!: string;
  private profile!: string;

  constructor(readonly server: FakeServer) {}

  async launch(overrides: Partial<Settings> = {}): Promise<this> {
    this.profile = mkdtempSync(join(tmpdir(), 'ptbr-e2e-'));
    this.context = await chromium.launchPersistentContext(this.profile, {
      channel: 'chromium',
      headless: true,
      args: [
        `--disable-extensions-except=${EXTENSION}`,
        `--load-extension=${EXTENSION}`,
        // No DNS: a test never leaves the machine, and an https host fails fast with an error page.
        '--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1, EXCLUDE localhost',
      ],
    });
    this.worker = this.context.serviceWorkers()[0] ?? (await this.context.waitForEvent('serviceworker'));
    this.id = new URL(this.worker.url()).host;
    const base = this.server.base;
    await this.setSettings({
      enabled: true,
      provider: 'claude',
      keys: { claude: 'sk-test', deepseek: 'sk-test', ollama: '', ollaya: '' },
      apiBase: { claude: base, deepseek: base, ollama: base, ollaya: base },
      sites: { '127.0.0.1': { enabled: true, level: 'descritivo' } },
      ...overrides,
    });
    return this;
  }

  async close() {
    await this.context?.close();
    if (this.profile) rmSync(this.profile, { recursive: true, force: true });
  }

  async settings(): Promise<Settings> {
    return (await this.worker.evaluate(() => chrome.storage.local.get('settings'))).settings as Settings;
  }

  /** Replaces the stored settings; a content script follows the change through storage.onChanged. */
  async setSettings(patch: Partial<Settings>) {
    await this.worker.evaluate(async (p) => {
      const { settings } = await chrome.storage.local.get('settings');
      await chrome.storage.local.set({ settings: { ...(settings ?? {}), ...p } });
    }, patch);
  }

  async registered(): Promise<string[]> {
    return this.worker.evaluate(() => chrome.scripting.getRegisteredContentScripts().then((s) => s.map((x) => x.id)));
  }

  // Opens a page of the fake server and waits until the content script listens to it (its module
  // loaded, its settings read, its listeners on): typing earlier would go unseen. The wait runs in
  // the script's own world, found through CDP, on the promise content.js leaves there.
  async open(page: string, host = '127.0.0.1'): Promise<Page> {
    const p = await this.context.newPage();
    const cdp = await this.context.newCDPSession(p);
    const world = new Promise<number>((ok) =>
      cdp.on('Runtime.executionContextCreated', ({ context: c }) => {
        if (c.name === 'gramatico') ok(c.id);
      }),
    );
    await cdp.send('Runtime.enable');
    await p.goto(`http://${host}:${this.server.port}/${page}`);
    await cdp.send('Runtime.evaluate', {
      contextId: await world,
      expression: 'globalThis.ptbrWriter.then(() => true)',
      awaitPromise: true,
    });
    await cdp.detach();
    return p;
  }

  /** An extension page (options.html, popup.html) in a tab. */
  async page(file: string): Promise<Page> {
    const p = await this.context.newPage();
    await p.goto(`chrome-extension://${this.id}/${file}`);
    return p;
  }

  /** The popup rendered for a tab at `url` (an https host that never resolves: its error page keeps the URL). */
  async popupOn(url: string): Promise<Page> {
    const tab = await this.context.newPage();
    await tab.goto(url).catch(() => {});
    const popup = await this.page('popup.html');
    await tab.bringToFront();
    // The popup reads the active tab: wait until Chrome reports this one, or it may render for itself.
    const host = new URL(url).host;
    await expect
      .poll(() => this.worker.evaluate(async () => (await chrome.tabs.query({ active: true, currentWindow: true }))[0]?.url ?? ''))
      .toContain(host);
    await popup.reload();
    return popup;
  }
}

export const test = base.extend<{ ext: Ext }, { server: FakeServer }>({
  server: [
    async ({}, use) => {
      const server = await startFakeServer();
      await use(server);
      await server.close();
    },
    { scope: 'worker' },
  ],
  ext: async ({ server }, use) => {
    server.reset();
    const ext = new Ext(server);
    await use(ext);
    await ext.close();
  },
});

/** Types into the field and waits for its underline. */
export async function typeAndWait(page: Page, text = TYPED, selector = '#field') {
  await page.locator(selector).click();
  await page.keyboard.type(text);
  await expect(page.locator('.ptbr-underline')).not.toHaveCount(0, { timeout: UI_TIMEOUT });
}

/** Opens the card of the `i`-th underline and clicks Aceitar. */
export async function accept(page: Page, i = 0) {
  await page.locator(`.ptbr-underline[data-i="${i}"] .ptbr-hit`).first().click();
  await page.locator('.ptbr-card [data-action=accept]').click();
}
