// The extension's sound, off by default (Settings.sound): a short cue on each event, made in code
// with the Web Audio API (no audio file, no third-party license), and reading a suggestion aloud
// with one of the system's own voices. Nothing here talks to a server.

export type Cue = 'found' | 'accept' | 'ignore' | 'error';

// Each cue is a few sine notes, in Hz, played one after the other. Rising is good news, falling is
// a step back, and the low pair is a problem.
export const CUES: Readonly<Record<Cue, readonly number[]>> = {
  found: [660, 880], // E5 then A5: suggestions appeared
  accept: [523.25, 659.25, 783.99], // C5, E5, G5: the text changed
  ignore: [440, 349.23], // A4 then F4: a suggestion was dropped
  error: [220, 196], // A3 then G3: the badge shows an error
};

const NOTE_S = 0.09; // each note's length, in seconds
const GAIN = 0.06; // quiet: a cue, not an alarm

type AudioContextLike = Pick<AudioContext, 'currentTime' | 'destination' | 'createOscillator' | 'createGain'>;
let audio: AudioContextLike | null = null;

// Plays a cue. The context is made on first use; a page that has no Web Audio plays nothing.
export function playCue(cue: Cue, make: () => AudioContextLike | null = defaultContext): void {
  audio ??= make();
  if (!audio) return;
  const t0 = audio.currentTime;
  CUES[cue].forEach((hz, i) => {
    const start = t0 + i * NOTE_S;
    const osc = audio!.createOscillator();
    const gain = audio!.createGain();
    osc.type = 'sine';
    osc.frequency.value = hz;
    // A short fade in and out, so the note does not click.
    gain.gain.setValueAtTime(0, start);
    gain.gain.linearRampToValueAtTime(GAIN, start + 0.01);
    gain.gain.linearRampToValueAtTime(0, start + NOTE_S);
    osc.connect(gain).connect(audio!.destination);
    osc.start(start);
    osc.stop(start + NOTE_S);
  });
}

function defaultContext(): AudioContextLike | null {
  const Ctor = globalThis.AudioContext ?? (globalThis as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  return Ctor ? new Ctor() : null;
}

// Only for tests: forget the shared context.
export function resetAudio(): void {
  audio = null;
}

// A pt-BR voice that runs on this machine. Chrome's "Google …" voices are remote (localService is
// false): with them the text would go to Google's servers, so they are never used.
export function localVoice(voices: readonly SpeechSynthesisVoice[]): SpeechSynthesisVoice | null {
  const local = voices.filter((v) => v.localService);
  return (
    local.find((v) => v.lang.replace('_', '-').toLowerCase() === 'pt-br') ??
    local.find((v) => v.lang.toLowerCase().startsWith('pt')) ??
    null
  );
}

// Reads `text` aloud with a local pt-BR voice. False, and silent, when there is none.
export function speak(text: string, synth: SpeechSynthesis | undefined = globalThis.speechSynthesis): boolean {
  if (!synth) return false;
  const voice = localVoice(synth.getVoices());
  if (!voice) return false;
  synth.cancel();
  const u = new SpeechSynthesisUtterance(text);
  u.voice = voice;
  u.lang = voice.lang;
  synth.speak(u);
  return true;
}
