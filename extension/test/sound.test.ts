import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CUES, localVoice, playCue, resetAudio, speak } from '../src/content/sound';

// A fake AudioContext: it records each oscillator's frequency and start time.
function fakeAudio() {
  const notes: { hz: number; start: number }[] = [];
  const param = () => ({ value: 0, setValueAtTime: vi.fn(), linearRampToValueAtTime: vi.fn() });
  const ctx = {
    currentTime: 10,
    destination: {},
    createGain: () => ({ gain: param(), connect: (x: unknown) => x }),
    createOscillator: () => {
      const osc = {
        type: '',
        frequency: param(),
        connect: (x: unknown) => x,
        start: (t: number) => notes.push({ hz: osc.frequency.value, start: t }),
        stop: vi.fn(),
      };
      return osc;
    },
  };
  return { ctx: ctx as unknown as AudioContext, notes };
}

const voice = (name: string, lang: string, localService: boolean) =>
  ({ name, lang, localService }) as SpeechSynthesisVoice;

beforeEach(() => resetAudio());

describe('playCue', () => {
  it('plays each note of the cue, one after the other', () => {
    const { ctx, notes } = fakeAudio();
    playCue('accept', () => ctx);
    expect(notes.map((n) => n.hz)).toEqual([...CUES.accept]);
    expect(notes[1].start).toBeGreaterThan(notes[0].start);
  });
  it('plays nothing on a page with no Web Audio', () => {
    expect(() => playCue('found', () => null)).not.toThrow();
  });
  it('rises for good news and falls for a step back', () => {
    expect(CUES.found[1]).toBeGreaterThan(CUES.found[0]);
    expect(CUES.ignore[1]).toBeLessThan(CUES.ignore[0]);
    expect(Math.max(...CUES.error)).toBeLessThan(Math.min(...CUES.found));
  });
});

describe('localVoice', () => {
  it('takes a local pt-BR voice and never a remote one', () => {
    const remote = voice('Google português do Brasil', 'pt-BR', false);
    const local = voice('Luciana', 'pt-BR', true);
    expect(localVoice([remote, local])).toBe(local);
    expect(localVoice([remote])).toBeNull();
  });
  it('falls back to another local Portuguese voice', () => {
    const pt = voice('Joana', 'pt-PT', true);
    expect(localVoice([voice('Samantha', 'en-US', true), pt])).toBe(pt);
  });
});

describe('speak', () => {
  class Utterance {
    voice: SpeechSynthesisVoice | null = null;
    lang = '';
    constructor(public text: string) {}
  }
  beforeEach(() => vi.stubGlobal('SpeechSynthesisUtterance', Utterance));
  afterEach(() => vi.unstubAllGlobals());

  const synth = (voices: SpeechSynthesisVoice[]) =>
    ({ getVoices: () => voices, cancel: vi.fn(), speak: vi.fn() }) as unknown as SpeechSynthesis & {
      speak: ReturnType<typeof vi.fn>;
    };

  it('reads the text with the local voice', () => {
    const local = voice('Luciana', 'pt-BR', true);
    const s = synth([voice('Google português do Brasil', 'pt-BR', false), local]);
    expect(speak('verifique os logs', s)).toBe(true);
    const u = s.speak.mock.calls[0][0] as Utterance;
    expect(u.text).toBe('verifique os logs');
    expect(u.voice).toBe(local);
  });
  it('stays silent, and says so, with no local voice', () => {
    const s = synth([voice('Google português do Brasil', 'pt-BR', false)]);
    expect(speak('verifique os logs', s)).toBe(false);
    expect(s.speak).not.toHaveBeenCalled();
    expect(speak('x', undefined)).toBe(false);
  });
});
