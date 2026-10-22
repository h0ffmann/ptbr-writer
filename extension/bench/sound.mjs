// The runtime cost of the sound (src/content/sound.ts), measured in real Chromium, headless.
// `just ext-bench-sound` runs it; docs/EXTENSION_SOUND.md keeps the numbers and the machine.
//
// The real sound.ts is bundled with esbuild into a blank page, as the content script would carry it,
// and timed with performance.now(): the first cue (which makes the AudioContext), the next cues
// (scheduling only), the local-voice lookup, and the JS heap before and after.
import { build } from 'esbuild';
import { chromium } from '@playwright/test';

const RUNS = 200;

const bundle = await build({
  entryPoints: [new URL('../src/content/sound.ts', import.meta.url).pathname],
  bundle: true,
  format: 'iife',
  globalName: 'sound',
  write: false,
  minify: true,
});
const code = bundle.outputFiles[0].text;

const browser = await chromium.launch({ args: ['--autoplay-policy=no-user-gesture-required', '--enable-precise-memory-info'] });
const page = await browser.newPage();
await page.setContent('<!doctype html><title>sound bench</title>');
await page.addScriptTag({ content: code });

const r = await page.evaluate((runs) => {
  const heap = () => (performance.memory ? performance.memory.usedJSHeapSize : null);
  const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
  const p95 = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length * 0.95)];
  const heapBefore = heap();
  let t = performance.now();
  sound.playCue('found');
  const first = performance.now() - t;
  const next = [];
  for (let i = 0; i < runs; i++) {
    t = performance.now();
    sound.playCue(['found', 'accept', 'ignore', 'error'][i % 4]);
    next.push(performance.now() - t);
  }
  const voices = speechSynthesis.getVoices();
  const look = [];
  for (let i = 0; i < runs; i++) {
    t = performance.now();
    sound.localVoice(voices);
    look.push(performance.now() - t);
  }
  return {
    firstCueMs: first,
    nextCueMedianMs: median(next),
    nextCueP95Ms: p95(next),
    voiceLookupMedianMs: median(look),
    voices: voices.length,
    localPtVoices: voices.filter((v) => v.localService && v.lang.toLowerCase().startsWith('pt')).length,
    heapBefore,
    heapAfter: heap(),
  };
}, RUNS);

const audioState = await page.evaluate(() => new AudioContext().state);
console.log(JSON.stringify({ chromium: browser.version(), runs: RUNS, soundJsBytes: code.length, audioState, ...r }, null, 2));
await browser.close();
