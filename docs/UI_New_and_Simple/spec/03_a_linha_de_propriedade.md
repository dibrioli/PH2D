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

## §6-bis — A linha de VÁRIAS COMPONENTES: o controlo REFLUI, a coluna do rótulo não

⛔⛔ **Duas ordens do dono contrariam-se numa row de N campos, e a spec tem de escolher:** o §3 põe o
nome ao lado (*«Label acima do campo numérico! Muito ruim!»*, 2026-09-14), o que entrega ao controlo
**metade** da linha; o §5 põe um piso de `72 px` em cada caixa (*«não permita que a caixa seja
redimencionada para menor que isso»*, 2026-05-24). À largura de omissão do Inspector a coluna do
controlo mede `128 px` e **dois** campos ao piso pedem `148`.

⭐ **A saída NÃO é encolher a coluna do rótulo para esta linha** — a granularidade dela é a SECÇÃO
(§6), e uma coluna por linha devolve a coluna esfarrapada que o §6 recusa.

⇒ **as componentes que não cabem ao piso DESCEM, dentro da coluna do controlo** — é a lei que a
`seg_row` já praticava para um segmentado (*«o controlo reflui e a coluna do rótulo não»*), e é como
o Blender desenha um vector num painel estreito.

| interior | coluna do controlo | 2 campos | 4 campos |
|---|---|---|---|
| `200` (mínimo do dock) | `86,0` | 1×2 linhas | 1×4 linhas |
| `253,3` (o dock do dono) | `112,7` | 1×2 linhas | 1×4 linhas |
| `284` (omissão) | `128,0` | 1×2 linhas | 1×4 linhas |
| `380` | `176,0` | **2×1** | 2×2 linhas |
| `700` (máximo do dock) | `336,0` | **2×1** | **4×1** |

⚠️ **A largura é a MESMA em todas as linhas** — a última fica curta em vez de esticar o campo que
sobra; um `Bounds` de quatro componentes acabaria com o `H` ao dobro da largura do `X`.

⚠️ **Quando nem UM campo cabe ao piso, o campo recebe a coluna inteira.** Ali o recurso que falta é
o painel, e encolher mais só apagaria o número.

⭐⭐ **UM ponto de animação por LINHA, nunca por campo** — um par `X`/`Y` é *uma* propriedade com duas
componentes, e dois pontos diriam que são duas.

### O `lead`: quando a componente tem decoração própria

⛔⛔ **Ordem do dono, 2026-09-15:** *«a mesma formatação do Position X/Y que fez para Anchor vou
querer para todo o Transform»*. As linhas do Transform trazem uma **letra de eixo colorida**
(`X` / `Y`) antes de cada caixa — e ela é parte da **componente**, não do rótulo da linha.

⇒ a porta leva um `lead`: a largura que a decoração consome antes da caixa. **O piso continua a ser
o da CAIXA**; a célula precisa de `lead + piso` e a caixa mede `célula − lead`. ⛔ Somar o `lead` ao
piso pareceria igual e mentiria sobre o recurso — quem tem dono é a caixa (§0.0).

⛔⛔ **E o Transform DEIXOU de o usar no dia seguinte, por report do dono** (*«A disposição ficou
diferente. VC tinha colocado x e y na mesma linha. Position X/Y Caixa Caixa»*): com `lead = 14` as
duas componentes só ficavam lado a lado acima de um painel de **`~400`**, e sem decoração ficam
acima de **`~348`**. O dock dele está em **`369,74`** — *a letra de eixo custava-lhe exactamente a
disposição que ele tinha aprovado nas Âncoras.* ⇒ o `X`/`Y` viaja no NOME (`"Position X / Y"`), e
hoje **nenhuma linha do app usa `lead ≠ 0`**.

⚠️ **O parâmetro FICA, e o gate varre os dois regimes** (`lead ∈ {0, 14}`): a decoração por
componente é uma pergunta que volta, e a lei dela está medida. ⛔ Mas *ela não é grátis, e o número é
`~52 px` de largura de painel* — quem a reintroduzir paga isso.

