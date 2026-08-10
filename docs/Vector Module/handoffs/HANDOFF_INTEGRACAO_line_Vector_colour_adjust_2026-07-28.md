# Handoff de integração — `line/Vector`: COLOR ADJUST (plano 24 W8)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Estado:** fechada, **pendente de smoke**

⚠️ **Esta é a QUARTA wave da mesma abertura de linha.** As três primeiras têm handoff próprio —
[`…_fx_blend_…`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md) (a lei de mistura),
[`…_turbulence_…`](HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md) (a turbulência) e
[`…_morphology_…`](HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md) (o Grow / Shrink) — e
as quatro **compartilham o bump de schema**. Integre as quatro juntas: o `main` nunca viu nenhuma.

⚠️ **A frase abaixo foi corrigida pela wave seguinte.** Ela vale para as quatro famílias que o
`apply_op` da W2 nomeou — e a **W9** (Duotone + Luma to Alpha,
[handoff](HANDOFF_INTEGRACAO_line_Vector_duotone_2026-07-29.md)) veio **por pedido do Enio**, fora
daquela lista, e integra com estas.

**Com esta, o catálogo do plano 24 FECHA.** O `apply_op` da W2 nomeou quatro famílias
(*color-matrix · morphology · displacement+turbulence · bevel*); não sobra nenhuma.

---

## O que o artista ganha

Um degrau **Color Adjust** com a ficha que ele já conhece — três sliders, o neutro no meio:

| knob | faixa | o que faz |
|---|---|---|
| **Hue** | −180°..+180° | gira a matiz (rotação **rígida** do croma em OKLab) |
| **Saturation** | −1..+1 | −1 drena até o cinza · +1 dobra o croma |
| **Brightness** | −1..+1 | −1 é preto **exacto** · +1 é branco **exacto** |

E compõe: `Glow → Adjust` gira a forma **e o halo**; `Adjust → Glow` gira só a forma. As mesmas
duas operações, desenhos diferentes — a tese da pilha.

---

## A pesquisa decidiu a FORMA; o REPO decidiu a LEI

**O `type` de quatro valores do `feColorMatrix` é interface de formato DECLARATIVO.** Quem o
desenhou como CONTROLE convergiu na ficha *Hue/Saturation*: Photoshop, After Effects, Krita,
Blender. O `luminanceToAlpha` ficou **de fora com motivo** — ele converte luminância em
**COBERTURA**, e a pista pontual existe precisamente para não mover cobertura: é outro verbo.

⚠️ **E então a pergunta mudou de lugar.** A rotação de matiz já estava escrita no repo:
`AdjustmentKind::HueSaturationBrightness`, no `ph2d-painter-effects`, rotulada literalmente
*"Hue/Saturation"*, com lei de CPU (`apply_hsb`) **e** kernel de GPU. Escrever uma segunda daria
ao app **duas respostas para *"o que o slider de matiz faz?"***.

⚠️ **O repo já tinha decidido QUAL lei, pagando por isso:** o doc do `apply_hsb` diz porque a
matiz roda em **OKLab** e não em HSL — *"HSL hue is numerically unstable for near-gray pixels …
the colored speckle Enio hit on the gray background"*. É uma cicatriz, não uma preferência.

---

## A wave é uma EXTRAÇÃO — a segunda vez que este módulo faz exactamente esta

