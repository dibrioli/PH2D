//! ⭐⭐ **A TABELA DE ESTADOS do widget — a única porta de «que fundo, que borda, que raio tem
//! este controlo AGORA?».**
//!
//! A forma é a do `egui` (`Widgets × WidgetVisuals`, Apache-2.0/MIT — `pesquisa/08 §2`): **cinco
//! estados × seis campos** dizem tudo o que um controlo interactivo pode ser, e é isso que torna
//! *plano* e *coerente* a mesma coisa — hoje cada pintor decide fundo e borda sozinho
//! (`stroke_rounded_rect` em 271 sítios), e o resultado é o app «com a mesma cara» depois de
//! redesenhar os widgets um a um.
//!
//! # As duas famílias, uma tabela
//!
//! - **Moderna** ([`Theme::is_modern`]): os valores saem dos papéis derivados
//!   ([`crate::derive::Roles`]) — botão em repouso é a `base` sobre um painel `dark_1`, o hover
//!   sobe um degrau de contraste, o pressionado é o `highlight`; **borda `0`** salvo com *Draw
//!   Extra Borders*; raio **4** (`interface/theme/corner_radius` do Godot).
//! - **Clássica**: os valores são os que os pintores já usam (`Bg2` · `BgElev` · `AccentSoft` ·
//!   `Border` · `Radius::Md`) — a tabela DESCREVE o clássico, não o muda. ⚠️ Os pintores clássicos
//!   **não a lêem** (mantêm o caminho de sempre, byte-idêntico); ela existe aqui para que um
//!   pintor novo tenha uma resposta nas duas aparências.
//!
//! ⛔ **Não há campo `expansion`** (o do egui cresce o rect no hover): o `HitIndex` deste app é
//! também o denominador do gesto (§5 do handoff de 03/09) — crescer o pintado sem crescer o
//! registado desalinharia o clique.

use crate::color::{Color, ColorToken};
use crate::derive::{Inputs, Rgb};
use crate::radius::Radius;
use crate::theme::Theme;

/// Um traço: largura em px e cor. `width == 0` ⇒ não se pinta.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stroke {
    pub width: f32,
    pub color: Color,
}

impl Stroke {
    pub const NONE: Self = Self {
        width: 0.0,
        color: Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    };

    #[must_use]
    pub const fn new(width: f32, color: Color) -> Self {
        Self { width, color }
    }

    /// Há traço a pintar?
    #[must_use]
    pub fn is_visible(self) -> bool {
        self.width > 0.0 && self.color.a > 0
    }
}

/// O aspecto de um controlo num estado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetVisuals {
    /// Fundo de um controlo com corpo (botão com preenchimento, campo, chip).
    pub bg_fill: Color,
    /// Fundo de um controlo *fantasma* (botão sem preenchimento em repouso): transparente até o
    /// rato chegar.
    pub weak_bg_fill: Color,
    /// A borda do corpo.
    pub bg_stroke: Stroke,
    /// O traço do CONTEÚDO — texto, ícone, marca.
    pub fg_stroke: Stroke,
    /// Raio das quinas do corpo.
    pub corner_radius: f32,
}

/// Os cinco estados.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Widgets {
    /// Um rótulo, um painel, uma superfície sem interacção.
    pub noninteractive: WidgetVisuals,
    /// Um controlo interactivo em repouso.
    pub inactive: WidgetVisuals,
    /// Sob o rato.
    pub hovered: WidgetVisuals,
    /// A ser pressionado / arrastado.
    pub active: WidgetVisuals,
    /// Aberto (um menu, uma secção, um dropdown).
    pub open: WidgetVisuals,
}

/// O cromo que dá a CARA: painel, cabeçalho de secção, campo de texto.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Chrome {
    /// Raio de um painel que FLUTUA (o docado é `0` nas duas famílias).
    pub panel_radius: f32,
    /// Borda de um painel — `0` no redesenho (o Godot só a traça com *Draw Extra Borders*).
    pub panel_border: Stroke,
    /// Raio da placa de um cabeçalho de secção dobrado.
    pub plate_radius: f32,
    /// Raio de um campo de texto / número.
    pub field_radius: f32,
    /// Borda de um campo em REPOUSO — `0` no redesenho: a moldura só aparece no foco.
    pub field_border: Stroke,
    /// A borda de um campo FOCADO (o `focus_style` do Godot: acento a 2 px).
    pub field_focus: Stroke,
    /// O fundo de um campo em repouso.
    pub field_fill: Color,
}

