//! ⭐ As SEGUNDAS legendas do numérico (dono, onda 4 item 7a; foto `docs/InputMap/maquetes/06_teclado_notebook_dono.png`):
//! o que a tecla faz com o Num Lock apagado — End ↓ PgDn ← → Home ↑ PgUp Ins Del —, pequena, por baixo do número, mais
//! fraca. A ligação continua pela tecla FÍSICA; a legenda é a `cap_short` da tecla de navegação (a tabela única de
//! `ph2d_input`) ou a seta desenhada. Só onde cabe, MEDIDO: abaixo disso o grupo inteiro fica sem ela (nunca elide).

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::paint_icon;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::Key;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::geometry as g;
use super::keys_fit::{self as fit, Fit, Group, LEGEND_MARGIN, Legend, Level};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;

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
/// altura da maiúscula, a linha empilhada não precisa do respiro de um parágrafo).
fn sub_size(ts: &mut TextSystem, s: &SubLegend) -> (f32, f32) {
    match s {
        SubLegend::Text(t) => (ts.prefix_width(t, SUB_PX), SUB_PX),
        SubLegend::Icon(_) => (SUB_PX, SUB_PX),
    }
}

/// A legenda principal `main` e a segunda `sub` EMPILHADAS no centro da tecla `kr`: `(caixa do número, caixa da
/// segunda)`, ou `None` se não cabem as duas (em altura, na folga da plaquinha; em largura, com a barra à vista). As
/// duas caixas têm a altura do corpo: com a entrelinha `Tight` (25,3 px) não cabiam em tecla nenhuma das áreas reais.
pub(crate) fn stack(
    ts: &mut TextSystem,
    kr: Rect,
    main: &Legend,
    sub: &SubLegend,
) -> Option<(Rect, Rect)> {
    let mut lb = fit::legend_box(ts, kr, main);
    if let Legend::Text(_, px) = main {
        lb.h = *px;
    }
    let (sw, sh) = sub_size(ts, sub);
    let total = lb.h + sh;
    if total > fit::label_h(kr.h) || sw + LEGEND_MARGIN * 2.0 > kr.w {
        return None;
    }
    let top = kr.y + (kr.h - total) * 0.5;
    let s = Rect::new(kr.x + (kr.w - sw) * 0.5, top + lb.h, sw, sh);
    Some((Rect::new(lb.x, top, lb.w, lb.h), s))
}

/// ⭐ O numérico mostra as segundas legendas ao passo de `f`? Só se TODA tecla com uma a tem inteira (o grupo nunca
/// mistura teclas com e sem).
#[must_use]
pub fn group_shows(ts: &mut TextSystem, f: Fit) -> bool {
    let level = fit::levels(ts, f)
        .into_iter()
        .find(|(gr, _)| *gr == Group::Numpad)
        .map_or(Level::ShortSmall, |l| l.1);
    fit::placed(f).into_iter().all(|(c, x, y, w, h)| {
        let Some(sub) = sub_of(c.key) else {
            return true;
        };
        let kr = Rect::new(x, y, w, h);
        let main = fit::choose(ts, c.key, (level, true), w, fit::label_h(h))
            .unwrap_or_else(|| fit::legend_at(c.key, level));
        stack(ts, kr, &main, &sub).is_some()
    })
}

/// A tinta da segunda legenda: um degrau mais fraca que a do número (medido nos 12 temas, `onda4_numerico`).
#[must_use]
pub fn sub_ink(main: ColorToken) -> ColorToken {
    match main {
        ColorToken::Text1 => ColorToken::Text2,
        t if t == fit::LIT_INK => ColorToken::Bg3,
        // Sem acção o número já está no degrau mais fraco que se lê (`Text3`): a segunda fica nele, menor. Medido:
        // `TextDisabled` dava 1,77:1 no Oled e lia-se MAIS que o `Text3` no Light e no Candy.
        other => other,
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
    (s, sb): (&SubLegend, Rect),
) {
    let tok = sub_ink(main_ink);
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
