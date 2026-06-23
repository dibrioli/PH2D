# HANDOFF — Painter **Texture** section (Brush texture, Blender-parity, 2D-adapted)

> **Atualização 2026-06-23** — P0-P3 + follow-ups FECHADOS. Pós-entrega:
> - **Size corrigido p/ paridade Blender** (MTex `texvec = size·co`): **maior número = textura MENOR**
>   (antes estava invertido). Afeta `texture::sample`/`sample_unit`.
> - **Conjunto de texturas expandido p/ 19 kinds** (`COUNT=19`): set Blender (Clouds, Distorted Noise,
>   Magic, Marble, Musgrave, Wood, Stucci, Gradient/Blend) + extras de pintura (Grain/papel,
>   Crosshatch, Dots/halftone, Grid, Bricks), além dos originais (Noise/Checker/Voronoi/Stripes/Image).
>   Samplers em **`crates/ph2d-painter-brush/src/texture/patterns.rs`** (novo módulo), todos
>   **transcendental-free** (HR-5: `sin/cos` do Marble/Wood/Magic → `wave01` polinomial). Dropdown +
>   decode auto-escalam por `0..COUNT` (zero mudança de painel/id).
> - **Perf Tiled/Stencil**: cache de textura em espaço-canvas (`stamp::blit_canvas_cached`) — textura
>   computada 1× por pixel por traço (commit `af8974d0`).