`oklab_from_linear` / `oklab_to_linear` / o corpo do `case 0u` saíram para
**`crates/ph2d-render/src/shaders/colour_adjust.wgsl`**, prefixado pela `composite_source()` do
compositor **e** pelo `module_sources()` da pilha. É, ao pé da letra, o movimento que o cabeçalho
do `blend_modes.wgsl` já descreve (*"extraído do `layer_composite.wgsl` quando ganhou o SEGUNDO
consumidor"*), e o gate novo é o irmão exacto do que aquela extração deixou.

| pergunta | porta única | consumidores |
|---|---|---|
| o que matiz/saturação/brilho fazem a uma cor? | **`adjust_hsb`** (WGSL compartilhado) | o compositor de CAMADAS · a pilha de FX |
| este degrau ajusta cor? | `FxKindSpec::adjust_labels` | o painel (OFERECER) · o produtor (HONRAR) |
| ele está no neutro? | `FxOp::adjust_is_neutral` | o gate · o kernel |
| quanto ele espalha? | `op_reach` → **0** (pontual, a pista do Color Overlay) | o `stack_reach` |

⚠️ **O que a extração NÃO quebrou, provado:** os 37 gates de `layer_compositor_gpu` (que incluem
a paridade GPU↔CPU dos ajustes do Painter) e o `shader_adjustment_coefficients_bit_identical_with_rust`
seguem verdes — este último por ler a **fonte MONTADA**, que é precisamente a razão pela qual o
doc dele diz que ele lê a `composite_source()` e não o corpo solto.

---

## O oráculo não é uma tolerância: é a OUTRA implementação

**`the_adjust_is_the_law_the_painter_already_ships`** roda o degrau na GPU e o
`ph2d_painter_effects::adjustments::apply_adjustment` na CPU, sobre as mesmas cores e os mesmos
knobs, atravessando as mesmas duas transferências sRGB.

> **Pior divergência: 1 nível de byte**, em 5 combinações × 9 cores.

A força do oráculo é ele não ter sido escrito para esta wave: `apply_hsb` existe desde a W4 do
Painter, noutro crate, para outro consumidor.

---

## Três afirmações minhas que a medição derrubou

1. ***"o brilho move um pixel de qualquer cor"*** — falso. A fixture da varredura do catálogo é
   **BRANCA**, e `+brilho` é `out + (1−out)·b`, que em branco é branco: **0 de 12800 texels**. Um
   ajuste pontual tem pontos FIXOS por construção. A varredura passou a empurrar o brilho para
   BAIXO, e nasceu um gate para pinar onde eles estão.
2. ***"a rotação preserva o croma"*** — falso em cor viva: o vermelho da paleta cai a **0,641** num
   quarto de volta (a rotação é rígida em OKLab e o gamut do sRGB **corta** na volta a 8 bits).
   Reescrevi para *"nas duas que ficam no gamut"* e isso **também** era falso: eu tinha medido UM
   ângulo, e o âmbar cai a **0,817** a ⅜ de volta, o azul a **0,736** a −⅛. **Estar no gamut é
   propriedade do par (cor, ângulo), não da cor.** A fixture que contém o fenômeno é uma cor de
   croma BAIXO — no giro inteiro, razão **0,989..1,010**.
3. ***"o early-out do neutro é load-bearing para a byte-identidade"*** — falso, e foi uma MUTAÇÃO
   que o mostrou: sem o ramo, uma rampa sRGB COMPLETA sai com **0 de 4096 bytes diferentes**. O
   ramo é exactidão no FLOAT (que compõe numa pilha longa) e CUSTO — a frase que o `apply_hsb` do
   Painter já usava e que eu tinha lido sem a ler.

---

## Gates — 12 novos, 12 mutações, **11 sangram**

| gate | onde |
|---|---|
| `the_adjust_is_the_law_the_painter_already_ships` | GPU (cross-crate) |
| `a_neutral_adjust_is_byte_identical_to_no_adjust_at_all` | GPU |
| `the_hue_turns_the_colour_without_draining_it` | GPU |
| `the_hue_rotation_is_rigid_where_the_result_still_fits_in_the_gamut` | GPU |
| `the_saturation_drains_to_grey_and_doubles_the_chroma` | GPU |
| `the_brightness_reaches_exact_black_and_exact_white` | GPU |
| `an_achromatic_pixel_is_untouched_by_hue_and_saturation` | GPU |
| `the_adjust_reads_the_straight_colour_not_the_premultiplied_one` | GPU |
| `the_adjust_never_moves_the_coverage` | GPU |
| `the_colour_adjust_comes_from_the_shared_file_not_a_copy` | CPU (`ph2d-render`) |
| o bloco de ajuste em `a_row_paints_only_the_controls_its_kind_uses` · `the_three_adjust_knobs_reach_the_bus_with_a_bipolar_map_when_dragged` | seam (painel) |
| `the_adjust_knobs_cross_the_camera_unscaled` · `hit_of_decodes_the_grow_knob` (estendido) | shell |

**Rodar os de GPU:** `cargo test -p ph2d-render --test fx_stack_adjust_gpu --release -- --ignored`
(sem adapter fazem *skip gracioso*, que **não é verde**).

### O sobrevivente, e o que M5 ensinou

O sobrevivente é o item (3) acima: ele não expôs buraco de gate, expôs uma **frase errada** em três
doc-comments, corrigidos com o número.

⚠️ **E a mutação M5 mostrou a divisão de trabalho entre os gates:** apagar o braço do Color Adjust
do `cs_op_point` mata **8** gates desta wave e **não** mata o `every_kind_draws_something` — sem o
braço, o degrau cai na pista do Color Overlay, que repinta com o `tint` da varredura e portanto
*desenha alguma coisa*. A varredura pergunta **se** um tipo desenha; os dedicados perguntam **o quê**.

### Duas lições de gate que valem para a próxima wave

- **A régua de um slider tem DUAS portas.** O `event_filters` mapeia o valor que vai ao bus e o
  `populate_filters` regista o link que produz o número que o artista LÊ. Uma mutação que tornou só
  o `populate` unipolar **passou** pelo gate de bus — o sintoma seria o documento receber −90
  enquanto o chip mostra 0. O gate passou a afirmar as duas metades no mesmo gesto.
- **O painel tem tabela de fixture PRÓPRIA** (ele não alcança o `ph2d-ecs`), então tirar
  `adjust_labels` da tabela real não move o seam — quem o mata é a varredura de GPU. As duas
  metades são necessárias.

---

## Schema, contratos, ids

- **`PROJECT_SCHEMA` fica em 38** — a QUARTA leva da mesma linha (`blend`, `scale`/`detail`/`seed`,
  `grow`, agora `hue`/`sat`/`bright`). Um save v37 já é recusado pelo 38. **Uma linha, um bump.**
  ⚠️ O valor se **CONTA** a partir do `main` do dia.
- **Contrato congelado §6: INTACTO** — `architecture_contract_surface` (3 ✓) e
  `architecture_tool_contract_surface` (4 ✓) verdes, não auto-relato.
- `VEC_SCENE_SCHEMA_VERSION`: **intocado**.
- **`MAX_FILTER_KINDS` 11 → 12** (sem isso o "Add Color Adjust" não é pintado — o `paint` faz
  `.take()`; o gate de seam pega).
- **Ids novos (6):** `filter_{hue,sat,bright}_id` e os gêmeos `_num_id`.
- **`Globals` 96 → 112 bytes** (uma linha de 16). O `UNIFORM_STRIDE` (256) acomoda com folga.
- **Superfície pública nova:** `FxOp::COLOR_ADJUST` · `FxOp::{hue,sat,bright}` ·
  `FxKindSpec::adjust_labels` · `FxOp::{reads_adjust,adjust_is_neutral}` ·
  `FxOpGpu::{hue,sat,bright}` · `layer_compositor::COLOUR_ADJUST_WGSL` (pub(crate)).
  ⚠️ **`FxOpGpu` mudou de módulo** (`fx_stack_op`) — mesmo caminho de import.

### Um `assert` de tripwire foi RETIRADO, como ele próprio mandava

`the_blend_is_offered_where_a_colour_lands_on_something` pinava a coincidência
`takes_blend == !grows` com a mensagem *"se um dia deixarem de coincidir … apague este assert, não
o campo"*. O **Color Adjust é o primeiro tipo que não cresce E não toma a lei** (não tem cor
PRÓPRIA: a saída dele é a entrada ajustada — o argumento do Blur e do Feather). O assert caiu e no
lugar ficou a afirmação direta sobre o tipo que quebrou a coincidência.