⭐⭐⭐ **E o alinhamento que o dono pediu em 2026-05-24 — *«a caixa única de Rotation deve se alinhar
à caixa de X à esquerda e à direita»* — passa a sair de GRAÇA.** Era uma fórmula escrita à mão
(`2 × two_chip_w + col_gap + axis_col_w + tag_box_gap`), e o comentário dela registava que já
estivera **errada**. Com `n = 1` a célula **é** a coluna do controlo: ela começa onde o `X` começa e
acaba onde o `Y` acaba, nos dois regimes. *Uma propriedade obtida por construção não pode ficar
errada* — e tem gate mesmo assim, porque a refactoração seguinte pode perdê-la em silêncio.

⛔ **DUAS excepções, e as duas com a mesma medição:** a grelha 3×3 do 9-slice e as quatro amostras de
canto do *Color/Tint* ficam com o nome POR CIMA, porque o controlo delas não é o bloco — é o bloco
**mais um companheiro à direita dele** (os dois atalhos · a prévia do gradiente), e o par mede
`~144 px`. A coluna do controlo vale `0,5 × interior − 14`, logo só lá chega acima de um painel de
**`336`**; o dock do dono está em `273,3`. Elas são **blocos com legenda**, não linhas de
propriedade — e há gate com a lista, com a metade da obsolescência.

---

## §6-ter — A CEDÊNCIA: a metade encolhe antes de deixar a linha quebrar

