# PLANO — Seção **Texture** do pincel (Blender-parity, adaptado a 2D)

> **Status:** **P1 + P2 + P3 FEITOS** (commits locais `f7f40df1` engine, `a0862665` tool+panel+ids,
> `2936075f` Stencil; sem push). Plano **aprovado pelo Enio** (2026-06-22). **Seção Texture COMPLETA**
> (View/Tiled/Random/**Stencil**). Stencil = adaptação **espaço-de-imagem** (engine puro). Restam só 2
> follow-ups opcionais (gesto drag do Stencil + imagem importada) —
> [`HANDOFF_painter_texture_section.md`](../HANDOFF_painter_texture_section.md) §7.2.
>
> Todo file:line aqui foi **verificado no código real** (não é spec aspiracional) — os caminhos de
> costura batem com Stroke Methods, que é o template canônico.

---

## 1 — Objetivo e escopo

Adicionar uma seção **Texture** na view **Brush** do dock (mesma view de Stroke/Falloff), que
**modula o dab ao vivo**: uma máscara por-pixel multiplicando a cobertura, exatamente como o falloff
faz. Paridade comportamental com o Texture do Blender (`brush_painter_2d_tex_mapping`), **adaptada a
2D**: sem `3D`, sem eixo `Z`.

**Entrega visível:** o traço ganha textura (grão/ruído/quadriculado) que segue o pincel (View),
fica preso à imagem (Tiled) ou randomiza por dab (Random), com Angle/Rake/Random/Offset/Size.

**Fora do P1/P2:** Stencil (precisa overlay de shell → P3), imagem importada (pipeline de asset →
P3 opcional), modulação de **cor** (Blender tem; em 2D começamos só por **alpha/cobertura**).

---

## 2 — Decisões fechadas (§5 do handoff) — padrão-ouro + justificativa

### D1 — Fonte da textura ("+ New") → **procedurais embutidas** (imagem é follow-up P3)
`BrushSpec` é `#[derive(Copy)]` ([`spec.rs:28`](../../crates/ph2d-painter-brush/src/spec.rs)). Uma
textura **procedural** é só um enum + poucos params → **cabe em `Copy` sem alocar**; uma imagem
importada precisaria de pixels pesados no `PaintState` (como curve/circle/polygon) + handle leve no
spec. Logo procedural-first **alinha com a restrição de tipo E entrega o efeito sem pipeline de
asset**. Conjunto P1 (4 procedurais determinísticas, transcendental-free):

| Kind | Uso | Amostragem |
|---|---|---|
| **Noise** (default do "+ New") | grão de pincel (lápis/carvão) — o caso canônico | value-noise via hash inteiro + interp sqrt-free |
| **Checker** | debug do mapeamento + padrão duro | paridade do par `(⌊u⌋+⌊v⌋) & 1` |
| **Voronoi** (cells) | textura orgânica | menor dist² a pontos de célula (sqrt no fim) |
| **Stripes** | linhas/hachura | onda triangular de `u` (sem `sin`) |

**"+ New"** atribui a procedural default (**Noise**) quando não há textura (placeholder quadriculado =
`Kind::None`). **Clicar o thumbnail** abre um popover para escolher entre as 4 (idioma de dropdown já
existente no painel — não viola o screenshot, que não tem seletor de "tipo" explícito). Enquanto
`Kind::None`, **só** thumbnail + "New" aparecem (gating DIRETIVA §2 — sem controle no-op).

### D2 — Conjunto de mapeamentos → **View Plane · Tiled · Random** no P1/P2; **Stencil** no P3
`3D` **removido** (degenerado em raster 2D — §3 do handoff). **Stencil entra só no P3** porque exige
overlay em espaço de tela + gesto (shell). **Consequência DIRETIVA §2:** no P1/P2 o dropdown Mapping
lista **apenas View/Tiled/Random** — `Stencil` é **adicionado à lista no P3**, junto com o overlay,
para nunca existir opção morta.

### D3 — Determinismo do ângulo (HR-5) → **vetor-base, nunca ângulo no hot-path** (precedente Circle/Polygon)
O engine tem **zero transcendentais** hoje (verificado: grep limpo em `src/`); Circle subdivide arcos
por `normalize` (só `sqrt`), Polygon rotaciona por **constante baked** `POLY_STEP`
([`stroke/polygon.rs:24`](../../crates/ph2d-painter-brush/src/stroke/polygon.rs)). Sigo o mesmo:

- **Angle (slider, graus inteiros 0–360):** roto `(1,0)` por `angle_deg` aplicações da **constante
  baked** `DEG_STEP = (cos 1°, sin 1°) ≈ (0.999_847_7, 0.017_452_4)`. Uma constante, só `*`/`+`,
  bit-idêntico em todas as plataformas (drift acumulado é determinístico e visualmente irrelevante p/
  rotação de textura). Calculo o **vetor-base 1× por dab** (não por-pixel).
- **Rake:** usa a **tangente do traço** — que o engine **já computa** como vetor (sem ângulo, sem
  transcendental).
- **Random (flag):** vetor aleatório por-dab via **2 floats `splitmix64`**
  ([`stroke.rs:542 next_f32`](../../crates/ph2d-painter-brush/src/stroke.rs)) → reject-and-normalize
  (só `sqrt`). Determinístico dado o seed do traço.

**Sweep transcendental no fim** (como nos Stroke Methods); espero só `sqrt`. **Sub-tarefa P1
obrigatória:** confirmar se o caminho de dab/spec entra no **replay-hash** da CI — se entrar, o
esquema acima já é seguro; documento a confirmação.

### D4 — Como a textura multiplica → **alpha/cobertura por-pixel** (igual ao falloff)
Em [`dab.rs:79`](../../crates/ph2d-painter-brush/src/dab.rs) o loop faz
`let w = spec.falloff_weight(t);` e depois `let a = w * coverage` (linhas 93/95). A textura entra
**exatamente aí**: `let w = w * tex;` com `tex ∈ [0,1]`. Modula **só alpha** no P1 (cor é follow-up).

### D5 — Preview + perf → **o dab já é o preview; custo por-pixel controlado**
Modulação imediata (canvas CPU-residente pós-[ADR-0096], sem readback). Custo: amostragem dentro do
footprint do dab. Checker/Noise/Stripes são poucas ops inteiras + `sqrt`; **Voronoi** itera vizinhança
de células — **limito a 3×3 células** (custo O(1) por-pixel). Pré-computo o **basis + origem da
textura 1× por dab**; o per-pixel só faz 2 dot-products p/ UV (sem rotação/transcendental no loop
quente). Meço em `--release` se suspeitar (memory `feedback_measure_perf_symptom_scale`).

### D6 — Unidades + ranges (fonte única `TEX_*`, padrão dos sliders de Stroke)
| Param | Unidade | Range (const) | Default |
|---|---|---|---|
| Angle | graus inteiros | `0 ..= TEX_ANGLE_MAX_DEG (360)` | `0` |
| Offset X/Y | **fração de tile** (resolução-independente) | `TEX_OFFSET_MIN..MAX (−1.0 ..= 1.0)` | `0.0` |
| Size X/Y | escala (tiles no footprint) | `TEX_SIZE_MIN..MAX (0.1 ..= 10.0)` | `1.0` |
| (Tiled) tile base | px | `TEX_TILE_BASE_PX (256.0)` × Size | — |

Offset em fração de tile (não px nem "metros") — consistente com a natureza do mapeamento e
independente da resolução do canvas. Size = quantos tiles cabem no footprint (View) / por `TILE_BASE`
(Tiled).

**Coordenadas por mapeamento** (UV antes de samplear a procedural):
- **View Plane:** `uv = basis · ((pixel − dab_center) / (radius · size)) + offset` — textura presa ao
  footprint, centrada no cursor → *segue o pincel*.
- **Tiled:** `uv = basis · ((pixel − canvas_origin) / (TILE_BASE · size)) + offset` — presa à imagem.
- **Random:** View + **offset aleatório por-dab** (+ ângulo aleatório se flag Random).
- **Stencil (P3):** UV em espaço de tela, pos/rot/scale próprios do stencil.

---

## 3 — Modelo de dados (contrato novo, não-congelado)

**Engine — novo módulo [`crates/ph2d-painter-brush/src/texture.rs`](../../crates/ph2d-painter-brush/src/):**

```rust
pub enum TextureKind { None, Noise, Checker, Voronoi, Stripes }  // to_u8/from_u8/name()
pub enum TextureMapping { ViewPlane, Tiled, Random /*, Stencil (P3)*/ }  // to_u8/from_u8/name()

