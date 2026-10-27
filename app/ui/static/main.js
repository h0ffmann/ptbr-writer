// Loads the window's wasm; its start function (src/lib.rs `main`) draws the page. A failure to load
// it shows in the status line, not only in the console.
import init from './ptbr_app_ui.js';

try {
  await init();
} catch (e) {
  const s = document.getElementById('status');
  s.textContent = `A janela não carregou: ${e}`;
  s.classList.add('error');
}
