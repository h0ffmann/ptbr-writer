import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const read = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), 'utf8'));
const run = (...args: string[]) =>
  execFileSync('node', ['build.mjs', ...args], { cwd: new URL('..', import.meta.url), stdio: 'pipe' });

describe('build', () => {
  it('manifest segue a versão do plugin', () => {
    run();
    const plugin = read('../../plugins/ptbr-writer/.claude-plugin/plugin.json');
    const m = read('../dist/manifest.json');
    expect(m.version).toBe(plugin.version);
    expect(m.manifest_version).toBe(3);
    expect(m.permissions).toEqual(['storage', 'scripting', 'activeTab']);
    expect(m.host_permissions).toEqual(['https://api.anthropic.com/*', 'https://api.deepseek.com/*']);
    expect(m.optional_host_permissions).toEqual(['http://127.0.0.1:11434/*', 'http://127.0.0.1:11435/*', '<all_urls>']);
    expect(m.content_scripts[0].matches).toEqual([
      'https://web.whatsapp.com/*',
      'https://mail.google.com/*',
      'https://github.com/*',
    ]);
  });

  it('--test libera 127.0.0.1 e localhost', () => {
    run('--test');
    const m = read('../dist-test/manifest.json');
    expect(m.content_scripts[0].matches).toContain('http://127.0.0.1/*');
    expect(m.content_scripts[0].matches).toContain('http://localhost/*');
    expect(m.host_permissions).toContain('http://127.0.0.1/*');
    expect(m.host_permissions).toContain('http://localhost/*');
  });

  it('--firefox troca o service worker por scripts e tem um id de add-on', () => {
    run('--firefox');
    const m = read('../dist-firefox/manifest.json');
    expect(m.background).toEqual({ scripts: ['background.js'], type: 'module' });
    expect(m.browser_specific_settings.gecko.id).toBe('ptbr-writer@h0ffmann');
    expect(m.content_scripts[0].matches).not.toContain('http://localhost/*');
  });
});
