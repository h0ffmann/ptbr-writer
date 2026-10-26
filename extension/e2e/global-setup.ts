import { build } from 'esbuild';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// Bundles the Lexical page's script next to the pages the fake server serves.
export default async function globalSetup(): Promise<void> {
  const here = dirname(fileURLToPath(import.meta.url));
  await build({
    entryPoints: [join(here, 'lexical-app.ts')],
    outfile: join(here, 'pages', 'lexical.js'),
    bundle: true,
    format: 'iife',
    logLevel: 'warning',
  });
}
