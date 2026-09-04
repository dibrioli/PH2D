---
name: feedback-interval-error-is-first-order-so-a-uniform-grid-pays-n-cubed
description: "Erro de aritmética de intervalos é de 1.ª ordem na largura — grelha uniforme paga n³ para o dividir por n; a cura é branch-and-bound, e o minorante sai de graça de uma caixa degenerada"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-02T21:59:39.168Z
---

Aritmética de intervalos sobre-estima por **dependência** (a mesma variável entra várias vezes e o
intervalo trata cada ocorrência como independente), e o excesso é de **primeira ordem** na largura
da caixa. ⇒ uma **grelha uniforme** paga `n³` avaliações para dividir o excesso por `n`, o que é
inutilizável.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02 — o bound de Lipschitz da composição de três
deformadores; alvo `≈ 5`, produto de hoje `15,85`):

| grelha | caixas | bound | relógio |
|---:|---:|---:|---:|
| `8³` | `512` | `33,7` | `0,23 ms` |
| `16³` | `4 096` | `18,9` | `1,78` |
| `32³` | `32 768` | `12,6` | `14,1` |
| `64³` | `262 144` | `9,7` | **`110`** |

⭐⭐⭐ **Com branch-and-bound: `8,88` em `3,2 ms` com `3 000` caixas** — melhor do que a grelha de
`64³` e **34× mais barato**.

**How to apply:**
- ⭐ **Refine só quem disputa o máximo.** A esmagadora maioria das sub-caixas está muito abaixo do
  pior e parti-las não muda a resposta. Fila de candidatas, parte-se sempre a de maior majorante,
  pelo eixo **mais largo**.
- ⭐⭐ **O minorante que diz quando parar sai de GRAÇA**: a MESMA lei numa caixa **degenerada** (um
  ponto) é exacta, logo é um minorante válido do máximo verdadeiro. Pára-se quando
  `majorante ≤ minorante·(1+tol)`. *Sem ele não se sabe se refinar mais compra alguma coisa.*
- ⚠️ **Um orçamento de caixas** é obrigatório: o valor devolvido é um majorante válido refinado ou
  não, então desistir é seguro — e sem orçamento a lei corre para sempre numa peça patológica.
- ⭐ **O majorante de norma escolhe-se MEDINDO-O na família em que vai viver, não pelo nome**: aqui
  `‖M‖_F` ficou `5,1 %` acima do `σ_max` verdadeiro e `√(‖M‖₁‖M‖∞)` ficou `27,6 %` — porque estes
  jacobianos têm **um** valor singular grande e os outros `≤ 1`.
- ⛔ **A régua do próprio tipo de intervalo é a CONTENÇÃO**, e a metade da **derivada** é a que quase
  não se escreve: trocar o sinal da derivada do `cos` passa no gate que só olha valores e morre no
  que olha as parciais ([[feedback_a_gradient_gate_says_may_punch_only_the_image_says_punches]]).
