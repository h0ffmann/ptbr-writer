import type { CheckError, ProviderId, Settings } from './types';

const ENDPOINTS: Record<ProviderId, string> = {
  claude: 'https://api.anthropic.com/v1/messages',
  deepseek: 'https://api.deepseek.com/chat/completions',
  // Ollama's own chat API, not /v1: only it takes think: false, and a thinking model would spend
  // max_tokens on its reasoning.
  ollama: 'http://127.0.0.1:11434/api/chat',
};
// The host permission Ollama needs: optional in the manifest, asked when the user picks it.
export const OLLAMA_ORIGINS = { origins: ['http://127.0.0.1:11434/*'] };
const PATHS: Record<ProviderId, string> = {
  claude: '/v1/messages',
  deepseek: '/chat/completions',
  ollama: '/api/chat',
};

function urlFor(p: ProviderId, s: Settings): string {
  const base = __TEST__ ? s.apiBase?.[p] : undefined;
  return base ? base.replace(/\/+$/, '') + PATHS[p] : ENDPOINTS[p];
}

export function buildRequest(
  p: ProviderId,
  s: Settings,
  system: string,
  user: string,
): { url: string; init: RequestInit } {
  const url = urlFor(p, s);
  if (p === 'ollama') {
    return {
      url,
      init: {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          model: s.models.ollama,
          stream: false,
          think: false,
          options: { temperature: 0, num_predict: 1024 },
          messages: [
            { role: 'system', content: system },
            { role: 'user', content: user },
          ],
        }),
      },
    };
  }
  if (p === 'claude') {
    return {
      url,
      init: {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'x-api-key': s.keys.claude,
          'anthropic-version': '2023-06-01',
          'anthropic-dangerous-direct-browser-access': 'true',
        },
        body: JSON.stringify({
          model: s.models.claude,
          max_tokens: 1024,
          temperature: 0,
          system,
          messages: [{ role: 'user', content: user }],
        }),
      },
    };
  }
  return {
    url,
    init: {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        Authorization: `Bearer ${s.keys.deepseek}`,
      },
      body: JSON.stringify({
        model: s.models.deepseek,
        max_tokens: 1024,
        temperature: 0,
        messages: [
          { role: 'system', content: system },
          { role: 'user', content: user },
        ],
      }),
    },
  };
}

export function readText(p: ProviderId, json: unknown): string {
  const j = json as {
    content?: unknown;
    message?: { content?: unknown };
    choices?: Array<{ message?: { content?: unknown } }>;
  } | null;
  if (!j || typeof j !== 'object') return '';
  if (p === 'ollama') return typeof j.message?.content === 'string' ? j.message.content : '';
  if (p === 'claude') {
    if (!Array.isArray(j.content)) return '';
    return j.content
      .filter((b): b is { type: 'text'; text: string } => b?.type === 'text' && typeof b.text === 'string')
      .map((b) => b.text)
      .join('');
  }
  const c = Array.isArray(j.choices) ? j.choices[0]?.message?.content : undefined;
  return typeof c === 'string' ? c : '';
}

// True when the reply was cut at max_tokens: its JSON is incomplete, so "no suggestions" would be a lie.
export function isTruncated(p: ProviderId, json: unknown): boolean {
  const j = json as { stop_reason?: unknown; done_reason?: unknown; choices?: Array<{ finish_reason?: unknown }> } | null;
  if (!j || typeof j !== 'object') return false;
  if (p === 'claude') return j.stop_reason === 'max_tokens';
  if (p === 'ollama') return j.done_reason === 'length';
  return Array.isArray(j.choices) && j.choices[0]?.finish_reason === 'length';
}

// Ollama answers 403 to an extension origin it was not started to accept (OLLAMA_ORIGINS).
export function classifyError(status: number | null, p?: ProviderId): CheckError {
  if (p === 'ollama' && (status === null || status === 403)) return 'local';
  if (status === null) return 'network';
  if (status === 401 || status === 403) return 'auth';
  if (status === 429) return 'rate-limit';
  return 'provider';
}
