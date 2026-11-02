---
name: feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt
description: "Missão autónoma parou num «allow Bash command» de um subagente (20/10) — mesmo em bypass, `cd X && bash -c '… rm -rf $V …'` pede; cura = hook PreToolUse allow + formas proibidas no briefing"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 62b742d1-05c6-426f-9383-94fdb08c45f9
  modified: 2026-10-20T04:47:56.281Z
---

O dono (20/10, missão Pincéis): *«eu disse pra não pedir permissão mas uma permissão de "allow Bash Command" fez tudo
parar. não permita isso»*. O modo era `bypassPermissions` em todos os settings e no VSCode; o comando de um subagente
de fundo — `cd <dir> && bash -c '…; rm -rf $C; …'` — disparou mesmo assim o pedido do próprio Claude Code.

**Why:** numa missão sem ninguém ao teclado, um único pedido congela o agente (e quem espera por ele) por horas.

**How to apply:** (1) o hook `~/.claude/hooks/aprova-sem-perguntar.sh` (PreToolUse `*` → `allow`) está no
`.claude/settings.local.json` do PH2D e da worktree `line-PainterWatercolor` (git-ignorado); os guardas do repo que
dão `deny` continuam a vencer. (2) Todo briefing de agente leva as formas proibidas: `cd X && …`, `bash -c`/`sh -c`,
`rm -rf` com variável/glob, `kill` de PID alheio, escrever em `.git/`/`.claude/`/rc da shell. Ver
[[vscode-extension-refuses-bypass-and-edits-always-prompt-in-default]].
