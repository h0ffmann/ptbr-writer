// A content script is classic, not a module: the glue and the wasm come in through the extension's URL.
(async () => {
  const m = await import(chrome.runtime.getURL('ptbr_ext.js'));
  await m.default({ module_or_path: chrome.runtime.getURL('ptbr_ext_bg.wasm') });
  m.start_content();
})();
