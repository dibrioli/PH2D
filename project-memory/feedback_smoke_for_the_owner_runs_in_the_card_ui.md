---
name: feedback-smoke-for-the-owner-runs-in-the-card-ui
description: O smoke do dono corre na UI de cartões (a omissão); PH2D_CARDS=0 mostra o painel clássico e o dono lê isso como regressão
metadata:
  type: feedback
---

O comando de smoke dado ao dono nunca leva `PH2D_CARDS=0`; a cena corre e fotografa-se na UI de cartões, a omissão.

**Why:** dono 11/11, smoke da cena 120 + Outline Rock: a primeira dica usou `PH2D_CARDS=0`; ele viu o painel clássico, achou «bizarro, regressões», e eram só o painel antigo.

**How to apply:** `PH2D_CARDS=0` é saída de referência, só para comparar a mão; fora disso, nenhum comando de smoke o define, e `fotografa_cena.sh` fotografa na UI por omissão.
