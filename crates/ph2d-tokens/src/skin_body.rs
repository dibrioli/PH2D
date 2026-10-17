//! **O CORPO de uma pele vestida** (F18 — `docs/UI_New_and_Simple/vetor_nova_ui/29_plano_F18_o_desenho_e_o_corpo_do_widget.md`).
//!
//! Enquanto a pele de um widget é pintada sobre o DESENHO do artista, o tema responde pelo
//! [`Role`] de cada token: a superfície é o desenho, a linha de repouso some, o conteúdo escolhe a
//! tinta legível sobre ele. É o *theme override* local do Godot e a propriedade com escopo do CSS —
//! uma camada ACIMA da autorada ([`crate::overrides`]), consultada nas DUAS portas onde uma cor de
//! tema nasce: [`ColorToken::resolve`] e `Widgets::of`/`Chrome::of` ([`crate::visuals`]).
//!
//! Fora de um [`dress`] custa uma leitura de bool e a resposta é byte-idêntica.

use std::cell::{Cell, RefCell};

use crate::color::{Color, ColorToken};
use crate::theme::Theme;
use crate::visuals::{Chrome, Stroke, WidgetVisuals, Widgets};

/// O que está DEBAIXO do widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Body {
    /// O desenho do artista já pintou o corpo; `under` é a cor dele (`None` = sem preenchimento).
    Drawing { under: Option<Color> },
    /// Não há desenho debaixo (o painel gerado): a pele pinta o corpo na cor do artista.
    Flat { colour: Color },
}

impl Body {
    /// A cor contra a qual o conteúdo se lê.
    #[must_use]
    pub const fn ground(self) -> Option<Color> {
        match self {
            Self::Drawing { under } => under,
            Self::Flat { colour } => Some(colour),
        }
    }
}

/// O papel de um token numa pele vestida.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    /// Um corpo, no degrau de estado dele.
    Surface(Rank),
    /// Uma linha de repouso (moldura, grelha): parte do corpo.
    Line,
    /// Tinta de texto/ícone.
    Ink(InkLevel),
    /// Um gráfico de conteúdo (valor, marca, acento): fica enquanto se lê.
    Graphic,
}

/// O degrau de estado de uma superfície — a *state layer* do Material 3.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rank {
    Rest,
    Raised,
    Pressed,
}

/// A hierarquia de uma tinta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InkLevel {
    Primary,
    Tertiary,
    Disabled,
    /// A tinta SOBRE o acento (a marca do checkbox, o texto de um botão Accent): lê-se contra o
    /// acento já vestido, não contra o desenho.
    OnAccent,
}

/// Material 3, *state layers*: hover `8 %`.
const RAISED_LAYER: f32 = 0.08;
/// Material 3, *state layers*: pressed / focus `10 %`.
const PRESSED_LAYER: f32 = 0.10;
/// Material 3: conteúdo desactivado a `38 %`.
const DISABLED_INK: f32 = 0.38;
/// WCAG 2.2 AA 1.4.11 — componente não-textual.
const GRAPHIC_MIN: f64 = 3.0;
/// WCAG 2.2 AA 1.4.3 — texto.
pub const INK_MIN: f64 = 4.5;

const TRANSPARENT: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

