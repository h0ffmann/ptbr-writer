import { describe, expect, it } from 'vitest';
import { buildRequest, classifyError, readText } from '../src/providers';
import { DEFAULT_SETTINGS } from '../src/settings';
import type { Settings } from '../src/types';

const base: Settings = {
  ...DEFAULT_SETTINGS,
  keys: { claude: 'sk-test', deepseek: 'sk-test' },
};

describe('buildRequest', () => {
  it('claude: url, exact headers and body', () => {
    const { url, init } = buildRequest('claude', base, 'SYS', 'USER');
    expect(url).toBe('https://api.anthropic.com/v1/messages');
    expect(init.method).toBe('POST');
    expect(init.headers).toEqual({
      'content-type': 'application/json',
      'x-api-key': 'sk-test',
      'anthropic-version': '2023-06-01',
      'anthropic-dangerous-direct-browser-access': 'true',
    });
    const body = JSON.parse(init.body as string);
    expect(body).toEqual({
      model: base.models.claude,
      max_tokens: 1024,
      temperature: 0,
      system: 'SYS',
      messages: [{ role: 'user', content: 'USER' }],
    });
  });

  it('deepseek: url, bearer and messages', () => {
    const { url, init } = buildRequest('deepseek', base, 'SYS', 'USER');
    expect(url).toBe('https://api.deepseek.com/chat/completions');
    const h = init.headers as Record<string, string>;
    expect(h.Authorization).toBe('Bearer sk-test');
    expect(h['content-type']).toBe('application/json');
    const body = JSON.parse(init.body as string);
    expect(body.model).toBe(base.models.deepseek);
    expect(body.temperature).toBe(0);
    expect(body.messages[0]).toEqual({ role: 'system', content: 'SYS' });
    expect(body.messages[1]).toEqual({ role: 'user', content: 'USER' });
  });

  it('apiBase overrides the url (test builds only)', () => {
    const s = { ...base, apiBase: { claude: 'http://127.0.0.1:9', deepseek: 'http://127.0.0.1:8' } };
    expect(buildRequest('claude', s, 'a', 'b').url).toBe('http://127.0.0.1:9/v1/messages');
    expect(buildRequest('deepseek', s, 'a', 'b').url).toBe('http://127.0.0.1:8/chat/completions');
  });

  it('without apiBase the url is the real endpoint', () => {
    expect(buildRequest('claude', { ...base, apiBase: undefined }, 'a', 'b').url).toBe(
      'https://api.anthropic.com/v1/messages',
    );
    expect(buildRequest('deepseek', { ...base, apiBase: {} }, 'a', 'b').url).toBe(
      'https://api.deepseek.com/chat/completions',
    );
  });
});

describe('readText', () => {
  it('claude: concatenates text blocks', () => {
    const json = { content: [{ type: 'text', text: 'a' }, { type: 'tool_use' }, { type: 'text', text: 'b' }] };
    expect(readText('claude', json)).toBe('ab');
  });
  it('deepseek: first choice content', () => {
    expect(readText('deepseek', { choices: [{ message: { content: 'oi' } }] })).toBe('oi');
  });
  it('unexpected shapes give empty string', () => {
    expect(readText('claude', {})).toBe('');
    expect(readText('deepseek', {})).toBe('');
    expect(readText('claude', null)).toBe('');
    expect(readText('deepseek', 'x')).toBe('');
  });
});

describe('classifyError', () => {
  it('maps statuses', () => {
    expect(classifyError(401)).toBe('auth');
    expect(classifyError(403)).toBe('auth');
    expect(classifyError(429)).toBe('rate-limit');
    expect(classifyError(500)).toBe('provider');
    expect(classifyError(null)).toBe('network');
  });
});
