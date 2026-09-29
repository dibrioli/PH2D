---
name: a-daemon-born-under-the-gpu-door-inherits-its-lock
description: O servidor do sccache arrancado por um cargo sob `PH2D_GPU=1` herdava o fd 9 do `ph2d-run.sh` e segurava o cadeado da placa até ao prazo do scope (30 min) — curado com `9>&-`; diagnóstico por `fuser` no ficheiro do cadeado
metadata:
  type: feedback
---

Medido 2026-10-01 (`line/3DModeling`): um gate de GPU ficou 14 min parado no `flock -w` com a máquina
a `load 1`, e o `.dono` do cadeado nomeava o MEU comando. `fuser -v /run/user/1000/ph2d-gpu.lock`
mostrou o dono real: um **`/usr/bin/sccache`** dentro da fatia de OUTRA linha, cujo script já tinha
acabado. O `cargo` sob a porta arranca o servidor do `sccache`, ele **destaca-se**, herda o fd 9 aberto
pelo `exec 9>` do `ph2d-run.sh` e segura o `flock` até o scope morrer pelo `RuntimeMaxSec` (30 min).

- ⭐ **Cura:** o comando é lançado com `9>&-` nos dois ramos (`systemd-run` e `timeout`) — quem segura
  a placa é o bash da porta, que vive exactamente o tempo do comando. Controlo: com o fd 9 aberto no
  chamador, o comando vê `0 1 2 255` com a cura e `0 1 2 255 9` sem ela.
- ⭐ **A cura irmã (`line/components`, medida 2026-09-29, a 1.ª ocorrência — três linhas presas
  21 min com o `.dono` VAZIO):** o servidor do `sccache` é posto a correr ANTES do `exec 9>` e fora
  da fatia, e a mensagem de espera lista os seguradores (`fuser`) quando o `.dono` está vazio. As
  duas curas vivem juntas no `ph2d-run.sh` desde a rodada de 02/10 (as duas linhas acharam o mesmo
  defeito sem se ver, e o integrador fundiu-as): o `9>&-` cobre QUALQUER daemon, e o arranque
  antecipado impede o servidor partilhado de nascer na fatia de uma linha e morrer com ela.
- ⚠️ Desprender na hora: `sccache --stop-server` (com a máquina ociosa — ele volta sozinho no
  próximo `cargo`).
- ⚠️ Cada worktree tem a SUA cópia do script: a cura só vale nas árvores que a tiverem, e o `sccache`
  é um só para todas ⇒ uma linha velha ainda pode prender a placa até o integrador fundir.
- ⛔ **Não edite o `ph2d-run.sh` enquanto uma corrida sua o executa** — o bash lê o script aos
  bocados e a corrida morre com `erro de sintaxe` no meio (mordeu-me nesta mesma cura).

**Why:** o silêncio de um `flock -w` lê-se igual a «outra linha está na placa», e o `.dono` mente,
porque nomeia o comando que pegou o cadeado e não o processo que o segura.
**How to apply:** um gate de GPU parado à espera da placa com a máquina ociosa ⇒ `fuser -v` no
ficheiro do cadeado antes de esperar mais. Família: [[a-killed-cargo-can-leave-a-zombie-that-holds-the-build-lock]].
