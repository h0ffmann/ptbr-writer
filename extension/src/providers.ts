import type { CheckError, ProviderId, Settings } from './types';

const ENDPOINTS: Record<ProviderId, string> = {
  claude: 'https://api.anthropic.com/v1/messages',
  deepseek: 'https://api.deepseek.com/chat/completions',
};
const PATHS: Record<ProviderId, string> = {
  claude: '/v1/messages',
  deepseek: '/chat/completions',
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
    choices?: Array<{ message?: { content?: unknown } }>;
  } | null;
  if (!j || typeof j !== 'object') return '';
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

export function classifyError(status: number | null): CheckError {
  if (status === null) return 'network';
  if (status === 401 || status === 403) return 'auth';
  if (status === 429) return 'rate-limit';
  return 'provider';
}
