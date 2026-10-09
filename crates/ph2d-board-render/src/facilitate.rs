//! **A facilitação no desenho** (W7, `docs/MiroClone/05_facilitacao.md` §2 V2, V5, S1–S3): o selo do
//! resultado de uma votação (o medido do Miro), as fichas dos carimbos por baixo da nota e, por cima,
//! os `+`/`−` de votar da sessão aberta.

use ph2d_board_edit::VoteMark;
use ph2d_board_model::{Element, Rgba, RichText, Stamp};
use ph2d_text::TextSystem;
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color, Point, RoundedRect, Shape as _, Stroke, VectorScene,
};

use crate::{TextCache, doc_color};

/// V5 — o azul do selo do Miro, medido em `vote_05` (66, 98, 255).
// LITERAL-COLOR-OK: cor do PRODUTO de referência (o selo de resultado do Miro), a mesma em qualquer tema.
const BADGE_BLUE: Rgba = Rgba([0x42, 0x62, 0xff, 0xff]);
/// V2 — o cinzento do `+` de votar, medido em `vote_03` (o miolo de 190–200 com azul a mais).
// LITERAL-COLOR-OK: cor do produto de referência (o `+` de votar do Miro).
const PLUS_GREY: Rgba = Rgba([0xbe, 0xbe, 0xd2, 0xff]);
/// A letra do número em fracção do Ø do selo; a da contagem de uma ficha em fracção da altura dela.
const BADGE_TEXT: f64 = 0.5;
const CHIP_TEXT: f64 = 0.55;
/// O ícone na ficha: o lado em fracção da altura e a folga à esquerda (a mesma por cima).
const CHIP_ICON: f64 = 0.64;
const CHIP_PAD: f64 = 0.18;
/// O traço dos ícones Lucide: 2 em 24.
const LUCIDE_STROKE: f64 = 2.0;
const LUCIDE_BOX: f64 = 24.0;
/// A ponta descolada do selo: o arco de cima à esquerda, a esta transparência de branco.
const PEEL_ALPHA: u8 = 70;
/// Os sais das chaves de texto na cache (o texto da nota usa o id cru).
const BADGE_SALT: u64 = 0x766f_0000_0000_0000;
const CHIP_SALT: u64 = 0x7374_0000_0000_0000;

/// ⭐ Os carimbos — os caminhos SVG dos ícones **Lucide** 1.54 (ISC, `lucide-static`), num quadrado de
/// 24. O `<circle>` do `circle-help` está escrito como dois arcos.
pub(crate) const STAMP_SVG: [(Stamp, &[&str]); 8] = [
    (
        Stamp::ThumbsUp,
        &[
            "M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z",
            "M7 10v12",
        ],
    ),
    (
        Stamp::ThumbsDown,
        &[
            "M9 18.12 10 14H4.17a2 2 0 0 1-1.92-2.56l2.33-8A2 2 0 0 1 6.5 2H20a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-2.76a2 2 0 0 0-1.79 1.11L12 22a3.13 3.13 0 0 1-3-3.88Z",
            "M17 14V2",
        ],
    ),
    (
        Stamp::Heart,
        &[
            "M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5",
        ],
    ),
    (
        Stamp::Star,
        &[
            "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z",
        ],
    ),
    (
        Stamp::Flame,
        &[
            "M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4",
        ],
    ),
    (
        Stamp::Lightbulb,
        &[
            "M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5",
            "M9 18h6",
            "M10 22h4",
        ],
    ),
    (
        Stamp::Question,
        &[
            "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0",
            "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3",
            "M12 17h.01",
        ],
    ),
    (
        Stamp::Party,
        &[
            "M5.8 11.3 2 22l10.7-3.79",
            "M4 3h.01",
            "M22 8h.01",
            "M15 2h.01",
            "M22 20h.01",
            "m22 2-2.24.75a2.9 2.9 0 0 0-1.96 3.12c.1.86-.57 1.63-1.45 1.63h-.38c-.86 0-1.6.6-1.76 1.44L14 10",
            "m22 13-.82-.33c-.86-.34-1.82.2-1.98 1.11c-.11.7-.72 1.22-1.43 1.22H17",
            "m11 2 .33.82c.34.86-.2 1.82-1.11 1.98C9.52 4.9 9 5.52 9 6.23V7",
            "M11 13c1.93 1.93 2.83 4.17 2 5-.83.83-3.07-.07-5-2-1.93-1.93-2.83-4.17-2-5 .83-.83 3.07.07 5 2Z",
        ],
    ),
];

