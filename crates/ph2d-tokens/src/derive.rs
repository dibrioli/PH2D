//! ⭐⭐⭐ **A DERIVAÇÃO — um tema moderno nasce de CINCO entradas, e nenhum slot é escrito à mão.**
//!
//! Porte das regras de cor do tema *Modern* do Godot 4.6 (`editor/themes/theme_modern.cpp`,
//! `populate_shared_styles` + `_get_base_color` — **MIT**, vendorizado em
//! `docs/UI_New_and_Simple/referencias/godot-editor-src/`). A decisão do Enio (2026-09-04) está
//! em [`crate::theme`]; a pesquisa que a fundamenta em `pesquisa/08_modelos_com_codigo_para_seguir.md`.
//!
//! # As regras, uma a uma (as do Godot, com o nome dele ao lado)
//!
//! | papel | como nasce |
//! |---|---|
//! | `mono` | branco num tema escuro, preto num claro — *«will be used to generate the rest»* |
//! | `dark_color_1` | `base.lerp(preto, contraste · 1,15)` |
//! | `dark_color_3` | `base` com **V** de HSV puxado para 0 por `contraste · 0,8` e **S** × `0,9` |
//! | `contrast_color_1` | `base.lerp(mono, max(contraste, 0,3) · 1,15)` |
//! | `contrast_color_2` | `base.lerp(mono, max(contraste, 0,3) · 1,725)` |
//! | `highlight` | `acento @ 0,275` |
//! | `font` · `font_secondary` · `font_disabled` | `mono @ 0,75` · `@ 0,55` · `@ 0,35` |
//! | `info` · `success` · `warning` · `error` | fixos: `(0,7 0,8 1)` · `(0,45 0,95 0,5)` · `(0,83 0,78 0,62)` · `(1 0,47 0,42)` — e escurecidos num tema claro, como lá |
//! | `extra_border_1/2` | `mono @ 0,4` · `@ 0,2` — só com *Draw Extra Borders* (o preset OLED) |
//!
//! ⚠️ **O `lerp` é o do Godot: componente a componente em sRGB, não em luz linear.** Portar
//! «melhor» daria outras cores, e o que se quer é *as dele*.
//!
//! # ⚠️ ACHATAR o alfa é decisão deste porte, não do Godot
//!
//! O Godot escreve `font_color = mono × (1,1,1,0,75)` e deixa o alfa viajar até ao pintor. Aqui
//! os slots que hoje são **opacos** no `tokens.json` continuam opacos: a cor com alfa é
//! **composta sobre a base** na derivação (`over`). Duas razões, e nenhuma é gosto: o gate de
//! contraste ([`crate::contrast`]) mede a cor do token e não a compõe, e há pintores que
//! constroem o `Color` do Vello a partir dos três canais. Os slots que hoje carregam alfa
//! (`bg-scrim`, `rail-bg`, `focus-ring`, `grid-*`, `graph-grid`, `graph-backdrop-*`,
//! `graph-marquee`, `graph-inert`) continuam a carregá-lo.
//!
//! # ⚠️ As cores de DADO não se derivam — emprestam-se
//!
//! `node-cat-*`, `port-*`, `curve-*`, `wire-fire-glow`, `graph-backdrop-*`, `attr-write`: são as
//! únicas com matiz por direito (a lei que os quatro modelos planos partilham — `pesquisa/08 §3`),
//! e já estão calibradas em OKLCH *dark-safe* no `tokens.json`. Um tema moderno escuro lê-as da
//! tabela do `forge`, um claro da do `sunstone`. ⭐ Os eixos `axis-x/y/z` são os do Godot.

use crate::color::{Color, ColorToken};
use crate::theme::Theme;

/// As cinco entradas de um tema moderno (as três de cor aqui; raio e espaçamento vivem em
/// [`crate::visuals::Chrome`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Inputs {
    /// `interface/theme/base_color`.
    pub base: Rgb,
    /// `interface/theme/accent_color`.
    pub accent: Rgb,
    /// `interface/theme/contrast` — negativo num tema claro (a «elevação» escurece).
    pub contrast: f32,
    /// Tema escuro ⇒ `mono` é branco.
    pub dark: bool,
    /// `interface/theme/draw_extra_borders` — o preset OLED liga-o, porque num fundo preto o
    /// contraste já não separa nada.
    pub extra_borders: bool,
}

