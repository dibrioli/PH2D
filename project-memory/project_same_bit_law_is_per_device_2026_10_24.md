---
name: project-same-bit-law-is-per-device-2026-10-24
description: "Dono decidiu (24/10, bug 7 da Luz 2D): «não muda um bit» vale DENTRO de cada placa; entre placas aceita-se a diferença mínima que o compilador da placa faz (1 ULP do f16)"
metadata:
  node_type: memory
  type: project
  originSessionId: b64a7c69-7322-4a72-9d9b-5f89407b0e9d
  modified: 2026-10-24T11:12:51.574Z
---

Dono, 24/10, resposta «A» à pergunta do bug 7 (`docs/Luz2D/BUGS_luz2d.md`): uma cura de otimização prova-se
«igualzinho» (0 texels) em cada placa onde o compilador não reescreve a conta; numa placa cujo compilador contrai
ou reassocia (RADV: as diferenças morrem com `ACO_DEBUG=noopt`), aceita-se a diferença mínima — 1 ULP do f16, em
poucos texels — e o teste DIZ qual das duas leis aplicou. Recusada a (B): igualar em toda placa = lutar contra o
compilador de cada uma (mais lento, quebradiço).

**Why:** o WGSL permite a contração; a NVIDIA dava 0 por acaso do compilador dela, não por lei.
**How to apply:** teste «não muda um bit» novo ou velho mede o tamanho da diferença e não só a contagem; ver
[[project-onboard-radeon-runs-gpu-tests-2026-10-23]].
