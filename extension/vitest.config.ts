import { readFileSync } from 'node:fs';
import { defineConfig } from 'vitest/config';

// Loads *.md as `export default <string>`, like esbuild's `text` loader, so src/skills.ts works under Vitest.
const markdownAsText = {
  name: 'markdown-as-text',
  transform(_code: string, id: string) {
    if (!id.endsWith('.md')) return null;
    return { code: `export default ${JSON.stringify(readFileSync(id, 'utf8'))};`, map: null };
  },
};

export default defineConfig({
  plugins: [markdownAsText],
  define: { __TEST__: 'true' },
  test: { environment: 'node', include: ['test/**/*.test.ts'] },
});
