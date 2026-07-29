# Handoff de integração — `line/Vector`: GROW / SHRINK (plano 24 W7)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Commits desta wave:** 2 (`0f729608e` · `6c3359f4d`)
**Estado:** fechada, **pendente de smoke**

⚠️ **Esta é a TERCEIRA wave da mesma abertura de linha.** As duas primeiras têm handoff próprio —
[`HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md)
(a lei de mistura) e
[`HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md`](HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md)
(a turbulência) — e as três **compartilham o bump de schema**. Integre as três juntas: o `main` nunca
viu nenhuma delas.

A última família de primitivas do SVG que faltava ao catálogo. O `apply_op` da W2 nomeava quatro
(*color-matrix · morphology · displacement+turbulence · bevel*); com esta, sobra **uma**.

---

## O que o artista ganha

Um degrau **Grow / Shrink** com um knob só — **Amount**, bipolar:

| Amount | o que se vê |
|---|---|
| negativo | a silhueta **AFINA** (o *choke*) |
| zero | **nada**, byte a byte |
| positivo | a silhueta **ENGORDA**, com as quinas a arredondar |

E ela **compõe**: `Outline → Grow` engorda o TRAÇO; `Grow → Outline` engorda a forma e depois a
contorna. As mesmas duas operações, desenhos diferentes — a tese da pilha.

---

## A pesquisa decidiu a FORMA, e a RÉGUA

**O modal (`erode`/`dilate`) é a interface ANTIGA.** Um enum é barato num formato *declarativo* (o
SVG) e o Photoshop herdou dois itens de menu (*Minimum*/*Maximum*) da era dos filtros de 1990. Quem
desenhou isto como **CONTROLE** convergiu no bipolar: *Simple Choker* (AE), *Dilate/Erode* (Blender),
*Offset Path* (Illustrator).

E o SVG dilata com um elemento estruturante **RETANGULAR** — o Photoshop tem de oferecer *Preserve:
Roundness* como opção. Com o campo de distância euclidiano do W4 o conjunto novo é `{d ≤ r}`, ou seja
um **DISCO**: medido na quina de uma caixa, crescer 10 px alcança **10,00 px** na diagonal contra os
**14,14** de um retângulo.

Detalhe e tabela: [`docs/Vector Module/24_plano_fx_raster.md`](Vector%20Module/24_plano_fx_raster.md) §14.

---

## As portas únicas

| pergunta | porta | consumidores |
|---|---|---|
| este tipo engorda/afina? | `FxKindSpec::grow_label` | o painel (OFERECER) · o produtor (HONRAR) |
| **contra o QUÊ ele mede?** | `FxOp::measures_the_image` | `plan_of` (a semente) · o writer dos globals (o `n_segs`) |
| de que cor é a área nova? | `straight_colour` (WGSL) | o feather · a morfologia |
| quanto ele espalha? | `op_reach` | `stack_reach` → o tamanho do scratch |

### ⚠️ A decisão de desenho: ela mede a IMAGEM, não a FORMA

As outras quatro do campo são efeitos de **borda da SILHUETA** e querem o pé exato da geometria. O
`feMorphology` dilata **a entrada dele** — e é isso que faz `Outline → Grow` engordar o traço em vez
de o recortar de volta.

Com geometria disponível o produtor resolveria pela forma: a resposta certa para quatro tipos e a
errada para o quinto, **sem erro nenhum**. O `n_segs` do uniform passou a ser derivado do PLANO —
semear o raster e deixar o finalize consultar a geometria construiria um campo que ninguém lê.
Isto **preserva** o que já se fazia (`raster_seed` era `geom.is_empty()`, logo os dois já coincidiam).

### ⚠️ `seeds_shell()` ENUMERAVA os leitores, e apodreceu na primeira adição

Ela dizia `FEATHER || OUTLINE || BEVEL || GLOW`. A morfologia lê o campo dos **dois** lados e nasceu
a cair no `else` — o ramo que semeia só os texels de FORA. **Quatro gates ficaram vermelhos de uma
vez.** A pergunta certa já estava escrita uma linha acima: **os de DENTRO são a exceção**. Hoje é
`!is_inner()`, equivalente ao byte no dia da troca (os gates de contorno/feather/bevel/glow/inner são
a prova).

---

## O oráculo, e o número que NÃO é desta operação

O gate que fecha a wave não compara contra o ideal: **`Grow(r)` e `Outline(r)` descrevem o MESMO
conjunto**, logo têm de pôr o contorno no mesmo lugar. Medido **71,992 contra 71,992** — e o gate de
catálogo, que não sabe nada disto, conta os **mesmos 1152 texels** para os dois.

⚠️ **MEDIDO e nomeado:** o campo semeado pelo raster põe a fronteira **~0,5 px adiante** numa aresta
DURA alinhada aos eixos quando o JFA propaga longe. **Não é desta operação** — um Outline de 8 px
mede `+8,494` no mesmo caminho contra `+8,000` pelo pé exato. Como a morfologia mede a IMAGEM de
propósito, ela paga essa régua sempre; o número fica pinado na sonda
`measure_where_each_law_puts_the_contour`.

---

## Gates — 14 novos, 10 mutações, **10 sangram**

| gate | onde |
|---|---|
| `a_zero_amount_is_byte_identical_to_no_morphology_at_all` | GPU |
| `the_contour_moves_by_the_amount_it_was_given_and_the_sign_is_the_direction` | GPU |
| `the_grow_and_the_outline_agree_on_where_the_dilated_boundary_is` | GPU |
| `the_structuring_element_is_a_disc_not_a_rectangle` | GPU |
| `the_ring_that_grew_wears_the_shapes_colour` | GPU |
| `the_shape_keeps_its_own_colours_and_only_the_new_ring_borrows` | GPU |
| `the_morphology_measures_the_image_it_received_not_the_shape` | GPU |
| `the_morphology_is_a_field_op_that_always_seeds_from_the_raster` | CPU (`ph2d-render`) |
| `the_morphology_only_pays_margin_for_the_direction_that_grows` | CPU (`ph2d-render`) |
| o Amount nas rows de `a_row_paints_only_the_controls_its_kind_uses` | seam (painel) |
| `the_grow_knob_reaches_the_bus_with_a_bipolar_map_when_dragged` | seam (painel) |
| `the_grow_crosses_the_camera_with_its_sign` · `hit_of_decodes_the_grow_knob` | shell |

**Rodar os de GPU:** `cargo test -p ph2d-render --test fx_stack_morphology_gpu --release -- --ignored`
(sem adapter fazem *skip gracioso*, que **não é verde**).

### ⚠️ Três lições de fixture, e duas são minhas

1. **Uma mutação exigiu TRÊS iterações até a fixture conter o fenômeno.** Colapsar os dois braços de
   `straight_colour` sobrevive numa forma **monocromática** (*"a minha cor"* e *"a cor da borda"* são
   o mesmo número) e sobrevive de novo num texel do **miolo** (o JFA não alcança, e o braço de
   vizinhança amostra a si próprio). Só o **ENCOLHER** — onde os texels sobreviventes estão dentro do
   alcance do salto — os separa.
2. **O gate de catálogo pegou a wave:** `every_kind_draws_something` constrói cada tipo com o *raio*
   visível, e o knob visível deste é outro campo ⇒ ele entrava no ponto NEUTRO e "não desenhava
   nada". A fixture passou a perguntar à TABELA, como a linha do `offset` ao lado já fazia.
3. **A fixture do gate de composição TEM de trazer geometria** — sem segmentos o produtor já semeia
   pela cobertura, o defeito não existe e o gate ficaria verde sobre nada.

---

## Schema, contratos, ids

- **`PROJECT_SCHEMA` fica em 38** — a terceira leva da mesma linha (`blend`, depois
  `scale`/`detail`/`seed`, agora `grow`). Um save v37 já é recusado pelo 38. **Uma linha, um bump.**
  ⚠️ O valor se **CONTA** a partir do `main` do dia.
- **Contrato congelado §6: INTACTO** (conferido por grep) — `NodeOp=2` / `OpResolver=1` /
  `NodeManifest=8` / `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`.
- `VEC_SCENE_SCHEMA_VERSION`: **intocado**.
- **`MAX_FILTER_KINDS` 10 → 11** (sem isso o "Add Grow / Shrink" não seria pintado — o `paint` faz
  `.take()`; o gate de seam pega).
- **Ids novos** (2, derivados por linha, bloco append-only): `filter_grow_id{,_num}`.
- **Superfície pública nova:** `FxOp::MORPHOLOGY` · `FxOp::grow` · `FxKindSpec::grow_label` ·
  `FxOp::measures_the_image` · `FxOp::reads_grow` · `FxOpGpu::grow_px`. E `ph2d_render::kernel_half`
  **mudou de módulo** (mesmo caminho de import).

---

## LOC — dois splits, os dois por responsabilidade

| arquivo | antes | depois |
|---|---|---|
| `ph2d-ecs/src/vec_filter.rs` | 741 | **508** + `vec_filter_kinds.rs` (247) — *o que um degrau É* × *o CATÁLOGO dos tipos* |
| `ph2d-render/src/fx_stack_shader.rs` | 712 | **385** + `fx_stack_field.rs` (340) — *o FOLD* × *a RÉGUA* |

⚠️ E o `kernel_half` mudou-se para o `fx_stack_plan`, que **já o importava de volta** — a pergunta
*"quanto este passe percorre"* é a família do `op_reach`/`jump_count`. De passagem, um doc-comment
**órfão** (o dele, pousado sobre um `pub use` por um split anterior) foi reancorado.

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=36 cargo run -p ph2d-host-desktop --release
```

Quatro pares — o SINAL · o ELEMENTO (as pontas arredondam) · **a ORDEM** (`Outline → Grow` contra
`Grow → Outline`, o headline) · e o USO (o *choke*: encolher antes de borrar).

**As cenas `=34` (a lei de mistura) e `=35` (a turbulência) continuam válidas e precisam do mesmo
smoke.**

---

## Aberto, nomeado

- **Falta o `feColorMatrix`** (tint/duotone/saturate/`luminanceToAlpha`) — o último item da lista do
  §7. O Color Overlay com as vinte leis já cobre a maior parte do que ele entrega.
- **Os joins do offset**: o Illustrator oferece miter/round/bevel no *Offset Path*. Aqui a quina é
  sempre redonda porque a régua é a distância; um miter exige geometria, e a pilha de LPE
  (`VecOffset { join }`) já tem essa resposta no eixo certo.
- **A régua do campo de raster** (~0,5 px numa aresta dura, medida acima) é pré-existente e
  partilhada; curá-la é mexer no `edge_offset`, que move o contorno, o feather e o bevel — wave
  própria, com oráculo de aparência.
