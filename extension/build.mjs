import { build } from 'esbuild';
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';

const test = process.argv.includes('--test');
const out = test ? 'dist-test' : 'dist';
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
  target: 'chrome120',
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
  optional_host_permissions: ['<all_urls>'],
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
writeFileSync(`${out}/manifest.json`, JSON.stringify(manifest, null, 2) + '\n');
