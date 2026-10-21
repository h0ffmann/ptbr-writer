import { build } from 'esbuild';
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';

const test = process.argv.includes('--test');
const firefox = process.argv.includes('--firefox');
const out = test ? 'dist-test' : firefox ? 'dist-firefox' : 'dist';
const plugin = JSON.parse(readFileSync('../plugins/ptbr-writer/.claude-plugin/plugin.json', 'utf8'));

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

await build({
  entryPoints: {
    background: 'src/background.ts',
    content: 'src/content/main.ts',
    options: 'src/options.ts',
    popup: 'src/popup.ts',
  },
  outdir: out,
  bundle: true,
  format: 'esm',
  target: firefox ? 'firefox128' : 'chrome120',
  loader: { '.md': 'text' },
  define: { __TEST__: test ? 'true' : 'false' },
  logLevel: 'warning',
});

cpSync('static', out, { recursive: true });

const local = test ? ['http://127.0.0.1/*', 'http://localhost/*'] : [];
const manifest = {
  manifest_version: 3,
  name: 'ptbr-writer',
  version: plugin.version,
  description: plugin.description,
  permissions: ['storage', 'scripting', 'activeTab'],
  host_permissions: ['https://api.anthropic.com/*', 'https://api.deepseek.com/*', ...local],
  optional_host_permissions: ['http://127.0.0.1:11434/*', '<all_urls>'],
  background: { service_worker: 'background.js', type: 'module' },
  options_page: 'options.html',
  action: { default_popup: 'popup.html' },
  content_scripts: [
    {
      matches: ['https://web.whatsapp.com/*', 'https://mail.google.com/*', 'https://github.com/*', ...local],
      js: ['content.js'],
      css: ['content.css'],
    },
  ],
};
// Firefox runs no background service worker: an event page (scripts) and an add-on id instead.
if (firefox) {
  manifest.background = { scripts: ['background.js'], type: 'module' };
  manifest.browser_specific_settings = { gecko: { id: 'ptbr-writer@h0ffmann', strict_min_version: '128.0' } };
}
writeFileSync(`${out}/manifest.json`, JSON.stringify(manifest, null, 2) + '\n');