#[derive(Clone, Copy, ...)]            // Copy — sem pixels
pub struct TextureSettings {
    pub kind: TextureKind,
    pub mapping: TextureMapping,
    pub angle_deg: u16,                 // 0..=360
    pub rake: bool,
    pub random_angle: bool,
    pub offset: [f32; 2],              // −1..1, fração de tile
    pub size:   [f32; 2],             // 0.1..10, default [1,1]
}
pub const TEX_ANGLE_MAX_DEG: u16 = 360;
pub const TEX_OFFSET_MIN/MAX: f32 = -1.0 / 1.0;
pub const TEX_SIZE_MIN/MAX:   f32 = 0.1 / 10.0;
pub const TEX_TILE_BASE_PX:   f32 = 256.0;
pub const DEG_STEP: (f32, f32) = (0.999_847_7, 0.017_452_4);  // baked cos/sin 1°

// 1× por dab: resolve basis (rotação) + origem (offset/random) a partir do tangente do dab + rng.
pub struct TexDabBasis { u: [f32;2], v: [f32;2], origin: [f32;2] }
pub fn dab_basis(s: &TextureSettings, dab_dir: [f32;2], rng: &mut u64) -> TexDabBasis;
// por-pixel: retorna [0,1]; chamado em dab.rs no lugar do falloff.
pub fn sample(s: &TextureSettings, b: &TexDabBasis, px: i64, py: i64,
              dab_center: [f32;2], radius: f32, canvas_origin: [f32;2]) -> f32;
