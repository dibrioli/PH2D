---
name: a-ruler-with-n-columns-answers-n-questions
description: Régua multi-coluna — citar a coluna errada devolve 0,00% sobre o defeito que está na foto
metadata:
  type: feedback
---

⛔⛔ **Uma régua com N colunas responde a N perguntas, e citar a errada devolve `0,00 %` sobre o
defeito que está no ecrã.**

Medido 2026-09-15 (`line/Vector`, a cena do canvas preso a ossos): o dono devolveu uma foto da arte
**RASGADA**. O doc-comment da constante do ângulo justificava a escolha dizendo que ela estava *«bem
abaixo do ponto em que o mapa dobra sobre si mesmo»*, citando a `ph2d_skeleton::fold`. Era **verdade
e irrelevante**:

| coluna da MESMA régua | leitura sobre a foto | o que ela mede |
|---|---:|---|
| `inverted` — a que eu citei | **`0,00 %`** | arte do avesso |
| `orphan` — a que gritava | **`33,85 %`** | arte fora do alcance de todo osso |

⚠️ **E a coluna certa era aquela que o doc da própria régua chama de ANTI-VACUIDADE** (ela existe
para impedir que uma «cura» que deixe a arte toda órfã se leia perfeita). Eu li-a como escrituração
em vez de diagnóstico.

**Why:** rasgar e inverter são defeitos diferentes do mesmo mapa. A leitura estava correcta; a
pergunta não era aquela. É a mesma forma de
[[feedback_a_leak_ruler_masked_by_the_products_own_predicate_hides_the_leak]] e de *«uma medição
sobre o eixo que não dobra mede o caso que não existe»*.

**How to apply:** antes de citar uma régua para justificar um número, **imprima TODAS as colunas dela
sobre a fixtura** e diga qual delas seria diferente se o defeito estivesse presente. Se nenhuma
mudaria, a régua não é testemunha daquela escolha. ⇒ ver também
[[reference_topic_measurement_discipline]].
