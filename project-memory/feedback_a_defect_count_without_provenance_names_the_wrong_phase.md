---
name: feedback-a-defect-count-without-provenance-names-the-wrong-phase
description: Um número de defeitos sem a PROVENIÊNCIA diz que há trabalho e não diz em que fase — e o palpite sobre a fase custou uma correção de rumo no mesmo dia
metadata:
  type: feedback
---

Quando um passo final de uma cadeia reporta *"N defeitos"*, faça cada item
carregar **de onde veio**. Sem isso, `N` mede a cadeia inteira e a atribuição da
culpa é palpite.

**Why:** medido em 2026-08-21 (quad remesher). A montagem final reportava
`47 vértices irregulares`, e o plano tinha acabado de apontar a fase do **campo**
como o próximo trabalho, com uma medição real a apoiá-lo. Marcar cada vértice da
saída com a origem respondeu noutro sítio:

| origem | esfera | toro | cubo |
|---|---|---|---|
| canto do layout | 32 | 46 | 90 |
| centro de patch | 15 | 20 | 38 |
| **arco · raio · grade** | **0** | **0** | **0** |

⭐ **100 % vinham do layout, e a fase final não criava nenhum.** O campo estava
bom na configuração que o produto de facto corre; o número mau daquela medição era
de uma configuração que ninguém usa. *A correção de rumo veio no mesmo dia, mas só
porque a proveniência existiu.*

⚠️ **E a decomposição fez mais do que atribuir:** ela deu a **conta do chão**.
15 centros = um por patch de valência ≠ 4; 32 cantos = junções em T. Zero de cada
um daria 8, que é exactamente onde a referência fica. *Uma anatomia do defeito é um
plano de trabalho com números.*

**How to apply:**

1. O vetor de proveniência cresce **junto** com o de dados, num `struct` com um
   único `push` que exige as duas coisas — ⛔ nunca dois `Vec` paralelos com um
   comentário a pedir que assim fiquem. Um `push` esquecido num de cinco sítios dá
   uma decomposição **deslocada**, que soma certo e culpa a fase errada.
2. Confira uma das classes contra uma **fonte independente** (aqui: o nº de
   centros irregulares tem de bater com o nº de patches de valência ≠ 4, que o
   layout sabe). Sem essa conferência, mutar o rótulo do centro **sobrevivia**.
3. ⚠️ Classes cujos membros são **sempre** saudáveis são in-matáveis por
   construção — trocar-lhes o rótulo não move número nenhum. Escreva isso ao lado
   do gate em vez de construir instrumento para uma afirmação que ninguém faz.

Irmã de [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] (aqui as
duas leituras aparecem lado a lado) e de
[[feedback_a_round_that_never_reports_its_residual_is_a_silent_lie]] — nos dois
casos o instrumento que faltava era **dentro** do passo, não à volta dele.
