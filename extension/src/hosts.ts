/** Turns what the user typed into a lowercase hostname, or null when it is not acceptable. */
export function normalizeHost(input: string): string | null {
  const text = input.trim();
  if (!text) return null;
  const scheme = /^([a-z][a-z0-9+.-]*):\/\//i.exec(text);
  if (scheme && !/^https?$/i.test(scheme[1])) return null;
  const rest = scheme ? text.slice(scheme[0].length) : text;
  const authority = rest.split(/[/?#]/)[0];
  if (!authority || /[*:@\s]/.test(authority)) return null;
  let host: string;
  try {
    host = new URL(`https://${authority}`).hostname.toLowerCase();
  } catch {
    return null;
  }
  if (!host.includes('.') || host.endsWith('.')) return null;
  if (/^\d+(\.\d+){3}$/.test(host) || host.startsWith('[')) return null;
  return host;
}
