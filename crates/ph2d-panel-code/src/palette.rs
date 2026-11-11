//! As cores de UM quadro, resolvidas uma vez pelo tema: cada papel tem o seu token, e o texto
//! pergunta à porta única [`Ink::token`].

use crate::style::Ink;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::section_cards::CardDepth;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, VectorScene};

/// A fracção de tinta da faixa parada e da selecção nos cartões (o `HIGHLIGHT_A` dos `*-soft` do tema).
pub const BAND_INK: f32 = 0.275; // LITERAL-PX-OK: opacidade de realce, a do tema, não uma medida

#[derive(Clone)]
pub(crate) struct Palette {
    inks: [Color; Ink::ALL.len()],
    pub bg: Color,
    pub gutter: Color,
    pub border: Color,
    pub current_line: Color,
    pub selection: Color,
    pub match_: Color,
    pub match_current: Color,
    pub match_edge: Color,
    /// As ocorrências do nome sob o cursor (I10): neutro, abaixo da busca e da selecção.
    pub occurrence: Color,
    pub bracket: Color,
    pub caret: Color,
    pub caret_other: Color,
    pub number: Color,
    pub number_current: Color,
    pub error: Color,
    pub warn: Color,
    /// A faixa da linha onde o depurador parou: âmbar translúcido (o `warn-soft` da pele é cinza).
    pub stopped_band: Color,
    pub info: Color,
    pub fold: Color,
    pub pill: Color,
    pub pill_text: Color,
    pub status_bg: Color,
    pub status_text: Color,
    pub success: Color,
    /// O tema do quadro — as quinas perguntam-lhe ([`ph2d_editor_core::paint::frame_radius`]).
    pub theme: Theme,
}

impl Palette {
    pub fn new(theme: Theme) -> Self {
        let c = |t: ColorToken| resolve(t, theme);
        let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
        let well = Well::new(theme, cartoes);
        Self {
            inks: Ink::ALL.map(|i| c(i.token())),
            bg: well.bg,
            gutter: well.gutter,
            border: c(ColorToken::Border),
            current_line: well.current_line,
            selection: well.selection,
            match_: c(ColorToken::InfoSoft),
            match_current: c(ColorToken::WarnSoft),
            match_edge: c(ColorToken::Warn),
            occurrence: well.occurrence,
            bracket: c(ColorToken::BorderStrong),
            caret: c(ColorToken::Text1),
            caret_other: c(ColorToken::Text2),
            number: c(ColorToken::Text3),
            number_current: c(ColorToken::Text2),
            error: c(ColorToken::Danger),
            warn: c(ColorToken::Warn),
            stopped_band: c(ColorToken::Warn).with_alpha(BAND_INK),
            info: c(ColorToken::Info),
            fold: c(ColorToken::Text2),
            pill: well.pill,
            pill_text: c(ColorToken::Text2),
            // A barra de estado fica no pé do cartão do código (dono 08/11): o tom do cartão.
            status_bg: if cartoes {
                c(ColorToken::KanbanCard)
            } else {
                CardDepth::Section.fill(theme)
            },
            status_text: c(if cartoes {
                ColorToken::CardLabel
            } else {
                ColorToken::Text2
            }),
            success: c(ColorToken::Success),
            theme,
        }
    }

    pub fn ink(&self, ink: Ink) -> Color {
        self.inks[ink as usize]
    }

    /// Troca a cor de `ink` (o «mudou» em âmbar na mono, [`crate::table_text`]).
    pub(crate) fn tint(&mut self, ink: Ink, color: Color) {
        self.inks[ink as usize] = color;
    }
}

/// As superfícies do poço do código. Nos cartões o poço é o tom do cartão — o MESMO das linhas de
/// Problems (dono 10/11: o texto claro sobre o quase-preto lia-se mais fino); a calha desce ao campo
/// do cartão, a linha do cursor sobe ao cartão erguido e a selecção é a lei do `selection` sobre ele.
/// Com `PH2D_CARDS=0`, os `code-*` de sempre.
pub(crate) struct Well {
    pub bg: Color,
    pub gutter: Color,
    pub current_line: Color,
    pub selection: Color,
    pub occurrence: Color,
    pub pill: Color,
}

impl Well {
    pub fn new(theme: Theme, cartoes: bool) -> Self {
        let c = |t: ColorToken| resolve(t, theme);
        if cartoes {
            return Self {
                bg: c(crate::cards::card_tone()),
                gutter: c(ColorToken::CardField),
                current_line: c(ColorToken::KanbanCardRaised),
                selection: c(ColorToken::Accent).with_alpha(BAND_INK),
                occurrence: c(ColorToken::KanbanSlot),
                pill: c(ColorToken::CardField),
            };
        }
        Self {
            bg: c(ColorToken::CodeBg),
            gutter: c(ColorToken::CodeGutter),
            current_line: c(ColorToken::CodeLineCurrent),
            selection: c(ColorToken::Selection),
            occurrence: c(ColorToken::Bg3),
            pill: c(ColorToken::Bg3),
        }
    }
}

