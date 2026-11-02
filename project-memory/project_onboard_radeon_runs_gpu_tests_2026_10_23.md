---
name: project-onboard-radeon-runs-gpu-tests-2026-10-23
description: "A Radeon 610M do 9950X corre 5 573/5 580 testes de placa (só Luz 2D falha) sem tocar na NVIDIA; a fila da placa custou ≥ 2 h 45 em 6 dias, segurada sobretudo pelo nextest-impacted inteiro"
metadata:
  node_type: memory
  type: project
  originSessionId: bf5b7b6f-71f3-47eb-8a24-37b8f49014aa
  modified: 2026-10-23T22:43:23.931Z
---

Medido em 23/10 (dono: «teste tudo e a placa onboard, sem atrapalhar as linhas»).

- **Fila (18–23/10):** ≥ 36 esperas, ≥ 2 h 45 min somadas (transcripts + saídas de tarefas de fundo), máx. 25 min.
  Quem segura: sobretudo `nextest-impacted.sh` inteiro — o porteiro do `ph2d-run.sh` segura a vez até o
  COMANDO acabar, mesmo nos minutos em que a suíte só corre testes de CPU. Depois: fotos (`fotografa_cena`) e oráculos.
- **Radeon só:** `VK_DRIVER_FILES`/`VK_ICD_FILENAMES=radeon_icd.json` + `__EGL_VENDOR_LIBRARY_FILENAMES=50_mesa.json`
  + `__GLX_VENDOR_LIBRARY_NAME=mesa` ⇒ o wgpu vê só RADV (Vulkan) e radeonsi (GL). Amostra de 1 s no `nvidia-smi`
  durante 22 min: zero processos do `main`.
- **Resultado (20 crates de placa, `--profile smoke`):** 5 573 passam, 7 falham — TODAS da Luz 2D (bit a bit
  `a_luz_de_fora_nao_atravessa` ×2, holográfica ×3, G5 aloca `hrc_r0` 524288 vs teto 3072, custo do miolo 1,001 vs 0,8).
  Na NVIDIA (junit do ship 05:28) as 7 passam. Tempo: a maioria igual ou mais rápida; a Luz 2D ~5,6× mais lenta.

**Why:** decide se uma 2.ª placa (ou a onboard) tira as linhas da fila.
**How to apply:** a onboard serve aos testes comuns; Luz 2D e medições ficam na NVIDIA. O ganho maior e grátis é o
porteiro largar a vez quando o processo de placa acaba, não no fim da suíte. Ver [[feedback_full_suite_only_before_integration]].