/// Uma cor sRGB em `0..1`, o espaço em que o Godot deriva.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// `Color::lerp` do Godot — componente a componente.
    #[must_use]
    pub fn lerp(self, to: Self, t: f32) -> Self {
        Self::new(
            self.r + (to.r - self.r) * t,
            self.g + (to.g - self.g) * t,
            self.b + (to.b - self.b) * t,
        )
        .clamp()
    }

    #[must_use]
    fn clamp(self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    /// `self` com alfa `a`, **composto sobre** `under` — o achatamento descrito no topo.
    #[must_use]
    pub fn over(self, under: Self, a: f32) -> Self {
        under.lerp(self, a)
    }

    /// `_get_base_color` do Godot: V de HSV puxado para 0 por `contrast · dim`, S multiplicado.
    #[must_use]
    pub fn dimmed(self, contrast: f32, dim: f32, sat_mult: f32) -> Self {
        let final_contrast = if dim < 0.0 {
            contrast.clamp(-0.1, 0.5)
        } else {
            contrast
        };
        let (h, s, v) = self.to_hsv();
        let v = (v + (0.0 - v) * (final_contrast * dim)).clamp(0.0, 1.0);
        Self::from_hsv(h, s * sat_mult, v)
    }

    fn to_hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let v = max;
        let s = if max > 0.0 { delta / max } else { 0.0 };
        let h = if delta <= f32::EPSILON {
            0.0
        } else if (max - self.r).abs() <= f32::EPSILON {
            ((self.g - self.b) / delta).rem_euclid(6.0)
        } else if (max - self.g).abs() <= f32::EPSILON {
            (self.b - self.r) / delta + 2.0
        } else {
            (self.r - self.g) / delta + 4.0
        } / 6.0;
        (h, s, v)
    }

    fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let s = s.clamp(0.0, 1.0);
        if s <= 0.0 {
            return Self::new(v, v, v);
        }
        let h6 = (h.rem_euclid(1.0)) * 6.0;
        let i = h6.floor();
        let f = h6 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        match i as i32 {
            0 => Self::new(v, t, p),
            1 => Self::new(q, v, p),
            2 => Self::new(p, v, t),
            3 => Self::new(p, q, v),
            4 => Self::new(t, p, v),
            _ => Self::new(v, p, q),
        }
    }

    /// Opaca.
    #[must_use]
    pub fn color(self) -> Color {
        self.with_alpha(1.0)
    }

    /// Com o alfa que o slot carrega hoje.
    #[must_use]
    pub fn with_alpha(self, a: f32) -> Color {
        let c = self.clamp();
        Color {
            r: (c.r * 255.0).round() as u8,
            g: (c.g * 255.0).round() as u8,
            b: (c.b * 255.0).round() as u8,
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }
}

/// O `default_contrast` do Godot — o piso das duas cores de contraste.
const DEFAULT_CONTRAST: f32 = 0.3;

