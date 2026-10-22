import { describe, expect, it } from 'vitest';
import { buildRequest, classifyError, isTruncated, readText } from '../src/providers';
import { DEFAULT_SETTINGS } from '../src/settings';
import type { Settings } from '../src/types';

const base: Settings = {
  ...DEFAULT_SETTINGS,
  keys: { claude: 'sk-test', deepseek: 'sk-test', ollama: '', ollaya: '' },
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
    expect(body.max_tokens).toBe(1024);
    expect(body.messages[0]).toEqual({ role: 'system', content: 'SYS' });
    expect(body.messages[1]).toEqual({ role: 'user', content: 'USER' });
  });

  it('ollama: local chat API, no key, thinking off, reply and truncation', () => {
    const { url, init } = buildRequest('ollama', base, 'SYS', 'USER');
    expect(url).toBe('http://127.0.0.1:11434/api/chat');
    expect(init.headers).toEqual({ 'content-type': 'application/json' });
    expect(JSON.parse(init.body as string)).toEqual({
      model: 'qwen3.6:27b',
      stream: false,
      think: false,
      options: { temperature: 0, num_predict: 1024 },
      messages: [
        { role: 'system', content: 'SYS' },
        { role: 'user', content: 'USER' },
      ],
    });
    expect(readText('ollama', { message: { content: '[]' }, done_reason: 'stop' })).toBe('[]');
    expect(isTruncated('ollama', { done_reason: 'length' })).toBe(true);
    expect(isTruncated('ollama', { done_reason: 'stop' })).toBe(false);
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

describe('isTruncated', () => {
  it('claude: stop_reason max_tokens', () => {
    expect(isTruncated('claude', { stop_reason: 'max_tokens', content: [] })).toBe(true);
    expect(isTruncated('claude', { stop_reason: 'end_turn', content: [] })).toBe(false);
  });
  it('deepseek: finish_reason length', () => {
    expect(isTruncated('deepseek', { choices: [{ finish_reason: 'length', message: { content: '[' } }] })).toBe(true);
    expect(isTruncated('deepseek', { choices: [{ finish_reason: 'stop', message: { content: '[]' } }] })).toBe(false);
  });
  it('unexpected shapes are not truncated', () => {
    expect(isTruncated('claude', null)).toBe(false);
    expect(isTruncated('deepseek', {})).toBe(false);
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
  it('Ollama: down or refusing the extension origin is local', () => {
    expect(classifyError(null, 'ollama')).toBe('local');
    expect(classifyError(403, 'ollama')).toBe('local');
    expect(classifyError(500, 'ollama')).toBe('provider');
    expect(classifyError(403, 'claude')).toBe('auth');
  });
  it('Ollaya: down or refusing the extension origin is local-ollaya', () => {
    expect(classifyError(null, 'ollaya')).toBe('local-ollaya');
    expect(classifyError(403, 'ollaya')).toBe('local-ollaya');
    expect(classifyError(404, 'ollaya')).toBe('provider');
  });
});