impl ColorToken {
    /// **O papel deste token numa pele vestida.** `match` exaustivo: um token novo não compila
    /// sem dizer o que é.
    #[must_use]
    pub const fn role(self) -> Role {
        use ColorToken as T;
        match self {
            T::Bg0
            | T::Bg1
            | T::Bg2
            | T::BgScrim
            | T::RailBg
            | T::PanelBg
            | T::WindowGround
            | T::Canvas
            | T::GraphBg
            | T::GraphBackdrop1
            | T::GraphBackdrop2
            | T::GraphBackdrop3
            | T::GraphBackdrop4
            | T::GraphBackdrop5
            | T::GraphBackdrop6
            | T::GraphBackdrop7
            | T::GraphBackdrop8
            | T::TimelineRulerBg
            | T::CodeBg
            | T::CodeGutter => Role::Surface(Rank::Rest),
            T::Bg3 | T::BgElev | T::TimelineRowAlt | T::CodeLineCurrent => {
                Role::Surface(Rank::Raised)
            }
            T::AccentSoft
            | T::DangerSoft
            | T::SuccessSoft
            | T::WarnSoft
            | T::InfoSoft
            | T::Selection
            | T::GraphMarquee
            | T::StageTintUi
            | T::TimelineLoopRegion => Role::Surface(Rank::Pressed),
            T::Border
            | T::BorderStrong
            | T::GridLine
            | T::GridAxis
            | T::GraphGrid
            | T::TimelineRulerTick
            | T::TimelineHandleLine => Role::Line,
            T::Text1 | T::Text2 | T::PersonInk => Role::Ink(InkLevel::Primary),
            T::AccentFg => Role::Ink(InkLevel::OnAccent),
            T::Text3 => Role::Ink(InkLevel::Tertiary),
            T::TextDisabled => Role::Ink(InkLevel::Disabled),
            T::BorderEmph
            | T::Accent
            | T::AccentHover
            | T::AccentPress
            | T::Danger
            | T::Success
            | T::Warn
            | T::Info
            | T::FocusRing
            | T::CurveR
            | T::CurveG
            | T::CurveB
            | T::AxisX
            | T::AxisY
            | T::AxisZ
            | T::NodeCatSource
            | T::NodeCatDistribute
            | T::NodeCatTransform
            | T::NodeCatFocus
            | T::NodeCatFx
            | T::NodeCatOutput
            | T::NodeCatUtility
            | T::PortInstances
            | T::PortVector
            | T::PortField
            | T::PortSignal
            | T::PortControl
            | T::PortEvent
            | T::PortStatic
            | T::PortValue
            | T::WireFireGlow
            | T::GraphInert
            | T::AttrWrite
            | T::TimelinePlayhead
            | T::TimelineKey
            | T::TimelineKeySelected
            | T::TimelineKeyActive
            | T::TimelineCurve
            | T::TimelineHandle
            | T::TimelineLoopBrace
            | T::TimelineMarker
            | T::TimelineSummaryKey
            | T::TimelineSummaryRing
            | T::TimelineMissing
            | T::BoneHandle
            | T::ShapeHandle
            | T::Person0
            | T::Person1
            | T::Person2
            | T::Person3
            | T::Person4
            | T::Person5
            | T::Person6
            | T::Person7
            | T::CodeKeyword
            | T::CodeFunction
            | T::CodeBuiltin
            | T::CodeType
            | T::CodeString
            | T::CodeNumber
            | T::CodeConstant
            | T::CodeProperty
            | T::CodeOperator
            | T::CodeComment => Role::Graphic,
        }
    }
}

thread_local! {
    static STACK: RefCell<Vec<Body>> = const { RefCell::new(Vec::new()) };
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
}

/// O escopo aberto por [`dress`]; fechá-lo (drop, também num `panic`) devolve o tema de antes.
#[must_use = "o escopo fecha quando a guarda morre"]
pub struct Dressed(());

impl Drop for Dressed {
    fn drop(&mut self) {
        STACK.with(|s| {
            let mut s = s.borrow_mut();
            s.pop();
            ACTIVE.with(|a| a.set(!s.is_empty()));
        });
    }
}

/// **Abre o escopo**: até a guarda morrer, este fio pinta sobre `body`.
pub fn dress(body: Body) -> Dressed {
    STACK.with(|s| s.borrow_mut().push(body));
    ACTIVE.with(|a| a.set(true));
    Dressed(())
}

fn current() -> Option<Body> {
    if !ACTIVE.with(Cell::get) {
        return None;
    }
    STACK.with(|s| s.borrow().last().copied())
}

/// **A tinta que se lê sobre `under`** — o `contrast-color()` do CSS dentro da paleta do tema.
///
/// Entre os dois polos do tema (`Text1`, `Bg0`) o de maior contraste; se ele não chega a
/// [`INK_MIN`], o extremo puro do mesmo lado (preto ou branco dão sempre `≥ 4,58:1`).
#[must_use]
pub fn readable_on(under: Color, theme: Theme) -> Color {
    let opaque = |c: Color| Color { a: 255, ..c };
    let (a, b) = (
        opaque(ColorToken::Text1.resolve_unscoped(theme)),
        opaque(ColorToken::Bg0.resolve_unscoped(theme)),
    );
    let best = if a.contrast_ratio(&under) >= b.contrast_ratio(&under) {
        a
    } else {
        b
    };
    if best.contrast_ratio(&under) >= INK_MIN {
        return best;
    }
    let white = Color::from_hex(0xFF_FF_FF); // LITERAL-COLOR-OK: o extremo do contrast-color()
    let black = Color::from_hex(0x00_00_00); // LITERAL-COLOR-OK: o extremo do contrast-color()
    if white.contrast_ratio(&under) >= black.contrast_ratio(&under) {
        white
    } else {
        black
    }
}

fn with_alpha(c: Color, k: f32) -> Color {
    Color {
        a: (f32::from(c.a) * k).round() as u8,
        ..c
    }
}

/// `top` composto por cima de `base` com opacidade `k` (source-over, em sRGB).
fn over(base: Color, top: Color, k: f32) -> Color {
    let mix = |b: u8, t: u8| (f32::from(b) + (f32::from(t) - f32::from(b)) * k).round() as u8;
    Color {
        r: mix(base.r, top.r),
        g: mix(base.g, top.g),
        b: mix(base.b, top.b),
        a: base.a,
    }
}