/// A razão de contraste da WCAG 2.2 entre duas cores opacas — a régua da lei de contraste da casa
/// (`contrast.rs`), usada aqui para DERIVAR o que a lei exige em vez de o escrever à mão.
fn wcag_ratio(a: Rgb, b: Rgb) -> f64 {
    let la = a.color().relative_luminance();
    let lb = b.color().relative_luminance();
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Os papéis DERIVADOS — a tabela intermédia entre as cinco entradas e os ~83 slots.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Roles {
    pub base: Rgb,
    pub accent: Rgb,
    pub mono: Rgb,
    pub mono_inv: Rgb,
    pub dark_1: Rgb,
    /// ⭐⭐ **A FAMÍLIA `surface_*` do Godot Modern** (`_get_base_color(ofs, sat)`,
    /// `theme_modern.cpp:191-203`): ofs **positivo** escurece (poços · popups · fundo das abas),
    /// **negativo** clareia (cartões · botões). É a pilha real do editor dele: o `PanelContainer`
    /// é a `base`, um cartão (a faixa de um bus de áudio) é `surface_high`, um botão em repouso é
    /// `button_normal`, o `LineEdit` é `surface_lower`, o fundo do `GraphEdit` é `surface_lowest`.
    /// ⚠️ A wave 1 tinha posto o painel em `dark_1` e os cartões em `dark_3` — 4/255 entre os dois
    /// (report do Enio, 2026-09-05).
    pub surface_lowest: Rgb,
    pub surface_lower: Rgb,
    pub surface_low: Rgb,
    pub surface_high: Rgb,
    pub surface_higher: Rgb,
    pub surface_highest: Rgb,
    pub surface_popup: Rgb,
    pub button_normal: Rgb,
    pub button_hover: Rgb,
    pub button_pressed: Rgb,
    pub button_disabled: Rgb,
    pub contrast_1: Rgb,
    pub contrast_2: Rgb,
    pub font: Rgb,
    pub font_secondary: Rgb,
    pub font_hover: Rgb,
    pub font_disabled: Rgb,
    pub info: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub error: Rgb,
    /// A alfa das bordas extra (0 sem *Draw Extra Borders*).
    pub extra_border_a: f32,
    pub contrast: f32,
}

impl Inputs {
    /// As entradas de um tema moderno; `None` para a família clássica.
    #[must_use]
    pub fn of(theme: Theme) -> Option<Self> {
        // ⚠️ Os quatro vêm da tabela `color_preset` do Godot 4.6 (MIT), literalmente — nenhuma
        // cor nova. `Default` é o preset que o editor aplica a uma instalação nova.
        Some(match theme {
            Theme::Dark => Self {
                base: Rgb::new(0.161, 0.161, 0.161),
                accent: Rgb::new(0.337, 0.62, 1.0),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            Theme::Gray => Self {
                base: Rgb::new(0.24, 0.24, 0.24),
                accent: Rgb::new(0.44, 0.73, 0.98),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            Theme::Light => Self {
                base: Rgb::new(0.9, 0.9, 0.9),
                accent: Rgb::new(0.18, 0.50, 1.00),
                contrast: -0.06,
                dark: false,
                extra_borders: false,
            },
            Theme::Oled => Self {
                base: Rgb::BLACK,
                accent: Rgb::new(0.45, 0.75, 1.0),
                contrast: 0.0,
                dark: true,
                extra_borders: true,
            },
            _ => return None,
        })
    }

    /// Os papéis, pelas regras do Godot.
    #[must_use]
    pub fn roles(self) -> Roles {
        let mono = if self.dark { Rgb::WHITE } else { Rgb::BLACK };
        let mono_inv = if self.dark { Rgb::BLACK } else { Rgb::WHITE };
        let c = self.contrast;
        let c_floor = c.max(DEFAULT_CONTRAST);
        let base = self.base;
        let (info, success, warning, error) = if self.dark {
            (
                Rgb::new(0.7, 0.8, 1.0),
                Rgb::new(0.45, 0.95, 0.5),
                Rgb::new(0.83, 0.78, 0.62),
                Rgb::new(1.0, 0.47, 0.42),
            )
        } else {
            // «Darken some colors to be readable on a light background.»
            (
                Rgb::new(0.35, 0.6, 0.9),
                Rgb::new(0.45, 0.95, 0.5).lerp(mono, 0.35),
                Rgb::new(0.83, 0.49, 0.01),
                Rgb::new(0.8, 0.22, 0.22),
            )
        };
        // O CARTÃO (`Bg1`): é sobre ele que a lei de contraste da casa mede o texto secundário e o
        // acento (`CONTRAST_PAIRS`), e é a superfície que a família multiplicativa mais move.
        let surface_high = base.dimmed(c, -1.3, 0.8);
        // ⭐ **Dois números que a lei DERIVA em vez de os escrever.** O Godot fixa o texto
        //    secundário em `mono@0.55` e dá o acento do preset por inteiro — e nenhum dos dois
        //    cumpre a WCAG sobre o cartão em todos os presets: no *Gray* a base `0.24` eleva o
        //    cartão a `#555555` (a família é multiplicativa) e o `0.55` dá 3,5:1; o azul do *Light*
        //    (`0.18, 0.50, 1.00`) faz 2,5:1 sobre o cartão e **2,99:1 sobre o próprio painel** do
        //    Godot. A resposta do §0 é medir: a alfa sobe do piso só até cumprir 4,5:1, e o
        //    acento escurece 2 % de cada vez só até 3:1 (só o *Light* se move; o `#569eff` do
        //    Dark fica intacto, e há gate).
        // ⭐ O PISO é `0.65`, não o `0.55` do Godot: os títulos e rótulos dos cartões são todos
        //    `Text2`, e o dono pediu-os *«um pouco mais claros»* (2026-09-05). O `Text1` fica em
        //    `0.75`, para a hierarquia continuar a ler-se. Medido: Dark `0.65` · Light `0.65` ·
        //    Gray `0.72` (o Gray já precisava de mais pela lei).
        let font_secondary = (65..=80)
            .map(|pct| mono.over(base, pct as f32 / 100.0))
            .find(|t| wcag_ratio(*t, surface_high) >= 4.5)
            .unwrap_or(mono.over(base, 0.8));
        let mut accent = self.accent;
        for _ in 0..30 {
            if wcag_ratio(accent, surface_high) >= 3.0 {
                break;
            }
            accent = accent.lerp(Rgb::BLACK, 0.02);
        }
        let button_normal = base.dimmed(c, -2.0, 0.85);
        // ⚠️ **O OLED (base preta, contraste 0) colapsa a família multiplicativa numa cor só**: a
        //    `_get_base_color` escala o V, e `0 × k = 0`. O Godot separa os estados por bordas e
        //    pela cor da fonte; aqui um botão que não muda sob o rato é um botão morto com cara de
        //    vivo (gate `the_three_interactive_states_are_distinguishable`), então o hover e o
        //    pressionado caem num degrau ADITIVO em `mono` quando o multiplicativo não os move.
        let raised_or = |multiplicative: Rgb, additive_k: f32| {
            if multiplicative == button_normal {
                base.lerp(mono, c_floor * additive_k)
            } else {
                multiplicative
            }
        };
        Roles {
            base,
            accent,
            mono,
            mono_inv,
            dark_1: base.lerp(Rgb::BLACK, c * 1.15),
            // Os offsets e as saturações são os do `theme_modern.cpp` (191-209), literalmente.
            surface_lowest: base.dimmed(c, 1.7, 0.9),
            surface_lower: base.dimmed(c, 1.1, 0.9),
            surface_low: base.dimmed(c, 0.8, 1.0),
            surface_high,
            surface_higher: base.dimmed(c, -1.5, 0.8),
            surface_highest: base.dimmed(c, -2.2, 0.6),
            surface_popup: base.dimmed(c, 1.9, 0.9),
            button_normal,
            button_hover: raised_or(base.dimmed(c, -2.9, 0.75), 0.35),
            button_pressed: raised_or(base.dimmed(c, -3.2, 0.75), 0.5),
            button_disabled: base.dimmed(c, -1.4, 0.75),
            contrast_1: base.lerp(mono, c_floor * 1.15),
            contrast_2: base.lerp(mono, c_floor * 1.725),
            font: mono.over(base, 0.75),
            font_secondary,
            font_hover: mono.over(base, 0.85),
            font_disabled: mono.over(base, if self.dark { 0.35 } else { 0.5 }),
            info,
            success,
            warning,
            error,
            extra_border_a: if self.extra_borders { 0.2 } else { 0.0 },
            contrast: c,
        }
    }
}

/// A alfa do `highlight_color` do Godot — `Color(accent, 0.275)`.
const HIGHLIGHT_A: f32 = 0.275;

/// **A cor de um slot num tema moderno.**
///
/// ⚠️ Cobre TODA chave de [`ColorToken`] — o gate `every_token_derives_in_every_modern_theme`
/// percorre `ColorToken::ALL` × [`Theme::MODERN`], então uma chave nova que não entre aqui
/// reprova em vez de estourar no primeiro quadro.
#[must_use]
pub(crate) fn colour(theme: Theme, token: ColorToken) -> Color {
    let inputs = Inputs::of(theme).expect("um tema moderno tem entradas");
    let r = inputs.roles();
    let key = token.key();
    // As cores de DADO: emprestadas da família clássica (ver o topo).
    let borrowed = if inputs.dark {
        Theme::Forge
    } else {
        Theme::Sunstone
    };
    let borrow = || token.factory(borrowed);
    let c = r.contrast.max(DEFAULT_CONTRAST);
    match key {
        // ── superfícies ──
        // ⭐⭐ A ESCADA assenta no painel = `base` (o `PanelContainer` do Godot) e sobe pela
        //    família `surface_*`/`button_*` dele: `PanelBg → Bg1 → Bg2 → Bg3 → BgElev` é monótona
        //    (a subir no escuro, a descer no claro — o contraste negativo do *Light*), e o degrau
        //    do cartão é o que se vê (Dark: `#292929 → #393939`). Gate:
        //    `a_card_stands_off_its_panel_and_the_surface_ladder_climbs`. ⚠️ `Bg3` e `BgElev`
        //    ficam a 3/255 no Dark (`button_hover` · `button_pressed`): o Godot não tem um quinto
        //    nível elevado, e o `BgElev` só pousa sobre o painel ou sobre um cartão.
        // ⛔ O `bg-0` é o FUNDO DO CANVAS (`hero/canvas.rs`), e fica no `dark_1` que o dono viu e
        //    aprovou (Enio, 2026-09-05: *«mudou a cor do canvas — volte ao que era antes»*): a
        //    escada de superfícies começa no painel, não aqui.
        "bg-0" => r.dark_1.color(),
        "bg-1" => r.surface_high.color(),
        "bg-2" => r.button_normal.color(),
        "bg-3" => r.button_hover.color(),
        "bg-elev" => r.button_pressed.color(),
        "panel-bg" => r.base.color(),
        "rail-bg" => r.dark_1.with_alpha(0.85),
        "canvas" => r.base.lerp(Rgb::BLACK, c * 1.5).color(),
        "bg-scrim" => Rgb::BLACK.with_alpha(if inputs.dark { 0.6 } else { 0.4 }),
        // ── bordas ──
        "border" => r
            .mono
            .with_alpha(r.extra_border_a.max(if inputs.dark { 0.08 } else { 0.12 })),
        "border-strong" => r
            .mono
            .with_alpha((r.extra_border_a * 2.0).max(if inputs.dark { 0.2 } else { 0.25 })),
        "border-emph" => r.accent.color(),
        // ── texto ──
        "text-1" => r.font.color(),
        "text-2" => r.font_secondary.color(),
        "text-3" => r.mono.over(r.base, 0.45).color(),
        "text-disabled" => r.font_disabled.color(),
        // ── acento ──
        "accent" => r.accent.color(),
        "accent-hover" => r.accent.lerp(r.mono, 0.15).color(),
        "accent-press" => r.accent.lerp(r.mono_inv, 0.15).color(),
        "accent-soft" => r.accent.over(r.base, HIGHLIGHT_A).color(),
        "accent-fg" => r.mono_inv.color(),
        "selection" => r.accent.over(r.base, HIGHLIGHT_A).color(),
        "focus-ring" => r.accent.with_alpha(0.55),
        // ── estado ──
        "danger" => r.error.color(),
        "danger-soft" => r.error.over(r.base, HIGHLIGHT_A).color(),
        "success" => r.success.color(),
        "success-soft" => r.success.over(r.base, HIGHLIGHT_A).color(),
        "warn" => r.warning.color(),
        "warn-soft" => r.warning.over(r.base, HIGHLIGHT_A).color(),
        "info" => r.info.color(),
        "info-soft" => r.info.over(r.base, HIGHLIGHT_A).color(),
        // ── grelha e eixos ──
        "grid-line" => r.mono.with_alpha(0.12),
        "grid-axis" => r.accent.with_alpha(0.30),
        "axis-x" => Rgb::new(0.96, 0.20, 0.32).color(),
        "axis-y" => Rgb::new(0.53, 0.84, 0.01).color(),
        "axis-z" => Rgb::new(0.16, 0.55, 0.96).color(),
        // ── o grafo de nós ──
        // O fundo do grafo é um canvas como o outro: fica no `dark_1` aprovado (ver `bg-0`).
        "graph-bg" => r.dark_1.color(),
        "graph-grid" => r.mono.with_alpha(0.06),
        "graph-marquee" => r.accent.with_alpha(0.18),
        "graph-inert" => r.base.with_alpha(0.62),
        // ── a timeline: os 16 apelidos, por construção ──
        "timeline-curve"
        | "timeline-handle"
        | "timeline-key-selected"
        | "timeline-loop-brace"
        | "timeline-playhead"
        | "timeline-summary-ring" => r.accent.color(),
        "timeline-handle-line" | "timeline-loop-region" => {
            r.accent.over(r.base, HIGHLIGHT_A).color()
        }
        // Apelido POR CONSTRUÇÃO do `bg-2` (a lei dos 16 slots da timeline): repetir a fórmula
        // aqui foi o que os desalinhou no dia em que a escada de superfícies se reassentou.
        "timeline-row-alt" | "timeline-ruler-bg" => colour(theme, ColorToken::Bg2),
        "timeline-marker" | "timeline-summary-key" => r.warning.color(),
        "timeline-key-active" => r.accent.lerp(r.mono_inv, 0.15).color(),
        "timeline-missing" => r.error.color(),
        "timeline-key" => r.font.color(),
        "timeline-ruler-tick" => r.mono.over(r.base, 0.45).color(),
        // ── as cores de DADO: emprestadas ──
        k if k.starts_with("node-cat-")
            || k.starts_with("port-")
            || k.starts_with("curve-")
            || k.starts_with("graph-backdrop-")
            || k == "wire-fire-glow"
            || k == "attr-write" =>
        {
            borrow()
        }
        other => panic!(
            "color token {other:?} nao tem regra de derivacao para o tema {theme:?} — \
             acrescente-a em ph2d-tokens/src/derive.rs"
        ),
    }
}
