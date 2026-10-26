import { expect, test } from './helpers';

// The options page and the popup. Headless Chromium shows no permission prompt, so these walk only
// the granted path: the test manifest grants exemplo.test and the local ports beforehand, and a
// request for them resolves at once. A refused permission is covered by the Rust unit tests.

const KEYS = { claude: 'sk-claude-1111', deepseek: 'sk-deep-2222', ollama: '', ollaya: '' };

test.describe('opções', () => {
  test('trocar o provider mostra a chave e os modelos dele; um servidor local lista o que tem e baixa', async ({ ext, server }) => {
    await ext.launch({ keys: KEYS });
    const page = await ext.page('options.html');
    const key = page.locator('#key');
    await expect(key).toHaveAttribute('type', 'password');
    await expect(key).toHaveValue('••••1111');
    await expect(page.locator('#model option')).toHaveText(['claude-haiku-5-5', 'claude-sonnet-5-5', 'claude-opus-5-5']);
    await page.locator('#provider').selectOption('deepseek');
    await expect(key).toHaveValue('••••2222');
    await expect(page.locator('#model option')).toHaveText(['deepseek-flash', 'deepseek-v4-pro']);
    await expect(page.locator('#ollaya-style')).toBeHidden();
    await page.locator('#provider').selectOption('ollama');
    await expect(key).toBeDisabled();
    await expect(key).toHaveAttribute('placeholder', 'sem chave: just ext-ollama');
    await expect(page.locator('#ollaya-style')).toBeVisible();
    await expect(page.locator('#model option')).toHaveText([
      'qwen3.6:27b (sugerido) (baixar)',
      'qwen3:14b (baixar)',
      'qwen3:8b (baixar)',
      'qwen3:4b (baixar)',
      'llama3.2:1b',
    ]);
    await expect(page.locator('#pull-model')).toBeVisible();
    await page.locator('#pull-model').click();
    await expect(page.locator('#pull-status')).toHaveText('pronto');
    expect(server.calls.map((c) => [c.path, c.body.model])).toEqual([['/api/pull', 'qwen3.6:27b']]);
    await expect(page.locator('#model option').first()).toHaveText('qwen3.6:27b (sugerido)');
    await expect(page.locator('#pull-model')).toBeHidden();
  });

  test('Salvar guarda a chave do provider escolhido, mascarada, e mantém a outra', async ({ ext }) => {
    await ext.launch({ keys: KEYS });
    const page = await ext.page('options.html');
    await page.locator('#key').fill('sk-nova-9999');
    await page.locator('#model').selectOption('claude-opus-5-5');
    await page.locator('#save').click();
    await expect(page.locator('#status')).toHaveText('Salvo.');
    await expect(page.locator('#key')).toHaveValue('••••9999');
    const s = await ext.settings();
    expect(s.keys).toEqual({ ...KEYS, claude: 'sk-nova-9999' });
    expect(s.models.claude).toBe('claude-opus-5-5');
  });

  test('adicionar um site: inválido avisa na linha; válido salva como descritivo e registra; remover desfaz', async ({ ext }) => {
    await ext.launch();
    const page = await ext.page('options.html');
    await page.locator('#new-host').fill('sem ponto');
    await page.locator('#add-site').click();
    await expect(page.locator('#site-error')).toHaveText('Site inválido. Use um domínio como exemplo.com.');
    expect((await ext.settings()).sites['sem ponto']).toBeUndefined();

    await page.locator('#new-host').fill('https://Exemplo.test/caixa');
    await page.locator('#add-site').click();
    const row = page.locator('tr[data-host="exemplo.test"]');
    await expect(row.locator('select')).toHaveValue('descritivo');
    await expect(row.locator('input[type=checkbox]')).toBeChecked();
    expect((await ext.settings()).sites['exemplo.test']).toEqual({ enabled: true, level: 'descritivo' });
    await expect.poll(() => ext.registered()).toEqual(['site:exemplo.test']);

    await row.locator('button', { hasText: 'Remover' }).click();
    await expect(row).toHaveCount(0);
    expect((await ext.settings()).sites['exemplo.test']).toBeUndefined();
    await expect.poll(() => ext.registered()).toEqual([]);
  });

  test('os sites padrão não têm Remover; o nível de um site muda na hora', async ({ ext }) => {
    await ext.launch();
    const page = await ext.page('options.html');
    await expect(page.locator('tr[data-host]')).toHaveCount(4);
    for (const host of ['web.whatsapp.com', 'mail.google.com', 'github.com']) {
      await expect(page.locator(`tr[data-host="${host}"] button`)).toHaveCount(0);
    }
    await expect(page.locator('tr[data-host="127.0.0.1"] button', { hasText: 'Remover' })).toHaveCount(1);
    await page.locator('tr[data-host="github.com"] select').selectOption('estrito');
    await expect.poll(async () => (await ext.settings()).sites['github.com']?.level).toBe('estrito');
  });
});

