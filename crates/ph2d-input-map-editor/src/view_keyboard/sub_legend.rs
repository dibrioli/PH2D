//! ⭐ As SEGUNDAS legendas do numérico (dono, onda 4 item 7a; foto `docs/InputMap/maquetes/06_teclado_notebook_dono.png`):
//! o que a tecla faz com o Num Lock apagado — End ↓ PgDn ← → Home ↑ PgUp Ins Del —, pequena, por baixo do número, mais
//! fraca. A ligação continua pela tecla FÍSICA; a legenda é a `cap_short` da tecla de navegação (a tabela única de
//! `ph2d_input`) ou a seta desenhada. Arranjo da foto: número em cima à esquerda, segunda em baixo à direita. As de
//! TEXTO são um conjunto (todas ou nenhuma: manda a mais larga, «Home»), as SETAS outro; a única exceção por tecla é a
//! tecla ligada com barra na faixa de baixo (as barras vencem). Nunca elide.

use ph2d_editor_core::icons::{IconId, cmd_to_path};
use ph2d_editor_core::paint::{icon_side_for_glyph, paint_icon};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::Key;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::{BezPath, Shape, VectorScene};

use super::geometry as g;
use super::keys_fit::{self as fit, Fit, LEGEND_MARGIN, Legend};
use crate::doc::InputMapDoc;
use crate::view_list::text::{body, body_width};
use ph2d_card_stack::metrics as m;

/// O tamanho da segunda legenda: o menor texto da app (`TypeToken::Xxs`), abaixo do número (`VALUE_PX`).
pub const SUB_PX: f32 = fit::MIN_LEGEND_PX;

/// Numpad 0..9 e o ponto → a tecla de navegação que o notebook do dono imprime por baixo (Num Lock apagado).
const NUMPAD_NAV: [(u32, u32); 10] = [
    (0xF740, 0xF727), // 0 → Insert
    (0xF741, 0xF72B), // 1 → End
    (0xF742, 0xF701), // 2 → ↓
    (0xF743, 0xF72D), // 3 → PgDn
    (0xF744, 0xF702), // 4 → ←
    (0xF746, 0xF703), // 6 → →
    (0xF747, 0xF729), // 7 → Home
    (0xF748, 0xF700), // 8 → ↑
    (0xF749, 0xF72C), // 9 → PgUp
    (0xF74A, 0xF728), // . → Delete
];

const KP5: u32 = 0xF745;

/// A tecla de navegação impressa por baixo de `key` (`None` fora do numérico, e no 5).
#[must_use]
pub fn nav_of(key: Key) -> Option<Key> {
    NUMPAD_NAV
        .iter()
        .find(|(kp, _)| *kp == key.0)
        .map(|(_, nav)| Key(*nav))
}

/// A segunda legenda: a palavra curta da tabela ou a seta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubLegend {
    Text(String),
    Icon(IconId),
}

/// A segunda legenda de `key`, se a tecla tem uma.
#[must_use]
pub fn sub_of(key: Key) -> Option<SubLegend> {
    let nav = nav_of(key)?;
    Some(g::arrow_icon(nav).map_or_else(
        || SubLegend::Text(nav.cap_short().to_owned()),
        SubLegend::Icon,
    ))
}

/// ⭐ A altura das MAIÚSCULAS da fonte activa a `px`, lida do [`ph2d_text::cap_centre_px`] (topo → meio das
/// maiúsculas `= round(subida) − maiúscula/2`): a inclinação em `px` dá a maiúscula. Entre dois intervalos vizinhos
/// fica o que não salta de píxel (o arredondamento da subida).
#[must_use]
pub fn cap_height_px(px: f32) -> f32 {
    let step = m::FIELD_BORDER * 0.1; // LITERAL-PX-OK: passo de diferença finita, um décimo de píxel
    let d = ph2d_text::displayed_font_px;
    let slope = |a: f32, b: f32| {
        -2.0 * (ph2d_text::cap_centre_px(b) - ph2d_text::cap_centre_px(a)) / (d(b) - d(a))
    };
    let (s1, s2) = (slope(px - step, px), slope(px, px + step));
    let k = if (0.0..1.0).contains(&s1) { s1 } else { s2 };
    k * d(px)
}

