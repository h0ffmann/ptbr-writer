// ptbr-writer for OpenCode — the ask-levels hook: a message that starts with `ptbr` gets one
// instruction, built by the pinned `ptbr ask-levels` binary (the same one the Claude Code hook
// runs); any other message gets nothing. Installed by scripts/install-opencode.sh.
import { spawnSync } from "node:child_process"
import { randomBytes } from "node:crypto"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

// plugins/ptbr-writer.js sits next to ptbr-writer/ (bin, scripts, default toml) and skills/.
const base = join(dirname(fileURLToPath(import.meta.url)), "..")
const home = join(base, "ptbr-writer")

export const PtbrWriter = async () => ({
  "chat.message": async (input, output) => {
    const text = output.parts.filter((p) => p.type === "text").map((p) => p.text).join("\n").replace(/^\s*["']/, "") // `opencode run` quotes the message
    if (!/^\s*ptbr/i.test(text)) return
    const r = spawnSync(
      "bash",
      [join(home, "bin/run.sh"), "ask-levels", join(base, "skills/ptbr-writer/SKILL.md"), join(home, "scripts/to-pdf.sh"),
       join(home, "ptbr-writer.default.toml"), `bash ${join(home, "bin/run.sh")} reading-time`],
      { input: JSON.stringify({ prompt: text }), encoding: "utf8" },
    )
    let ctx
    try { ctx = JSON.parse(r.stdout).hookSpecificOutput.additionalContext } catch { return }
    if (!ctx) return
    // The instruction names Claude Code's tool; in OpenCode it is `question`.
    const note = "No OpenCode, a ferramenta AskUserQuestion é a `question` (header, pergunta, opções)."
    output.parts.push({
      id: "prt_" + randomBytes(13).toString("hex"),
      sessionID: input.sessionID,
      messageID: output.message.id,
      type: "text",
      text: `${ctx}\n${note}`,
      synthetic: true,
    })
  },
})
