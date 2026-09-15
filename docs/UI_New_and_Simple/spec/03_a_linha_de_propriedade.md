# Spec — a LINHA DE PROPRIEDADE (2026-09-14)

> ⛔⛔ **Esta página é o «modelo pronto e bem estabelecido» que o Enio pediu** em 2026-09-14:
> *«falta para nós um modelo pronto e bem estabelecido com todas as regras para todos os widgets …
> o inspector está assustador de horrível»*. Ela cobre **uma** superfície — a linha de propriedade,
> que é a que mais se repete no app — e é o referente da ordem permanente dele: *«se temos o manual
> precisamos converter o APP todo a ele»*.
>
> ⚠️ **Cada lei tem PORTA e GATE, e a tabela do §9 é VERIFICADA por um teste**
> ([`the_property_row_manual_names_only_doors_that_exist`](../../../crates/ph2d-editor-core/tests/it/the_property_row_manual_names_only_doors_that_exist.rs)):
> um nome que deixe de existir reprova. *Um doc que enuncia a lei que o código não implementa lê-se
> como auditado* — e esta casa já pagou essa forma.
>
> ⛔ **Nenhum número desta página é escolhido.** Cada um traz a medição ou o dono ao lado.

---

## §1 — O que é uma linha de propriedade

Uma faixa de altura [`ROW_H_PX`] que reparte a largura entre **um nome** e **um controlo** que edita
uma propriedade de um objecto. É a unidade que o Inspector, o painel de vetor, o de física, o de
modelação e outros dez repetem dezenas de vezes por ecrã.

⛔ **Não é** uma linha de lista (a Hierarquia), nem uma faixa de faixa (a timeline), nem um cabeçalho
de secção. Esses têm leis próprias e **não** devem ser convertidos a esta.

---

## §2 — HÁ DUAS FORMAS, e a regra que escolhe é uma pergunta sobre o VALOR

| forma | quando | como se lê |
|---|---|---|
| **Caixa única** | o valor tem uma **fracção** para mostrar (um intervalo: `0..1`, `0..360`, um slider) | `[ Nome ················ 62% ]` — rótulo **DENTRO**, à esquerda; valor dentro, à direita; o **preenchimento** diz a fracção |
| **Linha de propriedade** | o valor **não** tem fracção (um número livre, um texto, uma caixa de verificação, um selector) | `Nome ····· [ 2 m ]` — rótulo **FORA**, à direita; controlo a começar no meio |

⚠️ **A partição é principiada, não histórica.** A caixa única existe porque o preenchimento **é**
informação; onde não há fracção, uma caixa larga só diz *«o número é o assunto»*, que foi a queixa
do dono em 2026-09-14 (*«as caixas numéricas são muito grandes. Maiores que as labels»*).

⛔⛔ **E as duas NÃO são convertíveis uma na outra na largura que o artista usa.** Medido: pôr o
rótulo fora numa linha com trilho deixaria, no mínimo do dock, **`72 px` para o trilho E o número
juntos** — que é exactamente o cromo fixo (`rótulo 70 | trilho | caixa 72` = `154 px`) que a caixa
única nasceu para matar em 2026-09-02. ⇒ *quem uniformizar as duas tem de trazer a medição que
desfaz esta.*

**Censo (2026-09-14):** caixa única em **15** painéis / 53 chamadas · linha de propriedade em **5**
painéis / 22 chamadas · **seis** painéis usam as duas, e está certo.

---

## §3 — A COLUNA: o controlo começa no MEIO da linha

⛔ **Ordem do dono, 2026-09-14:** *«Melhor alinhar no meio do painel e as labels alinhadas todas à
direita (no centro do painel)»*.

- `LABEL_COL_FRAC = 0,5` — a fracção é da **LINHA**, não do utilizável: a coluna de animação sai do
  lado do controlo, e *«o meio do painel» é o meio do que o artista vê*.
- O rótulo acaba **um vão [`Spacing::Md`] antes** do meio, para o controlo começar exactamente nele.

⚠️ **Ela já foi MEDIDA e deixou de o ser, e a troca é honesta.** Nasceu a reproduzir o literal que a
casa mais escrevia (`96/276 = 0,348`) — arqueologia correcta do produto de então. *Uma medição diz o
que o produto FAZ; ela nunca disse o que ele DEVIA fazer* (`CLAUDE.md` §0.8).

⛔ **Uma largura FIXA está errada por construção:** a coluna docada é arrastável
([`PANEL_MIN_W_PX`]`..720`). Antes desta lei a mesma pergunta tinha **onze** respostas no app
(`96` · `78` · `84` · `76` · `72` · `64` · `150` · `176`), cada uma um literal com dispensa.

---

## §4 — O RÓTULO: à direita, elidido, medido no peso em que pinta

