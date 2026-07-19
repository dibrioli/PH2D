---
name: feedback-a-ship-x-can-be-the-environment-not-the-code
description: O target/ do primário é symlink pra /dev/shm e evapora no reboot — o ship.sh marca ✗ em clippy E nextest sem que nenhum dos dois tenha rodado
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 50723bb0-6f74-4589-81dc-ee242a680d8c
---

O `target/` do checkout **primário** é um symlink para `/dev/shm/ph2d-target` (tmpfs, DIRETRIZ §6).
O tmpfs **evapora no reboot**, e o symlink fica pendurado. Aí o `ship.sh` marca:

```
✗ clippy (workspace, all-targets, CI features)
✗ nextest run --workspace (ci-test)
```

…e **nenhum dos dois rodou**. A causa está enterrada acima, uma linha só:
`error: failed to create directory .../target` · `Caused by: Not a directory (os error 20)`.

**Why:** dois `✗` de gates pesados convidam a caçar regressão no código que acabou de ser integrado —
e o código pode estar impecável. O `✗` é do **ambiente**. Pior: o único `✗` legítimo da mesma rodada
(fmt) fica no meio de dois falsos, então a lista de "o que consertar" nasce com 3 itens e 2 são fumaça.
Isto morde exatamente o integrador, que roda o ship no primário logo depois de um reboot ou de uma
limpeza de fim de dia.

**How to apply:** ship com `✗` em clippy/nextest → **antes de ler o código, leia o log**: se o texto for
`failed to create directory ... Not a directory`, é o symlink. `mkdir -p /dev/shm/ph2d-target` e re-rode.
Confira com `ls -ld target` (symlink) + `ls -ld /dev/shm/ph2d-target` (o alvo existe?) — o `readlink -f`
**mente aqui**: ele resolve o caminho e imprime, exista o destino ou não.

O corolário geral: **um gate que não conseguiu RODAR não é um gate vermelho** — é um gate ausente, e a
diferença some no resumo, que só mostra ✓/✗. Vale para qualquer runner que reporte por linha-resumo.
Relacionado: [[feedback_pipe_masks_script_exit_code]] (o `EXIT=$?` depois de um pipe também mente sobre
quem falhou) · [[project_modo_l_speed_hole_worktree_targets_slow_path]] (por que o target mora em tmpfs).
