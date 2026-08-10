# Handoff de integração — `line/Vector`: Color Harmonies no picker compartilhado

**Data:** 2026-07-25 · **Branch:** `line/Vector` · **Commit:** `746afdfbd` · **Estado:** fechado, **pendente de smoke do Enio**.

Increment sobre a linha REABERTA (depois da integração de Falloff/Twist/Knot). Um único
commit. Todo o trabalho é em `ph2d-editor-core` + fiação mínima na shell.

## O que landou

O **picker de cor baseado no Blender** — a superfície ÚNICA de cor do app (Painter, Vector e
Inspector o abrem via `register_picker_swatch`) — ganhou uma seção **Color Harmonies**. Foi a
decisão do Enio de fazer disto uma ferramenta **global** melhorando o widget que todos já usam,
em vez de uma tool nova: zero duplicação, e toda superfície de cor herda a feature.

- **Seletor de 7 esquemas** (RadioGroup segmentado): Off / Comp / Anlg / Triad / Split / Tetra / Mono.
- **Tira de parceiras DERIVADAS** (aparece com um esquema ativo) + **botão "+"** que soma todas à paleta.
- Clicar numa parceira **a adota** como cor ativa (a base gira para ela — modelo "linked" do Corel).

## A espinha (porta única)

`widget::blender_color_picker::harmony::partners(base, scheme) -> Vec<ColorValue>` é a **porta
ÚNICA**: o painel a desenha e o dispatch a consome pela MESMA função, então o que o artista vê é o
que o clique pega. Rotações de matiz na roda **HSV** (a que o artista vê — complementar oposto,
tríade nos cantos); `Monochromatic` é o inverso (mantém matiz, varia o VALOR). As parceiras nunca
são guardadas — mover a base gira TODAS pelo mesmo Δ (a propriedade "linked").

`Harmony` é **view-state** (como `ChannelMode`): NÃO é serde. Palettes persistem à parte
(`~/.ph2d/palettes.txt`), então **nenhum `PROJECT_SCHEMA`**, **nenhum contrato §6**, **nenhum ADR**.

## Superfície tocada

- **Motor:** `widget/blender_color_picker/harmony.rs` (`Harmony` enum + `partners` + `paint_harmony_section`)
  + `harmony_tests.rs`. Re-exports `Harmony`/`harmony_partners` em `widget/mod.rs`.
- **State:** `BlenderColorPicker.harmony` (state.rs) + `blender_harmony`/`set_blender_harmony`
  (blender_ops.rs) + campo `harmony` na variante `InteractiveState::BlenderPicker` (state/mod.rs).
- **Ids:** `BLENDER_HARMONY_SCHEMES[7]`/`_SWATCHES[4]`/`_ADD` (ids/menus.rs) + campos no `BlenderSubIds`
  (sub_ids.rs), preenchidos no ÚNICO sítio (`color_picker_demo.rs`).
- **Hit kinds:** `BlenderHitKind::{HarmonyScheme(u8),HarmonySwatch(u8),HarmonyAdd}`.
- **Dispatch:** 3 braços novos em `dispatch/blender.rs`.
- **Registro:** `pre_populate_blender.rs` (hits em laço) + `PICKER_H` 560→620 (para a seção caber).
- **Shell:** `harmony_smoke.rs` + `mod` + call no prólogo + `harmony_smoke_done`.

⚠️ **LOC — split por responsabilidade:** os variants novos empurraram `interaction/types.rs` para
707 > 700. O vocabulário de hit do PRÓPRIO picker (`PaletteIoKind` + `BlenderHitKind`) saiu para o
irmão **`types_blender.rs`** (espelho do `types_menu.rs` que já existia), re-exportado por `types.rs`
⇒ `types::BlenderHitKind` e `interaction::BlenderHitKind` seguem resolvendo sem churn. `types.rs` 707→623.

## Gates (todos verdes, mutação-provados onde marcado)

- **6 de engine** (`harmony_tests.rs`): base sempre 1ª · contagem por esquema · **ângulos da roda
  medidos** (oráculo = MATIZ de volta por `rgba_to_hsv`, não a regra do offset; mutação `h`→`h+off`
  removida deixa toda parceira na base = RED) · mono mantém matiz varia valor · rotação da base gira
  todas pelo mesmo Δ.
- **3 de seam por PONTEIRO REAL** (`dispatch/tests/harmony.rs`, via `dispatch_pointer`): clicar
  segmento seleciona o esquema · **clicar parceira adota a cor que a seção derivou** (a MESMA porta)
  · "+" cresce a paleta pela contagem de parceiras. **Mutação provada:** o braço `HarmonySwatch`
  adotando `cur` em vez da parceira → o gate de adoção **RED**, restaurado → GREEN.
- **Colisão de id** (`node_id_collisions.rs`): `BLENDER_HARMONY_ADD` como const + os 2 arrays
  dobrados no teste de unicidade com **label distinto por elemento** (label compartilhado deixaria o
  dedup `(id,label)` mascarar uma colisão intra-array).
- **a11y** (`hr12_widgets_a11y.rs`): `harmony.rs` fia a11y; `harmony_tests.rs` (math pura) no `A11Y_OPT_OUT`.
- **LOC** (`architecture_workspace_file_loc_cap`) · **wiring parity** · **clippy --all-targets** ·
  **`cargo check --workspace`** · **shell `file_loc_caps`** — todos verdes.

## Smoke

**`PH2D_HARMONY_SMOKE=1 cargo run -p ph2d-host-desktop --release`** — abre o picker flutuante já
semeado com base laranja saturada (matiz ~30°) e o esquema **Triad**, então a seção Color Harmonies
aparece de cara com 3 parceiras. Conferir: trocar de esquema muda a tira · clicar parceira a adota ·
"+" soma à paleta · mover a base (roda/hex/chips) gira TODAS. Hues medidos (base 29,9°):
Comp `[29,9, 209,9]` · Triad `[29,9, 149,9, 269,9]` · Tetrad `[29,9, 119,7, 209,9, 299,7]`.

(O picker é compartilhado, então o mesmo comportamento aparece em qualquer swatch de cor de
Painter/Vector/Inspector — o smoke só o abre turnkey.)

## Aberto / decisões de produto (não construído sem pedido)

- **a11y por-swatch das parceiras:** as parceiras são hit-registradas como as swatches de paleta
  (que também não emitem nó a11y próprio) — o picker anuncia coarse. Consistente com o existente;
  se quiser nós por-parceira, é a mesma wave para as swatches de paleta.
- **Modelo do clique numa parceira:** hoje adota a parceira como nova BASE (a roda re-ancora). A
  alternativa (adotar a cor mas manter a roda ancorada na base original) é uma decisão de produto.
- **Persistir o esquema por-projeto:** deliberadamente NÃO (é view-state, como `ChannelMode`).