/// ⭐ O desenho de um carimbo: o ícone Lucide num quadrado de 24 (traço de 2, pontas redondas).
#[must_use]
pub fn stamp_path(s: Stamp) -> BezPath {
    let mut out = BezPath::new();
    let svg = STAMP_SVG
        .iter()
        .find(|(k, _)| *k == s)
        .map_or(&[][..], |(_, d)| *d);
    for d in svg {
        if let Ok(p) = BezPath::from_svg(d) {
            out.extend(p);
        }
    }
    out
}

/// A cor de um carimbo (a do emoji que ele substitui) — DADO do documento, a mesma em qualquer tema.
#[must_use]
pub fn stamp_color(s: Stamp) -> Rgba {
    // LITERAL-COLOR-OK: as cores dos emoji que os carimbos substituem (amarelo, vermelho, laranja,
    // azul e roxo dos ramos do mapa do Miro).
    Rgba(match s {
        Stamp::ThumbsUp | Stamp::ThumbsDown | Stamp::Star | Stamp::Lightbulb => {
            [0xe8, 0xa3, 0x00, 0xff]
        }
        Stamp::Heart => [0xe5, 0x39, 0x35, 0xff],
        Stamp::Flame => [0xff, 0x7a, 0x00, 0xff],
        Stamp::Question => [0x42, 0x62, 0xff, 0xff],
        Stamp::Party => [0xb7, 0x36, 0xff, 0xff],
    })
}

/// Pinta o carimbo `s` com o canto superior esquerdo em `at` e o lado `side` (por `t`).
pub fn paint_stamp(scene: &mut VectorScene, s: Stamp, at: [f64; 2], side: f64, t: Affine) {
    let k = side / LUCIDE_BOX;
    let path = stamp_path(s);
    let stroke = Stroke::new(LUCIDE_STROKE)
        .with_caps(ph2d_vector::Cap::Round)
        .with_join(ph2d_vector::Join::Round);
    let into = t * Affine::translate((at[0], at[1])) * Affine::scale(k);
    let brush = Brush::Solid(doc_color(stamp_color(s)));
    scene.inner_mut().stroke(&stroke, into, &brush, None, &path);
}

/// Um número centrado em `c` (referencial de `t`), na cache de texto sob `key`.
#[allow(clippy::too_many_arguments)]
fn number(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut TextCache,
    key: (u64, u64),
    n: u32,
    size: f64,
    c: [f64; 2],
    t: Affine,
    ink: Color,
) {
    let layout = text.get_rich(
        ts,
        key,
        &RichText::plain(&n.to_string()),
        size as f32,
        f32::MAX,
        false,
    );
    let (w, h) = (f64::from(layout.width()), f64::from(layout.height()));
    ph2d_board_layout::paint(
        scene,
        layout,
        t * Affine::translate((c[0] - w / 2.0, c[1] - h / 2.0)),
        ink,
    );
}

/// ⭐ V5 e S3 — por cima de um elemento (por `t` = caixa → ecrã): o selo do resultado, se `range` (o
/// menor e o maior total do quadro, `None` = escondidos) e ele tem votos; as fichas dos carimbos.
pub(crate) fn paint_marks(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut TextCache,
    board: u64,
    el: &Element,
    t: Affine,
    range: Option<[u32; 2]>,
) {
    for (c, [x0, y0, x1, y1]) in ph2d_board_model::stamp_chips(el) {
        let h = y1 - y0;
        let chip = RoundedRect::new(x0, y0, x1, y1, h / 2.0).to_path(0.1);
        // LITERAL-COLOR-OK: a ficha é papel branco com o contorno da sombra da nota (documento).
        scene.fill_path(
            &chip,
            &Brush::Solid(Color::from_rgba8(255, 255, 255, 255)),
            t,
        );
        let edge = Brush::Solid(Color::from_rgba8(0, 0, 0, 40)); // LITERAL-COLOR-OK: idem
        scene
            .inner_mut()
            .stroke(&Stroke::new(h * 0.04), t, &edge, None, &chip);
        let icon = h * CHIP_ICON;
        paint_stamp(
            scene,
            c.stamp,
            [x0 + h * CHIP_PAD, y0 + (h - icon) / 2.0],
            icon,
            t,
        );
        let digits_x0 = x0 + h * (CHIP_PAD * 2.0 + CHIP_ICON);
        let key = (board ^ CHIP_SALT, (el.id.0 << 4) | c.stamp as u64);
        let ink = doc_color(Rgba(ph2d_board_model::DEFAULT_INK));
        number(
            scene,
            ts,
            text,
            key,
            c.count,
            h * CHIP_TEXT,
            [(digits_x0 + x1 - h * CHIP_PAD * 0.5) / 2.0, (y0 + y1) / 2.0],
            t,
            ink,
        );
    }
    let Some(range) = range.filter(|_| el.votes() > 0) else {
        return;
    };
    let (c, d) = ph2d_board_model::vote_badge(el, el.votes(), range);
    let disc = Circle::new(Point::new(c[0], c[1]), d / 2.0);
    scene.fill_path(&disc.to_path(0.1), &Brush::Solid(doc_color(BADGE_BLUE)), t);
    // A ponta descolada (o Miro): o arco de cima à esquerda, mais claro, cortado pela corda.
    let r = d / 2.0;
    let arc = format!(
        "M{} {} A{r} {r} 0 0 1 {} {} Z",
        c[0] - r,
        c[1],
        c[0],
        c[1] - r
    );
    let peel = BezPath::from_svg(&arc).unwrap_or_default();
    let light = Color::from_rgba8(255, 255, 255, PEEL_ALPHA); // LITERAL-COLOR-OK: o brilho da ponta
    scene.fill_path(&peel, &Brush::Solid(light), t);
    let white = Color::from_rgba8(255, 255, 255, 255); // LITERAL-COLOR-OK: o número do selo do Miro
    number(
        scene,
        ts,
        text,
        (board ^ BADGE_SALT, el.id.0),
        el.votes(),
        d * BADGE_TEXT,
        c,
        t,
        white,
    );
}

