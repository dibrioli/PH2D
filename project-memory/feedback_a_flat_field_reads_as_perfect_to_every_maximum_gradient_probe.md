---
name: feedback-a-flat-field-reads-as-perfect-to-every-maximum-gradient-probe
description: Campo ACHATADO tem gradiente ZERO — toda sonda de máximo de ‖∇f‖ o lê como perfeito; o defeito oposto precisa de outra régua
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T21:58:02.199Z
---

Uma sonda que mede o **máximo** de `‖∇f‖` e reprova acima de `1` apanha o campo que cresce
**depressa demais** (a marcha atravessa a superfície). ⛔ **Ela é estruturalmente cega ao defeito
oposto:** um campo que **não cresce** tem gradiente **zero**, e zero passa em qualquer barra de
máximo. *Um `0,00` de «achatado» e um `0,00` de «aqui não há nada» são o mesmo byte.*

Caso medido (PH2D, `line/3DModeling`, 2026-08-31, foto do Enio): a banda da dobra estava escrita no
`z` de **entrada** em vez de no ângulo. Com `b` saturado, `x` e `z` de saída deixam de depender do
eixo ⇒ a peça ganha uma **cauda semi-infinita** cujo campo é uma planície à superfície:

| `z` | campo em `x = −0,2` |
|---:|---:|
| `0,50` | `+0,00047` |
| `20,0` | `+0,00047` |

A sonda de gradiente lia **`0,1161`** e chamava-lhe seguro.

⚠️ **E o instrumento óbvio também não serve:** o contador de raios que esgotam o orçamento
(`EXHAUSTED`) **sobreviveu à mutação** — a planície custa ~850 passos e o orçamento é
`MAX_STEPS × shrink = 4 000`. *A cauda não morre de fome; ela é cortada pela caixa de recorte*, que
é finita porque a bola de bordo é finita.

**Why:** cada régua responde a uma pergunta, e a ausência de uma resposta lê-se igual a uma resposta
boa. Quem só tem sondas de «demasiado» nunca vê «de menos».

**How to apply:**
- Ao escrever um gate de máximo, escreva a frase que ele **não** diz e pergunte quem a diz.
- ⭐ A régua que apanhou isto foi **a imagem contra um oráculo sem recorte**, contando os pixels que
  o **produto não desenha** — e note que a sonda de normais é cega a isso **por construção**: ela só
  compara pixels que os *dois* acertaram, então uma peça que o produto não desenha **sai da
  população** em vez de reprovar ([[feedback_a_gradient_gate_says_may_punch_only_the_image_says_punches]]).
- ⚠️ Antes de escrever a barra, **corra a régua na fixtura que já sabe boa**: a 1.ª versão desta
  (crescimento por unidade de distância, normalizado pelo divisor) acusou **tudo**, inclusive a
  configuração que a imagem prova correcta — porque amostrava **fora** da caixa de recorte, que é
  onde o `piso` congela a secção de propósito
  ([[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]).
