export type Level = 'estrito' | 'descritivo' | 'leve' | 'científico';
export type ProviderId = 'claude' | 'deepseek' | 'ollama' | 'ollaya';
export type Severity = 'erro' | 'aviso';
// The conversation style of a rewrite, which Ollaya picks for Ollama (Settings.ollayaStyle).
export type Style = 'conciso' | 'socrático' | 'amigável';
export interface Suggestion {
  original: string;
  replacement: string;
  detectOnly?: true; // a decision model (Ollaya) names the rule but writes no replacement: no Aceitar
  rule: string;
  severity: Severity;
  occurrence: number; // 1-based
}
export interface Located extends Suggestion {
  start: number; // UTF-16, in the text sent
  end: number;
}
export interface SiteConfig {
  enabled: boolean;
  level: Level;
}
export interface Settings {
  enabled: boolean;
  provider: ProviderId;
  keys: Record<ProviderId, string>;
  models: Record<ProviderId, string>;
  // A feature flag: Ollaya picks the style before Ollama writes. Off, or Ollaya down, Ollama writes alone.
  ollayaStyle: boolean;
  // Sound, off by default: a cue on each event and 🔊 on the card (content/sound.ts). No server.
  sound: boolean;
  sites: Record<string, SiteConfig>; // key: hostname
  apiBase?: Partial<Record<ProviderId, string>>; // honored only when __TEST__
}
export type CheckError = 'no-key' | 'local' | 'local-ollaya' | 'auth' | 'rate-limit' | 'provider' | 'network';
export interface CheckRequest {
  type: 'check';
  key: string;
  text: string;
  level: Level;
}
export type CheckResult = { ok: true; suggestions: Suggestion[] } | { ok: false; error: CheckError };
