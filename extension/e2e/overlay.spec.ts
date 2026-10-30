import { SUGGESTION } from './fake-server';
import { expect, SETTLE, test, TYPED, typeAndWait, UI_TIMEOUT } from './helpers';

// What the overlay draws in its shadow root: underlines, the card, the badge.

test('um sublinhado por sugestão no shadow root, nada dentro do editor', async ({ ext, server }) => {
  server.reply = [SUGGESTION, { original: 'vai ser', replacement: 'será', rule: 'PTW-2', severity: 'aviso' }];
  await ext.launch();
  const page = await ext.open('editable.html');
  await typeAndWait(page, TYPED, '#reply');
  await expect(page.locator('.ptbr-underline')).toHaveCount(2);
  await expect(page.locator('#gramatico-root .ptbr-underline-erro')).toHaveCount(1);
  await expect(page.locator('#gramatico-root .ptbr-underline-aviso')).toHaveCount(1);
  expect(await page.locator('#field').innerHTML()).not.toContain('ptbr');
  expect(await page.evaluate(() => document.getElementById('gramatico-root')?.parentElement?.tagName)).toBe('HTML');
});

test('o cartão de um clique: regra com título, troca, Aceitar, Ignorar e Sempre ignorar; Ignorar tira a sugestão', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  await page.locator('.ptbr-hit').first().click();
  const card = page.locator('.ptbr-card');
  await expect(card.locator('.ptbr-rule')).toHaveText('PTW-1 · Quem faz, aparece');
  await expect(card.locator('.ptbr-change s')).toHaveText('Foi decidido');
  await expect(card.locator('.ptbr-change strong')).toHaveText('A equipe decidiu');
  await expect(card.locator('button')).toHaveText(['Aceitar', 'Ignorar', 'Sempre ignorar']);
  await card.locator('[data-action=ignore]').click();
  await expect(card).toHaveCount(0);
  await expect(page.locator('.ptbr-underline')).toHaveCount(0);
  await expect(page.locator('#field')).toHaveValue(TYPED);
  await expect(page.locator('.ptbr-badge')).toHaveText('0');
  await page.waitForTimeout(SETTLE);
  expect(server.calls.length).toBe(1);
});

test('o selo: a contagem, e o erro do provider com seu rótulo', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  const badge = page.locator('.ptbr-badge');
  await expect(badge).toHaveText('1');
  await expect(badge).toHaveAttribute('data-state', 'idle');
  server.reply = 401;
  await page.keyboard.type(' ');
  await expect(badge).toHaveText('chave inválida', { timeout: UI_TIMEOUT });
  await expect(badge).toHaveAttribute('data-state', 'error');
  server.reply = 429;
  await page.keyboard.type(' ');
  await expect(badge).toHaveText('limite atingido', { timeout: UI_TIMEOUT });
  await expect(page.locator('.ptbr-underline')).toHaveCount(0);
});

test('o selo abre o cartão da primeira sugestão', async ({ ext }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  await page.locator('.ptbr-badge').click();
  await expect(page.locator('.ptbr-card .ptbr-rule')).toHaveText('PTW-1 · Quem faz, aparece');
});

test('Sempre ignorar tira a sugestão e guarda o trecho na lista nunca corrigir', async ({ ext, server }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  await page.locator('.ptbr-hit').first().click();
  await page.locator('.ptbr-card [data-action=never]').click();
  await expect(page.locator('.ptbr-underline')).toHaveCount(0);
  await expect(page.locator('#field')).toHaveValue(TYPED);
  await expect.poll(async () => (await ext.settings()).neverFix).toEqual(['Foi decidido']);
  await page.waitForTimeout(SETTLE);
  expect(server.calls.length).toBe(1);
});

test('o cartão fica acima do texto quando não cabe abaixo', async ({ ext }) => {
  await ext.launch();
  const page = await ext.open('bottom.html');
  await typeAndWait(page);
  const seg = (await page.locator('.ptbr-seg').first().boundingBox())!;
  await page.locator('.ptbr-hit').first().click();
  const card = (await page.locator('.ptbr-card').boundingBox())!;
  expect(card.y + card.height).toBeLessThanOrEqual(seg.y);
  const tall = await ext.open('textarea.html');
  await typeAndWait(tall);
  const seg2 = (await tall.locator('.ptbr-seg').first().boundingBox())!;
  await tall.locator('.ptbr-hit').first().click();
  const below = (await tall.locator('.ptbr-card').boundingBox())!;
  expect(below.y).toBeGreaterThanOrEqual(seg2.y + seg2.height);
});

test('o cartão do mouse fecha quando o mouse sai; Escape fecha o do clique', async ({ ext }) => {
  await ext.launch();
  const page = await ext.open('textarea.html');
  await typeAndWait(page);
  const box = (await page.locator('.ptbr-seg').first().boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  const card = page.locator('.ptbr-card');
  await expect(card).toHaveCount(1, { timeout: UI_TIMEOUT });
  await page.mouse.move(400, 400);
  await expect(card).toHaveCount(0);
  await page.locator('.ptbr-hit').first().click();
  await expect(card).toHaveCount(1);
  await page.mouse.move(400, 400);
  await page.waitForTimeout(500);
  await expect(card).toHaveCount(1);
  await page.keyboard.press('Escape');
  await expect(card).toHaveCount(0);
});