/// A caixa da TINTA do ícone na viewbox 24×24, em fracção do lado do desenho.
fn ink_frac(icon: IconId) -> (f32, f32, f32, f32) {
    let mut p = BezPath::new();
    for c in icon.cmds() {
        p.extend(cmd_to_path(*c));
    }
    let b = p.bounding_box();
    let v = 24.0; // LITERAL-PX-OK: a viewbox dos ícones da casa
    (
        (b.x0 / v) as f32,
        (b.y0 / v) as f32,
        (b.width() / v) as f32,
        (b.height() / v) as f32,
    )
}

/// ⭐ O lado do DESENHO da seta para que a tinta dela tenha o tamanho da maiúscula do texto secundário (o peso visual
/// de «End»): a dimensão maior da tinta = [`cap_height_px`]`(SUB_PX)`.
fn arrow_drawn(icon: IconId) -> f32 {
    let (_, _, fw, fh) = ink_frac(icon);
    cap_height_px(SUB_PX) / fw.max(fh)
}

/// A caixa da tinta da segunda (o que ocupa a faixa): o texto, a largura medida e a altura do corpo; a seta, a tinta.
fn sub_size(ts: &mut TextSystem, s: &SubLegend) -> (f32, f32) {
    match s {
        SubLegend::Text(t) => (body_width(ts, t, SUB_PX), cap_height_px(SUB_PX)),
        SubLegend::Icon(i) => {
            let (_, _, fw, fh) = ink_frac(*i);
            let dr = arrow_drawn(*i);
            (fw * dr, fh * dr)
        }
    }
}

/// O quadrado onde pintar a seta para que a sua tinta caia em `ink` (o inverso do encaixe com o respiro do ícone).
fn icon_rect(icon: IconId, ink: Rect) -> Rect {
    let dr = arrow_drawn(icon);
    let side = icon_side_for_glyph(dr);
    let (fx, fy, _, _) = ink_frac(icon);
    let pad = (side - dr) * 0.5;
    Rect::new(ink.x - pad - fx * dr, ink.y - pad - fy * dr, side, side)
}

/// O numérico no arranjo da foto do dono: o número no canto de cima à esquerda (o 5 também) e a segunda legenda na
/// faixa de baixo, à direita. `/ * - + Enter Num Lock` ficam ao centro.
#[must_use]
pub fn corner(key: Key) -> bool {
    nav_of(key).is_some() || key.0 == KP5
}

/// O recuo mínimo de cima e de baixo no canto: a aresta da tecla. Medido (onda 4): número 12 + segunda 10 = 22 px; a
/// tecla mede 25 a 1280 (o `KEY_MIN_H`) e 26,5–30,5 a 1600 — cabe sempre com ≥ 1,5 px.
const CORNER_INSET_MIN: f32 = m::FIELD_BORDER;

/// A folga da segunda à aresta, de cada lado: a mesma barra à vista que a plaquinha do número deixa à esquerda
/// ([`fit::BAR_VISIBLE_MIN`], 2 px). Na faixa de baixo não há barras: a folga cheia ([`fit::LEGEND_PAD`]) não se paga.
pub const SUB_PAD: f32 = fit::BAR_VISIBLE_MIN;

/// O rectângulo `r` fica fora da curva dos cantos de baixo da tecla `kr` (raio `SEG_RADIUS`, o da face pintada)?
#[must_use]
pub fn clear_of_corners(r: Rect, kr: Rect) -> bool {
    let rad = m::SEG_RADIUS;
    let dy = (kr.y + kr.h) - (r.y + r.h);
    [r.x - kr.x, (kr.x + kr.w) - (r.x + r.w)]
        .into_iter()
        .all(|dx| {
            let (cx, cy) = ((rad - dx).max(0.0), (rad - dy).max(0.0));
            dx >= 0.0 && dy >= 0.0 && cx * cx + cy * cy <= rad * rad
        })
}

/// Que conjuntos de segundas legendas o numérico mostra na última pintura.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubSets {
    /// As de texto (End PgDn Home PgUp Ins Del): todas ou nenhuma.
    pub text: bool,
    /// As setas (↑ ↓ ← →): todas ou nenhuma.
    pub icons: bool,
}

