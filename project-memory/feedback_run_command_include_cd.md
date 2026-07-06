---
name: feedback-run-command-include-cd
description: "Sempre prefixar comandos de rodar/executável com o `cd` da pasta (worktree copiável de uma vez)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f54a187a-2ffb-45e8-84f1-c5dc8fb4e843
---

Ao dar ao Enio QUALQUER comando pra rodar o app/executável (smoke, `cargo run`, etc.), **inclua o `cd <pasta>` junto** no mesmo bloco, copiável de uma vez.

**Why:** no [[project_multiagent_modo_l_2026_07_05]] (Modo L) o código vive no worktree (ex. `Worktrees/line-audio`), NÃO no repo principal; o Enio roda de outro diretório, então sem o `cd` o comando falha ou roda a árvore errada.

**How to apply:** formato `cd <caminho-absoluto-do-worktree> && <ENV=..> cargo run -p <bin>`. Ex.:
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && PH2D_AUDIO_FILE=/som.wav PH2D_AUDIO_SMOKE=1 cargo run -p ph2d-host-desktop`