test.describe('opções: o texto', () => {
  test('tom, o quanto corrigir e espera salvam na hora', async ({ ext }) => {
    await ext.launch();
    const page = await ext.page('options.html');
    await expect(page.locator('#tone option')).toHaveText(['Informal', 'Neutro', 'Formal']);
    await expect(page.locator('#rewrite option')).toHaveText(['Só erros', 'Erros e reescrita']);
    await expect(page.locator('#delay option')).toHaveText(['1 s', '2 s', '3 s']);
    await page.locator('#tone').selectOption('formal');
    await page.locator('#rewrite').selectOption('false');
    await page.locator('#delay').selectOption('3000');
    await expect
      .poll(async () => {
        const s = await ext.settings();
        return [s.tone, s.rewrite, s.delayMs];
      })
      .toEqual(['formal', false, 3000]);
  });

  test('nunca corrigir: mais de 3 palavras avisa; adicionar e remover', async ({ ext }) => {
    await ext.launch();
    const page = await ext.page('options.html');
    await page.locator('#new-never-fix').fill('um dois três quatro');
    await page.locator('#add-never-fix').click();
    await expect(page.locator('#never-fix-error')).toHaveText('Use de 1 a 3 palavras.');
    await page.locator('#new-never-fix').fill('  Kafka ');
    await page.locator('#add-never-fix').click();
    await expect(page.locator('#never-fix li')).toHaveText(['Kafka Remover']);
    expect((await ext.settings()).neverFix).toEqual(['Kafka']);
    await page.locator('#never-fix li button').click();
    await expect(page.locator('#never-fix li')).toHaveCount(0);
    expect((await ext.settings()).neverFix).toEqual([]);
  });
});

test.describe('popup', () => {
  test('o site da aba: nível, Desativar e Ativar; a chave geral', async ({ ext }) => {
    await ext.launch({ sites: { 'exemplo.test': { enabled: true, level: 'leve' } } });
    const popup = await ext.popupOn('https://exemplo.test/caixa');
    await expect(popup.locator('#app')).toContainText('exemplo.test');
    await expect(popup.locator('#site-level')).toHaveValue('leve');
    await popup.locator('#site-level').selectOption('estrito');
    await expect.poll(async () => (await ext.settings()).sites['exemplo.test'].level).toBe('estrito');
    await popup.locator('#toggle-site', { hasText: 'Desativar neste site' }).click();
    await expect(popup.locator('#toggle-site')).toHaveText('Ativar neste site');
    expect((await ext.settings()).sites['exemplo.test']).toEqual({ enabled: false, level: 'estrito' });
    await popup.locator('#toggle-site').click();
    await expect(popup.locator('#toggle-site')).toHaveText('Desativar neste site');
    expect((await ext.settings()).sites['exemplo.test'].enabled).toBe(true);
    await popup.locator('#enabled').uncheck();
    await expect.poll(async () => (await ext.settings()).enabled).toBe(false);
  });

  test('uma página http não é suportada; o provider muda na hora; Todas as configurações abre a página', async ({ ext, server }) => {
    await ext.launch({ keys: { ...KEYS, claude: '' } });
    const popup = await ext.popupOn(`${server.base}/textarea.html`);
    await expect(popup.locator('#app')).toContainText('Site não suportado.');
    await expect(popup.locator('#app')).toContainText('Sem chave do Claude.');
    await expect(popup.locator('#toggle-site')).toHaveCount(0);
    await popup.locator('#provider').selectOption('deepseek');
    await expect(popup.locator('#key')).toHaveValue('••••2222');
    await expect.poll(async () => (await ext.settings()).provider).toBe('deepseek');
    const opened = ext.context.waitForEvent('page');
    await popup.locator('#open-options').click();
    expect((await opened).url()).toBe(`chrome-extension://${ext.id}/options.html`);
  });
});
