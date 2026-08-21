---
name: feedback-a-ruler-that-deduplicates-cannot-report-duplication
description: Um gate que conta num Set não pode denunciar duplicação — a característica de Euler saía CERTA sobre uma malha que já não era variedade
metadata:
  type: feedback
---

Se o defeito que você teme é **duplicação**, a régua tem de contar por
**OCORRÊNCIA**. Um `BTreeSet`/`HashSet` na régua funde as duas cópias e devolve o
número certo sobre o estado errado.

**Why:** medido em 2026-08-21 (quad remesher, F1). O gate `the_genus_survives`
calculava `χ = V − E + F` com as arestas num `BTreeSet`. Quando o flip passou a
criar **duas** arestas entre o mesmo par de vértices (a mesma diagonal criada por
duas trocas da mesma rodada), o `Set` fundiu-as: `E` saiu como se a malha fosse
variedade, `χ` deu `2` na esfera, **o gate ficou verde** — e a malha tinha 18
vértices não-manifold. Três fases a jusante liam 33 singularidades falsas.

⚠️ **E o sintoma a jusante não aponta para cá.** Quem via o defeito era o traçado,
duas crates adiante, e a nota que ele escreveu culpava a fase errada.

**How to apply:** num gate de integridade de malha (ou de qualquer estrutura em que
"o mesmo elemento duas vezes" seja o defeito), conte com `BTreeMap<chave, usize>` e
afirme a **multiplicidade**, não a presença. Faça as perguntas **separadas** — elas
têm causas diferentes e curas diferentes:

| pergunta | conta | o que denuncia |
|---|---|---|
| aresta não-dirigida com 1 face | `== 1` | **borda** / rasgo |
| aresta não-dirigida com ≥ 3 faces | `> 2` | **não-variedade** |
| aresta DIRIGIDA usada > 1 vez | `> 1` | orientação inconsistente ou **face duplicada** |

⚠️ A terceira não move nenhuma das duas primeiras — um gate que só conta arestas
não-dirigidas passa por cima dela. Família:
[[reference_topic_gate_discipline]]; irmã de
[[feedback_a_green_gate_may_be_green_by_accident]] e de
[[feedback_a_ratio_between_two_sick_channels_is_green_by_construction]]. E a régua
que sobrou depois disto está em
[[feedback_a_conserved_invariant_cannot_grade_quality]].
