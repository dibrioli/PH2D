---
name: feedback_if_relaxation_cannot_move_the_median_the_defect_is_in_the_connectivity
description: Uma cura que falha delimita a causa — se mover posições N vezes não move a mediana, o defeito está na estrutura, não nas posições
metadata:
  type: feedback
---

**Uma cura construída e REJEITADA pode valer mais que a feature — porque a forma
como ela falha delimita a causa.**

⛔ **Caso medido (quad-remesh, 2026-08-22).** Os quads saíam enviesados (mediana
`27°` de desvio de 90°, contra `6°` do oráculo). Construí a relaxação por **ajuste
de quadrado**: cada face pede o quadrado mais próximo de si (forma fechada, o
primeiro harmónico da DFT de quatro pontos), cada vértice vai para a média dos
pedidos. Dezasseis rondas:

| | aspecto máximo | ⭐ **enviesamento p50** | dobras |
|---|---|---|---|
| 0 rondas | `122,7` | **`27°`** | 171 |
| 16 rondas | `30,3` | **`26°`** | 576 |

⇒ **A cauda melhora `4×`; a mediana não se mexe, e o preço é `3,4×` as dobras.**

**Why:** uma relaxação **move vértices e mais nada**. Se dezasseis rondas de um
método cuja *função-objectivo é exactamente a propriedade que falta* não movem a
mediana, então endireitar um elemento desendireita o vizinho — o sistema já está no
ponto fixo que a estrutura permite. ⇒ o defeito está na **CONECTIVIDADE** (quem se
liga a quem, em que direcção as linhas correm), e nenhum alisador lhe toca.

⚠️ **E o preço tinha mecanismo, não ruído:** num vértice **irregular** o pedido é
contraditório — três quads a pedir 90° somam 270° e têm de fechar 360°. A relaxação
puxa com força onde não existe solução.

**How to apply:**
1. ⭐ **Meça a MEDIANA e a CAUDA em separado.** Uma cura que só melhora a cauda está
   a tratar outliers; o que o artista vê é a mediana.
2. ⭐ **Uma cura que não cura é um EXPERIMENTO com resultado** — guarde a tabela e o
   código vivo e testado, e ponha a constante a zero com o porquê ao lado. Ver
   [[feedback_documented_decision_chesterton_fence]] e a política de recusas
   medidas do `CLAUDE.md` §5.0.
3. ⚠️ **Toda cura leva a coluna do PREÇO** — aqui dobras, aresta, detalhe e relógio.
   *Uma cura sem a coluna do preço é meia medição.*
4. Depois de eliminar «posições», a sonda seguinte tem de olhar para a **estrutura**
   — e ⚠️ **certifique-se de que ela discrimina**: a primeira versão da nossa media
   **uma** família de linhas de grade e dizia que estávamos melhor que o oráculo,
   sobre uma malha com quatro vezes o enviesamento dele. Ver
   [[feedback_a_gate_that_asserts_what_construction_guarantees_is_a_tautology]] e
   [[feedback_a_global_extreme_is_not_a_per_face_ruler]].
