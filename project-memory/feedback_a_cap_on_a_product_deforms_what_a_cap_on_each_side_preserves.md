---
name: a-cap-on-a-product-deforms-what-a-cap-on-each-side-preserves
description: "Clampar `rows × cols` trunca em row-major e transforma um quadrado numa FAIXA; clampar cada lado preserva a forma — e nenhum gate desta casa mede a forma de uma grelha"
metadata:
  type: feedback
---

Ao efectivar o tecto de `16 384` objectos por nó (ordem do dono, 2026-09-22), a 1.ª implementação
clampava o **PRODUTO** (`(rows × cols).min(max)`). O `build_grid` constrói em ordem row-major, logo
esse `min` entrega **as primeiras `16 384` células**: um `512 × 512` sai como **`32` linhas de
`512`** — uma faixa, não um quadrado.

Medido: **seis cenas de demo do produto** ficavam assim, e **nenhum gate o via** — todos contam
linhas (`count()`) e nenhum mede a **FORMA** da grelha. A contagem estava certa; a geometria não.

**Why:** um artista que escreve `512 × 512` quer um quadrado. Truncar um produto em ordem
row-major é a operação mais barata de escrever e a única que muda a coisa que ele desenhou. *Uma
grandeza composta tem uma FORMA, e um tecto sobre o produto não a conhece.*

**How to apply:** ao pôr um tecto sobre uma grandeza que é um PRODUTO de factores autorados, clampe
**cada factor**, não o produto — a forma mantém-se e o produto fica no tecto por construção. Guarde
o `.min(produto)` como cerca: depois disso ela deixa de poder morder, que é o que se quer de uma
cerca.

⚠️ E a pista de que isto aconteceu é a própria ordem de quem pediu: aqui ele escreveu *«num nó como
grid o limite máximo é **Rows = 128 e Columns = 128**»* — por LADO, e eu implementei por produto na
primeira passagem. *Reler o pedido depois de medir o efeito valeu mais do que a medição.*

⭐ E há um corolário para os gates: **uma suíte que só conta elementos é cega a deformação**. Se uma
lei pode mudar a forma de uma grelha, de uma malha ou de uma imagem, alguma régua tem de medir a
forma — senão a mudança passa em silêncio.

[[reference-topic-measurement-discipline]] · [[feedback-perfection-no-deferrals]]