/// O tamanho do número do canto: o `Md` da casa (15 px). MEDIDO (onda 4, revisão): maiúscula 10,71 px contra os
/// 7,14 da segunda a `SUB_PX` = 67 % (alvo ≤ 75 %); o `Base` (13 px) dava 77 % e o `VALUE_PX` 83 %.
pub const CORNER_NUMBER_PX: f32 = ph2d_tokens::TypeToken::Md.px();

/// O vão mínimo entre a tinta do número e a da segunda.
const CORNER_GAP: f32 = m::SEG_PAD;

/// A geometria do canto pela TINTA: `(x do número, topo da tinta do número, topo da maiúscula da segunda, fundo da
/// faixa)`. `None` = a tecla não tem a altura: recuo + maiúscula do número + vão + maiúscula e descendente da segunda
/// + recuo (24,7 px; a tecla mede 25 a 1280 e 26,7 a 1600).
fn geometry(kr: Rect) -> Option<(f32, f32, f32, f32)> {
    let (cap_n, cap_s) = (cap_height_px(CORNER_NUMBER_PX), cap_height_px(SUB_PX));
    let desc = SUB_PX - cap_s;
    let need = CORNER_INSET_MIN * 2.0 + cap_n + CORNER_GAP + cap_s + desc;
    if kr.h < need {
        return None;
    }
    let inset =
        (CORNER_INSET_MIN + (kr.h - need) * 0.5).min(fit::PLATE_PAD_Y.max(CORNER_INSET_MIN));
    let bottom = kr.y + kr.h - inset;
    let x = kr.x + LEGEND_MARGIN + fit::PLATE_PAD_X;
    Some((x, kr.y + inset, bottom - desc - cap_s, bottom))
}

/// A caixa de TINTA `(w, h)` à direita na faixa de baixo (maiúscula a partir de `cap_top`, até `bottom`), e se cabe:
/// a folga [`SUB_PAD`] à direita, nunca a começar à esquerda do número (`num_x`), fora da curva dos cantos.
fn band(kr: Rect, (num_x, cap_top, bottom): (f32, f32, f32), (w, h): (f32, f32)) -> (Rect, bool) {
    let cap_s = cap_height_px(SUB_PX);
    let r = Rect::new(kr.x + kr.w - SUB_PAD - w, cap_top + (cap_s - h) * 0.5, w, h);
    let whole = Rect::new(r.x, cap_top.min(r.y), w, bottom - cap_top.min(r.y));
    (r, r.x >= num_x - 0.01 && clear_of_corners(whole, kr))
}

/// ⭐ Os conjuntos ao passo de `f`: o de texto se a MAIS LARGA das palavras cabe em TODA tecla com palavra; o das setas
/// se a maior seta cabe em toda tecla com seta. Nunca um subconjunto (a 1280, End/Ins/Del sem Home liam-se como erro).
#[must_use]
pub fn sets(ts: &mut TextSystem, f: Fit) -> SubSets {
    let mut widest = (0.0_f32, 0.0_f32, 0.0_f32);
    for (kp, _) in NUMPAD_NAV {
        if let Some(s) = sub_of(Key(kp)) {
            let (w, h) = sub_size(ts, &s);
            match s {
                SubLegend::Text(_) => widest.0 = widest.0.max(w),
                SubLegend::Icon(_) => (widest.1, widest.2) = (widest.1.max(w), widest.2.max(h)),
            }
        }
    }
    let mut out = SubSets {
        text: true,
        icons: true,
    };
    for (c, x, y, w, h) in fit::placed(f) {
        let Some(s) = sub_of(c.key) else {
            continue;
        };
        let kr = Rect::new(x, y, w, h);
        let Some((x, _, cap_top, bottom)) = geometry(kr) else {
            return SubSets::default();
        };
        let at = (x, cap_top, bottom);
        match s {
            SubLegend::Text(_) => out.text &= band(kr, at, (widest.0, cap_height_px(SUB_PX))).1,
            SubLegend::Icon(_) => out.icons &= band(kr, at, (widest.1, widest.2)).1,
        }
    }
    out
}

