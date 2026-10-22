export const NEVER_FIX_MAX = 200;
export const NEVER_FIX_MAX_WORDS = 3;

// Adds a phrase to the "never fix" list: trimmed, 1 to 3 words, no repeats (case-insensitive),
// and only the NEVER_FIX_MAX most recent entries stay.
export function addNeverFix(list: readonly string[], phrase: string): string[] {
  const p = phrase.trim();
  if (!p || p.split(/\s+/).length > NEVER_FIX_MAX_WORDS) return [...list];
  const low = p.toLowerCase();
  if (list.some((e) => e.toLowerCase() === low)) return [...list];
  return [...list, p].slice(-NEVER_FIX_MAX);
}

// Whole word: letters and digits (with accents) are word characters; `flags` adds 'i' to ignore case.
function wholeWord(entry: string, flags: string): RegExp {
  const esc = entry.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`(?<![\\p{L}\\p{N}])${esc}(?![\\p{L}\\p{N}])`, `u${flags}`);
}

// Whole-word match, ignoring case but not accents.
export function containsEntry(text: string, entry: string): boolean {
  const e = entry.trim();
  return e !== '' && wholeWord(e, 'i').test(text);
}

// True when `original` holds the entry (any case) and `replacement` does not keep each matched
// text exactly as written, case included, as a whole word: `maria` -> `Maria` breaks it.
export function breaksEntry(original: string, replacement: string, entry: string): boolean {
  const e = entry.trim();
  if (!e) return false;
  const found = [...original.matchAll(wholeWord(e, 'gi'))].map((m) => m[0]);
  return found.some((f) => !wholeWord(f, '').test(replacement));
}
