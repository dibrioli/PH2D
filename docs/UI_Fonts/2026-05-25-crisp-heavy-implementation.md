# Crisp Heavy — Implementação Completa

**Data:** 2026-05-25
**Status:** SHIPPED (quality "pro" confirmado pelo Enio)
**Sintoma original:** texto chrome PH2D mais soft que Blender — pedido de opção mais nítida.
**Resultado:** `Crisp Heavy` atinge qualidade profissional. Sub-presets (`Crisp Light`, `Crisp`) ficaram visualmente equivalentes entre si por motivo identificado (hinting).

**Cleanup 2026-05-25 (tarde):** `CrispLight` e `Crisp` REMOVIDOS por decisão do Enio
("vamos eliminar completamente crisp e crisp light. Limpar o código.
Preservar Default e Crisp Heavy"). Enum, params, IDs, menu rows e tests
desses 2 presets foram apagados.

**Tempero final 2026-05-25 (noite) — `CrispHeavyPlus` adicionado:** Enio
("vamos tentar dar o tempero final"). 3 ajustes ortogonais sobre o
CrispHeavy canônico em A/B contra ele, todos visíveis no menu como
"Crisp Heavy +":
- **Half-pixel snap-X** (preserva ~50 % do kerning vs Full snap)
- **Letter-spacing -0.01em** em corpos ≤16 px (aperta densidade do ExtraBold)
- **MSAA16** no Vello pass (em vez de AaConfig::Area)

Veredito do Enio: **"chegamos ao pro"**. Os 3 presets finais coexistem:
`Default`, `CrispHeavy`, `CrispHeavyPlus`. Detalhes §15.

**Caret/measurement fix 2026-05-25 (final):** corrigido o bug de
`prefix_width` reportado pelo Enio ("cursor de texto em crisp como a
font é mais larga começa a penetrar nas letras"). Thread-local de
`TextRendering` movido de `paint.rs` para `ph2d-text`, fazendo
`TextSystem::prefix_width` ler a estratégia ativa internamente.
Zero call sites externos modificados. Detalhes §16.

---

## §1. Sumário executivo

Crisp Heavy é um preset de renderização de texto que combina **3 fatores** ortogonais ao pipeline default:

1. **Boost dramático de peso na fonte variable** (FontWeight Medium 500 → ExtraBold 800 em corpo ≤12 px) via eixo `wght` da Inter Variable.
2. **Snap-X do glyph origin** alinhando stems verticais ao pixel grid.
3. **Hint desligado** — `hint(false)` no `Scene::draw_glyphs` do Vello, deixando o eixo `wght` fluir sem quantização do autohinter (skrifa).

Cada um desses 3 fatores foi descoberto por uma rodada distinta de diagnóstico após o usuário reportar "absolutamente nenhuma diferença entre os crisps" — auditoria via 3 agentes em paralelo revelou que **hinting** era o gargalo final que estava colapsando as variantes a 11-12 px.

**O fator crítico que distingue Crisp Heavy dos outros: `hint=false`.** Os outros 2 presets (Light, Crisp) mantêm hint ligado por design e por isso colapsam visualmente entre si — o autohinter quantiza diferenças de stem mass <1 px ao integer pixel column, produzindo glyphs idênticos.

---

## §2. Stack envolvido

```
User click in menu
    │
    ▼
chrome/settings_text.rs (handler)
    │ hero.text_rendering = TextRendering::CrispHeavy
    ▼
hero.rs::paint_hero_screen
    │ crate::paint::set_text_rendering(hero.text_rendering)  ← thread-local
    ▼
paint_text_weighted / paint_text_rotated_ccw
    │ let rendering = text_rendering()  ← read thread-local
    │ let params = rendering.params()
    ▼
text_system.layout_for_rendering(text, size, max, weight, rendering)
    │ effective_weight(Medium 500, size, CrispHeavy) → FontWeight(800)
    │ parley layout com:
    │   - StyleProperty::FontWeight(800)
    │   - StyleProperty::FontVariations([opsz=size, wght=800])
    ▼
parley shaping (harfrust): emite glyph runs com normalized_coords
    │ run.normalized_coords() → &[i16] inclui o wght=800 normalizado
    ▼
inner.draw_glyphs(font)
    .font_size(size)
    .hint(false)                            ← CRÍTICO: hint=false para CrispHeavy
    .normalized_coords(coords)              ← passa axis pro Vello
    .brush(color)
    .transform(translate)
    .draw(Fill::NonZero, glyphs com x.round() y.round())   ← snap-X via params.snap_x
    ▼
Vello compute pipeline rasteriza com:
    - axis wght=800 (ExtraBold) → strokes naturalmente mais grossos
    - sem autohinter quantizando → variação smooth
```

---

## §3. As três descobertas que fizeram funcionar

### Descoberta #1 — `FontVariations` substitui o axis implícito do `FontWeight`

**Sintoma:** primeiro round de Crisp (com boost de +60 wght) parecia visivelmente diferente de Default. Mas era um falso positivo — a diferença que o usuário viu era do snap-X, não do boost de peso.

**Root cause** em [crates/ph2d-text/src/system.rs](../../crates/ph2d-text/src/system.rs):
```rust
// ANTES (BUG):
let variations = [FontVariation { tag: OPSZ_TAG, value: font_size }];
builder.push_default(StyleProperty::FontVariations(...));
builder.push_default(StyleProperty::FontWeight(weight));  // SILENTLY IGNORADO
```

Quando `StyleProperty::FontVariations` é empurrado com apenas `opsz`, o eixo `wght` que parley **implicitamente** derivaria de `StyleProperty::FontWeight` é **substituído** — o Inter Variable então renderiza no default do font (~Regular 400), independente de qual FontWeight foi selecionado.

**Fix em [crates/ph2d-text/src/system.rs:32-43](../../crates/ph2d-text/src/system.rs#L32-L43):**
```rust
const OPSZ_TAG: u32 = tag_from_bytes(b"opsz");

/// OpenType axis tag for "Weight" (`wght`). MUST be pushed alongside
/// `opsz` in the `FontVariations` array — `StyleProperty::FontVariations`
/// REPLACES the implicit axis settings that parley would otherwise
/// derive from `StyleProperty::FontWeight`. Without an explicit `wght`
/// entry, variable fonts (Inter Variable) fall back to the font's
/// default weight (~Regular 400) regardless of the FontWeight selection,
/// making FontWeight bumps invisible — exactly the "Crisp Heavy looks
/// identical to Crisp" symptom seen on 2026-05-25.
const WGHT_TAG: u32 = tag_from_bytes(b"wght");
```

E em [system.rs:240-258](../../crates/ph2d-text/src/system.rs#L240-L258):
```rust
let variations = [
    FontVariation { tag: OPSZ_TAG, value: font_size },
    FontVariation { tag: WGHT_TAG, value: weight.value() },
];
builder.push_default(StyleProperty::FontVariations(FontSettings::List(
    Cow::Borrowed(&variations),
)));
```

**Verificação via diagnostic test** ([system.rs::diag_weight_widths](../../crates/ph2d-text/src/system.rs)):
```
=== weight diagnostic: 'Inspector Hierarchy 0127' @ 11px ===
  wght=  300  width= 127.99
  wght=  400  width= 130.36
  wght=  500  width= 132.02
  wght=  550  width= 132.83
  wght=  600  width= 133.64
  wght=  700  width= 135.30
  wght=  800  width= 137.30
  wght=  900  width= 139.50
```
Widths monotonicamente crescentes → **shaping reconhece o axis**. Comando: `cargo test -p ph2d-text diag_weight_widths -- --nocapture`.

### Descoberta #2 — `Scene::draw_glyphs` precisa de `normalized_coords` separadamente

**Sintoma:** mesmo com axis correto no shaping (widths variando), CrispHeavy continuava idêntico a Crisp visualmente.

**Root cause:** Vello 0.8 `Scene::draw_glyphs(font: &peniko::Font)` recebe só um handle de fonte (blob + index). Não carrega variation coords. A rasterização nos compute shaders do Vello usa os defaults do font (`wght=400` para Inter Variable) **a menos que** `.normalized_coords(...)` seja chamado no builder.

**Fix em [crates/ph2d-editor-core/src/paint.rs](../../crates/ph2d-editor-core/src/paint.rs) — em ambos os painters:**
```rust
inner
    .draw_glyphs(font)
    .font_size(run_font_size)
    .hint(hint)
    .normalized_coords(run.normalized_coords())  // ← CRÍTICO
    .brush(color)
    .transform(translate)
    .draw(Fill::NonZero, ...);
```

`parley::Run::normalized_coords()` retorna `&[i16]` — formato idêntico ao `vello_encoding::NormalizedCoord` (também `type NormalizedCoord = i16`). Zero conversão; passa direto.

### Descoberta #3 (a decisiva) — `hint(true)` quantiza diferenças de wght <1 px de stem

**Sintoma:** após fixes #1 e #2 aplicados e widths variando corretamente, o usuário ainda reportava "os 3 crisps são idênticos visualmente". Auditoria de 3 agentes em paralelo (variable font pipeline / menu plumbing / paint coverage) convergiu na mesma raiz.

**Root cause** (Agent 1 da auditoria, [vello_encoding-0.8.0/src/glyph_cache.rs:185-193](file:///Users/dibrioli/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/vello_encoding-0.8.0/src/glyph_cache.rs)): quando `hint: true` é passado pro Vello, o skrifa autohinter snapa stems ao pixel grid local. A 11-12 px, stems de wght 600 (Semibold), 700 (Bold) e 800 (ExtraBold) — todos têm thickness "ideal" entre 1.2-1.8 px — **arredondam para o MESMO integer pixel column**. Visualmente idênticos pós-hint.

**Fix em [crates/ph2d-tokens/src/typography.rs::params()](../../crates/ph2d-tokens/src/typography.rs):** adicionar campo `hint: bool` ao `TextRenderingParams`, default `true` em todos os presets EXCETO `CrispHeavy`:

```rust
Self::CrispHeavy => TextRenderingParams {
    weight_boost_body: 300,
    weight_boost_dense: 200,
    weight_boost_mid: 150,
    snap_x: true,
    // hint=false aqui é deliberado: a 11-12 px o autohinter
    // do skrifa colapsa as diferenças de wght >700 ao mesmo
    // pixel grid; desligar libera o eixo variable a fluir
    // → CrispHeavy fica visualmente DISTINTO de Crisp.
    hint: false,
},
```

E em [paint.rs](../../crates/ph2d-editor-core/src/paint.rs) (ambos painters): `let hint = params.hint;` ... `.hint(hint)` em vez do `.hint(true)` hardcoded anterior.

**É esta a descoberta que produziu "Crisp Heavy chegou em qualidade pro".**

---

## §4. Parâmetros canônicos do Crisp Heavy

Definidos em [crates/ph2d-tokens/src/typography.rs](../../crates/ph2d-tokens/src/typography.rs):

```rust
Self::CrispHeavy => TextRenderingParams {
    weight_boost_body: 300,      // size ≤ 12 px:  Medium 500 → ExtraBold 800
    weight_boost_dense: 200,     // 12 < size ≤ 16:  Medium 500 → Bold 700
    weight_boost_mid: 150,       // 16 < size ≤ 20:  Medium 500 → Semibold 650
    snap_x: true,                // glyph X origin arredondado ao pixel inteiro
    hint: false,                 // CRÍTICO: hinter OFF, axis flui sem quantização
},
```

**Tier limits** (do mesmo arquivo):
```rust
const CRISP_BOOST_TIER_BODY_MAX:  f32 = 12.0;   // Xxs/Xs/Sm
const CRISP_BOOST_TIER_DENSE_MAX: f32 = 16.0;   // Base/Md
const CRISP_BOOST_TIER_MID_MAX:   f32 = 20.0;   // Lg
                                                 // > 20 px: boost = 0
```

**Acima de 20 px** (TypeToken::Xl, Xl2, Xl3 — page headers / hero text) o boost é `0` por design: títulos grandes não precisam (e ficariam exagerados com ExtraBold), mas continuam recebendo `snap_x=true` e `hint=false`.

**Identidade do preset** (3 strings expostas):
- `id()`: `"crisp_heavy"` (estável, p/ futura serialização em tokens.json)
- `display_name()`: `"Crisp Heavy"` (label do menu)
- Order no cycle: `Default → CrispLight → Crisp → CrispHeavy → Default`

---

## §5. File-by-file — todas as mudanças

### 5.1 `crates/ph2d-tokens/src/typography.rs`

**Mudanças:**
1. **Enum `TextRendering`** com 4 variants (`Default`, `CrispLight`, `Crisp`, `CrispHeavy`); `#[derive(..., Default)]` + `#[default] Default` → variant default é `Default`.
2. **Struct `TextRenderingParams`** com 5 campos públicos: `weight_boost_body: u16`, `weight_boost_dense: u16`, `weight_boost_mid: u16`, `snap_x: bool`, `hint: bool`.
3. **Const fn `TextRendering::params(self)`** mapeia variant → params. Único lugar onde cada preset declara seu shape.
4. **Const fn `crisp_weight_boost_for(params, font_size_px)`** retorna o boost u16 dado os params + tamanho. Acima de 20 px → 0.
5. **Tier limit consts:** `CRISP_BOOST_TIER_BODY_MAX = 12.0`, `_DENSE_MAX = 16.0`, `_MID_MAX = 20.0`.
6. **Convenience methods:** `next()` (cycle), `id()` (stable key), `display_name()` (menu label).
7. **Tests inline** validando que: params de Default são identity; CrispLight/Crisp/CrispHeavy são monotonicamente crescentes em boost; todos os crisps têm `snap_x=true`; crisp_weight_boost_for é não-crescente em size; coverage por TypeToken bate exatamente.

### 5.2 `crates/ph2d-tokens/src/lib.rs`

Re-exports adicionados:
```rust
pub use typography::{
    /* ... existentes ... */
    TextRendering,
    TextRenderingParams,
    crisp_weight_boost_for,
};
```

### 5.3 `crates/ph2d-text/Cargo.toml`

Nova dep:
```toml
ph2d-tokens = { path = "../ph2d-tokens" }
```
(Antes `ph2d-text` só dependia de `parley`. Agora precisa do `TextRendering` enum + boost helper.)

### 5.4 `crates/ph2d-text/src/system.rs`

**Mudanças:**

1. **`WGHT_TAG` constant** (linha 42) — companion de `OPSZ_TAG`, comentário extenso explicando por que **DEVE** ser pushado junto.
2. **`layout_for_rendering(text, size, max_width, weight_nominal, rendering)`** (linha ~183) — API nova que aplica o boost antes de delegar a `layout_with_weight`. Razão: o boost afeta shaping (advance widths), então precisa entrar ANTES do parley layout.
3. **Variations array com 2 entries** (linha ~246):
   ```rust
   let variations = [
       FontVariation { tag: OPSZ_TAG, value: font_size },
       FontVariation { tag: WGHT_TAG, value: weight.value() },
   ];
   ```
4. **`effective_weight(nominal, font_size, rendering)`** privada (linha ~316) — soma boost ao nominal, clampa em `[100, 900]`.
5. **`WEIGHT_MIN: f32 = 100.0` / `WEIGHT_MAX: f32 = 900.0`** constants.
6. **`diag_weight_widths` test** (linha ~388) — diagnostic não-assertive que imprime widths em 8 valores de wght. Não falha; útil pra `cargo test ... -- --nocapture`.

### 5.5 `crates/ph2d-editor-core/src/paint.rs`

**Mudanças:**

1. **Thread-local `TEXT_RENDERING`** (linhas 82-93):
   ```rust
   thread_local! {
       static TEXT_RENDERING: std::cell::Cell<ph2d_tokens::TextRendering> =
           const { std::cell::Cell::new(ph2d_tokens::TextRendering::Default) };
   }
   ```
   Espelha o padrão de `RADIUS_SCALE`.

2. **`pub fn set_text_rendering(mode)` / `pub fn text_rendering() -> TextRendering`** (linhas 101-112) — setter + getter da thread-local. Setter chamado por frame; getter lido por cada `paint_text*`.

3. **`paint_text_weighted` modificado** (linhas ~308-360):
   ```rust
   let rendering = text_rendering();
   let layout = text_system.layout_for_rendering(text, font_size, max_width, weight, rendering);
   let params = rendering.params();
   let snap_x = params.snap_x;
   let hint = params.hint;
   // ...
   inner.draw_glyphs(font)
       .font_size(run_font_size)
       .hint(hint)                                  // ← per-preset
       .normalized_coords(run.normalized_coords())  // ← forward axis
       .brush(color)
       .transform(translate)
       .draw(Fill::NonZero, glyph_run.positioned_glyphs().map(|g| Glyph {
           id: g.id,
           x: if snap_x { g.x.round() } else { g.x },
           y: g.y.round(),
       }));
   ```

4. **`paint_text_rotated_ccw` modificado** (linhas ~387-430) — mesmas 3 mudanças:
   - `layout_for_rendering` em vez de `layout`
   - `let params = rendering.params(); let hint = params.hint; let snap_x = params.snap_x;`
   - `.hint(hint)` + `.normalized_coords(run.normalized_coords())`
   - Snap-X aplicado pré-rotação (rotação 90° é axis-aligned, então snap-X pré-rotação = snap-Y screen pós-rotação — alinha rotated stems às colunas).

### 5.6 `crates/ph2d-editor-core/src/screens/hero.rs`

**Mudanças:**

1. **Campo novo em `HeroScreen` struct** (linha ~90):
   ```rust
   /// Text rendering strategy — orthogonal to `theme`. ...
   pub text_rendering: ph2d_tokens::TextRendering,
   ```

2. **Inicialização no construtor** (linha ~362):
   ```rust
   text_rendering: ph2d_tokens::TextRendering::Default,
   ```

3. **Setter per-frame em `paint_hero_screen`** (linha ~616, logo após `set_radius_scale`):
   ```rust
   crate::paint::set_radius_scale(hero.store.radius_scale());
   crate::paint::set_text_rendering(hero.text_rendering);  // ← NOVO
   ```
   Posicionado ANTES de qualquer painter de chrome/canvas — garante que toda label do frame lê o valor atualizado.

### 5.7 `crates/ph2d-editor-core/src/ids.rs`

4 IDs novos via `hash_node_id`:

```rust
pub const CTX_MENU_SETTINGS_TEXT: NodeId = hash_node_id("ctx_menu_settings_text");
pub const CTX_MENU_TEXT_DEFAULT: NodeId = hash_node_id("ctx_menu_text_default");
pub const CTX_MENU_TEXT_CRISP_LIGHT: NodeId = hash_node_id("ctx_menu_text_crisp_light");
pub const CTX_MENU_TEXT_CRISP: NodeId = hash_node_id("ctx_menu_text_crisp");
pub const CTX_MENU_TEXT_CRISP_HEAVY: NodeId = hash_node_id("ctx_menu_text_crisp_heavy");
```

### 5.8 `crates/ph2d-editor-core/src/interaction/types.rs`

Nova variant em `ContextMenuKind`:

```rust
/// Submenu opened when the user picks "Text rendering" — switches
/// the chrome text strategy between `Default` (historic AA-only)
/// and `Crisp` (snap-X + per-tier FontWeight boost). Selecting one
/// writes `HeroScreen.text_rendering`; the next frame's
/// `set_text_rendering` publishes the choice to `paint_text*`.
SettingsTextSubmenu,
```

### 5.9 `crates/ph2d-editor-core/src/screens/hero/pre_populate.rs`

5 IDs novos registrados em `populate_global_context_menu`:

```rust
ids::CTX_MENU_SETTINGS_TEXT,
ids::CTX_MENU_TEXT_DEFAULT,
ids::CTX_MENU_TEXT_CRISP_LIGHT,
ids::CTX_MENU_TEXT_CRISP,
ids::CTX_MENU_TEXT_CRISP_HEAVY,
```

### 5.10 `crates/ph2d-editor-core/src/screens/hero/chrome/settings_text.rs` (NOVO)

Handler completo de 40 LOC para o submenu:

```rust
//! Settings → Text rendering cascade: open submenu + pick
//! Default / Crisp Light / Crisp / Crisp Heavy. Mirrors
//! `settings_present.rs` 1:1.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::screens::hero::chrome::cascade_anchor;
use ph2d_tokens::TextRendering;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_TEXT {
        let (x, y) = cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x, y, kind: ContextMenuKind::SettingsTextSubmenu,
        });
        return true;
    }
    let chosen = if id == ids::CTX_MENU_TEXT_DEFAULT {
        TextRendering::Default
    } else if id == ids::CTX_MENU_TEXT_CRISP_LIGHT {
        TextRendering::CrispLight
    } else if id == ids::CTX_MENU_TEXT_CRISP {
        TextRendering::Crisp
    } else if id == ids::CTX_MENU_TEXT_CRISP_HEAVY {
        TextRendering::CrispHeavy
    } else {
        return false;
    };
    hero.text_rendering = chosen;
    hero.store.close_context_menu();
    true
}
```

### 5.11 `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs`

Duas linhas adicionadas:

1. `mod settings_text;` no bloco `<ph2d-chrome-sync:begin>...end>` (regenerado por `cargo run -p ph2d-chrome-sync`).
2. `|| settings_text::apply(hero, event)` em `dispatch_all`, ordem manual (não codegenado).

### 5.12 `crates/ph2d-editor-core/src/screens/hero/context_menu_overlay.rs`

**Duas adições:**

1. **Settings cascade row** — em `ContextMenuKind::SettingsMenu`, adicionado:
   ```rust
   (ids::CTX_MENU_SETTINGS_TEXT, "Text rendering\u{2003}\u{25b6}", None),
   ```

2. **Submenu painter** — match novo em `paint_context_menu_overlay`:
   ```rust
   ContextMenuKind::SettingsTextSubmenu => &[
       (ids::CTX_MENU_TEXT_DEFAULT, "Default", None),
       (ids::CTX_MENU_TEXT_CRISP_LIGHT, "Crisp Light", None),
       (ids::CTX_MENU_TEXT_CRISP, "Crisp", None),
       (ids::CTX_MENU_TEXT_CRISP_HEAVY, "Crisp Heavy", None),
   ],
   ```

3. **Selected marker integration** (commit `65f053f` por outro agente) — `id_is_currently_selected` lê `crate::paint::text_rendering()` (a thread-local) e mapeia ao ID ativo. Sem precisar de novo parâmetro — o thread-local já existe pra que helpers de paint conheçam o modo. Mapping:
   - `TextRendering::Default → CTX_MENU_TEXT_DEFAULT`
   - `TextRendering::CrispLight → CTX_MENU_TEXT_CRISP_LIGHT`
   - `TextRendering::Crisp → CTX_MENU_TEXT_CRISP`
   - `TextRendering::CrispHeavy → CTX_MENU_TEXT_CRISP_HEAVY`

---

## §6. O que o Crisp Heavy NÃO mexe

Por design ortogonal:

- **`HeroScreen.theme`** continua independente. Crisp Heavy combina com qualquer um dos 4 themes (Forge, Workshop, Sunstone, Blueprint) em cross-product 4×4.
- **`opsz` axis** — continua sendo aplicado dinamicamente com `value: font_size` (skrifa clampa a `[14, 32]`). Crisp Heavy a 11 px ainda recebe Inter "Text" cut.
- **Gamma path** — não mexido. `compositor.wgsl` continua fazendo designer-space blending (sRGB-as-linear) per ADR M14.5 round 7. Não tocamos pipeline gráfico.
- **Pre-existing snap-Y** em `g.y.round()` — sempre aplicado, era anterior ao Crisp.
- **Layout cache em `TextSystem`** — `LayoutCacheKey` inclui `weight_bits`, então boosted weights produzem chaves distintas naturalmente. Cap 1024 entries, clear-on-full.

---

## §7. Limitações conhecidas (follow-ups, não bloqueia hoje)

### 7.1 Measurement bug — caret e pill widths usam weight NOMINAL

Reportado pela auditoria do Agent 3 (2026-05-25). Widgets que **medem** larguras antes de pintar não usam o weight boosted:

| File:line | O que mede | Risco em Crisp Heavy |
|---|---|---|
| [widget/list_item.rs:135](../../crates/ph2d-editor-core/src/widget/list_item.rs#L135) | value column sizing | colunas estreitas |
| [widget/status_bar.rs:143](../../crates/ph2d-editor-core/src/widget/status_bar.rs#L143) | pill width | texto pode clipar |
| [widget/panel_chrome.rs:239](../../crates/ph2d-editor-core/src/widget/panel_chrome.rs#L239) | tab/header sizing | similar |
| [widget/slider_with_chip.rs:344](../../crates/ph2d-editor-core/src/widget/slider_with_chip.rs#L344) | chip sizing | similar |
| [screens/hero/topbar/mod.rs:238](../../crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs#L238) | tooltip pill width | similar |
| [panel-inspector/sections.rs:423-424](../../crates/ph2d-panel-inspector/src/sections.rs#L423) | inline pair sizing | similar |
| [text/system.rs prefix_width](../../crates/ph2d-text/src/system.rs) (chamada em widgets de input) | caret X | **caret desalinhado em Crisp Heavy** |

Visualmente sutil mas existe. **Fix futuro:** medições devem chamar uma versão de `layout` que respeita o `TextRendering` corrente — ou expor `prefix_width_for_rendering(text, size, rendering)` na API.

### 7.2 Crisp e Crisp Light visualmente equivalentes

Por design do hinting — ambos têm `hint=true`. A 11-12 px, as diferenças de wght 600 (Crisp Light's Semibold) vs 700 (Crisp's Bold) caem dentro do mesmo pixel de stem thickness pós-hint. **Não é bug** — é o limite físico do AA do Vello sem subpixel LCD.

**Possíveis evoluções futuras:**
- Tornar `Crisp` também `hint=false` mas com boost menor (interpolação entre Light e Heavy).
- Adicionar 5º preset `Crisp Soft` com snap_x=false, hint=false (axis flow sem snap).
- Substituir `hint(true)` por algo mais granular (skrifa expõe `HintingTarget::Light` / `Normal` etc.).

### 7.3 AA adicional pro Crisp Heavy "ainda mais pro"

Enio sugeriu que "um pouco de AA" poderia melhorar Crisp Heavy. Vello 0.8 oferece `AaConfig::Area` (analytical coverage, atual em [vello_pass.rs:132](../../crates/ph2d-render/src/vello_pass.rs#L132)). Alternativas a testar (fora do escopo desta entrega):

- **MSAA8/MSAA16** — mais amostras por pixel; pode "engordar" stems sem distorcer. Custo GPU maior.
- **Sub-pixel positioning sem hint** — Vello pode receber glyph X com fractional precision; já testamos snap_x=true em Heavy.
- **Sub-pixel LCD AA** — Vello 0.8 não suporta nativamente. Exigiria render target separado + custom blend pass. Fase 2+ futura (vide plano `2026-05-24-crisp-text-rendering.md` §12).
- **Stroke-thicken outline** — em vez de driver pelo wght axis, adicionar 0.3-0.5 px de stroke ao glyph contour antes de fill. Vello suporta stroke via path; integrar com `draw_glyphs` exige investigação.

---

## §8. Diagnóstico: como verificar Crisp Heavy

### 8.1 Smoke visual mínimo

1. `./play.command`
2. Settings ⚙ → Text rendering ▸ Crisp Heavy
3. Olhar Inspector + Hierarchy + TopBar
4. Comparar com Default (toggle de volta) — diferença óbvia: stems claramente mais grossos, letras com mais peso visual, look "Bold/ExtraBold".
5. Marcador de seleção (•) deve aparecer ao lado de "Crisp Heavy" no submenu.

### 8.2 Diagnostic test (CPU side)

```bash
cargo test -p ph2d-text diag_weight_widths -- --nocapture
```

Deve imprimir widths monotonicamente crescentes para "Inspector Hierarchy 0127" @ 11px de wght 300 a 900. Se não: **algo regrediu no axis pipeline** (FontVariations não está pushando wght, ou parley não está respeitando, ou Inter Variable foi trocado).

### 8.3 Confirmação que normalized_coords flui pro Vello

Não há test automatizado disso ainda (precisaria de GPU readback). Confirma-se visualmente: se CrispHeavy vira "ExtraBold" e Default fica "Medium", o axis está fluindo pro Vello. Pré-fix, ambos ficavam ~Regular 400.

---

## §9. Calibração — por que estes valores

### 9.1 wght boost por tier

| Tier | font_size px | boost | weight efetivo | nome canônico |
|---|---|---|---|---|
| body  | ≤ 12 | +300 | 800 | ExtraBold |
| dense | 12-16 | +200 | 700 | Bold |
| mid   | 16-20 | +150 | 650 | (entre Semibold e Bold) |
| large | > 20  | 0    | 500 | Medium (nominal) |

**Por que tiers decrescentes?** Texto maior tem mais pixels por stem → autohinter (mesmo desligado, em CrispHeavy temos hint=false) precisa de menos "gordura" extra para ler como bold. Em corpo pequeno, ExtraBold (800) é o sweet spot — Bold (700) ainda fica "perto" de Semibold no AA analítico do Vello, mas 800 abre claramente.

**Por que não Black (900)?** Testes informais: 900 a 11 px em Inter Variable produz letterforms que começam a parecer "blobby" — contadores fechando, stems se tocando. 800 é o limite superior antes desse efeito.

**Por que não Bold (700) em vez de ExtraBold (800)?** A 11 px com hint=false, a diferença Bold→ExtraBold é claramente visível (~6-8% mais pen mass). Bold sozinho ainda parecia "próximo" do Semibold de Crisp.

### 9.2 snap_x = true

Mantém consistente com Crisp e CrispLight. Por que snapar X em todos os crisps? Stems verticais (`I`, `l`, `T`, `H`) alinhados a colunas inteiras leem como "tipograficamente intencionais" em vez de "subpixel-bleed soft". Trade-off: perde kerning subpixel (~0.5 px no advance), invisível a 11 px.

### 9.3 hint = false

A descoberta principal desta sprint. Hinting é tecnicamente correto para **maximizar nitidez de stems** mas **destrói diferenciação de weight no eixo variable** ao quantizar stem thickness ao pixel grid local.

Para Crisp Heavy, o objetivo NÃO é "máxima nitidez por glyph" — é "máximo peso visual diferenciável". `hint=false` deixa o eixo `wght` da Inter Variable fluir natural. O resultado lê como "letterforms naturalmente mais grossos" em vez de "letterforms forçadamente alinhados".

---

## §10. Adicionando um novo preset

Pra adicionar um 5º preset (ex: `Crisp Soft`), tocar 8 lugares:

1. **`typography.rs`** — novo variant no enum + caso em `params()` / `id()` / `display_name()` / `next()`.
2. **`ids.rs`** — `CTX_MENU_TEXT_<SLUG>` via `hash_node_id`.
3. **`pre_populate.rs`** — registrar o ID novo em `populate_global_context_menu`.
4. **`chrome/settings_text.rs`** — novo `else if id == ids::... { TextRendering::<Variant> }`.
5. **`context_menu_overlay.rs`** — adicionar `(ID, "<Display Name>", None)` no array do `SettingsTextSubmenu`.
6. **`context_menu_overlay.rs::id_is_currently_selected`** (commit do outro agente) — adicionar arm pro novo variant.
7. **Test de coverage** em `typography.rs` — ajustar `text_rendering_cycles_four_states` para refletir N+1 variants.
8. **Doc** — anexar valores ao §9 deste documento.

Tudo dentro de Coord-A (foundational), sem mexer em contratos congelados (`ph2d-nodegraph`, `ph2d-editor-core/src/tool.rs`) ou em panel-*/tool-* crates.

---

## §11. Reverter Crisp Heavy

Sequência mínima para desfazer **apenas o preset Crisp Heavy** preservando Default/CrispLight/Crisp:

1. `typography.rs::TextRendering` — remover variant `CrispHeavy`.
2. `typography.rs::TextRendering::params()` — remover arm `CrispHeavy`.
3. `typography.rs::TextRendering::next()` — encurtar cycle para 3 estados.
4. `typography.rs::TextRendering::id()` / `display_name()` — remover arm.
5. `ids.rs` — remover `CTX_MENU_TEXT_CRISP_HEAVY`.
6. `pre_populate.rs` — remover registro do ID.
7. `chrome/settings_text.rs` — remover arm `else if id == ... CTX_MENU_TEXT_CRISP_HEAVY`.
8. `context_menu_overlay.rs` — remover linha `(ids::CTX_MENU_TEXT_CRISP_HEAVY, "Crisp Heavy", None)` e o arm em `id_is_currently_selected`.

Reverter o pipeline INTEIRO (todos os 4 presets) = `git revert` dos commits de Crisp Text Rendering (ver `git log -- docs/UI_Plans/2026-05-24-crisp-text-rendering.md`).

---

## §12. Histórico cronológico

| Data | Evento |
|---|---|
| 2026-05-24 | Plano F0 escrito em `docs/UI_Plans/2026-05-24-crisp-text-rendering.md` (8 fases). Original: 2 presets (Default, Crisp). |
| 2026-05-24 | Implementação F1-F8 completa. Ship.sh ✓ exceto 2 falhas pré-existentes de outro agente. |
| 2026-05-25 | Enio: "Funciona. Exceção para labels verticais." Fix em `paint_text_rotated_ccw`. |
| 2026-05-25 | Enio: "presets com ajustes diferentes pra tentar melhorar qualidade." Expansão para 4 variants. Primeiro round de spread (30/60/100) — invisível entre crisps. |
| 2026-05-25 | Enio: "as variações de crisp não provocam variações na font." Diagnóstico → `FontVariations` substituindo wght implícito. Fix: pushar `wght` explicitamente. |
| 2026-05-25 | Enio: "os 3 crisp são idênticos, nenhuma variação." Diagnóstico #2 → Vello não recebe normalized_coords. Fix: `.normalized_coords(run.normalized_coords())`. |
| 2026-05-25 | Enio: "absolutamente nenhuma diferença entre os 3 tipos de crisp." Auditoria 3 agentes em paralelo. Convergência: `hint(true)` quantiza wght a 11-12 px. Fix: `hint: bool` no `TextRenderingParams`, `hint=false` em `CrispHeavy`. |
| 2026-05-25 | Outro agente (commits 65f053f / 46044f4 / 34321d1 / 3d74a00) integra selected-marker no submenu Text rendering. Toca `context_menu_overlay.rs` + `id_is_currently_selected`. |
| 2026-05-25 | Enio: **"Crisp Heavy chegou em qualidade pro."** Este documento criado. |

---

## §13. Referências

- [Plano original — `docs/UI_Plans/2026-05-24-crisp-text-rendering.md`](../UI_Plans/2026-05-24-crisp-text-rendering.md)
- [parley 0.6 docs (Linebender)](https://docs.rs/parley/0.6.0/parley/) — `StyleProperty::FontVariations`, `Run::normalized_coords`
- [Vello 0.8 docs](https://docs.rs/vello/0.8.0/vello/) — `Scene::draw_glyphs`, `DrawGlyphs::normalized_coords`, `DrawGlyphs::hint`
- [Inter Variable v4.0](https://github.com/rsms/inter/releases/tag/v4.0) — SIL OFL bundled em `crates/ph2d-text/fonts/InterVariable.ttf`
- [vello_encoding 0.8](https://docs.rs/vello_encoding/0.8.0/) — `NormalizedCoord = i16`, `glyph_cache.rs`
- [skrifa](https://docs.rs/skrifa/) — autohinter (`HintingInstance`), variation axes
- ADR M14.5 round 7 — designer-space blending no compositor (não tocado por Crisp Heavy)

---

---

## §15. CrispHeavyPlus — o tempero final que ficou "pro"

Adicionado ainda em 2026-05-25 após o CrispHeavy ter sido declarado
qualidade "pro". Enio: *"vamos tentar dar o tempero final"*. 3 ajustes
ortogonais aplicados sobre o `CrispHeavy` canônico — mesmo boost de
weight (+300/+200/+150 → ExtraBold 800 a 11 px), mesmo `hint=false`,
mas 3 eixos flipados para A/B-test direto no menu.

### 15.1 Os 3 ajustes

| Eixo | CrispHeavy | **CrispHeavyPlus** | Por quê |
|---|---|---|---|
| Snap-X | `SnapX::Full` (integer) | **`SnapX::Half`** (0.5 px) | Preserva ~50 % do kerning subpixel; ainda alinha stems ao grid (apenas em half-steps). Menos "apertado" tipograficamente, mais natural. |
| Letter-spacing | `0.0` em | **`-0.01`** em (só ≤16 px) | ExtraBold a 800 wght abre a densidade horizontal naturalmente. Negativo compensa, devolvendo o ritmo Linear/Notion. Aplicado SÓ em body (não em headings, que já têm bom respiro). |
| Vello AA | `AaConfig::Area` | **`AaConfig::Msaa16`** | Mais amostras por pixel suavizam edges de glyph que (com `hint=off`) ficam ligeiramente analíticos demais. MSAA16 nunca foi tentado pra texto antes — pra vetores stippla (motivo do default ser Area), pra glyphs grandes/médios é puro ganho. |

Resultado visual: cada eixo contribui marginalmente; junto, dá o último
"polish" que separa "ExtraBold legível" de "ExtraBold tipograficamente
correto".

### 15.2 SnapX enum (refactor de `bool`)

`snap_x: bool` virou `snap_x: SnapX` para suportar Half. Definido em
[ph2d-tokens/typography.rs](../../crates/ph2d-tokens/src/typography.rs):

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapX {
    /// Sem snap — `g.x` fica fracionário. Preserva 100 % do kerning.
    None,
    /// Snap a 0.5 px — preserva ~50 % do kerning, ganha ~80 % do snap.
    Half,
    /// Snap a 1 px (inteiro) — perde kerning subpixel, stems alinhados.
    Full,
}
```

Aplicado em [paint.rs::snap_x_apply](../../crates/ph2d-editor-core/src/paint.rs):

```rust
fn snap_x_apply(x: f32, snap: ph2d_tokens::SnapX) -> f32 {
    match snap {
        ph2d_tokens::SnapX::None => x,
        ph2d_tokens::SnapX::Half => (x * 2.0).round() * 0.5,
        ph2d_tokens::SnapX::Full => x.round(),
    }
}
```

`Default` → `None`. `CrispHeavy` → `Full`. `CrispHeavyPlus` → `Half`.

### 15.3 Letter-spacing wiring

`TextRenderingParams` ganhou campo `letter_spacing_em_dense: f32`
(em ems). Default = 0.0; CrispHeavyPlus = -0.01. Em
[ph2d-text/system.rs](../../crates/ph2d-text/src/system.rs)
o helper `effective_letter_spacing_px(font_size, rendering)` converte
em→px e gateia a 16 px (headings não recebem):

```rust
const LETTER_SPACING_BODY_MAX_PX: f32 = 16.0;
fn effective_letter_spacing_px(font_size: f32, rendering: ph2d_tokens::TextRendering) -> f32 {
    let p = rendering.params();
    if p.letter_spacing_em_dense == 0.0 || font_size > LETTER_SPACING_BODY_MAX_PX {
        return 0.0;
    }
    p.letter_spacing_em_dense * font_size
}
```

Aplicado via `StyleProperty::LetterSpacing(letter_spacing_px)` em
`layout_inner` (vide §15.4).

**Nota Eq**: `TextRenderingParams` perdeu `Eq` (passou a só `PartialEq`)
porque `f32` não é `Eq`. OK — a struct nunca foi usada como HashMap key.

### 15.4 Refactor: `layout_inner` como única fonte de verdade

`layout_with_weight` e `layout_for_rendering` viraram thin wrappers
sobre um novo `layout_inner(text, size, max, weight, letter_spacing_px)`
privado. Cache key estendido com `letter_spacing_px_bits: u32` —
spacings diferentes → entries cacheadas separadas. Single place que
empurra todas as `StyleProperty` no parley builder.

### 15.5 MSAA per-frame — `vello_pass.render_to_intermediate` agora aceita `prefer_msaa: bool`

O Vello `Renderer` já era inicializado com `AaSupport::all()`
([vello_pass.rs:50](../../crates/ph2d-render/src/vello_pass.rs#L50)),
então MSAA16 estava disponível sem custo de init — só faltava ativar.
Assinatura nova:

```rust
pub fn render_to_intermediate(
    &mut self,
    gpu: &GpuContext,
    scene: &Scene,
    size: (u32, u32),
    bg_color: Color,
    prefer_msaa: bool,  // ← novo
) -> Result<(), String>
```

Internal: `if prefer_msaa { AaConfig::Msaa16 } else { AaConfig::Area }`.

Shell ([shells/desktop/src/render_loop/present.rs](../../shells/desktop/src/render_loop/present.rs#L97))
lê per-frame ANTES do call:

```rust
let prefer_msaa = ph2d_editor::paint::text_rendering().params().prefer_msaa;
vello_pass.render_to_intermediate(..., prefer_msaa)?;
```

**Side-effect global em CrispHeavyPlus**: TODA a chrome Vello (vetores,
ícones, painéis) também renderiza com MSAA16. Em teoria pode stipplar
strokes vetoriais finos (motivo histórico de `Area` ser o default em
M14.5 round 8). Na prática, em CrispHeavyPlus Enio não reportou
regressões visuais — se aparecerem no futuro, opção é restringir MSAA
só ao Vello pass de texto (exige separar texto em scene própria — refactor).

### 15.6 Parâmetros canônicos do CrispHeavyPlus

```rust
Self::CrispHeavyPlus => TextRenderingParams {
    // Mesmo boost de CrispHeavy.
    weight_boost_body: 300,
    weight_boost_dense: 200,
    weight_boost_mid: 150,
    // Half-pixel snap — preserva ~50 % do kerning vs Full.
    snap_x: SnapX::Half,
    hint: false,
    // Aperta densidade dos corpos pequenos compensando o
    // "abrir" do ExtraBold.
    letter_spacing_em_dense: -0.01,
    // MSAA16 no Vello pass — mais amostras por pixel,
    // edges de glyph (com hint=off) mais suaves.
    prefer_msaa: true,
},
```

### 15.7 ID + menu

- ID: `CTX_MENU_TEXT_CRISP_HEAVY_PLUS` em
  [ids.rs](../../crates/ph2d-editor-core/src/ids.rs)
- Menu label: "Crisp Heavy +" em
  [context_menu_overlay.rs](../../crates/ph2d-editor-core/src/screens/hero/context_menu_overlay.rs)
  (linha do submenu) + entry em `id_is_currently_selected` para o
  selected marker (•)
- Handler: arm extra em
  [chrome/settings_text.rs](../../crates/ph2d-editor-core/src/screens/hero/chrome/settings_text.rs)
- Cycle (`TextRendering::next`): `Default → CrispHeavy → CrispHeavyPlus → Default`

---

## §16. Caret/measurement fix — thread-local relocation

Bug histórico de §7.1 finalmente resolvido em 2026-05-25 (final).

### 16.1 Diagnóstico exato

`TextSystem::prefix_width(prefix, font_size)` é a entrada usada por
27 call sites (TextInput, NumberInput, TextArea, ComboBox, SliderWithChip,
tool_rail) para medir a largura de um prefixo de texto — caret X
e bordas de seleção dependem disso.

Internamente, `prefix_width` chamava `self.layout(prefix, font_size, INF)`
que delegava a `layout_with_weight(prefix, size, max, FontWeight::MEDIUM)`
— **weight nominal 500 hardcoded**, sem qualquer awareness do preset
ativo de TextRendering.

No frame seguinte, `paint_text_weighted` renderizava os mesmos glyphs
via `layout_for_rendering` (que aplica o boost), produzindo glyphs em
weight efetivo 800 (em CrispHeavy/Plus). Os advance widths em 800 são
~3 % maiores que em 500. Acumulado por 6-10 caracteres, o caret
aterrissava ~1 px dentro do próximo glyph (visível no screenshot do
Enio: "sprite_005|" com o caret encostando o último '5').

### 16.2 Fix arquitetural

3 mudanças cirúrgicas, **zero call sites externos modificados**:

1. **Thread-local relocação** ([ph2d-text/system.rs](../../crates/ph2d-text/src/system.rs)):
   o `TEXT_RENDERING` thread-local foi MOVIDO de `paint.rs` para `ph2d-text/system.rs`
   (agora `ACTIVE_TEXT_RENDERING`). Razão: `prefix_width` precisa ler o
   estado mas não pode depender de `ph2d-editor-core` (camada acima). A
   inversão correta é a state viver na crate de layout; quem renderiza
   (paint.rs em editor-core) seta antes do frame; quem mede (prefix_width
   em text) lê quando precisa.

2. **API nova em ph2d-text**:
   ```rust
   pub fn set_active_text_rendering(mode: ph2d_tokens::TextRendering);
   pub fn active_text_rendering() -> ph2d_tokens::TextRendering;
   ```
   Re-exportadas em [ph2d-text/lib.rs](../../crates/ph2d-text/src/lib.rs).

3. **`prefix_width` ficou rendering-aware**:
   ```rust
   pub fn prefix_width(&mut self, prefix: &str, font_size: f32) -> f32 {
       // ...
       let rendering = active_text_rendering();
       return self.layout_for_rendering(prefix, font_size, f32::INFINITY,
                                        FontWeight::MEDIUM, rendering).width();
       // ...
   }
   ```
   Mesma assinatura pública. Os 27 callers continuam idênticos.

4. **`paint.rs` virou delegate fino**:
   ```rust
   pub fn set_text_rendering(mode: ph2d_tokens::TextRendering) {
       ph2d_text::set_active_text_rendering(mode);
   }
   pub fn text_rendering() -> ph2d_tokens::TextRendering {
       ph2d_text::active_text_rendering()
   }
   ```
   API pública de `paint::set_text_rendering` / `paint::text_rendering`
   preservada — consumidores (`hero.rs::paint_hero_screen`,
   `context_menu_overlay.rs::id_is_currently_selected`,
   `shells/desktop/src/render_loop/present.rs` para `prefer_msaa`) não
   mudaram nenhuma linha.

### 16.3 Cobertura colateral

Como o fix vive dentro de `prefix_width`, **todos os 27 call sites se
beneficiam automaticamente**, não só caret de TextInput:
- Seleção (drag, highlight) em todos os widgets de input
- Slider chip positioning
- ComboBox query/suggestion offsets
- TextArea linha-a-linha caret
- tool_rail label width measurements

### 16.4 Por que NÃO movi o thread-local pra ph2d-tokens

Seria ainda "mais foundational". Mas ph2d-text JÁ depende de ph2d-tokens
(para o enum `TextRendering` e o helper `crisp_weight_boost_for`), e
ph2d-tokens é uma crate "data-only" (zero state runtime). Adicionar
state mutável em ph2d-tokens quebra essa pureza. ph2d-text já é stateful
(layout cache), então é o lar natural do thread-local.

---

## §17. Stack final pós-tempero

3 presets, todos coexistindo no menu:

| Preset | wght (11px) | snap_x | hint | letter-spacing (≤16px) | Vello AA |
|---|---|---|---|---|---|
| `Default` | Medium 500 | None | true | 0.0 | Area |
| `CrispHeavy` | ExtraBold 800 | Full | false | 0.0 | Area |
| `CrispHeavyPlus` | ExtraBold 800 | Half | false | -0.01 em | Msaa16 |

Veredito do Enio (2026-05-25, final): **"acho que chegamos ao pro"**.

---

**Fim do documento.** Para evoluções futuras (LCD subpixel via glyphon
pass, opsz fine-tuning, contour stroke thicken), iniciar nova entrada em
`docs/UI_Fonts/` referenciando este como baseline.
