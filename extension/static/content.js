// A content script is classic, not a module, and runs under the page's CSP: the module comes in
// through the extension's URL, as JavaScript (xtask: wasm2js), never as wasm. The promise lives in
// the isolated world only; the e2e suite awaits it before typing.
globalThis.ptbrWriter = (async () => {
  const m = await import(chrome.runtime.getURL('ptbr_ext_js.js'));
  await m.start_content();
})();