> **Para o próximo agente.** Enio pediu a seção **Texture** do pincel, seguindo o padrão Blender
> (fonte vendorizada em `reference/blender-texture-paint/`), com **uma adaptação**: *o que não se
> aplica à pintura 2D deve ser retirado ou adaptado*.
>
> ## ⛔ PRIMEIRO PASSO OBRIGATÓRIO — escreva um PLANO DETALHADO antes de implementar
>
> Enio determinou explicitamente: **antes de tocar em código, escreva um plano detalhado** (em
> `docs/Painter/` ou como `docs/PLAN_painter_texture_section.md`) e **apresente ao Enio para
> aprovação**. O plano DEVE resolver as decisões em aberto de [§5](#5--decisões-em-aberto-que-o-plano-precisa-fechar)
> (fonte da textura, conjunto de mapeamentos adaptados, modulação no engine, determinismo do ângulo,
> overlay de stencil, preview), propor um **faseamento**, e listar arquivos/testes por fase. Não
> comece a implementar sem o plano aprovado. **Decida no padrão-ouro e proponha** (o Enio confia na
> sua leitura técnica), mas **não pule o plano**.

---

## 1 — Estado atual (de onde você parte)

A Painter é um **host de Layers + Efeitos + brush engine clean-room** (ADR-0099/0097). O pincel já
tem, **completo e testado** (commits LOCAIS não-pushados — ver `git log`):

- **9 Stroke Methods**: os 7 do Blender (Dots/Airbrush/Anchored/Space/Drag Dot/Line/Curve) + **Circle**
  e **Polygon** (extensões PH2D — editores de forma on-canvas). Ver
  [`HANDOFF_painter_stroke_section.md`](HANDOFF_painter_stroke_section.md) §2.4 — **é o melhor template
  da arquitetura por-camada** que você vai repetir aqui.
- Seções do painel (view **Brush** do dock): **Size, Strength, Blend, Falloff (+ curva editável),
  Stroke, Eraser, Colour** — com **scroll + scrollbar** já funcionando na view Brush.

A **Texture** é uma **nova seção** dentro da view Brush (mesma view onde estão Stroke/Falloff), entre
seções existentes (sugestão: depois de Falloff ou depois de Stroke — você decide no plano).

> ⚠️ Há **6 commits locais não-pushados** (Curve, undo-fix, Circle, Circle-click-fix, panel-scroll,
> Polygon). **Não pushe / não rode CI** sem o Enio mandar ("ship"/"CI"). Você acumula commit local.

---

## 2 — Referência Blender (clean-room) + o que o screenshot pede

> **⚠️ GPL-2.0 vs PH2D proprietário** ([memory `project_blender_texture_paint_reference`]): a fonte
> em `reference/blender-texture-paint/` é **referência COMPORTAMENTAL apenas**. **Proibido portar
> código literal** — leia, entenda o algoritmo, reimplemente clean-room (foi assim em todos os Stroke
> Methods).

**Arquivos-chave da fonte vendorizada:**

- **Mapeamento de textura no paint 2D** (o coração): `…/editors/sculpt_paint/mesh/paint_image_2d.cc`,
  função `brush_painter_2d_tex_mapping` (≈ linha 677). Define como cada **modo de mapeamento** gera as
  coordenadas de textura do dab. Os modos (`MTEX_MAP_MODE_*`):
  - **VIEW** ("View Plane"): textura presa ao footprint do pincel, centrada no cursor — *segue o
    pincel*. **Default do paint 2D.**
  - **TILED**: textura azulejada nas coordenadas do canvas — *fica fixa na imagem* enquanto você
    pinta por cima.
  - **RANDOM**: como VIEW mas com **offset/ângulo aleatório por dab**.
  - **STENCIL**: textura fixa em espaço de tela (você pinta "através" dela); tem posição/rotação/
    escala próprias (precisa de overlay de tela).
  - **3D**: mapeia coords 3D — **não se aplica a pintura 2D pura** (ver adaptação §3).
- Amostragem da textura por-texel: `BKE_brush_sample_tex_3d` + `brush_imbuf_tex_co` (mesmo arquivo,
  ≈ linhas 290/440/538) — a textura multiplica a máscara do dab por-pixel.
- Painel (UI Python, layout de referência): procure `brush_texture_settings` /
  `VIEW3D_PT_tools_brush_texture` na árvore `…/scripts/startup/bl_ui/` (se presente) ou use os
  **screenshots** abaixo como spec canônica de layout.

**Spec de UI (dos screenshots do Enio):**

| Controle | Tipo | Observação |
|---|---|---|
| Thumbnail da textura | preview | quadriculado = nenhuma textura atribuída (placeholder) |
| **+ New** | botão | cria/atribui uma textura nova |
| **Mapping** | dropdown | `View Plane`, `Tiled`, `Random`, `Stencil` (Blender também tem `3D` — **dropar**, ver §3) |
| **Angle** | grau (0–360) | rotação da textura |
| **Rake** | checkbox | o ângulo segue a direção do traço |
| **Random** | checkbox | randomiza o ângulo por dab |
| **Offset X / Y** | metros/px | deslocamento do mapeamento (Blender tem **Z** — **dropar**, ver §3) |
| **Size X / Y** | escala (1.00) | escala do mapeamento (Blender tem **Z** — **dropar**) |

---

## 3 — A adaptação 2D (regra do Enio: o que não se aplica, retira/adapta)

Proposta (CONFIRME/refine no plano, justificando cada uma):

- **Mapping `3D` → REMOVER.** É para coordenadas 3D de superfície / texturas procedurais 3D; em
  pintura 2D de raster é degenerado (o próprio Blender só o mapeia para "canvas 0..1"). Conjunto final
  sugerido: **View Plane · Tiled · Random · Stencil**. (DIRETIVA §2: não pinte controle no-op — se
  dropar 3D, ele some do dropdown, não fica desabilitado.)
- **Offset Z / Size Z → REMOVER.** 2D não tem Z. Mantém **X/Y** apenas.
- **Angle / Rake / Random → MANTER** (todos afetam rotação 2D da textura).
- **Unidade do Offset:** o Blender mostra "m" (metros, herança 3D). Em 2D use **px** ou fração do
  diâmetro — decida e seja consistente com o resto do painel (Spacing usa %, Jitter usa px/fração).

---

## 4 — Arquitetura por-camada (onde cada peça vai) — siga o padrão dos Stroke Methods

Repita EXATAMENTE o pipeline que os Stroke Methods usam. Mapa dos arquivos canônicos:

### Engine — `crates/ph2d-painter-brush/`
- **`spec.rs` → `BrushSpec`**: adicione os campos da textura (modo de mapeamento, ângulo, flags
  rake/random, offset XY, size XY, e a **referência/handle da textura**). É um `Copy` struct — a
  textura em si (pixels/procedural) provavelmente **não** cabe aqui; veja §5 (fonte da textura).
- **`dab.rs` → `stamp_dab`** (o **ponto de modulação**): no loop por-pixel há
  `let w = spec.falloff_weight(t); … let a = w * coverage;`. A textura entra **exatamente aqui**:
  `a = w * coverage * tex_sample(px, py, mapping)`, multiplicando a máscara como o falloff faz. Você
  precisa, por dab: as coords de textura por-pixel (do modo de mapeamento + angle/offset/size) e a
  amostragem (procedural ou imagem).
- Considere um módulo novo `texture.rs` (sibling em `ph2d-painter-brush`) com o modelo de textura + a
  amostragem + os modos de mapeamento (transcendental-free onde der — ver §5 determinismo). Mantenha
  arquivos **< 600 LOC** (testes inline contam no gate de painel; no engine os testes ficam em
  `*/tests.rs` siblings, já excluídos — ver os Stroke Methods).
- Exporte o que o tool/panel precisam nomear via `lib.rs`.

### Tool — `crates/ph2d-tool-painter/`
- **`tool/paint.rs` → `BrushSettings`** (snapshot que o painel lê): adicione os campos da textura.
- **`tool/paint/brush_settings.rs` → setters `set_brush_*`**: um setter por controle (clamp aqui é a
  fonte única; o painel só encaminha valores crus). Mirror dos `set_brush_spacing/jitter/…`.
- Se a textura tiver estado pesado (pixels), guarde no `PaintState` (como `curve`/`circle`/`polygon`)
  e exponha um snapshot leve no `BrushSettings`.

### editor-core — `crates/ph2d-editor-core/src/ids/chrome/painter.rs`
- **NodeIds novos** para cada widget (thumbnail, New, Mapping dropdown + factory de option-id, Angle
  slider, Rake/Random toggles, Offset X/Y, Size X/Y). Siga `painter_brush_stroke_method_option_id` /
  `PAINTER_BRUSH_*` como modelo. **IconId** novo se houver ícone (ordem alfabética — memory
  `feedback_new_tool_icon_needs_iconid`).

### Panel — `crates/ph2d-panel-painter-layers/`
- **Nova seção de paint** (ex.: `paint_texture.rs`, sibling de `paint_stroke.rs`/`paint_falloff.rs`),
  chamada de dentro de `paint_brush.rs::paint_brush_body` (entre Falloff e Stroke, ou após Stroke).
  Reuse os helpers `paint_param_row` / `paint_toggle_row` / `paint_dropdown_row` /
  `paint_dropdown_popover`. **Gate por relevância**: ex. controles de Stencil só quando Mapping=Stencil
  (DIRETIVA §2).
- **`populate.rs`**: **REGISTRE** cada widget novo (botões como `Button`, o Mapping como `Dropdown`).
  ⚠️ **Sem registro no populate, o clique não faz nada** (não-`is_focusable` → Down não arma →
  Up não emite Click) — memories `feedback_panel_populate_register` +
  `feedback_context_menu_closes_on_down_repaint`.
- **`event.rs`**: encaminhe cada evento via `EditorAction::ToolPanelEvent(PanelEvent::…)`. Para o
  dropdown de Mapping, escreva um `decode_*_option` e **cubra TODAS as opções** —
  ⚠️ **bug clássico desta sessão**: o decode do Stroke Method usava `0..7` e engolia o Circle (wire 7);
  o clique "não fazia nada". **Toda faixa de decode `0..N` precisa incluir o último valor.** Teste de
  round-trip obrigatório (ver `event/tests.rs`).
- **`trait_impls.rs` (no tool)**: roteie cada `PanelEvent::{Click,SetValue,SelectOption}` novo para o
  setter correspondente (mirror das linhas de Stroke).

### Shell — `shells/desktop/`
- **Mapping=Stencil** precisa de **overlay em espaço de tela** + provavelmente mover/rotacionar/
  escalar o stencil → trabalho de shell (`render_loop/painter_bridge.rs` para o overlay, padrão dos
  overlays de Circle/Polygon; `input_dispatch/painter_canvas_input.rs` para o gesto). Avalie no plano
  se Stencil entra já no P1 ou é fase posterior.
- Se a textura vier de **imagem importada**, o load de arquivo é shell (mirror dos importadores
  existentes). Se for **procedural**, não precisa.

---

## 5 — Decisões em aberto que o PLANO precisa fechar

1. **Fonte da textura ("+ New").** O screenshot mostra placeholder quadriculado (= sem textura).
   Opções: (a) **procedurais embutidas** (checker/noise/voronoi — deterministas, zero pipeline de
   asset), (b) **imagem importada**, (c) ambas. **Recomendação para o P1:** começar por **procedurais
   embutidas** (entrega o efeito visível sem pipeline de asset; imagem é follow-up). Justifique sua
   escolha. Defina o que "+ New" cria.
2. **Conjunto de mapeamentos.** Confirme View/Tiled/Random/Stencil; drope 3D (§3). Decida se **Stencil**
   entra no P1 (precisa de overlay de shell) ou em fase posterior.
3. **Determinismo do ângulo (HR-5).** Rotacionar a textura por `angle` pede `cos/sin`. O brush engine
   tem precedente de **evitar transcendentais** (snap_to_45, Circle/Polygon via vetor). Decida:
   (a) pré-computar `cos/sin` do ângulo **uma vez por dab** (determinístico dado o ângulo) e passar o
   vetor adiante, OU (b) confirmar que o caminho de dab **não** entra no replay-hash da CI e documentar
   a exceção. **Faça um sweep** de transcendentais ao fim (como nos Stroke Methods). Rake usa a direção
   do traço (vetor já disponível — sem ângulo); Random usa o RNG dep-free já existente no engine
   (splitmix64 — determinístico).
4. **Como a textura multiplica.** Confirmar: máscara por-pixel multiplicando `coverage` (como o
   falloff), em `dab.rs`. A textura modula **alpha** (e opcionalmente cor — Blender tem modos; em 2D
   comece só por alpha/cobertura, cor é follow-up).
5. **Preview ao vivo + perf.** A modulação é imediata (o dab já é o preview). Cuide do custo por-pixel
   (amostragem dentro do footprint do dab); meça em `--release` se suspeitar (memory
   `feedback_measure_perf_symptom_scale`).
6. **Offset unit + ranges** (px vs fração; ângulo 0–360; size 0–N) — defina os `BRUSH_*_MAX` como
   fonte única (padrão dos sliders de Stroke).

---

## 6 — Gotchas (lições desta sessão — leia antes de codar)

- **Velocidade:** inner loop = `cargo check -p <crate>` no **slot warm** (`bash scripts/slot-seed.sh
  slot-1` → prefixe `CARGO_TARGET_DIR=…/target-slots/slot-slot-1`). Teste/clippy/gates 1× no fim.
- **LOC caps (gate executável):** arquivo **≤ 600 LOC**; **função de painel ≤ 200 LOC**
  (`architecture_panel_loc_cap`). O gate **per-file do painel conta `mod tests` INLINE** → mova testes
  para **sibling** `paint_texture/tests.rs` + `#[cfg(test)] mod tests;` (foi preciso fazer isso com
  `paint_stroke`/`event` nesta sessão). Função grande → extraia helper (extraí `paint_brush_view`).
  `shells/**` **não** é escaneado pelo gate de LOC (só `crates/**`).
- **`no_magic_numeric`** (escaneia widget/screens/painel): zero `f32` literal de UI sem o marcador
  `// LITERAL-PX-OK: <motivo>` **na mesma linha do literal**.
- **`no_literal_color`** (escaneia shell/widget/screens): cores baked precisam `// LITERAL-COLOR-OK:`.
- **`hr12_widgets_a11y`** (HR-12): widgets novos precisam a11y wired.
- **Decode de dropdown:** cubra **todas** as opções (`0..N` inclusivo do último) — bug Circle-click.
  Teste round-trip em `event/tests.rs`.
- **populate.rs:** registre TODO widget novo (botão/dropdown), senão o clique é no-op silencioso.
- **fmt:** pino **rustfmt 1.95 `--edition 2024`**; rode em **arquivos específicos seus**
  (`rustup run 1.95 rustfmt --edition 2024 <meus arquivos>`) — `cargo fmt -p` reformata WIP alheio.
  Cheque com `rustup run 1.95 cargo fmt --all -- --check`.
- **Git anti-colisão:** `git add -- <seus paths>`; `git commit --no-verify -m "msg" -- <paths>`
  (`-m` ANTES do `--`); `git status` antes; há WIP alheio na árvore (docs deletados, `Cargo.lock`,
  `docs/Painter/` untracked) — **não stage** nada que não seja seu.
- **Não pushe / não rode CI** sem o Enio mandar. Acumule commit local.
- **Smoke é do Enio** (caneta/mouse) — você valida headless (engine/tool/painel); GUI fica para ele.

---

## 7 — Faseamento sugerido (refine no plano)

> **STATUS (2026-06-22):** **TUDO FEITO — P0–P3 + os 2 follow-ups** (commits locais `f7f40df1`,
> `a0862665`, `2936075f`, `8428006c` Stencil-gesture, `d49d13f2` imagem; sem push). Plano aprovado
> pelo Enio em [`docs/Painter/PLAN_texture_section.md`](Painter/PLAN_texture_section.md). **A seção
> Texture está COMPLETA** — Kind picker (None/Noise/Checker/Voronoi/Stripes/**Image**) + New +
> Mapping (**View/Tiled/Random/Stencil**) + Angle + Rake + Random + Offset X/Y + Size X/Y, gated;
> modula o dab ao vivo. **Stencil** com overlay + handles (arrastar canto=resize, centro=move).
> **Imagem importada** via file-picker (luminância → máscara). Nada pendente além de smoke do Enio.

- **P0 — PLANO** ✅ — decisões §5 fechadas no padrão-ouro + faseamento + arquivos/testes →
  aprovado.
- **P1 — Engine** ✅ — `crates/ph2d-painter-brush/src/texture.rs` (novo) + `texture/tests.rs`:
  `TextureKind` (None/Noise/Checker/Voronoi/Stripes) + `TextureMapping` (ViewPlane/Tiled/Random) +
  `TextureSettings` (Copy) + `dab_basis`/`sample` + 4 samplers procedurais. `BrushSpec` ganhou o
  campo `texture` (default None). Modulação em `dab.rs` via novo `stamp_dab_textured` (`stamp_dab`
  delega com `None` → tool segue verde). **Transcendental-free** (sweep limpo; só `sqrt`), rotação
  por `DEG_STEP` baked, Rake=tangente, Random=splitmix64. 16 testes verdes; clippy/fmt/LOC-cap
  verdes; tool crate compila.
- **P2 — Tool + Panel** ✅ (`a0862665`) — `BrushSettings` movido p/ `brush_settings.rs` (paint.rs
  estava em 600) + 7 campos de textura + snapshot + setters (clamp único; New=Noise); `stamp_dabs`
  resolve basis por-dab (tangente Rake de centros consecutivos + RNG `tex_rng` por-traço) e chama
  `stamp_dab_textured`. editor-core: 10 NodeIds + 2 factories em `painter_texture.rs` (sibling novo;
  fnv twin → `pub(super)`). Painel: `paint_texture.rs` (Kind picker + New sempre; resto gated em
  kind≠None) + populate + state + decoders movidos p/ `event/decode.rs` (event.rs estava em 600),
  range `0..=COUNT`. **Testes:** painel (visibilidade gateada none→picker / kind→tudo; round-trip
  Kind+Mapping) + tool (setters/clamp + e2e: dab texturizado mascara o footprint). Gates verdes
  (LOC/magic/color/a11y/node-id), clippy, fmt, sweep limpo; shell compila.
- **P3 — Stencil** ✅ (`2936075f`) — `TextureMapping::Stencil` (wire 3, COUNT→4): máscara retangular
  **em espaço de imagem** (adaptação 2D — engine puro, sem transform de tela) posicionada/rotacionada/
  escalada pelos sliders Offset (centro) / Size (extensão) / Angle; mascara fora do retângulo.
  `stencil_frame()` é fonte única (centro/half/rotação) p/ a máscara E o overlay. Painel: Stencil
  entra no dropdown sozinho (COUNT) + esconde Rake/Random p/ Stencil. Shell: overlay **read-only** do
  retângulo (espelho do Circle). **Testes:** engine (frame, mascara-fora + fixo-à-imagem, padrão lê) +
  tool (overlay + e2e mascara) + painel (gating Stencil). Gates/clippy/fmt/sweep verdes; shell compila.
  **Follow-ups deferidos (§7.2):** gesto drag-to-position + imagem importada.
- **Fim:** gates (LOC/magic/a11y/color), clippy, fmt, sweep transcendental, commit LOCAL. Atualize
  ESTE handoff (marque resolvido) + `HANDOFF_painter_stroke_section.md` se a Texture interagir com Stroke.

---

## 7.1 — Estado de costura que o P2 herda (engine pronto, falta ligar UI→engine)

A capacidade existe e é testada; o P2 só **liga**. Pontos exatos:

- **Engine API a chamar:** `ph2d_painter_brush::texture::dab_basis(&settings, dab_dir, &mut rng) ->
  TexDabBasis` (1× por dab) + `ph2d_painter_brush::stamp_dab_textured(buf, w, h, center, &spec,
  coverage, preserve_alpha, Some(&basis))`. Hoje o tool chama `stamp_dab` (sem textura) em
  `tool/paint.rs` (`stamp_dabs`, ~l.381) — **trocar p/ `stamp_dab_textured`** passando o basis.
- **`dab_dir` (Rake):** o `Dab` **não carrega direção**. Derive a tangente em `stamp_dabs` de
  `d[i].center − d[i-1].center` (normalizada; primeiro dab → `[0,0]`, que cai no ângulo). **Não
  precisa mudar o struct `Dab`.**
- **`rng` (Random):** use o seed por-traço já existente (`PaintState.seed`) com um contador por-dab,
  como o jitter faz — determinístico (HR-5).
- **Campos a expor no `BrushSettings` + setters (clamp único):** `kind`(u8), `mapping`(u8),
  `angle_deg`(u16), `rake`(bool), `random_angle`(bool), `offset`[x,y], `size`[x,y]. Ranges/consts
  já existem no engine: `TEX_ANGLE_MAX_DEG`, `TEX_OFFSET_MIN/MAX`, `TEX_SIZE_MIN/MAX`. Wire u8:
  `TextureKind::to_u8/from_u8` (COUNT=5, inclui None), `TextureMapping::to_u8/from_u8` (COUNT=3).
- **"+ New":** setter atribui `kind = Noise` (default real); thumbnail-picker cobre as 5 kinds.
- **Decode de dropdown:** cubra `0..=TextureMapping::COUNT-1` e `0..=TextureKind::COUNT-1` (inclui o
  último — bug Circle@7). Round-trip test usando `to_u8/from_u8`.
- **Restrições de LOC já mapeadas (verificadas):** `tool/paint.rs` 599/600 → mover `BrushSettings`
  p/ sibling antes de add campos; `panel/event.rs` 600/600 → decoders em `event/texture.rs`. Ver
  plano §6.

## 7.2 — Follow-ups (FEITOS: Stencil drag-gesture `8428006c` + imagem importada `d49d13f2`)

> **(A) e (B) abaixo foram implementados.** (A) Stencil drag: `paint/stencil.rs` (canto=resize,
> centro=move; handles desambiguam → sem modificador; Angle pelo slider, sem `atan2`); shell desenha
> os handles. (B) Imagem: `TextureKind::Image` + `ImageMask` (bilinear center-coord) + storage no
> `PaintState` + request-flag que o shell pollla → rfd + decode (`AssetDb`) + luminância Rec.601.
> **O texto histórico abaixo era o plano dos follow-ups** (mantido p/ referência + os refinamentos
> ainda em aberto: stencil fixo-à-tela; máscara alpha-aware em vez de só luminância).


**Decisão de arquitetura do Stencil (feita no P3):** Stencil é **espaço de IMAGEM**, não de tela. O
engine `ph2d-painter-brush` é puro (sem UI/tela/GPU); threadear o transform de viewport nele
quebraria essa fronteira. Então o stencil é um retângulo fixo ao canvas, posicionado por Offset/Size/
Angle. O Blender-style "fixo à tela" (fica parado na tela ao pan/zoom) é o que esses follow-ups
trariam se quisermos — mas exige threadear o transform tela→imagem (vem do `deliver_canvas_pointer`,
`scale=(hi_x-lo_x)/iw`) até o `sample`, e é onde mora a complexidade.

**(A) Gesto drag-to-position do Stencil (ergonomia; shell).** Hoje o stencil se posiciona por
sliders + overlay read-only. Para arrastar/rotacionar/escalar o retângulo no canvas:
- Estado: o frame já vem de `offset/size/angle` (não precisa novo estado no `PaintState`). O gesto
  edita esses 3 via os setters existentes (`set_brush_texture_offset_norm/size_norm/angle_norm`).
- Shell: dar handles ao overlay (`stencil_overlay()` já dá os 4 cantos; adicionar handles de canto/
  rotação como Circle) em `painter_bridge.rs`; hit-test + drag em `painter_canvas_input.rs`. ⚠️ O
  stencil **coexiste com pintura** (você pinta E reposiciona) → precisa de **modifier-gate** (ex.:
  segurar uma tecla para mover o stencil em vez de pintar), distinto do Circle/Polygon (que SÃO o
  stroke method). Esse interleave é o pedaço arriscado — daí ter sido deferido.
- Converter drag de tela→imagem: `scale` já existe no `deliver_canvas_pointer`; o delta de tela ÷
  scale = delta de imagem → vira delta de offset (÷canvas) / size / angle.

**(B) Imagem importada (opcional).** Pixels pesados no `PaintState` (como curve/circle) + handle leve
no `BrushSpec.texture` (id/`Arc`); `texture::sample` ganha um `TextureKind::Image` que lê a imagem
(bilinear, **center-coord** — memory `feedback_pixel_center_vs_edge_coord`). Load de arquivo = shell
(mirror dos importadores). "+ New" passa a oferecer "from image". `TextureKind::COUNT` sobe sozinho.

## 8 — Definition of done

- Seção **Texture** na view Brush do dock: thumbnail + New + Mapping (View/Tiled/Random/Stencil) +
  Angle + Rake + Random + Offset X/Y + Size X/Y (sem 3D, sem Z).
- A textura **modula o dab ao vivo** (máscara por-pixel multiplicando a cobertura), com os modos de
  mapeamento adaptados ao 2D.
- Clicar QUALQUER opção/controle **tem efeito** (populate + decode cobertos; teste round-trip).
- Engine/tool/painel **verdes** (cargo check + nextest dos crates tocados); **todos os arch-gates**
  (LOC file/fn, no_magic, no_literal_color, hr12) verdes; clippy + fmt verdes; **sweep de
  transcendentais** limpo (ou exceção documentada).
- **Commit LOCAL** (sem push/CI). Handoff atualizado.

---

### Apêndice — comandos úteis (desta sessão)

```bash
# slot warm
bash scripts/slot-seed.sh slot-1
export CARGO_TARGET_DIR=/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/target-slots/slot-slot-1

# inner loop
cargo check -p ph2d-painter-brush          # engine
cargo check -p ph2d-tool-painter           # tool
cargo check -p ph2d-panel-painter-layers   # painel
cargo check -p ph2d-host-desktop           # shell

# gates (rode no fim)
cargo test -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers
cargo test -p ph2d-editor-core --test architecture_panel_loc_cap \
  --test architecture_workspace_file_loc_cap --test no_magic_numeric \
  --test no_literal_color --test hr12_widgets_a11y
cargo clippy -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers -p ph2d-host-desktop --all-targets
rustup run 1.95 cargo fmt --all -- --check

# sweep transcendental (geometria nova; espere só sqrt)
grep -nE "\.(sin|cos|tan|atan2|exp|ln|log|powf|powi)\b" crates/ph2d-painter-brush/src/<novos>.rs
```
