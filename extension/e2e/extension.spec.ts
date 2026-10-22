import { accept, expect, FIXED, SETTLE, test, TYPED, typeAndWait, UI_TIMEOUT } from './helpers';

// The four providers, end to end: a textarea, the underline, Aceitar, the field changed.
for (const [provider, path] of [
  ['claude', '/v1/messages'],
  ['deepseek', '/chat/completions'],
  ['ollama', '/api/chat'],
] as const) {
  test(`${provider}: textarea, sublinha, aceita`, async ({ ext, server }) => {
    await ext.launch({ provider });
    const page = await ext.open('textarea.html');
    await typeAndWait(page);
    await accept(page);
    await expect(page.locator('#field')).toHaveValue(FIXED);
    expect(server.calls.map((c) => c.path)).toEqual([path]);
    expect(server.calls[0].text).toBe(TYPED);
  });
}

test('ollaya: uma pergunta por frase, só aponta', async ({ ext, server }) => {
  await ext.launch({ provider: 'ollaya' });
  const page = await ext.open('textarea.html');
  await typeAndWait(page, `${TYPED} A equipe valida o build hoje.`);
  await expect(page.locator('.ptbr-underline-aviso')).toHaveCount(2);
  expect(server.calls.map((c) => [c.path, c.text])).toEqual([
    ['/api/decide', TYPED],
    ['/api/decide', 'A equipe valida o build hoje.'],
  ]);
  await page.locator('.ptbr-underline[data-i="0"] .ptbr-hit').first().click();
  const card = page.locator('.ptbr-card');
  await expect(card.locator('.ptbr-rule')).toHaveText('PTW-1 (90%)');
  await expect(card.locator('button')).toHaveText(['Ignorar']);
});

test('contenteditable estilo Gmail', async ({ ext }) => {
  await ext.launch();
  const page = await ext.open('editable.html');
  await typeAndWait(page, TYPED, '#reply');
  await accept(page);
  await expect(page.locator('#reply')).toHaveText(FIXED);
  await expect(page.locator('#quote')).toHaveText('Foi decidido que a reunião é na segunda.');
});

test('site não liberado não chama a API', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('textarea.html', 'localhost');
  await page.locator('#field').click();
  await page.keyboard.type(TYPED);
  await page.waitForTimeout(SETTLE);
  await expect(page.locator('.ptbr-underline')).toHaveCount(0);
  expect(server.calls).toEqual([]);
});

test('desligar o site numa aba aberta para as chamadas', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  expect(server.calls.length).toBe(1);
  await ext.setSettings({ sites: { '127.0.0.1': { enabled: false, level: 'descritivo' } } });
  await page.keyboard.type(' E mais uma frase foi escrita.');
  await page.waitForTimeout(SETTLE);
  expect(server.calls.length).toBe(1);
  await expect(page.locator('.ptbr-underline')).toHaveCount(0);
});

test('passar o mouse na palavra abre o cartão', async ({ ext }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  const seg = page.locator('.ptbr-underline-erro .ptbr-seg').first();
  const box = (await seg.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.locator('.ptbr-card [data-action=accept]').click({ timeout: UI_TIMEOUT });
  await expect(page.locator('#field')).toHaveValue(FIXED);
});

test('uma página com CSP estrita roda a extensão', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('csp/textarea.html');
  await typeAndWait(page);
  await accept(page);
  await expect(page.locator('#field')).toHaveValue(FIXED);
  expect(server.calls.length).toBe(1);
});
