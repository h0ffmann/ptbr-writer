// The few things the content generator needs from Node: files, one git command and an exit code.
// Paths are relative to site/content, where `gleam run` runs.
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";

// The file's text, or "" when it does not exist.
export function read(path) {
  return existsSync(path) ? readFileSync(path, "utf8") : "";
}

export function write(path, text) {
  writeFileSync(path, text);
}

// Stop the generator with an exit code, so that `just site` and CI fail.
export function exit(code) {
  process.exit(code);
}

// The short hash of the commit that is built, or "unversioned" outside a Git checkout.
export function revision() {
  try {
    return execSync("git rev-parse --short HEAD", { stdio: ["ignore", "pipe", "ignore"] })
      .toString()
      .trim();
  } catch (_) {
    return "unversioned";
  }
}
