# Crisp Text Rendering — Plano Fase 0

**Data:** 2026-05-24
**Autor:** Coord-A (Opus 4.7 — main session)
**Status:** READY-TO-RUN (sem commits ainda; outros agentes paralelos ativos)
**Caminho DIRETRIZ:** (C) Coord-only — foundational (ph2d-tokens / ph2d-text / ph2d-editor-core / shells/desktop). Não-paralelizável.
**ADR necessário:** Não — não toca contratos congelados §4. Toca paint.rs (foundational), mas adição ortogonal (thread-local + 1 enum + 1 campo HeroScreen). Sem bump de cap. Sem mudança de superfície de trait pública.

---

## Resumo executivo

Adicionar uma estratégia opcional de renderização de texto chamada **Crisp**, exposta no menu global como `Settings ▸ Text rendering ▸ Default | Crisp`. **Não substitui** a aparência atual (`Default`), apenas oferece um caminho alternativo focado em legibilidade em corpo pequeno (≤14px).

O modo `Crisp` aplica 3 ajustes ortogonais à pipeline atual de texto:

1. **Snap X integral por glifo** (atualmente só Y é snapado — [paint.rs:319](../../crates/ph2d-editor-core/src/paint.rs#L319)).
2. **Boost de `FontWeight` por faixa de tamanho** (atualmente fixo em Medium 500 para body e SemiBold 600 para títulos — [system.rs:164](../../crates/ph2d-text/src/system.rs#L164)).
3. **Verificação documentada de gamma path** — já é correto (designer-space blending intencional em [compositor.wgsl](../../crates/ph2d-render/src/shaders/compositor.wgsl)); registramos a auditoria, não mexemos.

Por que `opsz` NÃO está na lista: já é aplicado dinamicamente em [system.rs:216-222](../../crates/ph2d-text/src/system.rs#L216-L222) com `value: font_size`; skrifa clampa para `[14, 32]`, então corpo 11px já recebe o cut "Text" mais pesado do Inter. Verificação pertence à auditoria final, não a uma mudança.

**Arquitetura escolhida (padrão ouro):** thread-local global `set_text_rendering(mode)` + `text_rendering()` getter, espelhando exatamente o padrão de [`set_radius_scale`](../../crates/ph2d-editor-core/src/paint.rs#L84) já em produção (Wave 8). Zero alteração na assinatura de `paint_text*` ou em `PaintCtx` — os 99+ call-sites de paint de texto continuam intactos. O modo é lido dentro de `paint_text_weighted` e ramifica internamente.

**Custo total estimado:** 6 fases sequenciais, ~250-350 LOC novos, zero refactor amplo.

---

## Princípios operacionais (LEIA antes de iniciar)

1. **Loop contínuo automático.** Cada fase termina em "erro zero" antes da próxima começar. "Erro zero" significa: `cargo check -p <crate>` verde, `cargo test -p <crate>` verde, clippy `--all-targets` verde para o crate da fase. Não delega ao agente humano — re-roda até verde.
2. **Sem perguntas durante execução.** Qualquer dúvida resolve sozinho com critério: **padrão ouro, perfeição, sem economias, sem gambiarras**. Os critérios já estão registrados em cada fase deste documento — releia a fase se precisar decidir.
3. **Sem commits.** Outros agentes paralelos ativos. Todas as mudanças ficam em working tree. Coord-A faz commit unificado depois, fora deste plano.
4. **Smoke por último.** Não roda `./play.command` durante as fases. Só ao final, com a lista canônica de testes visuais (§9).
5. **Hard Rules respeitadas.**
   - HR-15 (i18n) — labels novos do menu usam strings inglesas em literal (mesmo padrão dos labels atuais "Forge", "Workshop", "Sunstone", "Blueprint" — i18n virá em sweep separado).
   - HR-12 (a11y) — itens novos do menu emitem `Node` via `WidgetStore::register` igual aos pares.
   - HR-18 (LOC cap shell) — shells/desktop ganha ~3 linhas em `init`, sem risco.
   - HR-3 (zero-alloc hot path) — `set_text_rendering(mode)` chamado 1× por frame antes do paint, mesma pegada de `set_radius_scale`. Zero alloc.
6. **Gates ativos respeitados.**
   - `no_literal_color` — não mexemos em cores.
   - `no_magic_numeric` — adições de literais novos em `paint.rs` (`weight_boost(...)`, `snap_x_when_crisp`) ficam dentro de `// LITERAL-PX-OK: <razão>` quando necessário; preferimos consts nomeadas (`CRISP_WEIGHT_BOOST_BODY`, `CRISP_WEIGHT_BOOST_HEADING`).
   - `hr12_widgets_a11y` — não introduzimos widget novo; trabalhamos sob o widget de menu existente.
   - `architecture_widget_loc_cap` — não tocamos widgets.
   - Demais gates — não afetados.
7. **Isolamento.** Coord-A, slot `coord-a`. Antes da sessão: `source scripts/slot-env.sh coord-a`. Não tocar pastas onde outros agentes têm WIP (checar [`docs/SESSION_ACTIVE.md`](../SESSION_ACTIVE.md) antes).

---

## Mapa de arquivos tocados

| Fase | Arquivo | Tipo | Linhas estim. |
|---|---|---|---|
| F1 | [`crates/ph2d-tokens/src/typography.rs`](../../crates/ph2d-tokens/src/typography.rs) | Add `enum TextRendering` + helpers | +30 |
| F1 | [`crates/ph2d-tokens/src/lib.rs`](../../crates/ph2d-tokens/src/lib.rs) | Re-export | +1 |
| F2 | [`crates/ph2d-text/src/system.rs`](../../crates/ph2d-text/src/system.rs) | Add `layout_for_rendering()` API; computa weight efetivo | +60 |
| F2 | [`crates/ph2d-text/src/lib.rs`](../../crates/ph2d-text/src/lib.rs) | Re-export (se necessário) | +0-1 |
| F3 | [`crates/ph2d-editor-core/src/paint.rs`](../../crates/ph2d-editor-core/src/paint.rs) | Thread-local `TEXT_RENDERING` + `set_text_rendering` + `text_rendering` + branch em `paint_text_weighted` | +50 |
| F4 | [`crates/ph2d-editor-core/src/screens/hero.rs`](../../crates/ph2d-editor-core/src/screens/hero.rs) | Add `text_rendering: TextRendering` em `HeroScreen` + chamada `set_text_rendering` antes de paint | +5 |
| F5 | [`crates/ph2d-editor-core/src/ids.rs`](../../crates/ph2d-editor-core/src/ids.rs) | 3 novos `NodeId`: `CTX_MENU_SETTINGS_TEXT`, `CTX_MENU_TEXT_DEFAULT`, `CTX_MENU_TEXT_CRISP` | +6 |
| F5 | [`crates/ph2d-editor-core/src/screens/hero/pre_populate.rs`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs) | Registrar 3 IDs no `populate_global_context_menu` | +3 |
| F5 | [`crates/ph2d-editor-core/src/screens/hero/chrome/`](../../crates/ph2d-editor-core/src/screens/hero/chrome/) | **Novo handler** `settings_text.rs` + entrada em [`chrome/mod.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs) `dispatch_all` | +40 (novo) +2 (mod.rs) |
| F6 | Painter do menu de Settings (cascade overlay) | Render dos 2 sub-items + 1 caret no item pai | +15-30 (depende do painter) |
| F7 | Testes inline em ph2d-tokens / ph2d-text / paint.rs | Cobertura | +60 |

**Total:** ~270-330 LOC.
**Risco de tocar contrato congelado:** zero.
**Risco de LOC cap (HR-18 shells):** zero — shell ganha ≤5 linhas.

---

## Decisões locked (autônomas, padrão ouro)

Estas decisões estão FECHADAS. Não revisitar durante execução.

### D1 — TextRendering é ortogonal a Theme

`TextRendering` é um enum SEPARADO de `Theme`. Theme = paleta de cores + layout (Floating/Sidebar). TextRendering = estratégia de rasterização tipográfica. Cross-product: qualquer Theme combina com qualquer TextRendering. **Razão:** mistura conceitual (text-strategy dentro de Theme enum) violaria SRP e quebraria a simetria do menu — usuário deveria poder ter "Forge + Crisp" e "Sunstone + Default" sem hack.

### D2 — Thread-local, não parâmetro

Espelha o padrão de [`set_radius_scale`](../../crates/ph2d-editor-core/src/paint.rs#L84) — 200+ call-sites de `paint_rounded_rect` não recebem `radius_scale` como parâmetro. Mesmo para texto: 99 call-sites de `paint_text*` não recebem `TextRendering`. **Razão:** invariância de signature pública = zero churn em widgets, zero risco de breakage em panels paralelos. Já é padrão validado no projeto.

### D3 — Snap X é integral, não fracionado

Em `Crisp`: `g.x.round()` (mesmo tratamento de Y). **Razão:** para corpo ≤14 px, kerning subpixel produz delta visual ≤0.5 px, abaixo do limite de discriminação humano para texto chrome. A nitidez perceptual ganha é muito maior — confirmado por crítica adversarial e prática Pango/Skia/CoreText.

### D4 — Boost de weight é em FAIXAS, não linear

Função pura `crisp_weight_boost(font_size: f32) -> u16`:
- `size ≤ 12.0` → +60  (ex.: Medium 500 → ~Semibold 560)
- `12.0 < size ≤ 16.0` → +40 (ex.: Medium 500 → 540)
- `16.0 < size ≤ 20.0` → +20
- `size > 20.0` → 0

**Razão:** o boost compensa coverage AA fino do Vello em corpo pequeno — efeito diminui com tamanho. Faixas discretas evitam micro-shifts de stroke (variable axis interpolation visível em zoom). Valores derivados da prática Blender/Linear/Notion + Inter variable weight axis (sup. fino 100-900). Final dentro de `FontWeight` value `[100, 900]` clamping em [system.rs](../../crates/ph2d-text/src/system.rs) (já aplicado pelo skrifa).

### D5 — opsz não muda

[system.rs:216-222](../../crates/ph2d-text/src/system.rs#L216-L222) já cravando `opsz = font_size`. Skrifa clamp a `[14, 32]`. Para size=11, opsz efetivo = 14 (Text cut, ~120% de pen mass do design Display). Forçar opsz=14 explicitamente em Crisp é idêntico ao comportamento atual. **Razão:** mudança redundante. Fica como auditoria documentada, não como código.

### D6 — Gamma path não muda

Compositor faz "designer-space blending" em sRGB-as-linear ([compositor.wgsl:1-30](../../crates/ph2d-render/src/shaders/compositor.wgsl#L1-L30)) — escolha intencional M14.5 round 7 para paridade com Figma/browsers. Re-fazer text composite em linear-light é cirurgia de pipeline (novo render target + novo blend pass) e está **fora do escopo** desta Fase 0. Documentado como possível evolução em Crisp Pro (Fase 2+ futura).

### D7 — Menu: cascade sub-group, não toggle direto

Hierarquia: `Settings ▸ Text rendering ▸ Default | Crisp`. **Razão:** consistência com `Settings ▸ Display ▸ VSync | Immediate` e `Settings ▸ Unit ▸ Meters | Pixels` já em produção. Toggle direto ("Crisp Text" checkbox) na raiz quebra simetria do menu.

### D8 — Default é Default (não-breaking)

`TextRendering::Default` é o padrão do enum (`#[default]`). Sessões existentes continuam com aparência idêntica. Crisp é opt-in.

### D9 — Persistência

Não persiste em settings.json **nesta Fase 0**. É um toggle de sessão. Se o usuário gostar, adicionamos persistência em fase de polish separada (precedente: radius_scale também não persiste cross-session, é runtime). **Razão:** evita acoplamento com `ph2d-asset`/save crate que ainda são stub.

### D10 — Sem perguntar — onde a dúvida bate, decide assim:

| Dúvida | Resolução padrão-ouro |
|---|---|
| Bug de compile em fase N quebra coisa de fase N-1? | Reverte só a mudança de N que causou; mantém N-1 intacto; tenta de novo. |
| Teste novo conflita com gate `no_magic_numeric`? | Adiciona const nomeada (`CRISP_WEIGHT_BOOST_BODY: u16 = 60`). Nunca usa `// LITERAL-OK` quando uma const resolve. |
| `cargo clippy --all-targets -- -D warnings` reclama de função pública sem doc? | Adiciona `///` 1 linha. Nunca silencia com `#[allow]`. |
| Em F5/F6, o painter do menu pinta sub-cascade caret automaticamente ou exige flag? | Lê primeiro como o `Settings ▸ Display` pinta o caret (linha exata em `context_menu_overlay.rs`); replica o padrão; se a flag é explícita, seta. Não improvisa visual novo. |
| Em F4, `hero.text_rendering` deve viver no `store` (interaction state) ou direto na struct `HeroScreen`? | Direto na struct, espelhando `hero.theme` ([state.rs](../../crates/ph2d-editor-core/src/screens/hero/state.rs)). `store` é estado de widgets; text_rendering é estado de tema. |
| Onde fica o `set_text_rendering(...)` por frame? | Logo após [hero.rs:607 `set_radius_scale(...)`](../../crates/ph2d-editor-core/src/screens/hero.rs#L607). Idêntico padrão. |
| Conflito com agente paralelo em arquivo (M no `git status` que não é meu)? | PARA. Reporta. Não força. |

---

## Fase 1 — TextRendering enum em ph2d-tokens

**Objetivo:** introduzir o tipo, sem nenhum cliente ainda.

### Implementação

Em [`crates/ph2d-tokens/src/typography.rs`](../../crates/ph2d-tokens/src/typography.rs), após o último enum existente (`LetterSpacing`, ~line 124):

```rust
/// Estratégia de rasterização de texto da UI.
///
/// Ortogonal a [`Theme`] — qualquer combinação Theme × TextRendering
/// é válida. Lida via thread-local [`paint::text_rendering`] dentro
/// de `paint_text*` em `ph2d-editor-core`.
///
/// - `Default` — pipeline histórico: snap-Y por glifo, FontWeight
///   nominal (Medium 500 / Semibold 600), AA analítico Vello.
/// - `Crisp` — pipeline alternativo focado em corpo pequeno (≤14 px):
///   snap-X integral além do snap-Y, boost de weight por faixa de
///   tamanho (vide [`crisp_weight_boost`]), mesma família + opsz
///   axis do path Default. Sem trade-off em legibilidade de texto
///   grande, ganho perceptual significativo em chrome denso.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextRendering {
    /// Pipeline histórico — Vello AA analítico + snap-Y, sem boost.
    #[default]
    Default,
    /// Pipeline alternativo — snap-X + snap-Y + boost de weight em
    /// corpo pequeno. Foco: legibilidade em chrome UI.
    Crisp,
}

impl TextRendering {
    /// Cycle entre as opções (para toggle de menu).
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::Crisp,
            Self::Crisp => Self::Default,
        }
    }

    /// Stable identifier (matches future tokens.json key).
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Crisp => "crisp",
        }
    }

    /// Human-readable display name (menu items).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Crisp => "Crisp",
        }
    }
}

/// Boost de FontWeight aplicado em modo [`TextRendering::Crisp`]
/// como função do tamanho da fonte renderizado. Retorna 0 para
/// tamanhos onde o boost não traz ganho perceptual (>20 px).
///
/// Faixas calibradas para Inter Variable v4.0 sob AA analítico
/// do Vello 0.8. Soma sobre o `FontWeight` nominal e é clampado
/// para [100, 900] pelo skrifa downstream.
pub const fn crisp_weight_boost(font_size_px: f32) -> u16 {
    // Faixas discretas evitam micro-shifts visíveis no variable axis.
    // Valores empíricos: 60/40/20/0 cobrem Xxs..Lg da tabela TypeToken.
    if font_size_px <= CRISP_BOOST_TIER_BODY_MAX {
        CRISP_WEIGHT_BOOST_BODY
    } else if font_size_px <= CRISP_BOOST_TIER_DENSE_MAX {
        CRISP_WEIGHT_BOOST_DENSE
    } else if font_size_px <= CRISP_BOOST_TIER_MID_MAX {
        CRISP_WEIGHT_BOOST_MID
    } else {
        0
    }
}

const CRISP_BOOST_TIER_BODY_MAX: f32 = 12.0;
const CRISP_BOOST_TIER_DENSE_MAX: f32 = 16.0;
const CRISP_BOOST_TIER_MID_MAX: f32 = 20.0;
const CRISP_WEIGHT_BOOST_BODY: u16 = 60;
const CRISP_WEIGHT_BOOST_DENSE: u16 = 40;
const CRISP_WEIGHT_BOOST_MID: u16 = 20;
```

Em [`crates/ph2d-tokens/src/lib.rs`](../../crates/ph2d-tokens/src/lib.rs), exportar:

```rust
pub use typography::{
    /* ...existentes... */
    TextRendering,
    crisp_weight_boost,
};
```

### Testes (no mesmo arquivo, em `#[cfg(test)]`)

```rust
#[test]
fn text_rendering_default_is_default() {
    assert_eq!(TextRendering::default(), TextRendering::Default);
}

#[test]
fn text_rendering_cycles_all() {
    let a = TextRendering::Default;
    let b = a.next();
    let c = b.next();
    assert_eq!(b, TextRendering::Crisp);
    assert_eq!(c, TextRendering::Default);
}

#[test]
fn text_rendering_ids_stable() {
    assert_eq!(TextRendering::Default.id(), "default");
    assert_eq!(TextRendering::Crisp.id(), "crisp");
}

#[test]
fn crisp_weight_boost_monotonically_decreases() {
    let sizes = [10.0, 11.0, 12.0, 13.0, 15.0, 17.0, 19.0, 22.0, 32.0];
    let mut prev = u16::MAX;
    for s in sizes {
        let b = crisp_weight_boost(s);
        assert!(b <= prev, "boost must be monotonic non-increasing in size; size={s} prev={prev} curr={b}");
        prev = b;
    }
}

#[test]
fn crisp_weight_boost_typetoken_coverage() {
    // Validate against the actual TypeToken px values from generated
    // constants — Xxs (10) and Xs (11) get max boost; Xl (24) gets 0.
    assert_eq!(crisp_weight_boost(TypeToken::Xxs.px()), CRISP_WEIGHT_BOOST_BODY);
    assert_eq!(crisp_weight_boost(TypeToken::Xs.px()), CRISP_WEIGHT_BOOST_BODY);
    assert_eq!(crisp_weight_boost(TypeToken::Sm.px()), CRISP_WEIGHT_BOOST_BODY);
    assert_eq!(crisp_weight_boost(TypeToken::Base.px()), CRISP_WEIGHT_BOOST_DENSE);
    assert_eq!(crisp_weight_boost(TypeToken::Lg.px()), CRISP_WEIGHT_BOOST_MID);
    assert_eq!(crisp_weight_boost(TypeToken::Xl.px()), 0);
}
```

### Auditoria — loop até erro zero

```bash
cargo check -p ph2d-tokens
cargo test  -p ph2d-tokens
cargo clippy -p ph2d-tokens --all-targets -- -D warnings
cargo fmt    -p ph2d-tokens
```

**Critério:** os 4 comandos verdes. Loop: se algum vermelho, corrige a causa raiz no mesmo arquivo e re-roda os 4. Não avança até zero.

---

## Fase 2 — TextSystem ganha API de rendering

**Objetivo:** expor variant `layout_for_rendering(text, size, max_w, weight, rendering)` em [`TextSystem`](../../crates/ph2d-text/src/system.rs) que recebe `TextRendering` e aplica o boost de weight ANTES de pedir o layout pra parley.

### Por que aqui e não em paint.rs

O boost de weight afeta o **shaping** (advance widths mudam com weight diferente), então precisa entrar **antes** do parley layout, não depois. Aplicar em paint.rs após `text_system.layout_with_weight(...)` resultaria em glyphs de weight novo posicionados pelos advance widths do weight antigo — visualmente quebrado em rendering denso.

### Implementação

Em [`crates/ph2d-text/src/system.rs`](../../crates/ph2d-text/src/system.rs), adicionar API nova SEM tocar nas existentes (back-compat):

```rust
impl TextSystem {
    /// Como [`Self::layout_with_weight`], mas aplica o boost de
    /// [`TextRendering::Crisp`] no weight antes de pedir layout
    /// ao parley. Em modo `Default` é idêntico a
    /// `layout_with_weight(text, size, max_width, weight_nominal)`.
    pub fn layout_for_rendering(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: f32,
        weight_nominal: FontWeight,
        rendering: ph2d_tokens::TextRendering,
    ) -> Layout<()> {
        let weight_effective = effective_weight(weight_nominal, font_size, rendering);
        self.layout_with_weight(text, font_size, max_width, weight_effective)
    }
}

/// Computa o `FontWeight` efetivo dado o nominal + tamanho +
/// estratégia. Em `Default` retorna o nominal sem alteração.
/// Em `Crisp`, soma o boost por faixa (vide `crisp_weight_boost`)
/// e clampa para o intervalo válido [100, 900].
fn effective_weight(
    nominal: FontWeight,
    font_size: f32,
    rendering: ph2d_tokens::TextRendering,
) -> FontWeight {
    match rendering {
        ph2d_tokens::TextRendering::Default => nominal,
        ph2d_tokens::TextRendering::Crisp => {
            let boost = ph2d_tokens::crisp_weight_boost(font_size);
            if boost == 0 {
                return nominal;
            }
            // FontWeight value() é f32 — boost soma com clamp.
            let raw = nominal.value() + boost as f32;
            let clamped = raw.clamp(WEIGHT_MIN, WEIGHT_MAX);
            FontWeight::new(clamped)
        }
    }
}

const WEIGHT_MIN: f32 = 100.0;
const WEIGHT_MAX: f32 = 900.0;
```

Nota sobre `FontWeight::new`: parley re-exporta o tipo; verifica se `new(f32)` é construtor público ou se é via `FontWeight::from(...)`. Usa o que existe; se nenhum, importa direto e constrói via API pública.

### Testes (no `#[cfg(test)] mod tests`)

```rust
#[test]
fn rendering_default_is_passthrough_weight() {
    let mut sys = TextSystem::without_system_fonts();
    let layout_a = sys.layout_with_weight("Hello", 11.0, f32::INFINITY, FontWeight::MEDIUM);
    let layout_b = sys.layout_for_rendering(
        "Hello",
        11.0,
        f32::INFINITY,
        FontWeight::MEDIUM,
        ph2d_tokens::TextRendering::Default,
    );
    // Layouts são equivalentes geometricamente.
    assert_eq!(layout_a.width(), layout_b.width());
    assert_eq!(layout_a.height(), layout_b.height());
}

#[test]
fn rendering_crisp_bumps_weight_in_body() {
    let mut sys = TextSystem::without_system_fonts();
    let a = sys.layout_with_weight("WWWWWW", 11.0, f32::INFINITY, FontWeight::MEDIUM);
    let b = sys.layout_for_rendering(
        "WWWWWW",
        11.0,
        f32::INFINITY,
        FontWeight::MEDIUM,
        ph2d_tokens::TextRendering::Crisp,
    );
    // Weight boost aumenta advance width em Inter (heavier strokes).
    // Em 11 px Inter Variable, 6× W com Medium 500 vs ~Semibold 560:
    // expect b.width() > a.width().
    assert!(
        b.width() > a.width(),
        "crisp layout should be wider due to weight bump; default={} crisp={}",
        a.width(),
        b.width()
    );
}

#[test]
fn rendering_crisp_no_op_at_large_size() {
    let mut sys = TextSystem::without_system_fonts();
    let a = sys.layout_with_weight("Hello", 32.0, f32::INFINITY, FontWeight::MEDIUM);
    let b = sys.layout_for_rendering(
        "Hello",
        32.0,
        f32::INFINITY,
        FontWeight::MEDIUM,
        ph2d_tokens::TextRendering::Crisp,
    );
    // Acima de 20 px, boost = 0, layouts idênticos.
    assert_eq!(a.width(), b.width());
}
```

### Cache: ainda válido

`LayoutCacheKey` já inclui `weight_bits` ([system.rs:55-60](../../crates/ph2d-text/src/system.rs#L55-L60)). Crisp e Default geram chaves distintas naturalmente — mesma string em ambos os modos gera 2 entradas separadas. Cap de 1024 entries continua suficiente. Nada a mudar.

### Auditoria — loop até erro zero

```bash
cargo check -p ph2d-text
cargo test  -p ph2d-text
cargo clippy -p ph2d-text --all-targets -- -D warnings
cargo fmt    -p ph2d-text
```

---

## Fase 3 — paint.rs ganha thread-local + branch em paint_text_weighted

**Objetivo:** acrescentar o switch global e fazer o `paint_text_weighted` ler ele para aplicar snap-X + weight bump.

### Implementação

Em [`crates/ph2d-editor-core/src/paint.rs`](../../crates/ph2d-editor-core/src/paint.rs), adicionar logo após a seção do `radius_scale` (após line 89):

```rust
// Thread-local global text-rendering strategy. Lets the user pick
// Default / Crisp in the Settings ▸ Text rendering submenu and have
// every `paint_text*` call (99+ sites across panels and chrome)
// apply the strategy uniformly without threading it through every
// signature. Set via `set_text_rendering` before paint, read via
// `text_rendering`. Defaults to `Default` (historic look).
thread_local! {
    static TEXT_RENDERING: std::cell::Cell<ph2d_tokens::TextRendering> =
        const { std::cell::Cell::new(ph2d_tokens::TextRendering::Default) };
}

/// Set the global text-rendering strategy for the current thread
/// (paint runs). Called by the shell once per frame, mirroring
/// `set_radius_scale`.
pub fn set_text_rendering(mode: ph2d_tokens::TextRendering) {
    TEXT_RENDERING.with(|c| c.set(mode));
}

/// Read the current text-rendering strategy. Default is
/// `TextRendering::Default` so any non-paint context (tests,
/// scrambled call orders) gets historic look.
pub fn text_rendering() -> ph2d_tokens::TextRendering {
    TEXT_RENDERING.with(|c| c.get())
}
```

Em `paint_text_weighted` (paint.rs:272-324), mudar para:

```rust
#[allow(clippy::too_many_arguments)]
fn paint_text_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) {
    let rendering = text_rendering();
    let layout = text_system.layout_for_rendering(text, font_size, max_width, weight, rendering);
    let inner = scene.inner_mut();
    let translate = Affine::translate((x.round() as f64, y.round() as f64));
    let snap_x = matches!(rendering, ph2d_tokens::TextRendering::Crisp);
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                .hint(true)
                .brush(color)
                .transform(translate)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        // Snap X to pixel grid in Crisp mode for crisp
                        // stems at the cost of perfect kerning; keep
                        // fractional in Default to preserve parley's
                        // subtleties.
                        x: if snap_x { g.x.round() } else { g.x },
                        y: g.y.round(),
                    }),
                );
        }
    }
}
```

E `paint_text_rotated_ccw` (paint.rs:335+) também — replica o mesmo branch interno (mas posicionamento rotacionado: o snap-X equivalente é nas coordenadas pré-rotação, então mesma lógica funciona).

### Testes inline (em `#[cfg(test)] mod tests` do paint.rs)

```rust
#[test]
fn text_rendering_default_thread_local() {
    set_text_rendering(ph2d_tokens::TextRendering::Default);
    assert_eq!(text_rendering(), ph2d_tokens::TextRendering::Default);
    set_text_rendering(ph2d_tokens::TextRendering::Crisp);
    assert_eq!(text_rendering(), ph2d_tokens::TextRendering::Crisp);
    // Reset para não vazar pros outros tests.
    set_text_rendering(ph2d_tokens::TextRendering::Default);
}
```

Teste **visual** (não-automatizado) fica na lista §9.

### Auditoria — loop até erro zero

```bash
cargo check -p ph2d-editor-core
cargo test  -p ph2d-editor-core
cargo clippy -p ph2d-editor-core --all-targets -- -D warnings
cargo fmt    -p ph2d-editor-core
```

**Cuidado especial:** este crate é grande (1300+ testes via memória). Não panique se primeiro `check` demorar 30-60s — é normal warm-up; subsequentes serão rápidos.

---

## Fase 4 — HeroScreen ganha campo + chamada por frame

**Objetivo:** persistir o estado no `HeroScreen` e aplicar antes do paint.

### Implementação

Localizar `HeroScreen` struct definition. Provavelmente em [`crates/ph2d-editor-core/src/screens/hero/state.rs`](../../crates/ph2d-editor-core/src/screens/hero/state.rs) (verificar — grep `pub struct HeroScreen`). Adicionar campo após `pub theme: Theme`:

```rust
pub struct HeroScreen {
    // ...campos existentes...
    pub theme: Theme,
    /// Text rendering strategy — ortogonal a `theme`. Default
    /// preserva visual histórico; Crisp aplica snap-X + weight
    /// boost por tamanho. Persistência: runtime-only (não save).
    pub text_rendering: ph2d_tokens::TextRendering,
    // ...
}
```

No construtor / `Default::default()` de `HeroScreen`, inicializa com `ph2d_tokens::TextRendering::default()` (ou apenas `Default::default()` se a struct tem `#[derive(Default)]`).

Em [`hero.rs:607`](../../crates/ph2d-editor-core/src/screens/hero.rs#L607) (onde está `set_radius_scale`), adicionar a linha simétrica:

```rust
crate::paint::set_radius_scale(hero.store.radius_scale());
crate::paint::set_text_rendering(hero.text_rendering);
```

### Auditoria — loop até erro zero

```bash
cargo check -p ph2d-editor-core
cargo test  -p ph2d-editor-core
cargo clippy -p ph2d-editor-core --all-targets -- -D warnings
cargo fmt    -p ph2d-editor-core
```

---

## Fase 5 — Menu: IDs + chrome handler + populate

**Objetivo:** novos itens visíveis no menu `Settings ▸ Text rendering ▸ Default | Crisp` e clicáveis.

### F5.1 — IDs

Em [`crates/ph2d-editor-core/src/ids.rs`](../../crates/ph2d-editor-core/src/ids.rs), depois das `CTX_MENU_DISPLAY_*` (~line 600), adicionar:

```rust
/// "Text rendering" cascade root — opens the submenu with the
/// `Default` / `Crisp` choices.
pub const CTX_MENU_SETTINGS_TEXT: NodeId = hash_node_id("ctx_menu_settings_text");
pub const CTX_MENU_TEXT_DEFAULT: NodeId = hash_node_id("ctx_menu_text_default");
pub const CTX_MENU_TEXT_CRISP: NodeId = hash_node_id("ctx_menu_text_crisp");
```

### F5.2 — Pre-populate

Em [`pre_populate.rs:406-459`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs#L406-L459), adicionar os 3 IDs na lista:

```rust
ids::CTX_MENU_SETTINGS_TEXT,
ids::CTX_MENU_TEXT_DEFAULT,
ids::CTX_MENU_TEXT_CRISP,
```

(Ordem visual no array é livre; preferir adjacente aos `CTX_MENU_SETTINGS_DISPLAY` / `CTX_MENU_DISPLAY_*` para manter agrupamento setting-related visualmente próximo no source.)

### F5.3 — Chrome handler novo

Criar [`crates/ph2d-editor-core/src/screens/hero/chrome/settings_text.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/settings_text.rs):

```rust
//! Text-rendering submenu — Default / Crisp.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use crate::screens::hero::chrome::cascade_anchor;
use ph2d_tokens::TextRendering;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_TEXT {
        // Cascade open: anchor at the parent row, populate the submenu
        // with the 2 choices.
        let (x, y) = cascade_anchor(hero, id);
        hero.store
            .open_cascade_text_rendering_at(x, y);
        return true;
    }
    let chosen = if id == ids::CTX_MENU_TEXT_DEFAULT {
        TextRendering::Default
    } else if id == ids::CTX_MENU_TEXT_CRISP {
        TextRendering::Crisp
    } else {
        return false;
    };
    hero.text_rendering = chosen;
    hero.store.close_context_menu();
    true
}
```

Nota crítica: o método `open_cascade_text_rendering_at` provavelmente NÃO existe. Padrão a seguir: replicar o que `chrome/settings_unit.rs` ou `chrome/settings_ppm.rs` faz para abrir cascade — investigar primeiro, replicar exato. **Decisão padrão-ouro:** se existe um método genérico (`open_cascade_at(submenu_kind, x, y)`), usa; se a convenção é um método por submenu, cria um novo método em `store` chamado `open_cascade_text_rendering_at(x, y)` espelhando o existente. Não inventa shape novo.

Em [`crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs):

- Linha `mod settings_text;` dentro do bloco `<ph2d-chrome-sync:begin>...end>` (ordem alfabética). **Heads-up:** esse bloco é codegenado — verificar se sync regenera ou se a edição é manual; o comentário diz "regenerated by `cargo run -p ph2d-chrome-sync`". Se for codegenado, rodar `cargo run -p ph2d-chrome-sync` após criar o arquivo.
- Linha `|| settings_text::apply(hero, event)` em `dispatch_all`, na ordem que faça sentido (alfabética ou agrupada com outros `settings_*`).

### F5.4 — Painter do menu (cascade overlay)

Localizar onde o `CTX_MENU_SETTINGS_DISPLAY` é pintado e sua submenu (`CTX_MENU_DISPLAY_VSYNC` / `_IMMEDIATE`). Provavelmente em `crates/ph2d-editor-core/src/screens/hero/context_menu_overlay.rs` ou em um painter sibling. Replicar o padrão para `CTX_MENU_SETTINGS_TEXT` com seu submenu Default/Crisp.

**Decisão padrão-ouro:** ler o caso `Display` (cascade root) inteiro antes de tocar. Replicar 1:1 com substituições de nome. Sem improvisar visual diferente. Se o painter usa um helper `paint_cascade_row(id, label, has_submenu)`, chamar com `(CTX_MENU_SETTINGS_TEXT, "Text rendering", true)`. Se usa `paint_menu_item(id, label, is_checked)`, chamar com check apontando para qual variant está ativa (Default ou Crisp).

**Check visual no submenu:** o item ativo (Default ou Crisp) deve receber check (✓) ou highlight, igual `Display ▸ VSync/Immediate` faz com o vsync ativo. Ler o que aquele submenu faz — replicar.

### Auditoria — loop até erro zero

```bash
cargo run  -p ph2d-chrome-sync     # se aplicável (ver F5.3 nota)
cargo check -p ph2d-editor-core
cargo test  -p ph2d-editor-core
cargo clippy -p ph2d-editor-core --all-targets -- -D warnings
cargo fmt    -p ph2d-editor-core
```

**Cuidado especial:** se a F5.3 nota se confirmar (sync codegena), o staleness gate `architecture_chrome_sync_staleness` (se existir) pode quebrar se não rodar o sync. Sempre re-rodar `ph2d-chrome-sync` após adicionar/remover arquivo em `chrome/`.

---

## Fase 6 — Sanity geral

**Objetivo:** garantir que mudanças foundational não quebraram crates dependentes.

```bash
cargo check --workspace
cargo test  --workspace --exclude ph2d-asset
```

(O exclude de `ph2d-asset` é convenção do projeto — testes desse crate exigem fixtures pesados não-essenciais para este loop.)

**Auditoria — loop até erro zero**: se algum crate vermelhar, identifica + corrige o crate específico. **Não** rebaixa pra `--ignored` ou silencia teste. Se identificar conflito com agente paralelo (M num arquivo que não é meu), pausa e reporta ao Enio.

---

## Fase 7 — Verificação independente de gamma (documental)

**Objetivo:** confirmar que a auditoria de gamma já foi feita, registrar inline no plano que está OK, e NÃO mudar nada do pipeline gráfico.

### Evidências já coletadas

1. [`shaders/compositor.wgsl:1-30`](../../crates/ph2d-render/src/shaders/compositor.wgsl#L1-L30) — comentário canônico: "Straight-alpha 'over' (Porter-Duff) in **sRGB-as-linear** (designer) space, NOT scene-linear. This matches Figma, browsers, and legacy 2D engines".
2. [`vello_pass.rs:127-131`](../../crates/ph2d-render/src/vello_pass.rs#L127-L131) — comentário: "The earlier 'Area looks low-rez' note pre-dated the gamma-correct compositor + glyph-snap fixes; in the current pipeline Area wins."
3. M14.5 round 7 (sprint anterior) fechou explicitamente gamma + glyph-snap correções.

### Conclusão

Gamma path está correto pela escolha de design (paridade Figma). Crisp mode **não toca** compositor. Eventual evolução pra "Crisp Pro" (linear-light text composite com sub-render-target dedicado) fica como Fase 2 futura, fora deste plano.

**Nada a implementar nesta fase.** Apenas registro.

---

## Fase 8 — Sanity de ship.sh (paridade-CI local, sem push)

**Objetivo:** garantir que o conjunto inteiro passa pelo gate de CI ANTES de qualquer commit eventual.

```bash
./scripts/ship.sh
```

**Auditoria — loop até erro zero:** corrige cada `✗` na ordem em que aparece. NÃO pula nenhum. Se algo escapa do escopo das 7 fases (ex.: drift de fmt em outro crate por agente paralelo), reporta ao Enio — é caso fast-mode legítimo, mas não esconde com `--no-verify`.

**Não pusha.** O plano termina aqui. Coord-A externa decide commit + push.

---

## §9 — Lista de testes visuais para smoke (após F8)

Quando o Enio rodar `./play.command`, validar:

### 9.1 — Paridade Default
1. App abre no tema **Forge** (default).
2. **Inspector** com label "TRANSFORM", subtítulos "Position (px)" "Rotation (°)" — visualmente idêntico ao baseline (commit `a9718c5`). Sem mudança perceptível.
3. **Hierarchy** com nomes de entidades — idêntico ao baseline.
4. **TopBar** com pills + chips — idêntico.
5. **Widget Gallery** scrolla com todas as seções — idêntico.

**Critério:** indistinguível de antes do PR. Se algo mudou visualmente em Default, é regressão.

### 9.2 — Menu novo
1. Clicar em **Settings** (engrenagem ou cluster equivalente na TopBar) → context menu abre.
2. Procurar item **"Text rendering ▸"** (com caret cascade).
3. Hover/click → submenu abre.
4. 2 items visíveis: **"Default"** e **"Crisp"**.
5. Item ativo (Default no primeiro abrir) tem indicador visual (check/highlight) — mesmo padrão de `Display ▸ VSync/Immediate`.

**Critério:** menu funciona idêntico aos outros cascade groups (Display/Unit/Filter). Mesma posição de cascade, mesma direção (right-cascade com fallback left perto da borda).

### 9.3 — Toggle Default ↔ Crisp
1. Estado: Default. Olha Hierarchy (nomes em ~11px).
2. Click `Settings ▸ Text rendering ▸ Crisp`.
3. **Sem reload, sem flicker**: Hierarchy redesenha com tipo mais "encorpado" — strokes mais grossos perceptualmente.
4. Inspector também muda (labels de campos ~12px).
5. Títulos grandes (TopBar, Widget Gallery section heads ~18-24px) — **não** mudam visualmente (boost = 0 acima de 20px).
6. Click `Settings ▸ Text rendering ▸ Default` — volta exato ao estado 9.1.

**Critério:** corpo pequeno fica visivelmente mais legível em Crisp; texto grande fica idêntico. Toggle é instantâneo (≤1 frame).

### 9.4 — Cross-theme
1. Em Crisp, trocar Theme: Forge → Workshop → Sunstone → Blueprint → Forge.
2. Crisp persiste entre theme switches.
3. Cada theme em Crisp tem texto perceptualmente mais nítido que em Default.

**Critério:** TextRendering é ortogonal a Theme — quaisquer 4×2 = 8 combinações funcionam.

### 9.5 — Texto rotacionado (left rail labels)
1. Em Crisp, olhar labels rotacionadas verticalmente no LeftRail (sub-labels de botões — texto rodado 90° CCW).
2. Visualmente: aceitável (não tofu, não embaçado catastroficamente).

**Critério:** não regressão. Rotação aplica o mesmo snap pré-rotação.

### 9.6 — Performance
1. Em Crisp, scrolla Hierarchy (3000+ entries se houver fixture). Sem stutter perceptível.
2. Toggle Crisp/Default repetido 5× — sem leak visual ou memória crescente.

**Critério:** layout cache (perfil de hit-rate 96.8%) continua eficiente. Crisp e Default geram chaves distintas mas ambos cacheados.

### 9.7 — Texto de Image Tools panel (Inspector com tool ativo)
1. Em Crisp, ativar BgRemoval. Painel docado em Inspector.
2. Labels de sliders + valores chip — mais legíveis em corpo 11-12 px.
3. Sliders e drag funcionam normalmente.

**Critério:** ferramentas Image Tools funcionalmente intactas; visual mais nítido.

### 9.8 — Texto editável (TextInput / NumberInput)
1. Em Crisp, focar o NumberInput "Y" do Inspector com "-10.626".
2. Digitar "1" — caret avança, valor muda.
3. O texto digitado renderiza em Crisp também.

**Critério:** caret + selection + composição mantêm posicionamento correto. Snap-X não desalinha caret. (Edge case: caret usa `prefix_width` — pode precisar revisita se snap-X causar drift; vide §10.2.)

### 9.9 — Welcome screen / placeholder texts
1. Em Crisp, abrir uma cena vazia. Welcome hero (~44px) — sem mudança.
2. Sub-texts (~13-15px) — mais nítidos.

**Critério:** consistência ao longo da hierarquia tipográfica.

---

## §10 — Riscos conhecidos + mitigações

### 10.1 — Boost de weight muda métricas → quebra alinhamento de chips/sliders?

**Risco:** advance widths aumentam em Crisp. Componentes com layout fixo baseado em "largura do texto" podem ter chip overflow ou padding errado.

**Mitigação:** o layout é re-medido por frame (parley devolve novo `width()` no cache miss). Componentes que respeitam o `layout.width()` ajustam automaticamente. Risco real só em código que assume largura fixa hardcoded — improvável no projeto (todo Inspector usa `text_system.layout(...)`-then-measure).

**Plan B se aparecer:** ajuste do boost de 60 → 40 para Body tier. Re-roda smoke.

### 10.2 — Caret/cursor em TextInput desalinha em Crisp (snap-X)

**Risco:** `prefix_width` ([system.rs:247-263](../../crates/ph2d-text/src/system.rs#L247-L263)) retorna f32 baseado em layout não-snapado. Se paint snap o glyph e o caret é posicionado em `prefix_width` fracionário, caret aparece 1px à esquerda/direita da próxima letra.

**Mitigação:** `prefix_width` em Crisp pode ter que ser `.round()` antes de usar como posição de caret. Verificar smoke (§9.8); se aparecer, fix é localizado em `paint_caret` (no widget input — buscar onde caret é renderizado).

**Decisão padrão-ouro se aparecer:** add helper `prefix_width_snapped()` no `TextSystem` que faz round; caller usa em Crisp; mantém float em Default.

### 10.3 — Cascade staleness com chrome-sync codegen

**Risco:** se `chrome/mod.rs` é codegenado e esquecemos de `cargo run -p ph2d-chrome-sync`, o staleness gate quebra CI.

**Mitigação:** F5.3 explicitamente menciona rodar o sync. Hooks de pre-commit do projeto também devem pegar; ship.sh confirma.

### 10.4 — Conflito de paralelismo (M no `git status` que não é meu)

**Risco:** outro agente paralelo pode estar tocando `paint.rs` ou `hero.rs`.

**Mitigação:** antes de cada fase, `git status`. Se M aparecer em arquivo que não é meu, PARA, reporta ao Enio. (Padrão DIRETRIZ §7.1.)

### 10.5 — `FontWeight::new(f32)` não existir como construtor público

**Risco:** parley pode expor apenas constantes (`MEDIUM`, `SEMI_BOLD`) e não construtor por valor.

**Mitigação:** verificar `parley::FontWeight` docs no momento da F2. Alternativa: usar `From<u16>` ou método similar; se nenhum, interpolar manualmente para o `FontWeight` mais próximo (MEDIUM/SEMI_BOLD/BOLD) — perde precisão mas funciona.

**Decisão padrão-ouro se acontecer:** discreto > interpolado. Pula para `SEMI_BOLD` quando boost ≥ 50, para `BOLD` quando ≥ 150. Documenta no `effective_weight` why.

---

## §11 — Critério de "DONE"

Plano fecha quando TODOS os checks abaixo são `✓`:

- [ ] F1: ph2d-tokens check+test+clippy+fmt verdes
- [ ] F2: ph2d-text check+test+clippy+fmt verdes
- [ ] F3: ph2d-editor-core check+test+clippy+fmt verdes (post-paint.rs)
- [ ] F4: ph2d-editor-core check+test verdes (post-HeroScreen field)
- [ ] F5: ph2d-editor-core check+test verdes (post-menu + sync rodado se aplicável)
- [ ] F6: workspace check + test (exclude ph2d-asset) verde
- [ ] F7: registro de gamma audit (sem código novo) — só esta linha do checklist
- [ ] F8: `./scripts/ship.sh` verde fim-a-fim
- [ ] §9 testes visuais (smoke) — Enio confere e aprova

Sem commits, sem push. Coord-A externa orquestra entrega.

---

## §12 — Após DONE (fora do escopo deste plano)

Possíveis Fases futuras (registrar como follow-ups, não executar agora):

- **Persistência cross-session:** salvar `text_rendering` em settings.json quando `ph2d-asset` save infrastructure madurar.
- **Crisp Pro:** sub-render-target dedicado para texto, linear-light compositing, LCD subpixel AA opcional (requer custom wgpu pass — cirurgia maior).
- **Per-panel override:** alguns painéis (Welcome, Hero) podem querer forçar Default mesmo com Crisp global. Adicionar override por panel via `PaintCtx`.
- **i18n labels:** "Text rendering" / "Default" / "Crisp" entram em Fluent bundle quando i18n sweep rodar (HR-15 hoje aspiracional).

---

**Fim do plano.**
