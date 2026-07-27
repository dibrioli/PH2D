---
name: reference-prompt-deck-app
description: "O Prompt Deck (biblioteca de prompts do Enio) mora FORA do repo, em \"Meus Apps\"; a fonte única é prompts.json e três saídas são geradas"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 273e7943-ca95-473e-87b3-5ecf7108cd0c
  modified: 2026-07-25T20:24:34.158Z
---

Apps pessoais do Enio vivem em `~/Área de trabalho/Meus Apps/<Nome Legível>/`
(convenção: `.desktop` dentro da pasta, script `snake_case.py`, `icon.svg`).
Irmãos: GPU Sentinela, Teste Teclado.

O **Prompt Deck** (criado 2026-07-25) fica em `~/Área de trabalho/Meus Apps/Prompt Deck/`:

- `prompts.json` — **a fonte única**. É o ÚNICO arquivo a editar à mão.
- `prompt_deck_sync.py` — o gerador. Reescreve as três saídas:
  `index.html` (bloco entre marcadores `SEED:BEGIN/END`) · `espanso/` ·
  `~/.claude/commands/pd-*.md` (slash commands **de usuário**, valem em todo projeto).
  `--check` acusa drift; ele round-trip-verifica o YAML gerado.
- `prompt_deck.py` — o que a tecla **F3** chama: picker (fuzzel) → clipboard → Ctrl+V.
  Entrega o corpo com os campos reduzidos ao rótulo (`{{Objetivo}}`).

**Colar, nunca digitar**: uma quebra de linha injetada como tecla vira Enter, e Enter
SUBMETE na caixa do Claude Code e no terminal — um prompt de 20 linhas viraria 20
mensagens. Por isso a entrega é sempre clipboard, e o espanso usa `backend: Clipboard`.

Editar um prompt = editar `prompts.json` + rodar o sync. Editar direto o `index.html`,
o YAML do espanso ou os `pd-*.md` é trabalho que o próximo sync apaga.

Ver [[reference-kde-plasma6-global-shortcut]] para como o F3 foi amarrado.