---

## LOC

| arquivo | antes | depois |
|---|---|---|
| `ph2d-render/src/fx_stack.rs` | 711 | **656** + `fx_stack_op.rs` — *o que um degrau RESOLVIDO é* × *o FOLD que os executa* |

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=37 cargo run -p ph2d-host-desktop --release
```

Quatro pares — a MATIZ · a SATURAÇÃO · **a ORDEM** (`Adjust → Glow` contra `Glow → Adjust`, o
headline) · e os **PONTOS FIXOS** numa estrela CINZA (matiz e saturação não a movem; o brilho move).

⚠️ **A estrela é COLORIDA de propósito** — matiz e saturação são rotação e escala do CROMA, e um
pixel sem croma não tem o que girar. O quarto par mostra exactamente isso.

**As cenas `=34` (a lei de mistura), `=35` (a turbulência) e `=36` (o Grow / Shrink) continuam
válidas e precisam do mesmo smoke.**

---

## Aberto, nomeado

- **O DUOTONE de duas pontas** (mapear a luminância de uma cor escura a uma clara — o *Tint* do AE,
  o *Gradient Map* do Photoshop). O **Color Overlay com a lei `Color`** já entrega o tingimento
  monocromático (troca a matiz preservando a luminosidade) e o Painter tem um `gradient_map`
  próprio; o que falta é a PONTA ESCURA ser de outra matiz. É um degrau com uma **segunda cor**,
  não um knob — wave própria, se o uso a pedir.
- **`luminanceToAlpha`** — muda COBERTURA, então não cabe na pista pontual. Outro verbo.
- **A régua do campo de raster** (~0,5 px numa aresta dura) segue aberta desde a W7 — pré-existente
  e partilhada; curá-la move o contorno, o feather e o bevel.
