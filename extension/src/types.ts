export type Level = 'estrito' | 'descritivo' | 'leve' | 'científico';
export type ProviderId = 'claude' | 'deepseek';
export type Severity = 'erro' | 'aviso';
export interface Suggestion {
  original: string;
  replacement: string;
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
  sites: Record<string, SiteConfig>; // key: hostname
  apiBase?: Partial<Record<ProviderId, string>>; // honored only when __TEST__
}
export type CheckError = 'no-key' | 'auth' | 'rate-limit' | 'provider' | 'network';
export interface CheckRequest {
  type: 'check';
  key: string;
  text: string;
  level: Level;
}
export type CheckResult = { ok: true; suggestions: Suggestion[] } | { ok: false; error: CheckError };
