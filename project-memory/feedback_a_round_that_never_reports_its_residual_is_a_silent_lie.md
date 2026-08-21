---
name: feedback-a-round-that-never-reports-its-residual-is-a-silent-lie
description: Um `.round()` que não sabe dizer QUANTO teve de andar esconde um erro de fórmula por meses — e o resíduo dele é o instrumento que o nomeia
metadata:
  type: feedback
---

Quando o código faz `x.round()` (ou `snap`, ou `clamp`, ou "isto tem de ser um
inteiro a menos de erro numérico"), **guarde o resíduo** `|x − x.round()|` e
reporte-o. Sem ele, uma fórmula errada e uma fórmula certa produzem exactamente a
mesma saída na fixtura fácil.

**Why:** medido em 2026-08-21 (campo cruzado, F2). O índice de singularidade saía
de `round(total / (π/2))` e a fórmula esquecia uma parcela. Numa malha uniforme
essa parcela vale `4π/N` — invisível. Numa malha com triângulos de tamanhos
diferentes ela domina:

| malha | resíduo | ambíguos | sintoma |
|---|---|---|---|
| esfera uniforme (13 682 v) | 0,0009 | 0 | nenhum |
| ⛔ esfera sacudida | **0,4999** | **1 468** | soma dos índices `−147` onde a topologia exige `+8` |

⭐ **`0,5` é o máximo possível: um empate — o `round` a decidir por sorteio.** E o
resíduo da malha uniforme tinha a **ordem de grandeza da parcela que faltava**, o
que escreveu a hipótese sozinha. *O instrumento não só denunciou o erro: nomeou-o.*

⚠️ **E o gate topológico estava VERDE há dois meses**, porque as quatro fixturas
dele eram todas uniformes e a soma fechava por **cancelamento** dos
arredondamentos errados.

**How to apply:**

1. Toda conta que termina em `round`/`snap` porque *"tem de ser inteiro"* devolve
   também o pior resíduo e quantos casos ficaram acima de `0,25`. Um resíduo que
   cresce é um erro de modelo a aparecer, não ruído.
2. ⭐ **Instrumente as perguntas TODAS de uma vez.** A auditoria devolveu
   `desistiu 0 · colisões 0 · resíduo 0,4999` — as duas primeiras **excluíram** a
   hipótese óbvia (os `return 0` silenciosos) no mesmo instante em que a terceira
   apontou a real. Medir uma de cada vez teria custado três voltas.
3. Ao escolher entre variantes de fórmula, **corra as três lado a lado** e deixe o
   resíduo decidir — e dê à sonda um **controle** que prove que a grandeza nova
   está certa antes de ela julgar (aqui, Gauss–Bonnet: `Σ K_v/2π = χ`).

Irmã de [[feedback_a_conserved_invariant_cannot_grade_quality]] (a soma não podia
ver isto) e de [[feedback_a_green_gate_may_be_green_by_accident]]; a fixtura que
faltava é o caso de
[[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]].