```

`BrushSpec` ganha **um** campo: `pub texture: TextureSettings` (mantém `Copy`).
Imagem importada (P3): pixels no `PaintState` + handle leve no spec — **não** no P1.

---

## 4 — Arquitetura por-camada (o que muda em cada arquivo) — espelha Stroke Methods

### Engine — `crates/ph2d-painter-brush/`
- **`src/texture.rs` (novo, < 600 LOC):** enums + `TextureSettings` + constantes + 4 samplers
  procedurais + `dab_basis` + `sample` + math de mapeamento. Testes em **sibling `texture/tests.rs`**.
- **`spec.rs:28`** (`BrushSpec`): + `texture: TextureSettings` + default.
- **`dab.rs:79`** (`stamp_dab`): multiplica `w *= texture::sample(...)`. Precisa receber por dab:
  **tangente** (Rake) e **rng/seed** (Random). Avaliar estender assinatura de `stamp_dab` **ou** o
  `Dab` (hoje em `stroke.rs`) p/ carregar `dir` + `tex_seed`; o `dab_basis` é computado em
  **`stamp_dabs`** ([`tool/paint.rs:381`](../../crates/ph2d-tool-painter/src/tool/paint.rs)) **1× por
  dab** e passado adiante (per-pixel barato).
- **`stroke.rs`:** expor a tangente do dab se ainda não exposta (o walk já a conhece).
- **`lib.rs:26`:** `pub mod texture;` + re-exports (`TextureKind`, `TextureMapping`,
  `TextureSettings`, consts).

### Tool — `crates/ph2d-tool-painter/`
- **`tool/paint.rs:63` (`BrushSettings`):** + `texture_kind: u8`, `texture_mapping: u8`,
  `texture_angle_deg: u16`, `texture_rake: bool`, `texture_random: bool`, `texture_offset_x/y: f32`,
  `texture_size_x/y: f32`. ⚠️ **paint.rs está em 599 LOC** (cap 600) → **mover a struct `BrushSettings`
  p/ um sibling** (ex.: `tool/paint/brush_settings.rs`, hoje 243 LOC) para abrir espaço (ver §6).
- **`tool/paint/brush_settings.rs`:** `brush_settings()` (l.36-68) copia de `b.texture.*`; **setters
  (fonte única de clamp):** `set_brush_texture_kind/mapping(u8)`, `set_brush_texture_angle_norm(t)`
  (`t*360→u16`), `toggle_brush_texture_rake/random()`, `set_brush_texture_offset_norm(axis,t)`,
  `set_brush_texture_size_norm(axis,t)`.
- **`tool/trait_impls.rs:40` (`handle_panel_event`):** rotear `SelectOption→set_*`, `SetValue→set_*`,
  `Click→toggle_*`/`set_*`, espelhando as linhas de Stroke (l.156-233).

### editor-core — `crates/ph2d-editor-core/src/ids/chrome/painter.rs` (hoje 568 LOC, cap 600)
- **Consts** (após l.179): `PAINTER_BRUSH_TEXTURE_THUMB`, `_NEW`, `_MAPPING`, `_ANGLE`, `_RAKE`,
  `_RANDOM`, `_OFFSET_X`, `_OFFSET_Y`, `_SIZE_X`, `_SIZE_Y`.
- **Factories** (após l.197): `painter_brush_texture_mapping_option_id(m)` e
  `painter_brush_texture_kind_option_id(k)` (padrão `fnv_node_id_runtime`).
- **IconId** "+ New" / thumbnail: reusar ícone existente (Add/Plus) se houver; senão IconId novo
  **em ordem alfabética** (memory `feedback_new_tool_icon_needs_iconid`).

### Panel — `crates/ph2d-panel-painter-layers/`
- **`src/paint_texture.rs` (novo):** `paint_texture_section()` (dropdown Mapping + thumbnail/New +
  sliders Angle/Offset/Size + toggles Rake/Random, **gated em `kind != None`**) + `paint_texture_
  popovers()`. Reusa `paint_param_row`/`paint_toggle_row`/`paint_dropdown_row`/`paint_dropdown_popover`
  ([`paint_brush.rs:249/288/451/380`](../../crates/ph2d-panel-painter-layers/src/paint_brush.rs)).
  Testes em **sibling `paint_texture/tests.rs`** (gate de LOC conta `mod tests` inline).
- **`paint_brush.rs:185`:** inserir `y = crate::paint_texture::paint_texture_section(...)` entre Stroke
  (l.185) e Eraser (l.188); + drenar popover em `paint_brush_popovers()`.
- **`populate.rs:99`:** registrar Mapping + thumbnail-picker como `Dropdown`, "New" como `Button`,
  Angle/Offset/Size como `Slider`, Rake/Random como `Button`. ⚠️ **sem registro, clique é no-op
  silencioso** (memories `feedback_panel_populate_register` + `..._context_menu_closes_on_down`).
- **`event.rs` (hoje 600 LOC = NO CAP):** `decode_texture_mapping_option` + `decode_texture_kind_
  option` cobrindo **TODAS** as opções (`0..=N` inclusive do último — bug clássico Circle@7) + rotear
  clicks. ⚠️ event.rs **no cap** → decoders provavelmente vão p/ submódulo `event/texture.rs` (§6).
  **Teste round-trip obrigatório** em `event/tests.rs`.
- **`state.rs`:** par `set/take_pending_brush_texture_{mapping,kind}_dd` (espelha stroke-method dd).

### Shell — `shells/desktop/` (só P3)
- **Stencil:** overlay em espaço de tela (`render_loop/painter_bridge.rs`, padrão Circle/Polygon) +
  gesto mover/rotacionar/escalar (`input_dispatch/painter_canvas_input.rs`).
- **Imagem importada (opcional):** load de arquivo (mirror dos importadores) → pixels no `PaintState`.

---

## 5 — Faseamento (arquivos + testes por fase)

### **P1 — Engine** (modulação ao vivo headless-testável)
**Arquivos:** `texture.rs` (novo) + `texture/tests.rs` · `spec.rs` · `dab.rs` · `stroke.rs` (tangente) ·
`lib.rs`.
**Testes (engine, sibling):** (a) `texture=1.0 ⇒ dab idêntico ao sem-textura`; (b) `texture=0.0 ⇒
zero paint`; (c) View vs Tiled **divergem** p/ o mesmo pixel em 2 centros de dab distintos; (d) ângulo
determinístico + **transcendental-free** (grep limpo); (e) procedural determinística (mesmo px ⇒ mesmo
sample); (f) Random determinístico dado seed. **Sweep transcendental + confirmação replay-hash.**

### **P2 — Tool + Panel** (UI viva, View/Tiled/Random)
**Arquivos:** tool `paint.rs`(+split BrushSettings)/`brush_settings.rs`/`trait_impls.rs` ·
editor-core `chrome/painter.rs` · panel `paint_texture.rs`(+tests)/`paint_brush.rs`/`populate.rs`/
`event.rs`(+`event/texture.rs`?)/`event/tests.rs`/`state.rs`.
**Testes:** painel — **visibilidade gateada** (sem textura ⇒ só thumb+New; com textura ⇒ resto),
**decode round-trip** Mapping + Kind; tool — setters/clamp (offset/size/angle nos limites).

### **P3 — Stencil + (opcional) imagem importada**
Stencil entra no dropdown (não-no-op) + overlay/gesto de shell; imagem importada se o Enio quiser.
Testes headless onde der.

### **Fim de cada bloco**
Gates (`architecture_panel_loc_cap`, `architecture_workspace_file_loc_cap`, `no_magic_numeric`,
`no_literal_color`, `hr12_widgets_a11y`) + clippy `--all-targets` + `rustfmt 1.95 --edition 2024` nos
**meus** arquivos + sweep transcendental → **commit LOCAL** (sem push/CI). Atualizo este plano + o
handoff (marca resolvido) + `HANDOFF_painter_stroke_section.md` se houver interação.

---

## 6 — Riscos e restrições de LOC (verificados — precisam de manobra)

| Arquivo | LOC atual | Cap | Manobra planejada |
|---|---|---|---|
| `tool/paint.rs` | **599** | 600 | mover struct `BrushSettings` p/ `tool/paint/brush_settings.rs` antes de adicionar campos |
| `panel/event.rs` | **600** | 600 | extrair decoders de textura p/ submódulo `event/texture.rs` |
| `editor-core/chrome/painter.rs` | 568 | 600 | ~30 linhas de ids/factories cabem; verificar headroom no commit |
| `panel/paint_brush.rs` | 582 | 600 | só ganha ~4 linhas (call + popover); corpo da seção vai em `paint_texture.rs` |
| `engine/stroke.rs` | 595 | 600 | **não tocar volume**; tangente já existe ou extrair p/ sibling se preciso |

Outros: **`Dab`/`stamp_dab` assinatura** — decidir no P1 entre estender o struct `Dab` (carrega
`dir`+`tex_seed`) ou passar `TexDabBasis` por argumento; preferência: **`TexDabBasis` por argumento**
(não incha o `Dab`, mantém o per-pixel barato). **Determinismo** é o risco-rei: o sweep + a
confirmação de replay-hash são gate de P1, não "depois".

---

## 7 — Definition of done (do handoff §8)
Seção Texture na view Brush: thumbnail + New + Mapping (View/Tiled/Random; +Stencil no P3) + Angle +
Rake + Random + Offset X/Y + Size X/Y (sem 3D, sem Z) · textura **modula o dab ao vivo** (máscara
por-pixel × cobertura) · **qualquer** opção/controle tem efeito (populate + decode cobertos + teste
round-trip) · engine/tool/painel verdes + todos os arch-gates + clippy + fmt + **sweep transcendental
limpo** · **commit LOCAL** · handoff atualizado.