1. **Alinhado à direita** da coluna — encostado ao controlo. Com o controlo a começar no meio, um
   rótulo à esquerda deixa um rio de espaço variável: *quanto mais curto o nome, mais longe do valor
   que ele nomeia*.
2. **Elidido** quando não cabe, **nunca** transbordado nem quebrado em duas linhas.
3. **Medido no peso em que é pintado** (`Medium`). ⛔ Medir em `Medium` e pintar em `SemiBold` corta
   `0,74 %`–`1,69 %` curto — `1,3 %` de `110 px` é um caractere, exactamente na fronteira em que o
   corte existe.
4. **A reticência nunca fica pendurada num espaço**: o prefixo é aparado antes de a receber
   (`Air Jump …` era o defeito).
5. **Um rótulo que CABE nunca é pintado com reticências.** ⛔ A largura que o pintor recebe é a
   **medida** do texto que coube, nunca `x + col_w − recuo`: em `f32` essa diferença cancela um ULP
   abaixo em **8,6 %** das posições e o pintor corta um nome com `40`–`77 px` de folga.
6. Quando nem a reticência cabe, o rótulo **degrada para a esquerda** e o pintor **recusa** em vez de
   invadir o controlo.

---

## §5 — O PISO DO CONTROLO é o que o CONTROLO declara

⛔⛔ **Ordem do dono, 2026-05-24**, escrita no doc de [`NUMBER_INPUT_MIN_W_PX`]: *«não permita que a
caixa seja redimencionada para menor que isso»* ⇒ **`72 px`**.

⚠️ **Um piso que NOMEIA o recurso ainda pode estar errado sobre ele.** A 1.ª redacção desta lei disse
*«o piso é [`ICON_BTN_SIZE_PX`] (36) — a largura da coluna do stepper — mais um dígito»* ⇒ `48`. A
coluna do stepper é [`stepper_width`] = `clamp(0,6 × altura, 16, 22)`, **nunca 36**. ⇒ *quando o
recurso já tem dono, o piso é o dele*.

⭐⭐⭐ **E a lei do §3 sobrevive exactamente até ao mínimo do dock, sem ninguém a escolher:** o rótulo
só pode ser a metade enquanto `metade ≤ tecto`, o que dá `linha ≥ 2 × (piso + DECORATOR_W)` =
**`172`**; a linha de um cartão do Inspector no mínimo do dock mede
`220 − 2×18 − 2×6` = **`172`**. Os dois números encontram-se ao píxel, e há gate a afirmá-lo.

---

## §6 — O EMPRÉSTIMO: o rótulo toma a folga que o controlo não usa

`coluna = clamp(o rótulo mais largo da SECÇÃO, a metade, o que o controlo pode ceder)`.

- **O piso é a metade** — a ordem do §3 continua a valer, e numa coluna larga nada muda.
- **O tecto é o controlo** — ele nunca desce do piso do §5.
- **A granularidade é a SECÇÃO**, nunca a linha nem o painel. Uma coluna por linha põe cada controlo
  num `x` diferente e a coluna sai esfarrapada; uma por painel faz a secção de nomes curtos herdar a
  largura do nome mais comprido do painel inteiro.

**Medido** (§14 do Inspector, 52 rótulos, ao longo do curso do dock):

| painel | coluna | campo | rótulos elididos |
|---|---|---|---|
| `220` (mínimo do dock) | `78,00` | `72,00` | 16 |
| `245` | `103,00` | `72,00` | 3 |
| `273,3` | `113,88` | `89,42` | **0** |
| `304` (omissão) | `120,00` | `114,00` | 0 |
| `720` (máximo do dock) | `328,00` | `322,00` | 0 |

⏳ **NÃO CONVERTIDO:** o empréstimo alcança **1** das 59 chamadas da porta. Medido em 2026-09-14, à
largura que o dono usa, dar a mesma medição às outras secções corta para **cerca de metade** os
nomes elididos nos painéis de vetor, de escultura e no resto do Inspector; no **mínimo** do dock ele
compra **zero** (ali a metade e o tecto coincidem e não há folga nenhuma).

---

## §7 — O que a linha leva SEMPRE

| peça | lei |
|---|---|
| **coluna de animação** | [`DECORATOR_W`] = `14 px`, em **todas** as linhas, sempre — decisão do dono (*«vou querer animar tudo»*). É um **indicador**, não um controlo: não regista clique nenhum |
| **a unidade** | dentro do campo, **colada ao número**, em `Text2`, e **some enquanto se escreve**. ⛔ Nunca no rótulo: com a unidade lá, **20 de 39** rótulos do Inspector eram cortados; sem ela, **1** |
| **a superfície** | o preenchimento sai da porta do TEMA ([`body_fill`]), nunca do token do cartão. Num tema moderno o campo é um degrau **abaixo** da superfície em que assenta, e **não** leva moldura em repouso |
| **o valor** | alinhado à esquerda dentro do campo, com as setinhas na coluna da direita |

