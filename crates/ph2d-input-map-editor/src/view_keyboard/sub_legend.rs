//! ⭐ As SEGUNDAS legendas do numérico (dono, onda 4 item 7a; foto `docs/InputMap/maquetes/06_teclado_notebook_dono.png`):
//! o que a tecla faz com o Num Lock apagado — End ↓ PgDn ← → Home ↑ PgUp Ins Del —, pequena, por baixo do número, mais
//! fraca. A ligação continua pela tecla FÍSICA; a legenda é a `cap_short` da tecla de navegação (a tabela única de
//! `ph2d_input`) ou a seta desenhada. Arranjo da foto: número em cima à esquerda, segunda em baixo à direita. Só onde
//! cabe, MEDIDO, tecla a tecla: sem lugar some inteira (nunca elide).

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::paint_icon;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::Key;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::geometry as g;
use super::keys_fit::{self as fit, LEGEND_MARGIN, Legend};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;
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

/// A caixa da segunda: a largura medida e a ALTURA DO CORPO (`SUB_PX`, sem a entrelinha — o texto centra-se pela
/// altura da maiúscula).
fn sub_size(ts: &mut TextSystem, s: &SubLegend) -> (f32, f32) {
    match s {
        SubLegend::Text(t) => (ts.prefix_width(t, SUB_PX), SUB_PX),
        SubLegend::Icon(_) => (SUB_PX, SUB_PX),
    }
}

/// O numérico no arranjo da foto do dono: o número no canto de cima à esquerda (o 5 também) e a segunda legenda na
/// faixa de baixo, à direita. `/ * - + Enter Num Lock` ficam ao centro.
#[must_use]
pub fn corner(key: Key) -> bool {
    nav_of(key).is_some() || key.0 == KP5
}

const KP5: u32 = 0xF745;

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

/// O arranjo de uma tecla de [`corner`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Corner {
    /// O número (o tamanho pode descer a `VALUE_PX`, o do ponto) e a sua caixa no canto de cima.
    pub legend: Legend,
    pub main: Rect,
    /// A segunda e a sua caixa na faixa de baixo; `None` = não cabe NESTA tecla (largura, ou as barras na faixa).
    pub sub: Option<(SubLegend, Rect)>,
}

/// ⭐ O canto de `key` na tecla `kr`, com as barras `bars` já empilhadas: o número à esquerda, a [`LEGEND_MARGIN`]
/// mais a folga da plaquinha da aresta (a barra fica à vista dos dois lados dela); a segunda à direita na faixa de
/// baixo, medida contra a largura ÚTIL da tecla (menos [`SUB_PAD`] de cada lado, fora da curva dos cantos) — só se nenhuma barra entra
/// na faixa (ligada e sem lugar, as barras vencem). `None` = a tecla não é do canto, ou nem o número cabe em cima.
pub(crate) fn corner_of(
    ts: &mut TextSystem,
    kr: Rect,
    key: Key,
    legend: &Legend,
    bars: &[Rect],
) -> Option<Corner> {
    if !corner(key) {
        return None;
    }
    let Legend::Text(s, px) = legend else {
        return None;
    };
    let px = [*px, m::VALUE_PX]
        .into_iter()
        .find(|p| kr.h - p - SUB_PX >= CORNER_INSET_MIN * 2.0)?;
    let inset = ((kr.h - px - SUB_PX) * 0.5).min(fit::PLATE_PAD_Y);
    let mw = ts.prefix_width(s, px);
    let main = Rect::new(
        kr.x + LEGEND_MARGIN + fit::PLATE_PAD_X,
        kr.y + inset,
        mw,
        px,
    );
    let sub = sub_of(key).and_then(|sl| {
        let (sw, sh) = sub_size(ts, &sl);
        let sr = Rect::new(kr.x + kr.w - SUB_PAD - sw, kr.y + kr.h - inset - sh, sw, sh);
        let clear = bars.iter().all(|b| b.y + b.h <= sr.y || b.y >= sr.y + sr.h);
        (sw + SUB_PAD * 2.0 <= kr.w && clear && clear_of_corners(sr, kr)).then_some((sl, sr))
    });
    Some(Corner {
        legend: Legend::Text(s.clone(), px),
        main,
        sub,
    })
}

/// A tinta da segunda legenda, na FACE da tecla (fora da plaquinha): ligada `Text2` (um degrau abaixo do `Text1` do
/// número); sem acção fica no `Text3` do número, menor. Medido: `TextDisabled` dava 1,77:1 no Oled e lia-se MAIS que
/// o `Text3` no Light e no Candy.
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
    pub sub: Rect,
    pub legend: SubLegend,
    pub ink: ColorToken,
}

/// Pinta a segunda legenda `s` na caixa `sb` e regista-a.
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
            line(&mut doc.census, ts, scene, t, sb, SUB_PX, c);
        }
        SubLegend::Icon(i) => {
            paint_icon(scene, *i, sb, c, StrokeToken::Thin.px());
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
