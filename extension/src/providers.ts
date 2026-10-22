import { type Ask, decideAsks, styleBody } from './decisions';
import { CIENTIFICO, SKILL } from './skills';
import type { CheckError, Level, ProviderId, Settings } from './types';

const ENDPOINTS: Record<ProviderId, string> = {
  claude: 'https://api.anthropic.com/v1/messages',
  deepseek: 'https://api.deepseek.com/chat/completions',
  // Ollama's own chat API, not /v1: only it takes think: false, and a thinking model would spend
  // max_tokens on its reasoning.
  ollama: 'http://127.0.0.1:11434/api/chat',
  // Ollaya serves decision models: /api/decide answers questions, /api/chat is a 404.
  ollaya: 'http://127.0.0.1:11435/api/decide',
};
// The two servers on this machine; their /api/tags and /api/pull speak the same JSON.
const LOCAL = { ollama: 'http://127.0.0.1:11434', ollaya: 'http://127.0.0.1:11435' } as const;
export type LocalId = keyof typeof LOCAL;
// The host permission each one needs: optional in the manifest, asked when the user picks it.
export const LOCAL_ORIGINS: Record<LocalId, { origins: string[] }> = {
  ollama: { origins: [`${LOCAL.ollama}/*`] },
  ollaya: { origins: [`${LOCAL.ollaya}/*`] },
};

// The models this server has; none when it is down or refuses the extension.
export async function localModels(p: LocalId): Promise<string[]> {
  try {
    const res = await fetch(`${LOCAL[p]}/api/tags`);
    const j = (await res.json()) as { models?: Array<{ name?: unknown }> };
    return (j.models ?? []).map((m) => m.name).filter((n): n is string => typeof n === 'string');
  } catch {
    return [];
  }
}

// Downloads a model into the server; both resume a pull cut short, so an interrupted one can be asked again.
export async function localPull(p: LocalId, model: string): Promise<boolean> {
  try {
    const res = await fetch(`${LOCAL[p]}/api/pull`, { method: 'POST', body: JSON.stringify({ model, stream: false }) });
    return res.ok && ((await res.json()) as { status?: unknown }).status === 'success';
  } catch {
    return false;
  }
}
const PATHS: Record<ProviderId, string> = {
  claude: '/v1/messages',
  deepseek: '/chat/completions',
  ollama: '/api/chat',
  ollaya: '/api/decide',
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
          max_tokens: 2048,
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
        max_tokens: 2048,
        temperature: 0,
        // Both DeepSeek models think by default, and the thinking counts against max_tokens: it left the
        // answer empty. A correction needs no chain of thought.
        thinking: { type: 'disabled' },
        messages: [
          { role: 'system', content: system },
          { role: 'user', content: user },
        ],
      }),
    },
  };
}

// Ollaya's requests: one per sentence, asking which of the level's rules it breaks (decisions.ts).
export function decideRequests(s: Settings, level: Level, text: string): { ask: Ask; url: string; init: RequestInit }[] {
  const url = urlFor('ollaya', s);
  return decideAsks(s.models.ollaya, level, text, SKILL, CIENTIFICO).map((ask) => ({
    ask,
    url,
    init: { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(ask.body) },
  }));
}

// Ollaya + Ollama: the one question that picks the style Ollama writes in.
export function styleRequest(s: Settings, text: string): { url: string; init: RequestInit } {
  return {
    url: urlFor('ollaya', s),
    init: { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(styleBody(s.models.ollaya, text)) },
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
  const j = json as { stop_reason?: unknown; done_reason?: unknown; state_truncated?: unknown; choices?: Array<{ finish_reason?: unknown }> } | null;
  if (!j || typeof j !== 'object') return false;
  if (p === 'claude') return j.stop_reason === 'max_tokens';
  if (p === 'ollama') return j.done_reason === 'length';
  // Ollaya drops part of a long state to fit the model's context: a sentence may have gone unread.
  if (p === 'ollaya') return j.state_truncated === true;
  return Array.isArray(j.choices) && j.choices[0]?.finish_reason === 'length';
}

// Ollama and Ollaya answer 403 to an extension origin they were not started to accept (OLLAMA_ORIGINS,
// OLLAYA_ORIGINS).
export function classifyError(status: number | null, p?: ProviderId): CheckError {
  if (p === 'ollama' && (status === null || status === 403)) return 'local';
  if (p === 'ollaya' && (status === null || status === 403)) return 'local-ollaya';
  if (status === null) return 'network';
  if (status === 401 || status === 403) return 'auth';
  if (status === 429) return 'rate-limit';
  return 'provider';
}
