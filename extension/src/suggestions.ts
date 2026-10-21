import type { Located, Suggestion } from './types';

export function parseSuggestions(raw: string): Suggestion[] {
  const from = raw.indexOf('[');
  const to = raw.lastIndexOf(']');
  if (from < 0 || to < from) return [];
  let data: unknown;
  try {
    data = JSON.parse(raw.slice(from, to + 1));
  } catch {
    return [];
  }
  if (!Array.isArray(data)) return [];
  const out: Suggestion[] = [];
  for (const item of data) {
    if (typeof item !== 'object' || item === null) continue;
    const o = item as Record<string, unknown>;
    if (typeof o.original !== 'string' || o.original === '') continue;
    if (typeof o.replacement !== 'string' || typeof o.rule !== 'string') continue;
    if (o.severity !== 'erro' && o.severity !== 'aviso') continue;
    const occ = o.occurrence;
    out.push({
      original: o.original,
      replacement: o.replacement,
      rule: o.rule,
      severity: o.severity,
      occurrence: typeof occ === 'number' && Number.isInteger(occ) && occ >= 1 ? occ : 1,
    });
  }
  return out;
}

// NFC of `text` plus, for each NFC code unit, the offsets it spans in `text`.
function normalize(text: string): { nfc: string; starts: number[]; ends: number[] } {
  let nfc = '';
  const starts: number[] = [];
  const ends: number[] = [];
  const runs = /\P{M}\p{M}*|\p{M}+/gu;
  for (const m of text.matchAll(runs)) {
    const start = m.index ?? 0;
    const end = start + m[0].length;
    const part = m[0].normalize('NFC');
    nfc += part;
    for (let i = 0; i < part.length; i++) {
      starts.push(start);
      ends.push(end);
    }
  }
  return { nfc, starts, ends };
}

export function locate(text: string, s: Suggestion[]): Located[] {
  const { nfc, starts, ends } = normalize(text);
  const out: Located[] = [];
  for (const sug of s) {
    if (sug.original === sug.replacement) continue;
    const needle = sug.original.normalize('NFC');
    if (needle === '') continue;
    let at = -1;
    let from = 0;
    for (let n = 0; n < sug.occurrence; n++) {
      at = nfc.indexOf(needle, from);
      if (at < 0) break;
      from = at + needle.length;
    }
    if (at < 0) continue;
    out.push({ ...sug, start: starts[at], end: ends[at + needle.length - 1] });
  }
  return out.sort((a, b) => a.start - b.start);
}

export function dropOverlaps(l: Located[]): Located[] {
  const out: Located[] = [];
  let last = -Infinity;
  for (const x of l) {
    if (x.start < last) continue;
    out.push(x);
    last = x.end;
  }
  return out;
}

export function applyToString(text: string, l: Located): string {
  return text.slice(0, l.start) + l.replacement + text.slice(l.end);
}
