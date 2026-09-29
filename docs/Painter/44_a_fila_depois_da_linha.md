# 44 — A fila depois da `line/PainterWatercolor` (aberta 2026-09-29)

> A linha fechou e foi integrada em 2026-09-25. O §10 do
> [handoff dela](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md) listava
> dez itens em aberto, cinco deles marcados como «decisão do dono». O dono disse (29/09) que *algumas
> dessas decisões já foram tomadas*. ⇒ **nenhum item se constrói antes de ser CONFERIDO**:
> procura-se a decisão no diário, nos commits e na memória, e o dono confirma.
>
> Estado por item: `a conferir` · `aberto (conferido)` · `decidido` · `feito` · `recusado`.

| # | item | de quem | estado |
|---|---|---|---|
| 1 | Wet Paint: passos 3–4 do transfer em série (soma `f64` · arrasto Gauss-Seidel) | dono | **aberto (conferido 29/09)** — §1 |
| 2 | Aquarela: unificar a mistura seca (RYB) e a molhada (K–M) — a pista do *glaze*, diário §17.3 | dono | a conferir |
| 3 | O `Mixing` no Impasto | dono | a conferir |
| 4 | O `Pigment` que mudou de sítio na aquarela (do cartão *Water* para o *Mixing*) | dono | a conferir |
| 5 | Composite Brush: a pilha cheia numa tela grande (a alavanca é o RAIO) | dono | a conferir |
| 6 | Wet Paint: o *fork* do canvas no 1.º toque depois de soltar (`~9 ms`) — pede canvas em ladrilhos | linha | a conferir |
| 7 | Wet Paint: o tile do papel do motor (`~12 ms`, em série por impressão digital) | linha | a conferir |
| 8 | Composite Brush: o relevo fora da recomposição da pilha | linha | a conferir |
| 9 | Composite Brush: o resíduo Blur+Smear (`12/255`) | linha | a conferir |
| 10 | Composite Brush: metade dos bytes dos intermédios da pilha | linha | a conferir |

---

## §1 — Item 1: os passos 3–4 do transfer do Wet Paint

**Onde:** [`trail/transfer.rs`](../../crates/ph2d-wet-paint/src/trail/transfer.rs), `transfer_paint_impl`.

**O que são:**
- **Passo 3:** a média da janela do bico sobre as células com pigmento (`sum_pig`, `sum_water`), somada em `f64` numa ordem fixa. Ela só é lida no *soft cap*, isto é, quando uma célula passa de `3000` (`shed_pig`) ou de `WaterCap` (`shed_water`).
- **Passos 4+5:** pousar a janela e o **arrasto** (`drag`). O arrasto lê `g.susp[si]`, `g.sett[si]`, `g.film[si]` e `g.wet[si]` na posição da âncora ANTERIOR. Esses são os mesmos arrays que o próprio laço escreve.

**Custo registado** (diário §39.3, 2026-09-24): `~1,9 ms` por transfer e `~2` transfers por quadro a raio 250. Isto é cerca de metade dos `~8 ms` do quadro a pintar.

**Conferência — não há decisão.** O que foi procurado:
- no diário da linha, de §39.3 até §43: o item fica aberto e não volta a ser citado;
- em `git log main --since=2026-09-18 --grep=transfer`: só o `8a05c45cf` (ADR-0175), que paraleliza o depósito e os passos **1–2** e deixa estes dois de fora de propósito;
- em `project-memory/`: nenhuma entrada.

**Leitura do código, ainda NÃO medida:**
- **Passo 3.** Somas parciais por linha, somadas numa ordem fixa, mudam o `f64` no último bit. Esse valor só chega à tinta através de `old + v − shed` guardado em `f32`, com `old ≥ 3000`, onde o ulp do `f32` é `2,4e-4`. Um erro de `~1e-13` no `shed` quase nunca muda o `f32`. ⇒ paralelizar o passo 3 é, provavelmente, uma **decisão técnica**: cabe à linha medir se sai ao bit.
- **Passo 5.** O arrasto é Gauss-Seidel **dependente da direcção do traço**. O laço varre da esquerda para a direita e de cima para baixo. Quando o traço anda para a direita ou para baixo, `si` (a janela anterior) fica ANTES de `i` no varrimento e já foi escrita neste laço. Para a esquerda ou para cima, ainda não foi. ⇒ hoje o arrasto provavelmente puxa a tinta de maneira diferente conforme o sentido do traço. Mudá-lo para uma leitura sem ordem (Jacobi, como fez o ADR-0147 com o solver) muda a tinta. Isso é **produto**.

**Antes de o dono decidir, medir (sonda):**
1. o mesmo traço para a direita e para a esquerda, espelhado: quanto difere a tinta (bytes, píxeis);
2. a mesma cena com o arrasto Jacobi: quanto difere de hoje;
3. o relógio dos dois passos isolados, com a máquina calma.
