import { type FakeServer, SUGGESTION } from './fake-server';
import type { Page } from '@playwright/test';
import { accept, expect, FIXED, SETTLE, test, TYPED, typeAndWait, UI_TIMEOUT } from './helpers';

// The content script applies a settings change asynchronously (storage event, then a read), so a key
// typed right after setSettings can go unseen: type again until the change has taken, one key per
// wait longer than the 1 s debounce, so a seen key never gets a second request.
async function typeUntilChecked(page: Page, server: FakeServer) {
  await expect(async () => {
    await page.keyboard.type(' ');
    await expect.poll(() => server.calls.length, { timeout: 3_000 }).toBeGreaterThan(0);
  }).toPass({ timeout: UI_TIMEOUT });
}

// The content script: what it reads from a field (text model), when it sends (watcher) and what
// it does with the settings and an accepted suggestion (controller).

test.describe('modelo de texto', () => {
  test('aceita num input', async ({ ext }) => {
    await ext.launch();
    const page = await ext.open('input.html');
    await typeAndWait(page);
    await accept(page);
    await expect(page.locator('#field')).toHaveValue(FIXED);
  });

  test('aceita num contenteditable com vários nós de texto e um emoji antes', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('rich.html');
    await page.locator('#field').click();
    await page.keyboard.press('End');
    await page.keyboard.type(' ');
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    // A space typed at the end of a contenteditable is an NBSP.
    expect(server.calls[0].text.replace(/ /g, ' ')).toBe(`😀 ${TYPED} `);
    await accept(page);
    await expect(page.locator('#field')).toHaveText(`😀 ${FIXED}`);
  });

  test('um bloco vale uma quebra, um <br> também, e o citado não vai', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('blocks.html');
    await page.locator('#last').click();
    await page.keyboard.press('End');
    await page.keyboard.type(' x');
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    expect(server.calls[0].text).toBe(`${TYPED}\nSegunda linha\nterceira linha x`);
  });
});

test.describe('watcher', () => {
  test('três entradas viram um pedido, 1 s depois da última', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('textarea.html');
    await page.locator('#field').click();
    await page.keyboard.type('Foi decidido que o deploy');
    await page.waitForTimeout(400);
    await page.keyboard.type(' vai ser');
    await page.waitForTimeout(400);
    await page.keyboard.type(' na sexta.');
    expect(server.calls.length).toBe(0);
    await page.waitForTimeout(500);
    expect(server.calls.length).toBe(0);
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    await page.waitForTimeout(SETTLE);
    expect(server.calls.length).toBe(1);
    expect(server.calls[0].text).toBe(TYPED);
  });

  test('digitar limpa os sublinhados', async ({ ext }) => {
    await ext.launch();
    const page = await ext.open('textarea.html');
    await typeAndWait(page);
    await page.keyboard.type(' ');
    await expect(page.locator('.ptbr-underline')).toHaveCount(0);
  });

  test('senha e menos de 3 palavras não mandam nada', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('password.html');
    await page.locator('#field').click();
    await page.keyboard.type(TYPED);
    await page.waitForTimeout(SETTLE);
    const input = await ext.open('input.html');
    await input.locator('#field').click();
    await input.keyboard.type('Foi decidido');
    await input.waitForTimeout(SETTLE);
    expect(server.calls).toEqual([]);
    await expect(page.locator('.ptbr-underline')).toHaveCount(0);
    await expect(input.locator('.ptbr-underline')).toHaveCount(0);
  });

  test('acima de 4000 caracteres só o parágrafo do cursor vai', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('textarea.html');
    await page.locator('#field').fill(`${'a '.repeat(2100)}\n${TYPED}`);
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' ');
    await expect.poll(() => server.calls.length, { timeout: UI_TIMEOUT }).toBe(1);
    expect(server.calls.map((c) => c.text.slice(0, 60))).toEqual([`${TYPED} `]);
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
  });
});

test.describe('controller', () => {
  test('o site desligado e ligado de novo com a aba aberta', async ({ ext, server }) => {
    await ext.launch({ sites: { '127.0.0.1': { enabled: false, level: 'descritivo' } } });
    const page = await ext.open('textarea.html');
    await page.locator('#field').click();
    await page.keyboard.type(TYPED);
    await page.waitForTimeout(SETTLE);
    expect(server.calls).toEqual([]);
    await ext.setSettings({ sites: { '127.0.0.1': { enabled: true, level: 'descritivo' } } });
    await typeUntilChecked(page, server);
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    expect(server.calls.length).toBe(1);
  });

  test('a extensão desligada e ligada de novo com a aba aberta', async ({ ext, server }) => {
    await ext.launch({ enabled: false });
    const page = await ext.open('textarea.html');
    await page.locator('#field').click();
    await page.keyboard.type(TYPED);
    await page.waitForTimeout(SETTLE);
    expect(server.calls).toEqual([]);
    await ext.setSettings({ enabled: true });
    await typeUntilChecked(page, server);
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    await ext.setSettings({ enabled: false });
    await page.waitForTimeout(SETTLE);
    await page.keyboard.type(' ');
    await page.waitForTimeout(SETTLE);
    expect(server.calls.length).toBe(1);
    await expect(page.locator('.ptbr-underline')).toHaveCount(0);
  });

  test('segue o novo nível', async ({ ext, server }) => {
    await ext.launch();
    const page = await ext.open('textarea.html');
    await typeAndWait(page);
    expect(server.calls[0].system).toContain('Nível pedido: `descritivo`');
    await ext.setSettings({ sites: { '127.0.0.1': { enabled: true, level: 'estrito' } } });
    await page.keyboard.type(' ');
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    expect(server.calls.length).toBe(2);
    expect(server.calls[1].system).toContain('Nível pedido: `estrito`');
  });

  test('aceitar uma sugestão mantém as outras, deslocadas, sem nova chamada; digitar checa de novo', async ({ ext, server }) => {
    server.reply = [SUGGESTION, { original: 'vai ser', replacement: 'será', rule: 'PTW-2', severity: 'aviso' }];
    await ext.launch();
    const page = await ext.open('textarea.html');
    await typeAndWait(page);
    await expect(page.locator('.ptbr-underline')).toHaveCount(2);
    await accept(page, 0);
    await expect(page.locator('#field')).toHaveValue(FIXED);
    await expect(page.locator('.ptbr-underline')).toHaveCount(1);
    await page.waitForTimeout(SETTLE);
    expect(server.calls.length).toBe(1);
    await accept(page, 0);
    await expect(page.locator('#field')).toHaveValue('A equipe decidiu que o deploy será na sexta.');
    await expect(page.locator('.ptbr-underline')).toHaveCount(0);
    // Checked again: of the two replies only "Foi decidido" is still in the text.
    await page.keyboard.type(' Foi decidido também.');
    await expect(page.locator('.ptbr-underline')).toHaveCount(1, { timeout: UI_TIMEOUT });
    expect(server.calls.length).toBe(2);
  });
});
