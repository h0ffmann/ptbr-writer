// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { normalizeHost } from '../src/hosts';
import { render as renderOptions } from '../src/options';
import { render as renderPopup } from '../src/popup';
import { maskKey, resolveKey } from '../src/sites';
import { loadSettings } from '../src/settings';

let store: Record<string, unknown>;
let granted: boolean;
let tabUrl: string | undefined;
const send = vi.fn();
const request = vi.fn();
const remove = vi.fn();

beforeEach(() => {
  store = {};
  granted = true;
  tabUrl = 'https://example.com/x';
  send.mockReset();
  request.mockReset().mockImplementation(async () => granted);
  remove.mockReset().mockResolvedValue(true);
  (globalThis as any).chrome = {
    storage: { local: { get: async () => store, set: async (o: object) => void Object.assign(store, o) } },
    runtime: { sendMessage: send },
    permissions: { request, remove },
    tabs: { query: async () => [{ url: tabUrl }] },
  };
  document.body.innerHTML = '<div id="app"></div>';
});

const root = () => document.getElementById('app') as HTMLElement;
const flush = () => new Promise((r) => setTimeout(r, 0));
const addSite = async (text: string) => {
  (document.querySelector('#new-host') as HTMLInputElement).value = text;
  (document.querySelector('#add-site') as HTMLButtonElement).click();
  await flush();
};

describe('normalizeHost', () => {
  it('accepts hosts, urls and mixed case', () => {
    expect(normalizeHost('example.com')).toBe('example.com');
    expect(normalizeHost('https://example.com/path?q=1')).toBe('example.com');
    expect(normalizeHost('  EXAMPLE.com ')).toBe('example.com');
  });
  it('rejects wildcard, port, path-only, ip, localhost, empty, other schemes', () => {
    for (const bad of ['*.example.com', 'example.com:8080', 'https://example.com:443', '/path', '10.0.0.1',
      '[::1]', 'localhost', '', '   ', 'ftp://example.com', 'a b.com', 'user@example.com'])
      expect(normalizeHost(bad), bad).toBeNull();
  });
});

describe('keys', () => {
  it('masks and keeps on empty or masked input', () => {
    expect(maskKey('sk-test-1234')).toBe('••••1234');
    expect(resolveKey('sk-old', '')).toBe('sk-old');
    expect(resolveKey('sk-old', '••••1234')).toBe('sk-old');
    expect(resolveKey('sk-old', 'sk-new')).toBe('sk-new');
  });
  it('shows the mask after saving', async () => {
    await renderOptions(root());
    (document.querySelector('#key') as HTMLInputElement).value = 'sk-test-1234';
    (document.querySelector('#save') as HTMLButtonElement).click();
    await flush();
    expect((document.querySelector('#key') as HTMLInputElement).value).toBe('••••1234');
    expect((await loadSettings()).keys.claude).toBe('sk-test-1234');
  });
});

describe('provider, key and model', () => {
  const $ = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T;
  const choose = (provider: string) => {
    $<HTMLSelectElement>('#provider').value = provider;
    $<HTMLSelectElement>('#provider').dispatchEvent(new Event('change'));
  };
  const modelOptions = () => [...$<HTMLSelectElement>('#model').options].map((o) => o.value);

  it('shows one key field and one model list, for the chosen provider', async () => {
    await renderOptions(root());
    expect(document.querySelector('#key-claude')).toBeNull();
    expect(document.querySelector('#key-deepseek')).toBeNull();
    expect($('#model').tagName).toBe('SELECT');
    expect(modelOptions()).toEqual(['claude-haiku-5-5', 'claude-sonnet-5-5', 'claude-opus-5-5']);
    expect($<HTMLSelectElement>('#model').value).toBe('claude-haiku-5-5');
  });

  it('switching the provider shows its own key and models', async () => {
    store = { settings: { keys: { claude: 'sk-c-1111', deepseek: 'sk-d-2222' } } };
    await renderOptions(root());
    expect($<HTMLInputElement>('#key').value).toBe('••••1111');
    choose('deepseek');
    expect($<HTMLInputElement>('#key').value).toBe('••••2222');
    expect(modelOptions()).toEqual(['deepseek-flash', 'deepseek-v4-pro']);
    expect($<HTMLSelectElement>('#model').value).toBe('deepseek-flash');
  });

  it('saving writes only the chosen provider and keeps the other key', async () => {
    store = { settings: { keys: { claude: 'sk-c-1111', deepseek: '' } } };
    await renderOptions(root());
    choose('deepseek');
    $<HTMLInputElement>('#key').value = 'sk-new-9999';
    $<HTMLSelectElement>('#model').value = 'deepseek-v4-pro';
    $<HTMLButtonElement>('#save').click();
    await flush();
    const s = await loadSettings();
    expect(s.provider).toBe('deepseek');
    expect(s.keys).toEqual({ claude: 'sk-c-1111', deepseek: 'sk-new-9999' });
    expect(s.models.deepseek).toBe('deepseek-v4-pro');
    expect(s.models.claude).toBe('claude-haiku-5-5');
  });
});