const TRANSPARENT: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

/// O `corner_radius` por omissão do Godot 4.6 (`interface/theme/corner_radius`, faixa `0..6`).
pub const MODERN_CORNER_RADIUS_PX: f32 = 4.0;

/// A largura do anel de foco do Godot (`focus_style->set_border_width_all(2)`).
const MODERN_FOCUS_W: f32 = 2.0;

impl Widgets {
    /// A tabela deste tema.
    #[must_use]
    pub fn of(theme: Theme) -> Self {
        match Inputs::of(theme) {
            Some(inputs) => Self::modern(inputs),
            None => Self::classic(theme),
        }
    }

    fn modern(inputs: Inputs) -> Self {
        let r = inputs.roles();
        let c = r.contrast.max(0.3);
        let border = if inputs.extra_borders {
            Stroke::new(1.0, r.mono.with_alpha(0.2))
        } else {
            Stroke::NONE
        };
        let text = |rgb: Rgb| Stroke::new(1.0, rgb.color());
        let radius = MODERN_CORNER_RADIUS_PX;
        Self {
            noninteractive: WidgetVisuals {
                bg_fill: r.dark_1.color(),
                weak_bg_fill: TRANSPARENT,
                bg_stroke: border,
                fg_stroke: text(r.font),
                corner_radius: radius,
            },
            inactive: WidgetVisuals {
                bg_fill: r.base.color(),
                weak_bg_fill: TRANSPARENT,
                bg_stroke: border,
                fg_stroke: text(r.font),
                corner_radius: radius,
            },
            hovered: WidgetVisuals {
                bg_fill: r.base.lerp(r.mono, c * 0.35).color(),
                weak_bg_fill: r.mono.with_alpha(0.06),
                bg_stroke: border,
                fg_stroke: text(r.font_hover),
                corner_radius: radius,
            },
            active: WidgetVisuals {
                bg_fill: r.accent.over(r.base, 0.275).color(),
                weak_bg_fill: r.accent.with_alpha(0.275),
                bg_stroke: border,
                fg_stroke: text(r.mono),
                corner_radius: radius,
            },
            open: WidgetVisuals {
                bg_fill: r.base.lerp(r.mono, c * 0.35).color(),
                weak_bg_fill: r.mono.with_alpha(0.06),
                bg_stroke: border,
                fg_stroke: text(r.font_hover),
                corner_radius: radius,
            },
        }
    }

    /// O clássico, DESCRITO: os tokens que os pintores de hoje já escolhem, no mesmo sítio.
    fn classic(theme: Theme) -> Self {
        let t = |tok: ColorToken| tok.resolve(theme);
        let radius = Radius::Md.px();
        let border = Stroke::new(1.0, t(ColorToken::Border));
        Self {
            noninteractive: WidgetVisuals {
                bg_fill: t(ColorToken::Bg1),
                weak_bg_fill: TRANSPARENT,
                bg_stroke: border,
                fg_stroke: Stroke::new(1.0, t(ColorToken::Text1)),
                corner_radius: radius,
            },
            inactive: WidgetVisuals {
                bg_fill: t(ColorToken::Bg2),
                weak_bg_fill: TRANSPARENT,
                bg_stroke: border,
                fg_stroke: Stroke::new(1.0, t(ColorToken::Text1)),
                corner_radius: radius,
            },
            hovered: WidgetVisuals {
                bg_fill: t(ColorToken::BgElev),
                weak_bg_fill: t(ColorToken::BgElev),
                bg_stroke: Stroke::new(1.0, t(ColorToken::BorderStrong)),
                fg_stroke: Stroke::new(1.0, t(ColorToken::Text1)),
                corner_radius: radius,
            },
            active: WidgetVisuals {
                bg_fill: t(ColorToken::AccentSoft),
                weak_bg_fill: t(ColorToken::AccentSoft),
                bg_stroke: Stroke::new(1.0, t(ColorToken::Accent)),
                fg_stroke: Stroke::new(1.0, t(ColorToken::Text1)),
                corner_radius: radius,
            },
            open: WidgetVisuals {
                bg_fill: t(ColorToken::BgElev),
                weak_bg_fill: t(ColorToken::BgElev),
                bg_stroke: border,
                fg_stroke: Stroke::new(1.0, t(ColorToken::Text1)),
                corner_radius: radius,
            },
        }
    }
}

