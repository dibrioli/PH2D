---
name: feedback-a-derived-quantity-may-be-a-property-of-emission-order
description: Um tecto derivado de uma grandeza pode estar a medir a ORDEM em que o código foi emitido, e não o objecto — pergunte de que a grandeza é propriedade antes de escrever o limite
metadata:
  type: feedback
---

⛔⛔⛔ **Antes de escrever um tecto sobre uma grandeza DERIVADA, pergunte de que ela é propriedade.**

Medido 2026-09-15 (`line/3DModeling`, o modelador no dispositivo). O `vivos` de uma fita SSA — o
pico de valores vivos ao mesmo tempo, que numa GPU decide a **ocupação** — crescia linearmente com
as arestas de um contorno desenhado: `68` a 32 arestas, `492` a 256. Um tecto inteiro
(`MAX_VIVOS`) foi derivado disso, com tabela, travessia medida contra a CPU e cerca no produto.

⭐ **A causa não estava no grafo.** A travessia que achata a árvore empilha os filhos e emite o de
cima primeiro; sobre a cadeia esquerda `min(min(min(s₀,s₁),s₂),s₃)` que um contorno constrói, isso
calcula **todos** os `sᵢ` antes do primeiro `min` — logo os `N` segmentos ficam vivos ao mesmo
tempo. A ordem óptima da **mesma fita** acumula à medida que anda e tem pico **dois**.

Um escalonamento de lista (a mesma permutação de instruções, zero aritmética mudada, bit a bit a
mesma resposta) leva o pico a `28`–`48` em toda a faixa e o relógio do dispositivo a **`1,4×`–`9,2×`**
mais rápido. *O tecto estava a medir a ordem de iteração de uma dependência.*

**Why:** uma grandeza derivada tem sempre dois donos possíveis — o objecto e o **processo que a
produziu**. Um limite escrito sobre a segunda hipótese parece uma lei da física do problema e é uma
propriedade de quem escreveu o emissor.

**How to apply:**
- ao derivar um tecto, escreva **de que recurso ele é** (já é `CLAUDE.md` §0.0) **e de que o
  NÚMERO é propriedade**: se a resposta envolve «a ordem em que», há uma cura antes do tecto;
- o sinal barato: a grandeza cresce com um parâmetro que **não muda o trabalho total** (aqui, o
  número de arestas muda o trabalho, mas não muda quantos valores precisam de coexistir);
- ⚠️ e a cura PRESCRITA herda o erro: a nota mandava trocar o contorno por uma estrutura de dados,
  com um preço de produto declarado ao lado (perder os modificadores da peça). Esse preço **não era
  necessário** — *a nota descrevia uma procuração, e o preço dela também.*

Relacionado: [[reference-topic-measurement-discipline]] ·
[[feedback-a-constant-folded-into-a-tree-is-recomputed-wherever-the-tree-is]]
