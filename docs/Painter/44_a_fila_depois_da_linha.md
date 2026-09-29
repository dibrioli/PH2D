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

**Medido (2026-09-29, sonda [`measure_drag_direction`](../../crates/ph2d-wet-paint/tests/it/measure_drag_direction.rs)):**

⛔ **O teste de ESPELHO foi a 1.ª redacção e está RECUSADO.** O mesmo traço espelhado com `Drag = 0`
já diverge `67 %` da mudança que o traço faz, porque o papel e as cerdas têm ruído com semente e não
são simétricos. O fundo afoga o sinal: a assimetria até DESCE com o arrasto (`0,67 → 0,59` a
`Drag 0,4`).

⭐ **A régua que decide é a própria lei trocada.** A sonda grava o grid final de um traço horizontal
em cada sentido. Uma cópia temporária do `transfer.rs` lê a origem do arrasto de um instantâneo
tirado antes do laço (Jacobi), a sonda corre outra vez, e o ficheiro é reposto (`git diff` vazio
conferido). A previsão é o que dá o controlo: **para a ESQUERDA as duas leis têm de coincidir**, porque
ali a origem ainda não foi escrita.

| passo/quadro | sentido | Σ\|GS−Jacobi\| / Σ\|mudança\| (susp · sett · film) | pior célula susp | células > 0,1 % | pior RGB | pior `wet` |
|---|---|---|---|---|---|---|
| 3 px | direita | `1,99 %` · `1,18 %` · `1,90 %` | `64,3` | `611` | `3,29` | `7` |
| 3 px | **esquerda (controlo)** | `0,00 %` | `0,0028` | **`0`** | `0` | `0` |
| 12 px | direita | `0,22 %` · `0,22 %` · `0,21 %` | `21,0` | `23` | `0,38` | `5` |
| 12 px | **esquerda (controlo)** | `0,00 %` | `0,0285` | **`0`** | `0` | `0` |

**Leitura:**
- a dependência do sentido **existe** e é **pequena**: `~2 %` da tinta que um traço LENTO move, `~0,2 %` num traço rápido;
- a esquerda é o Jacobi a menos do ruído de `f32` (o resíduo `≤ 0,03` de massa não passa em nenhuma célula), logo **a lei sem ordem é a que o traço para a esquerda já pinta hoje**;
- mudá-la muda a tinta **só nos traços para a direita e para baixo** e só nesta ordem de grandeza.

⏳ **Por medir antes de se construir:** o relógio. O diário dá `~1,9 ms` por transfer e `~2` transfers
por quadro a raio 250, contra `~8 ms` de quadro, e a leitura sem ordem paga uma cópia da janela de
origem por transfer.

⛔ **E o preço que a tabela não mostra:** o Wet Paint é um porte 1:1 com impressão digital
(ADR-0134/0147). Trocar a leitura re-baseia as impressões desse caminho, como o ADR-0147 fez ao
solver, que manteve a rota antiga PREGADA (`PINNED_GAUSS_SEIDEL`).

**Decisão do dono** (§0.8), com os números acima:
- (a) fica como está;
- (b) passa a sem-ordem: os quatro sentidos pintam igual, os dois passos podem dividir-se pelos núcleos, e a tinta muda `≤ 2 %` nos traços lentos para a direita ou para baixo.
