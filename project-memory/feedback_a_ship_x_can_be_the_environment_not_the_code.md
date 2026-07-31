---
name: feedback-a-ship-x-can-be-the-environment-not-the-code
description: Um ✗ de gate pode ser o AMBIENTE — o target/ symlinkado que evaporou (✗ sem rodar) ou o disco a 100%, que se disfarça de "linking with clang failed"
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

## A segunda face, medida em 2026-07-30: o DISCO CHEIO, que se disfarça de erro de código

O símbolo `✗` da tmpfs pelo menos **diz** que não criou diretório. O disco cheio não: ele sai como

```
error: linking with `clang` failed: exit status: 1
error: could not compile `ph2d-host-desktop` (bin "ph2d-host-desktop")
```

— que se lê como **erro de link do código que você acabou de integrar**, e manda você caçar símbolo
duplicado, feature quebrada ou dep faltando. Medido: `/` a **100% (946G de 950G)**, com **842 GB** nos
`target/` das worktrees. Um `git commit` na mesma janela falhou com
`fatal: unable to write loose object file: Não há espaço disponível`, que é o único erro honesto do lote.

**Uma jornada de integração multi-linha ENCHE o disco**, e é previsível: cada worktree carrega um
`target/` de 46-159 GB, e o gate da árvore combinada roda em várias delas na mesma sessão.

**How to apply:** antes de diagnosticar QUALQUER falha de link/compilação num gate de integração, rode
`df -h /`. Se estiver perto de 100%: `du -sh Worktrees/*/target | sort -h`, e apague os das linhas
**já integradas** (`git rev-list --count main..line/<x>` = 0 **e** `git status --porcelain` vazio nas
duas — confira as duas coisas, não uma). ⚠️ **Nunca o `~/.cache/sccache`** (DIRETIVA_FIM_DE_DIA §3):
é ele que torna o rebuild barato. E depois de um disco cheio, **`git fsck --connectivity-only` antes
do próximo commit** — uma escrita de objeto interrompida é a coisa cara; hoje saiu só com *dangling*
(resíduo normal de rebase), mas isso se confere, não se presume.

O corolário geral: **um gate que não conseguiu RODAR não é um gate vermelho** — é um gate ausente, e a
diferença some no resumo, que só mostra ✓/✗. Vale para qualquer runner que reporte por linha-resumo.
Relacionado: [[feedback_pipe_masks_script_exit_code]] (o `EXIT=$?` depois de um pipe também mente sobre
quem falhou) · [[project_modo_l_speed_hole_worktree_targets_slow_path]] (por que o target mora em tmpfs).