impl Chrome {
    /// O cromo deste tema.
    #[must_use]
    pub fn of(theme: Theme) -> Self {
        match Inputs::of(theme) {
            Some(inputs) => Self::modern(inputs),
            None => Self::classic(theme),
        }
    }

    fn modern(inputs: Inputs) -> Self {
        let r = inputs.roles();
        let border = if inputs.extra_borders {
            Stroke::new(1.0, r.mono.with_alpha(0.2))
        } else {
            Stroke::NONE
        };
        Self {
            panel_radius: MODERN_CORNER_RADIUS_PX,
            panel_border: border,
            plate_radius: MODERN_CORNER_RADIUS_PX,
            field_radius: MODERN_CORNER_RADIUS_PX,
            field_border: border,
            field_focus: Stroke::new(MODERN_FOCUS_W, r.accent.color()),
            // O `LineEdit` do Godot assenta num degrau abaixo do painel.
            field_fill: r.dark_1.lerp(Rgb::BLACK, r.contrast.max(0.0) * 0.5).color(),
        }
    }

    fn classic(theme: Theme) -> Self {
        let t = |tok: ColorToken| tok.resolve(theme);
        Self {
            panel_radius: Radius::Sm.px(),
            panel_border: Stroke::new(1.0, t(ColorToken::Border)),
            plate_radius: Radius::Sm.px(),
            field_radius: Radius::Sm.px(),
            field_border: Stroke::new(1.0, t(ColorToken::Border)),
            field_focus: Stroke::new(2.0, t(ColorToken::BorderEmph)),
            field_fill: t(ColorToken::Bg1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No redesenho a moldura em repouso é ZERO — é a linha que separa «plano» de «com a mesma
    /// cara». O OLED é a excepção declarada (bordas extra), como no Godot.
    #[test]
    fn the_modern_family_has_no_resting_border_except_oled() {
        for theme in Theme::MODERN {
            let w = Widgets::of(theme);
            let c = Chrome::of(theme);
            let expect_border = theme == Theme::Oled;
            assert_eq!(
                w.inactive.bg_stroke.is_visible(),
                expect_border,
                "{theme:?} botao"
            );
            assert_eq!(
                c.panel_border.is_visible(),
                expect_border,
                "{theme:?} painel"
            );
            assert_eq!(
                c.field_border.is_visible(),
                expect_border,
                "{theme:?} campo"
            );
            assert!(
                c.field_focus.is_visible(),
                "{theme:?}: o foco tem de se ver"
            );
        }
    }

    /// O raio moderno é o do Godot, em todo o cromo — uma coisa só, não cinco números.
    #[test]
    fn the_modern_radius_is_godots_everywhere() {
        for theme in Theme::MODERN {
            let w = Widgets::of(theme);
            let c = Chrome::of(theme);
            for r in [
                w.inactive.corner_radius,
                w.hovered.corner_radius,
                w.active.corner_radius,
                c.panel_radius,
                c.plate_radius,
                c.field_radius,
            ] {
                assert_eq!(r, MODERN_CORNER_RADIUS_PX, "{theme:?}");
            }
        }
    }

    /// Os três estados interactivos têm fundos DISTINTOS — um botão que não muda sob o rato é
    /// um botão morto com a cara de vivo.
    #[test]
    fn the_three_interactive_states_are_distinguishable() {
        for theme in Theme::ALL {
            let w = Widgets::of(theme);
            assert_ne!(
                w.inactive.bg_fill, w.hovered.bg_fill,
                "{theme:?} repouso = hover"
            );
            assert_ne!(
                w.hovered.bg_fill, w.active.bg_fill,
                "{theme:?} hover = activo"
            );
            assert_ne!(
                w.inactive.bg_fill, w.active.bg_fill,
                "{theme:?} repouso = activo"
            );
        }
    }

    /// A tabela clássica descreve o clássico: os mesmos tokens que o `Button` escolhe hoje.
    #[test]
    fn the_classic_table_names_the_tokens_the_painters_use() {
        let w = Widgets::of(Theme::Forge);
        assert_eq!(w.inactive.bg_fill, ColorToken::Bg2.resolve(Theme::Forge));
        assert_eq!(w.hovered.bg_fill, ColorToken::BgElev.resolve(Theme::Forge));
        assert_eq!(
            w.active.bg_fill,
            ColorToken::AccentSoft.resolve(Theme::Forge)
        );
        assert_eq!(w.inactive.corner_radius, Radius::Md.px());
    }
}