/// O marcador de dobra na calha: ▸ fechada, ▾ aberta, centrado em `(cx, cy)`.
pub(crate) fn fold_marker(scene: &mut VectorScene, cx: f32, cy: f32, folded: bool, color: Color) {
    let h = f64::from(Spacing::Xs.px());
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let pts = if folded {
        [(cx - h * 0.5, cy - h), (cx + h, cy), (cx - h * 0.5, cy + h)]
    } else {
        [(cx - h, cy - h * 0.5), (cx + h, cy - h * 0.5), (cx, cy + h)]
    };
    let mut p = BezPath::new();
    p.move_to(pts[0]);
    p.line_to(pts[1]);
    p.line_to(pts[2]);
    p.close_path();
    scene.fill_path(&p, &Brush::Solid(color), Affine::IDENTITY);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
    use ph2d_tokens::Color as Rgba;

    const AA_TEXT: f64 = 4.5;
    const AA_GRAPHIC: f64 = 3.0;

    fn palette(theme: Theme, cartoes: bool) -> Palette {
        force_cards_on_this_thread(Some(cartoes));
        let p = Palette::new(theme);
        force_cards_on_this_thread(None);
        p
    }

    fn over(f: Rgba, a: f32, b: Rgba) -> Rgba {
        let m = |x: u8, y: u8| {
            let v = f32::from(x) * a + f32::from(y) * (1.0 - a);
            // CLAMP-OK: um canal de 8 bits.
            v.round().clamp(0.0, 255.0) as u8
        };
        Rgba {
            r: m(f.r, b.r),
            g: m(f.g, b.g),
            b: m(f.b, b.b),
            a: u8::MAX,
        }
    }

    /// ⭐ Nos cartões o poço do código é o tom do cartão das linhas de Problems (dono 10/11); com
    /// `PH2D_CARDS=0` fica o `code-bg`. *Mutação: devolver o ramo clássico nos cartões ⇒ vermelho.*
    #[test]
    fn o_poco_do_codigo_e_o_tom_do_cartao() {
        for tema in [Theme::Forge, Theme::Dark, Theme::Light] {
            let novo = palette(tema, true);
            assert_eq!(novo.bg, resolve(ColorToken::KanbanCard, tema), "{tema:?}");
            assert_eq!(novo.gutter, resolve(ColorToken::CardField, tema));
            assert_eq!(
                novo.current_line,
                resolve(ColorToken::KanbanCardRaised, tema)
            );
            let antes = palette(tema, false);
            assert_eq!(antes.bg, resolve(ColorToken::CodeBg, tema), "{tema:?}");
            assert_eq!(antes.gutter, resolve(ColorToken::CodeGutter, tema));
            assert_eq!(antes.selection, resolve(ColorToken::Selection, tema));
        }
    }

    /// ⭐ Sobre o poço novo, em todo tema: cada tinta do código a ≥ 4,5:1 (comentário e dica ≥ 3:1) no
    /// fundo e na linha do cursor; os números a ≥ 3:1 na calha; o texto a ≥ 4,5:1 na selecção; e a
    /// selecção, a linha do cursor e as ocorrências separam-se do fundo. O `Link` (o acento sob Ctrl,
    /// sublinhado) fica de fora: é o traço do VS Code, medido no handoff.
    #[test]
    fn as_tintas_leem_se_no_poco_dos_cartoes() {
        let mut falhas = Vec::new();
        for tema in Theme::ALL {
            let r = |t: ColorToken| t.resolve(tema);
            let (bg, line) = (r(ColorToken::KanbanCard), r(ColorToken::KanbanCardRaised));
            for ink in Ink::ALL.into_iter().filter(|i| *i != Ink::Link) {
                let min = if matches!(ink, Ink::Comment | Ink::Hint) {
                    AA_GRAPHIC
                } else {
                    AA_TEXT
                };
                for (onde, f) in [("bg", bg), ("line", line)] {
                    let k = r(ink.token()).contrast_ratio(&f);
                    if k < min {
                        falhas.push(format!("{tema:?}: {ink:?} sobre {onde} = {k:.2}"));
                    }
                }
            }
            let k = r(ColorToken::Text3).contrast_ratio(&r(ColorToken::CardField));
            if k < AA_GRAPHIC {
                falhas.push(format!("{tema:?}: text-3 sobre a calha = {k:.2}"));
            }
            let sel = over(r(ColorToken::Accent), BAND_INK, bg);
            let k = r(ColorToken::Text1).contrast_ratio(&sel);
            if k < AA_TEXT {
                falhas.push(format!("{tema:?}: text-1 sobre a selecção = {k:.2}"));
            }
            let slot = r(ColorToken::KanbanSlot);
            let occ = over(slot, f32::from(slot.a) / 255.0, bg);
            for (nome, s, min) in [
                ("selecção", sel, 1.4),
                ("cursor", line, 1.1),
                ("ocorrência", occ, 1.2),
            ] {
                let k = s.contrast_ratio(&bg);
                if k < min {
                    falhas.push(format!("{tema:?}: {nome} a {k:.2} do fundo"));
                }
            }
        }
        assert!(falhas.is_empty(), "{}", falhas.join("\n"));
    }
}