describe('add site', () => {
  it('denied permission: nothing saved, no message', async () => {
    granted = false;
    await renderOptions(root());
    await addSite('example.com');
    expect(request).toHaveBeenCalledWith({ origins: ['https://example.com/*'] });
    expect((await loadSettings()).sites['example.com']).toBeUndefined();
    expect(send).not.toHaveBeenCalled();
  });
  it('granted: saved as descritivo and enable-site sent', async () => {
    await renderOptions(root());
    await addSite('https://Example.com/a');
    expect((await loadSettings()).sites['example.com']).toEqual({ enabled: true, level: 'descritivo' });
    expect(send).toHaveBeenCalledWith({ type: 'enable-site', host: 'example.com' });
  });
  it('invalid host: inline error, nothing happens', async () => {
    await renderOptions(root());
    await addSite('*.example.com');
    expect(request).not.toHaveBeenCalled();
    expect(document.querySelector('#site-error')!.textContent).not.toBe('');
  });
  it('default sites toggle without permissions or messages and cannot be removed', async () => {
    await renderOptions(root());
    const row = document.querySelector('tr[data-host="github.com"]')!;
    expect(row.querySelector('button')).toBeNull();
    (row.querySelector('input[type=checkbox]') as HTMLInputElement).click();
    await flush();
    expect((await loadSettings()).sites['github.com'].enabled).toBe(false);
    expect(request).not.toHaveBeenCalled();
    expect(remove).not.toHaveBeenCalled();
    expect(send).not.toHaveBeenCalled();
  });
  it('remove: permission removed, disable-site sent, entry dropped', async () => {
    await renderOptions(root());
    await addSite('example.com');
    send.mockClear();
    (document.querySelector('tr[data-host="example.com"] button') as HTMLButtonElement).click();
    await flush();
    expect(remove).toHaveBeenCalledWith({ origins: ['https://example.com/*'] });
    expect(send).toHaveBeenCalledWith({ type: 'disable-site', host: 'example.com' });
    expect((await loadSettings()).sites['example.com']).toBeUndefined();
  });
});

describe('key fields', () => {
  it('are password inputs', async () => {
    await renderOptions(root());
    expect((document.querySelector('#key') as HTMLInputElement).type).toBe('password');
  });
});

describe('popup', () => {
  it('offers Ativar and follows the permission flow', async () => {
    await renderPopup(root());
    const b = document.querySelector('#toggle-site') as HTMLButtonElement;
    expect(b.textContent).toBe('Ativar neste site');
    b.click();
    await flush();
    expect(send).toHaveBeenCalledWith({ type: 'enable-site', host: 'example.com' });
    expect(document.querySelector('#toggle-site')!.textContent).toBe('Desativar neste site');
  });
  it('http, localhost and IP pages are not supported', async () => {
    for (const url of ['http://example.com/', 'https://localhost/', 'https://192.168.0.1/', 'http://127.0.0.1:8080/']) {
      tabUrl = url;
      await renderPopup(root());
      expect(document.querySelector('#toggle-site'), url).toBeNull();
      expect(root().textContent, url).toContain('Site não suportado.');
    }
  });
  it('always offers Configurações, which opens the options page', async () => {
    const openOptionsPage = vi.fn();
    (globalThis as any).chrome.runtime.openOptionsPage = openOptionsPage;
    for (const url of ['https://example.com/x', 'chrome://extensions', undefined]) {
      tabUrl = url;
      await renderPopup(root());
      const b = document.querySelector('#open-options') as HTMLButtonElement;
      expect(b?.textContent, String(url)).toBe('Configurações');
    }
    (document.querySelector('#open-options') as HTMLButtonElement).click();
    expect(openOptionsPage).toHaveBeenCalledTimes(1);
  });
  it('says when the chosen provider has no key', async () => {
    await renderPopup(root());
    expect(root().textContent).toContain('Sem chave do Claude.');
    store = { settings: { keys: { claude: 'sk-test-1234', deepseek: '' } } };
    await renderPopup(root());
    expect(root().textContent).not.toContain('Sem chave');
  });
  it('no button on non-http pages or without url', async () => {
    tabUrl = 'chrome://extensions';
    await renderPopup(root());
    expect(document.querySelector('#toggle-site')).toBeNull();
    tabUrl = undefined;
    await renderPopup(root());
    expect(document.querySelector('#toggle-site')).toBeNull();
  });
});
