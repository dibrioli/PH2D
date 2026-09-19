---
name: feedback-a-saturating-consumer-hides-a-divergence-a-linear-one-reveals
description: Uma divergência entre dois motores fica invisível enquanto o único consumidor dela SATURA
metadata:
  type: feedback
---

**Uma grandeza que dois motores calculam por caminhos diferentes pode divergir durante
meses sem nenhum gate acusar — se o único consumidor dela a passar por uma lei que
SATURA.** O primeiro consumidor **linear** nela é o primeiro instrumento que a vê, e
ele aparece como *«a minha wave partiu a paridade»*.

**Caso medido (2026-09-19, `line/3DModeling`).** A `W8` ligou uma tinta que lê a
curvatura linearmente. O gate de paridade CPU↔GPU reprovou à primeira, com `3`–`5`
bytes em oito píxeis. A **atribuição botão a botão** ilibou a lei nova e nomeou a
culpada:

```text
a fábrica                         →   0 píxeis fora, pior  0   (byte-idêntico)
só o contorno                     →   0,              pior  1
só as zonas + a saturação         →   0,              pior  1
a tinta por curvatura, nitidez 0,2→   0,              pior  1
a tinta por curvatura, nitidez 1  → 163,              pior  7
a tinta por curvatura, nitidez 2  → 168,              pior 13
a tinta por curvatura, nitidez 8  → 168,              pior 45
```

⭐⭐⭐ **A contagem SATURA em ~165 e a magnitude cresce LINEARMENTE com o ganho** — a
assinatura de *uma diferença pequena na grandeza, amplificada*. A divergência já lá
estava (fita `f64` achatada contra `field()` do WGSL) e o consumidor que ela tinha
passava-a por uma tabela pré-integrada com piso, que a **satura**.

**Why:** um gate de paridade não mede a grandeza — mede o pixel. Enquanto todo
consumidor a comprimir, a divergência não chega ao pixel, e um relatório verde diz
*«os dois motores concordam»* quando o que ele prova é *«os dois motores concordam no
que este consumidor deixa passar»*.

**How to apply:** quando uma wave nova parte uma paridade que estava verde, **varra o
GANHO antes de olhar para a lei**. Se o número de píxeis fora **satura** e a magnitude
**escala com o ganho**, o defeito é a montante e é pré-existente. ⛔ E então a barra
**não se afrouxa**: mede-se a lei nova no regime onde ela não amplifica, escreve-se a
tabela dose-resposta, e a dívida fica nomeada com o instrumento que falta — *medir a
GRANDEZA nos dois motores, e não o pixel*.

Irmão de [[reference_topic_measurement_discipline]] e de
[[feedback_a_barra_calibrada_sem_o_lado_aprovado]].
