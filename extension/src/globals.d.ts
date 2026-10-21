declare const __TEST__: boolean;

declare module '*.md' {
  const text: string;
  export default text;
}