/// O arranjo de uma tecla de [`corner`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Corner {
    /// O número (o tamanho pode descer a `VALUE_PX`, o do ponto) e a sua caixa no canto de cima.
    pub legend: Legend,
    pub main: Rect,
    /// A segunda e a caixa da sua TINTA na faixa de baixo; `None` = o conjunto dela está fora, ou as barras desta
    /// tecla ligada entram na faixa.
    pub sub: Option<(SubLegend, Rect)>,
}

/// ⭐ O canto de `key` na tecla `kr`, com as barras `bars` já empilhadas: o número à esquerda, a [`LEGEND_MARGIN`]
/// mais a folga da plaquinha da aresta (a barra fica à vista dos dois lados dela); a segunda à direita na faixa de
/// baixo, se o seu conjunto está em `on` e nenhuma barra entra na faixa (ligada e sem lugar, as barras vencem).
pub(crate) fn corner_of(
    ts: &mut TextSystem,
    (kr, key): (Rect, Key),
    legend: &Legend,
    bars: &[Rect],
    on: SubSets,
) -> Option<Corner> {
    if !corner(key) {
        return None;
    }
    let (x, top, cap_top, bottom) = geometry(kr)?;
    let Legend::Text(s, _) = legend else {
        return None;
    };
    let px = CORNER_NUMBER_PX;
    let main = Rect::new(x, top, ts.prefix_width(s, px), cap_height_px(px));
    let sub = sub_of(key).and_then(|sl| {
        let set = match sl {
            SubLegend::Text(_) => on.text,
            SubLegend::Icon(_) => on.icons,
        };
        let (sr, fits) = band(kr, (x, cap_top, bottom), sub_size(ts, &sl));
        let clear = bars.iter().all(|b| b.y + b.h <= cap_top || b.y >= bottom);
        (set && fits && clear).then_some((sl, sr))
    });
    Some(Corner {
        legend: Legend::Text(s.clone(), px),
        main,
        sub,
    })
}

/// A tinta da segunda legenda, na FACE da tecla: ligada `Text2` (um degrau abaixo do `Text1` do número, ≥ 5,46:1);
/// sem acção fica no `Text3` do número — o degrau mais fraco é o peso Regular e o tamanho (67 %). MEDIDO: nenhum token
/// de texto da casa fica abaixo do `Text3` nos 12 temas (`TextDisabled` passa-o no Light e no Candy, `CardLabel` no
/// Sunstone), e no Oled o próprio número é 2,62:1.
#[must_use]
pub fn sub_ink(bound: bool) -> ColorToken {
    if bound {
        ColorToken::Text2
    } else {
        ColorToken::Text3
    }
}

/// O que a última pintura pôs como segunda legenda (o censo dos testes).
#[derive(Clone, Debug, PartialEq)]
pub struct SubLook {
    pub key: Key,
    pub key_rect: Rect,
    pub main: Rect,
    pub main_px: f32,
    pub main_ink: ColorToken,
    pub plate: Rect,
    /// A caixa da TINTA (o texto: a largura e o corpo; a seta: o traço).
    pub sub: Rect,
    pub legend: SubLegend,
    pub ink: ColorToken,
}

/// Pinta a segunda legenda `s` na caixa de tinta `sb` e regista-a.
pub(super) fn paint(
    doc: &mut InputMapDoc,
    (ts, scene, ink): (&mut TextSystem, &mut VectorScene, CardInk),
    (key, kr, plate): (Key, Rect, Rect),
    (main, main_px, main_ink): (Rect, f32, ColorToken),
    (s, sb, bound): (&SubLegend, Rect, bool),
) {
    let tok = sub_ink(bound);
    let c = ink.color(tok);
    match s {
        SubLegend::Text(t) => {
            // Regular (o corpo da casa), um degrau abaixo do peso do número.
            body(&mut doc.census, ts, scene, t, sb, SUB_PX, c);
        }
        SubLegend::Icon(i) => {
            // O traço das setas das teclas (`keys::paint_key`): o mesmo peso que a palavra ao lado.
            paint_icon(scene, *i, icon_rect(*i, sb), c, StrokeToken::Default.px());
            doc.census.icons.push((sb, *i));
        }
    }
    doc.kb.subs.push(SubLook {
        key,
        key_rect: kr,
        main,
        main_px,
        main_ink,
        plate,
        sub: sb,
        legend: s.clone(),
        ink: tok,
    });
}
