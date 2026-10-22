// A service worker module may not await at top level, and its listeners must be registered
// synchronously, or the event that woke the worker is lost: the wasm loads behind a promise.
import init, { handle_message, start_background, sync_sites } from './ptbr_ext.js';
const ready = init().then(start_background);
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  ready
    .then(() => handle_message(msg, sender))
    .then(sendResponse, () => sendResponse({ ok: false, error: 'provider' }));
  return true;
});
// Absent on a Firefox event page.
const sync = () => ready.then(sync_sites).catch(() => {});
chrome.runtime.onStartup?.addListener(sync);
chrome.runtime.onInstalled?.addListener(sync);