---

## §8 — O que NÃO está decidido

1. **A reticência acaba ~2 px antes do fim da caixa dela** (o *side-bearing* do glifo `…`). Todas as
   reticências acabam na mesma posição; o que difere é onde a tinta pára. Curar pede métricas de
   glifo.
2. **Três nomes não cabem** em nenhuma coluna utilizável no mínimo do dock
   (`Corner Look-ahead`, `Weight on Ground`, `Swim Line (weights)`) — encurtar é decisão do dono.
3. **A coluna de animação na ponta estreita:** ela é `18 %` do que sobra para o nome a `220 px`.
   Escondê-la abaixo de uma largura levaria os 16 cortados para `~9` — ⛔ e um indicador que aparece
   e desaparece ensina a desconfiar dele. Decisão do dono.
4. **A unidade de EXIBIÇÃO** (`m` ⇄ `px`) ainda não conduz o sufixo do campo: mostrar `px` sobre um
   número em metros trocaria um rótulo comprido por um rótulo **mentiroso**. Precisa da conversão do
   valor, não de um sufixo.
5. **A coluna de nome de FAIXA da timeline** (`tracks.rs`) responde a outra pergunta e fica de fora
   **de propósito**.

---

## §9 — A tabela VERIFICADA: lei → porta → gate

> ⚠️ Esta tabela é lida por
> [`the_property_row_manual_names_only_doors_that_exist`](../../../crates/ph2d-editor-core/tests/it/the_property_row_manual_names_only_doors_that_exist.rs):
> cada nome da coluna **porta** e da coluna **gate** tem de existir no código. ⛔ Não acrescente uma
> linha sem o nome real.

| § | lei | porta | gate |
|---|---|---|---|
| §3 | o controlo começa no meio da linha | `property_row_columns` | `a_property_row_never_starves_its_control` |
| §3 | a coluna é uma resposta só, nunca um literal no sítio de pintura | `property_label_col_w` | `the_label_column_is_never_chosen_at_the_painting_site` |
| §4 | o rótulo é alinhado à direita e encostado ao controlo | `property_label_origin` | `a_property_label_is_flush_against_its_control` |
| §4 | um rótulo que cabe nunca leva reticências | `paint_property_label` | `a_label_that_fits_is_never_painted_with_dots` |
| §4 | a reticência nunca fica pendurada num espaço | `corte` | `the_ellipsis_never_hangs_off_a_space` |
| §5 | o campo nunca é pintado abaixo do que o dono declarou | `NUMBER_INPUT_MIN_W_PX` | `a_field_is_never_narrower_than_its_owner_declared` |
| §6 | o pintor pede emprestado a folga que o controlo não usa | `property_label_col_w_for` | `the_painter_borrows_the_slack_the_control_does_not_need` |
| §6 | quantos rótulos elidem, por largura do dock | `property_row_columns_for` | `the_elision_ladder_only_shrinks` |
| §7 | toda linha reserva a coluna de animação | `form_row_columns` | `every_form_row_reserves_the_animation_column` |
| §7 | o preenchimento do campo sai do tema | `body_fill` | `every_field_painter_asks_the_theme_for_its_fill` |
| §7 | um campo nunca tem a cor daquilo em que assenta | `SURFACE_STEP` | `a_field_is_never_the_colour_of_what_it_sits_on` |
| §7 | um sufixo mais longo nunca é ensombrado por um mais curto | `Unit` | `a_longer_suffix_is_never_shadowed_by_a_shorter_one` |
| §7 | a unidade é tinta dentro da caixa, nunca uma segunda caixa | `paint_number_input_with_buffer` | `the_unit_is_ink_inside_the_box_and_never_a_second_box` |

---

## §10 — Como converter uma linha a esta spec

1. `let row = property_row_columns(x, w, y, ROW_H_PX);` — ou `…_for(…, Some(mais_largo))` se a
   secção souber o rótulo mais largo que vai pintar (§6).
2. `paint_property_label(ts, scene, label, row.label.x, row.label.y + (row.label.h − font) * 0.5,
   font, row.label.w, resolve(ColorToken::Text2, theme));` — a assinatura é a do `paint_text_elided`
   **argumento a argumento**, de propósito: converter é trocar o nome da função.
3. `hit_index.register(id, row.control);` e pinte o controlo em `row.control`.
4. `paint_decorator_dot(scene, theme, row.dot);`
5. Se o campo tem unidade física, `input.suffix(Some(unit.suffix()))` — ⛔ **não** a ponha no rótulo.

⛔ **O que NÃO fazer:** escrever a largura da coluna no sítio de pintura (há censo com catraca) ·
pôr a unidade no rótulo · pintar o rótulo com `paint_text` (perde a elisão e o alinhamento) · dar ao
pintor um orçamento derivado por subtracção (§4.5).
