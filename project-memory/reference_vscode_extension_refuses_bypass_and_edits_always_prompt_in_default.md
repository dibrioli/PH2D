---
name: vscode-extension-refuses-bypass-and-edits-always-prompt-in-default
description: "A extensão VSCode ignora o defaultMode bypassPermissions dos settings e, em modo default, TODO edit pede aprovação no diff — allowlist não alcança; a cura são 2 chaves no settings DO VSCODE"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 566970c3-88e8-4ddc-b8bc-f9b1452eaa61
  modified: 2026-08-24T01:29:37.111Z
---

**O fenômeno (medido 2026-08-23):** toda jornada de integração era interrompida «após a integração
e antes da CI, pedindo permissão para editar docs» — apesar de `~/.claude/settings.json` **e**
`.claude/settings.local.json` terem `defaultMode: "bypassPermissions"` + `Edit`/`Write`/`Bash(*)`
liberados desde 18–21/08.

**O mecanismo (3 fatos, confirmados no package.json da extensão 2.1.241 + guia oficial):**

1. A extensão VSCode **recusa** `bypassPermissions` como modo inicial se o settings do VSCode não
   tiver `claudeCode.allowDangerouslySkipPermissions: true` — e cai **silenciosamente** em `default`.
   Transcripts provam: toda sessão nascia com `"permissionMode":"default"`.
2. Em modo `default`, **todo Edit/Write passa pelo diff de aprovação da extensão — a allowlist não
   alcança esse portão** (regra `Edit` liberada não suprime o prompt). Só o MODO libera edits:
   `acceptEdits` ou `bypassPermissions`.
3. O modo escolhido no seletor da janela **zera a cada conversa nova** (numa sessão de 23/08 o Enio
   virou para `acceptEdits` 5 vezes e ele voltou a `default`). Ajustar o seletor não é cura.

**A cura (onde ela mora — `~/.config/Code/User/settings.json`, o settings DO VSCODE, não os do Claude):**

```json
"claudeCode.allowDangerouslySkipPermissions": true,
"claudeCode.initialPermissionMode": "bypassPermissions"
```

Vale para conversas **novas**; janelas já abertas continuam no modo em que estão até reabrir.
Alternativa mais conservadora: `"acceptEdits"` no `initialPermissionMode` (libera edits; outras
ferramentas seguem a allowlist — mas ferramentas MCP fora da allowlist voltariam a pedir).

**A lição transferível:** CLI e extensão têm cadeias de resolução de modo **separadas** — os
settings do Claude Code mandam no CLI; o modo inicial da extensão só obedece ao settings do VSCode.
Sintoma-assinatura nos transcripts: `permissionMode:"default"` no nascimento de toda sessão apesar
de `defaultMode` dizer outra coisa. Irmã de [[a-rule-only-exists-if-it-is-on-the-path-of-who-executes-it]]:
a regra estava escrita, mas não no caminho de quem decide o modo.

⚠️ Multi-máquina: a cura é **por máquina** (settings do VSCode não viajam pelo repo). No Mac:
`~/Library/Application Support/Code/User/settings.json`.