⛔⛔ **Report do dono, 2026-09-15, com foto do Transform:** *«O painel ainda largo com espaço à
esquerda e as linhas já se quebram (caixa y passa para baixo. Isso não pode acontecer. Encontre a
solução»*.

Na foto, `Position X / Y` mede **`~88 px`** numa coluna de **`~154`** — **`~66 px` de vazio** à
esquerda do nome — e o `Y` desce porque ao controlo faltavam **`7 px`**. *A coluna do nome estava a
guardar espaço que não usava enquanto a do lado passava fome.*

⇒ **a metade (§3) deixa de ser um PISO e passa a ser um ALVO**, com três degraus:

1. **empréstimo** (§6): um nome mais largo que a metade passa dela;
2. **cedência** (esta): a metade encolhe até o controlo ter o que precisa;
3. **o piso da cedência é o que o NOME precisa** — medido, no peso em que pinta.

⚠️⚠️ **Só se cede quando a cedência RESOLVE.** Se nem com o nome no mínimo o controlo coubesse,
encolher a coluna troca *uma linha quebrada* por *um nome cortado* — e a linha continua quebrada.
Aí não se cede nada e a coluna fica onde o dono a pôs.

⚠️ **O `control_need` é da SECÇÃO, não da linha.** Se cada linha cedesse pelo que ELA precisa, a
linha de um campo (*Rotation*) não cederia e a de dois cederia — e a coluna saía esfarrapada, que é
o que *«as labels alinhadas todas à direita»* proíbe. ⇒ cada secção declara **a linha que ela não
quer ver quebrar**, e todas cedem o mesmo.

**Medido** (um par `X`/`Y`, nome de `88 px`):

| painel | coluna do nome | controlo | resultado |
|---|---|---|---|
| `220` | `92,0` | `86,0` | empilha (não há folga: `88 + 8 + 147 > 206`) |
| `273,3` | `118,7` | `112,7` | empilha (idem) |
| `304` | **`115,0`** (cedeu de `134`) | **`147,0`** | ⭐ **lado a lado** |
| `343` (a foto) | `153,5` | `147,5` | ⭐ **lado a lado** |
| `369,7` (o dock dele) | `166,9` | `160,9` | lado a lado |
| `720` | `342,0` | `336,0` | lado a lado |

⭐ A quebra passa a acontecer **só** onde `nome + vão + controlo > utilizável` — isto é, onde de
facto não há folga nenhuma.

---

## §7 — O que a linha leva SEMPRE

| peça | lei |
|---|---|
| **coluna de animação** | [`DECORATOR_W`] = `14 px`, em **todas** as linhas, sempre — decisão do dono (*«vou querer animar tudo»*). É um **indicador**, não um controlo: não regista clique nenhum |
| **a unidade** | dentro do campo, **colada ao número**, em `Text2`, e **some enquanto se escreve**. ⛔ Nunca no rótulo: com a unidade lá, **20 de 39** rótulos do Inspector eram cortados; sem ela, **1**. ⭐ Em 2026-09-15 o censo levou os restantes **28** (`Break Torque (N.m)`, `Init Vel X (m/s)`, `Non-Spatialized Radius (m)`, …) e o vocabulário ganhou `N`, `N.m` e `deg/s` |
| **o que o campo pinta, o campo lê** | o sufixo que se MOSTRA pode ter maiúscula (o `N` do SI) e o que o artista escreve não tem de a ter ⇒ a leitura é **insensível à caixa**. ⛔ Não há uma segunda string por unidade: duas strings para a mesma coisa divergem |
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
5. ⏳ **DUAS portas ainda não levam unidade**, e a catraca do
   `no_row_label_carries_its_own_unit` nomeia-as linha a linha: a `transform_row::paint_row` (o par
   de chips `X`/`Y` do Transform, onde o rótulo carrega ainda por cima a RÉGUA activa) e o
   `field_row` do 9-slice. ⚠️ **O `field_row` das âncoras JÁ leva** (2026-09-15) — a mesma unidade em
   todos os campos da row, porque uma row de `X`/`Y` mede a mesma grandeza nos dois.
   ⛔ E a razão de cada entrada da catraca é a **PORTA**, não *«é multi-campo»*: o `field_row` pinta
   uma row de UM campo (`Rotation (deg)`) e mesmo assim não levava sufixo. *O que separa não é a
   contagem de campos — é a porta ter, ou não, por onde a unidade entrar.*
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
| §6-bis | as componentes que não cabem ao piso descem, dentro da coluna do controlo | `property_fields_layout` | `a_row_of_many_fields_never_starves_them` |
| §6-bis | alargar o painel nunca faz caber menos campos por linha | `property_fields_layout` | `a_wider_panel_never_fits_fewer_fields` |
| §6-bis | a caixa sozinha cobre exactamente o que o par cobre | `property_row_columns` | `the_lone_field_of_a_row_spans_what_the_pair_spans` |
| §6-bis | dispor N campos numa linha tem UM chamador em todo o painel | `property_fields_layout` | `only_one_door_lays_out_a_row_of_fields` |
| §6-ter | uma linha nunca quebra enquanto a coluna do nome tem folga | `property_label_col_w_for` | `a_row_never_wraps_while_the_name_column_has_slack` |
| §6-bis | nenhuma linha do Inspector põe o nome por cima do controlo | `fields_row` | `no_row_paints_its_name_above_its_control` |
| §7 | toda linha reserva a coluna de animação | `form_row_columns` | `every_form_row_reserves_the_animation_column` |
| §7 | as portas que reservam a coluna DERIVAM-SE, nunca se enumeram | `property_label_row` | `the_door_census_derives_the_second_order_doors` |
| §7 | o preenchimento do campo sai do tema | `body_fill` | `every_field_painter_asks_the_theme_for_its_fill` |
| §7 | um campo nunca tem a cor daquilo em que assenta | `SURFACE_STEP` | `a_field_is_never_the_colour_of_what_it_sits_on` |
| §7 | um sufixo mais longo nunca é ensombrado por um mais curto | `Unit` | `a_longer_suffix_is_never_shadowed_by_a_shorter_one` |
| §7 | o campo lê de volta a unidade que ele próprio pinta, em qualquer caixa | `parse_suffix` | `every_unit_reads_back_what_it_paints` |
| §7 | nenhum rótulo do app carrega a unidade no texto | `num_row_unit` | `no_row_label_carries_its_own_unit` |
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
6. **Se a linha tem N campos, não faça nada disto:** chame `rows::fields_row(…, &[ids], passo,
   unidade, None)` — ela é a porta, e o reflúxo do §6-bis vem de dentro. ⛔ Uma aritmética de
   `(w − gap × (n−1)) / n` escrita no sítio de pintura é a quarta resposta à mesma pergunta (havia
   **três** em 2026-09-15, divergentes na altura da caixa e na existência do ponto de animação).
7. **Se o controlo é construído à mão** (um segmentado com «nenhum aceso», uma grelha), chame
   `rows::property_label_row(…)`, pinte em `row.control` e termine com o ponto.

⛔ **O que NÃO fazer:** escrever a largura da coluna no sítio de pintura (há censo com catraca) ·
pôr a unidade no rótulo · pintar o rótulo com `paint_text` (perde a elisão e o alinhamento) · dar ao
pintor um orçamento derivado por subtracção (§4.5).