/// ⭐ V2 — por cima: o `+` (e o `−`) de votar de cada objecto da sessão, com os votos da pessoa ao
/// ecrã no `+` (`v` = mundo → ecrã).
fn paint_vote_marks(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut TextCache,
    marks: &[VoteMark],
    v: Affine,
) {
    let grey = Brush::Solid(doc_color(PLUS_GREY));
    let blue = Brush::Solid(doc_color(BADGE_BLUE));
    let white = Color::from_rgba8(255, 255, 255, 255); // LITERAL-COLOR-OK: o sinal branco do Miro
    let sign = |scene: &mut VectorScene, (c, d): ([f64; 2], f64), vertical: bool| {
        let arm = d * 0.22;
        let mut p = BezPath::new();
        p.move_to((c[0] - arm, c[1]));
        p.line_to((c[0] + arm, c[1]));
        if vertical {
            p.move_to((c[0], c[1] - arm));
            p.line_to((c[0], c[1] + arm));
        }
        let s = Stroke::new(d * 0.08).with_caps(ph2d_vector::Cap::Round);
        scene
            .inner_mut()
            .stroke(&s, v, &Brush::Solid(white), None, &p);
    };
    for m in marks {
        let (c, d) = m.plus;
        let disc = Circle::new(Point::new(c[0], c[1]), d / 2.0).to_path(0.1);
        if m.mine == 0 {
            scene.fill_path(&disc, &grey, v);
            sign(scene, m.plus, true);
        } else {
            scene.fill_path(&disc, &blue, v);
            number(
                scene,
                ts,
                text,
                (BADGE_SALT ^ 1, m.id.0),
                m.mine,
                d * BADGE_TEXT,
                c,
                v,
                white,
            );
        }
        if let Some((mc, md)) = m.minus {
            let disc = Circle::new(Point::new(mc[0], mc[1]), md / 2.0).to_path(0.1);
            scene.fill_path(&disc, &grey, v);
            sign(scene, (mc, md), false);
        }
    }
}

impl crate::RenderCache {
    /// V5 — o menor e o maior total do quadro (recontado só quando o documento muda); `None` com os
    /// selos escondidos.
    pub(crate) fn vote_range(&mut self, doc: &ph2d_board_model::BoardDoc) -> Option<[u32; 2]> {
        if self.hide_votes {
            return None;
        }
        match self.vote_range {
            Some((rev, r)) if rev == doc.rev() => r,
            _ => {
                let r = ph2d_board_edit::vote_range(doc);
                self.vote_range = Some((doc.rev(), r));
                r
            }
        }
    }
}

/// ⭐ V2 — os `+`/`−` de votar da sessão aberta, por cima de tudo (`marks` do
/// [`ph2d_board_edit::Editor::vote_marks`]).
pub fn paint_votes(
    board: &ph2d_board_model::Board,
    area: ph2d_board_model::Area,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut crate::RenderCache,
    marks: &[VoteMark],
) {
    if marks.is_empty() {
        return;
    }
    let [x, y, w, h] = area;
    scene.push_clip(&ph2d_vector::Rect::new(x, y, x + w, y + h));
    paint_vote_marks(
        scene,
        ts,
        &mut cache.text,
        marks,
        crate::view(&board.camera, area),
    );
    scene.pop_layer();
}
