---
name: feedback_clear_the_heavy_queue_before_running_it
description: "`fila-pesada.sh correr` corre TUDO o que está na fila; veja com `ver` antes (05/11: uma entrada velha de 4 h arrancou)"
metadata:
  type: feedback
---

`bash scripts/fila-pesada.sh correr` executa tudo o que está lá, inclusive entradas esquecidas. Em 05/11 uma entrada velha de ~4 h (`corre_f0.sh`) arrancou por ~10 min e reescreveu os logs de 01/11 num scratchpad antigo (conclusões em E9 §8); foi parada, sem efeito na árvore.

**Why:** a fila não distingue o que é de hoje do que ficou de dias antes.

**How to apply:** `fila-pesada.sh ver` primeiro; tire o que não é seu; só então `correr`. Em scratchpad partilhado, confira de quem é cada script antes de o correr.