/// A fracção a que `Text3` está entre `Bg1` e `Text1` neste tema (luminância) — a hierarquia do
/// tema, levada para a tinta nova.
fn tertiary_alpha(theme: Theme) -> f32 {
    let l = |t: ColorToken| t.resolve_unscoped(theme).relative_luminance();
    let (lo, hi, mid) = (l(ColorToken::Bg1), l(ColorToken::Text1), l(ColorToken::Text3));
    if (hi - lo).abs() < f64::EPSILON {
        return 1.0;
    }
    (((mid - lo) / (hi - lo)) as f32).clamp(DISABLED_INK, 1.0)
}

fn ink(body: Body, theme: Theme) -> Option<Color> {
    body.ground().map(|g| readable_on(g, theme))
}

fn surface(body: Body, rank: Rank, theme: Theme) -> Color {
    let k = match rank {
        Rank::Rest => 0.0,
        Rank::Raised => RAISED_LAYER,
        Rank::Pressed => PRESSED_LAYER,
    };
    let layer = ink(body, theme).unwrap_or_else(|| ColorToken::Text1.resolve_unscoped(theme));
    match body {
        Body::Drawing { .. } if k == 0.0 => TRANSPARENT,
        Body::Drawing { .. } => with_alpha(Color { a: 255, ..layer }, k),
        Body::Flat { colour } => over(colour, layer, k),
    }
}

fn graphic(body: Body, base: Color, theme: Theme) -> Color {
    match body.ground() {
        Some(g) if base.a > 0 && base.contrast_ratio(&g) < GRAPHIC_MIN => readable_on(g, theme),
        _ => base,
    }
}

fn dressed(body: Body, role: Role, base: Color, theme: Theme) -> Color {
    match role {
        Role::Surface(rank) => surface(body, rank, theme),
        Role::Line => TRANSPARENT,
        Role::Ink(InkLevel::OnAccent) => {
            let raw = ColorToken::Accent.resolve_unscoped(theme);
            let worn = graphic(body, raw, theme);
            if worn == raw {
                base
            } else {
                readable_on(worn, theme)
            }
        }
        Role::Ink(level) => match (ink(body, theme), level) {
            (None, _) => base,
            (Some(c), InkLevel::Primary | InkLevel::OnAccent) => c,
            (Some(c), InkLevel::Tertiary) => with_alpha(c, tertiary_alpha(theme)),
            (Some(c), InkLevel::Disabled) => with_alpha(c, DISABLED_INK),
        },
        Role::Graphic => graphic(body, base, theme),
    }
}

/// A porta do [`ColorToken::resolve`]: `base` é a resposta sem escopo.
pub(crate) fn scoped(token: ColorToken, base: Color, theme: Theme) -> Color {
    match current() {
        None => base,
        Some(body) => dressed(body, token.role(), base, theme),
    }
}

fn dress_visuals(body: Body, v: WidgetVisuals, rank: Rank, theme: Theme) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: surface(body, rank, theme),
        weak_bg_fill: surface(body, rank, theme),
        bg_stroke: Stroke::NONE,
        fg_stroke: Stroke::new(
            v.fg_stroke.width,
            dressed(body, Role::Ink(InkLevel::Primary), v.fg_stroke.color, theme),
        ),
        corner_radius: v.corner_radius,
    }
}

/// A porta do `Widgets::of`.
pub(crate) fn scoped_widgets(w: Widgets, theme: Theme) -> Widgets {
    let Some(body) = current() else {
        return w;
    };
    Widgets {
        noninteractive: dress_visuals(body, w.noninteractive, Rank::Rest, theme),
        inactive: dress_visuals(body, w.inactive, Rank::Rest, theme),
        hovered: dress_visuals(body, w.hovered, Rank::Raised, theme),
        active: dress_visuals(body, w.active, Rank::Pressed, theme),
        open: dress_visuals(body, w.open, Rank::Raised, theme),
    }
}

/// A porta do `Chrome::of`.
pub(crate) fn scoped_chrome(c: Chrome, theme: Theme) -> Chrome {
    let Some(body) = current() else {
        return c;
    };
    let guard = |s: Stroke| Stroke::new(s.width, graphic(body, s.color, theme));
    Chrome {
        panel_border: Stroke::NONE,
        field_border: Stroke::NONE,
        field_focus: guard(c.field_focus),
        field_fill: surface(body, Rank::Rest, theme),
        selected: guard(c.selected),
        ..c
    }
}

#[cfg(test)]
#[path = "skin_body_tests.rs"]
mod tests;
