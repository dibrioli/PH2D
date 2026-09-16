---
name: the-gpu-cannot-be-divided-only-excluded
description: 50% de uma GPU de consumidor não é exprimível — a regra honesta é exclusão com prazo, e ela cobre o defeito real
metadata:
  type: feedback
---

Perguntado à máquina em 2026-09-15 (RTX 5060 Ti, driver proprietário), antes de
escrever a regra que o Enio pediu:

| porta | resposta |
|---|---|
| **MIG** (partição de placa) | `[N/A]` — placa de consumidor |
| cgroup **`dmem`** | existe no kernel, **não delegado** ao utilizador; e o driver proprietário não implementa contabilidade DRM |
| compute mode `EXCLUSIVE_PROCESS` | só cobre **CUDA**; este repo desenha por **Vulkan/wgpu** |
| quota em `sysfs` | nenhuma |

⛔ **Não há como dar 50 % da placa a um agente.** Escrevê-lo à mesma seria um
limite que não diz de que recurso é — o que o `CLAUDE.md §0.0` proíbe.

**Why:** e a regra honesta é **mais forte** que a pedida. O defeito real (14/09)
não foi partilhar a placa: foi **segurá-la** — uma sonda pendurada ficou `56 min`
com o driver e `2,6 GB`, em `S (sleeping)` com o tempo de CPU **parado**. Uma
quota de 50 % não teria ajudado em nada. *Antes de implementar a métrica pedida,
pergunte qual era o defeito: partilhar e segurar pedem curas opostas.*

**How to apply:** `PH2D_GPU=1 bash scripts/ph2d-run.sh <cmd>` — `flock` com espera
limitada (a recusa sai em `exit 75`/`EX_TEMPFAIL` e **diz quem segura**: linha,
pid, desde quando, comando) mais um prazo que mata o detentor pendurado e liberta
a fechadura sozinha. ⛔ Nunca forçar: duas linhas na placa ao mesmo tempo foi o que
pendurou o driver.

⚠️ E o prazo tem de ser do **scope**, não de um `timeout`: um binário de teste
reparenta-se ao `systemd --user` — medido com controlo, o scope deixa **0** órfãos
e o `timeout` deixa **1**.

Ver [[a-per-command-ceiling-does-not-compose-only-a-slice-does]] e
a família da régua em [[reference-topic-measurement-discipline]] (*o `ps` mostra a média da VIDA do processo: um binário BLOQUEADO lê-se ali como 95 %*).
