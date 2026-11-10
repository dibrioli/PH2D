//! O teclado de notebook medido: o passo para a caixa, a legenda que cabe em cada tecla (`cap` → `cap_short`, as
//! setas em ícone) e o arranjo — `Wide` se toda tecla tem legenda que cabe, senão `Narrow` (plano §13.2). Funções
//! puras da caixa e do texto: a altura pedida e a pintura chegam ao MESMO arranjo e ao MESMO passo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_input::Key;
use ph2d_text::TextSystem;

use super::geometry::{self as g, Arrangement, KeyCap};
use super::keys::{BAR_GAP, BAR_H};

/// A face do hover: a de repouso com o véu do hover do cartão por cima (`KanbanShadowHover`) — mais clara que o
/// repouso no tema escuro, mais funda no claro: sempre MAIS ênfase que o repouso, nunca um contorno.
pub const HOVER_FACE: ph2d_tokens::ColorToken = ph2d_tokens::ColorToken::KanbanShadowHover;

/// A tecla mais baixa: a letra na plaquinha com folga para a pilha de barras (o valor de antes, que a 1280 já cabia).
pub(super) const KEY_MIN_H: f32 =
    m::VALUE_PX + m::SEG_PAD + (BAR_H + BAR_GAP) * 2.0 + m::SEG_INSET * 0.5;

/// A margem mínima de cada lado da legenda, dentro da tecla (a plaquinha nunca passa dela).
pub(crate) const LEGEND_PAD: f32 = m::SEG_INSET * 0.5;

/// A folga da plaquinha («adesivo») à volta da legenda, e o raio dela.
pub(crate) const PLATE_PAD_X: f32 = m::SEG_PAD * 2.0;
pub(crate) const PLATE_PAD_Y: f32 = m::SEG_PAD;
pub(crate) const PLATE_RADIUS: f32 = m::SEG_RADIUS - m::FIELD_BORDER;

/// O recuo das barras às arestas da tecla: de lado, uma aresta dupla; em cima e em baixo, o raio (as barras nunca
/// entram nos cantos redondos).
pub(crate) const BAR_INSET_X: f32 = m::FIELD_BORDER * 2.0;
/// Quanto de cada barra se quer à vista de cada lado da plaquinha.
pub(crate) const BAR_PEEK: f32 = m::SEG_INSET;

/// O menor tamanho de legenda: o menor texto que a app já tem (`TypeToken::Xxs`), nada menor (a porta dos testes).
pub const MIN_LEGEND_PX: f32 = ph2d_tokens::TypeToken::Xxs.px();

/// A faixa vertical que a pilha de barras pode ocupar na tecla de altura `kh` (fora dos cantos redondos).
fn bar_band(kh: f32) -> f32 {
    (kh - m::SEG_RADIUS * 2.0).max(0.0)
}

/// Quantas barras cabem na tecla de altura `kh` à espessura cheia (`BAR_H`, vão `BAR_GAP`). Medido 10/11 nas
/// áreas reais: a tecla mede 25 a 1280 (4 barras cheias) e 26,5–30,5 a 1600 (4–5).
#[must_use]
pub fn bars_full(kh: f32) -> usize {
    ((bar_band(kh) + BAR_GAP) / (BAR_H + BAR_GAP)) as usize
}

/// ⭐ As `n` barras da tecla `r`, de cima para baixo: listras da largura interior, empilhadas e centradas no
/// centro vertical da tecla (o da legenda), por TRÁS da plaquinha. Até [`bars_full`] têm a espessura cheia; além
/// disso, listra e vão encolhem na mesma proporção para a pilha caber na faixa — nunca sai da tecla, nunca empurra
/// a legenda.
#[must_use]
pub fn bar_stack(r: ph2d_editor_core::zones::Rect, n: usize) -> Vec<ph2d_editor_core::zones::Rect> {
    if n == 0 {
        return Vec::new();
    }
    let pitch = BAR_H + BAR_GAP;
    let scale = ((bar_band(r.h) + BAR_GAP) / (pitch * n as f32)).min(1.0);
    let (h, gap) = (BAR_H * scale, BAR_GAP * scale);
    let total = h * n as f32 + gap * (n - 1) as f32;
    let top = r.y + (r.h - total) * 0.5;
    let w = (r.w - BAR_INSET_X * 2.0).max(0.0);
    (0..n)
        .map(|i| {
            ph2d_editor_core::zones::Rect::new(r.x + BAR_INSET_X, top + (h + gap) * i as f32, w, h)
        })
        .collect()
}

/// A plaquinha da legenda `l` (a caixa da legenda) na tecla `kr`: a legenda mais a folga, nunca além de
/// [`LEGEND_PAD`] das arestas (as barras espreitam de cada lado) nem fora da tecla em altura. `icon`: a caixa do
/// ícone já traz a folga dele ([`super::keys::ARROW_BOX`]), a de lado pode ir a zero.
#[must_use]
pub fn plate_of(
    kr: ph2d_editor_core::zones::Rect,
    l: ph2d_editor_core::zones::Rect,
    icon: bool,
) -> ph2d_editor_core::zones::Rect {
    // A folga de lado cede para as barras espreitarem `BAR_PEEK` de cada lado: nas setas e nas letras a 1280 a
    // folga cheia deixava 1–3 px de barra (fotos b1_1600/b1_1280, 10/11).
    let room = (kr.w - BAR_INSET_X * 2.0 - BAR_PEEK * 2.0 - l.w) * 0.5;
    let floor = if icon { 0.0 } else { PLATE_PAD_Y };
    let pad = room.clamp(floor, PLATE_PAD_X);
    let x0 = (l.x - pad).max(kr.x + LEGEND_PAD).min(l.x);
    let x1 = (l.x + l.w + pad)
        .min(kr.x + kr.w - LEGEND_PAD)
        .max(l.x + l.w);
    let y0 = (l.y - PLATE_PAD_Y).max(kr.y);
    let y1 = (l.y + l.h + PLATE_PAD_Y).min(kr.y + kr.h);
    ph2d_editor_core::zones::Rect::new(x0, y0, x1 - x0, (y1 - y0).max(0.0))
}

/// A altura de uma tecla de uma fila para o passo `p`.
pub(super) fn key_height(p: f32) -> f32 {
    (p * g::KEY_H).max(KEY_MIN_H)
}

/// O passo vertical entre filas para o passo `p` (a tecla e o vão).
pub(super) fn row_step(p: f32) -> f32 {
    key_height(p) + p * (g::ROW_PITCH - g::KEY_H)
}

/// A altura de `rows` filas para o passo `p`.
pub(super) fn rows_h(p: f32, rows: f32) -> f32 {
    row_step(p) * rows
}

/// O passo cujas `rows` filas medem `h` (o inverso de [`rows_h`]).
fn pitch_for_rows(h: f32, rows: f32) -> f32 {
    let linear = h / (g::ROW_PITCH * rows);
    if linear * g::KEY_H >= KEY_MIN_H {
        linear
    } else {
        (h / rows - KEY_MIN_H).max(0.0) / (g::ROW_PITCH - g::KEY_H)
    }
}

/// A largura pintada de uma tecla de `span` unidades.
pub(super) fn key_w(span: f32, p: f32) -> f32 {
    (span - (1.0 - g::KEY_FILL)) * p
}

/// A altura pintada de uma tecla de `rows` filas.
pub(super) fn key_h(rows: f32, p: f32) -> f32 {
    key_height(p) + row_step(p) * (rows - 1.0)
}

/// A legenda de uma tecla: texto (a `cap` ou a `cap_short`, ao seu tamanho) ou ícone.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Legend {
    Text(String, f32),
    /// Duas linhas ao mesmo tamanho (a PrtSc dos notebooks: «Prt» / «Sc»).
    Two(String, String, f32),
    Icon(IconId),
}

/// O menor tamanho de texto que a app já tem (`TypeToken::Xxs`), o segundo degrau da palavra.
pub(crate) const fn small_px() -> f32 {
    MIN_LEGEND_PX
}

/// Os grupos que partilham UM degrau de legenda: um grupo nunca mistura palavra e ícone, nem dois tamanhos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// A fila de cima: Esc, F1–F12, Insert, PrtSc, Delete.
    FnRow,
    /// O resto do bloco principal (os modificadores e as teclas da borda; as letras são glifos).
    Edge,
    /// As quatro setas (sempre ícone).
    Arrows,
    /// Home End PgUp PgDn, por cima do numérico.
    Nav,
    /// O numérico.
    Numpad,
}

/// O grupo de uma tecla desenhada.
#[must_use]
pub fn group_of(key: Key) -> Group {
    const F1: u32 = 0xF704;
    match key.0 {
        0x1B | 0xF727 | 0xF72E | 0xF728 => Group::FnRow,
        c if (F1..F1 + 12).contains(&c) => Group::FnRow,
        0xF729 | 0xF72B | 0xF72C | 0xF72D => Group::Nav,
        0xF717 | 0xF740..=0xF74F => Group::Numpad,
        _ if g::arrow_icon(key).is_some() => Group::Arrows,
        _ => Group::Edge,
    }
}

/// Os degraus da legenda, pela ordem: a palavra da foto a `LABEL_PX`, a mesma ao menor tamanho da app, a curta
/// (a `LABEL_PX` e depois ao menor), o símbolo ISO.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Cap,
    CapSmall,
    Short,
    ShortSmall,
    Icon,
}

const LEVELS: [Level; 5] = [
    Level::Cap,
    Level::CapSmall,
    Level::Short,
    Level::ShortSmall,
    Level::Icon,
];

fn word(key: Key, short: bool) -> String {
    let cap = key.cap();
    if cap.is_empty() {
        key.label()
    } else if short {
        key.cap_short().to_owned()
    } else {
        cap.to_owned()
    }
}

/// A legenda da tecla no degrau `level`. Um glifo único fica sempre a `VALUE_PX` (não é palavra).
pub(crate) fn legend_at(key: Key, level: Level) -> Legend {
    if let Some(icon) = g::arrow_icon(key) {
        return Legend::Icon(icon);
    }
    let short = level >= Level::Short;
    let s = word(key, short);
    if s.chars().count() == 1 {
        return Legend::Text(s, m::VALUE_PX);
    }
    match level {
        Level::Cap | Level::Short => Legend::Text(s, m::LABEL_PX),
        Level::CapSmall | Level::ShortSmall => Legend::Text(s, small_px()),
        Level::Icon => iso_icon(key).map_or(Legend::Text(s, small_px()), Legend::Icon),
    }
}

fn fits(ts: &mut TextSystem, l: &Legend, w: f32) -> bool {
    match l {
        Legend::Icon(_) => m::VALUE_PX + LEGEND_PAD * 2.0 <= w,
        Legend::Text(s, px) => ts.prefix_width(s, *px) + LEGEND_PAD * 2.0 <= w,
        Legend::Two(a, b, px) => {
            ts.prefix_width(a, *px).max(ts.prefix_width(b, *px)) + LEGEND_PAD * 2.0 <= w
        }
    }
}

/// Os tamanhos de uma legenda só de pontuação (`` ` `` `'` `,` `.` `;` `/` `-` `=` `[` `]` `\`), do maior ao menor:
/// os glifos dela são minúsculos a `VALUE_PX` e a tecla lia-se vazia; o primeiro que cabe, medido, vence.
pub const PUNCT_PX: [f32; 2] = [
    ph2d_tokens::TypeToken::Md.px(),
    ph2d_tokens::TypeToken::Base.px(),
];

/// A legenda é um só glifo de pontuação?
#[must_use]
pub fn is_punct(s: &str) -> bool {
    let mut c = s.chars();
    matches!((c.next(), c.next()), (Some(ch), None) if ch.is_ascii_punctuation())
}

/// A altura que duas linhas de `px` pedem.
pub(crate) fn two_lines_h(px: f32) -> f32 {
    px * ph2d_tokens::LineHeight::Tight.ratio() * 2.0
}

/// A altura livre para a legenda numa tecla de altura `kh`: a da tecla menos a folga da plaquinha (as barras
/// passam por trás dela e não tiram altura nenhuma).
pub(crate) fn label_h(kh: f32) -> f32 {
    (kh - PLATE_PAD_Y * 2.0).max(0.0)
}

/// A legenda da tecla no degrau `level` que cabe em `w` × `h`: a de uma linha; senão, se a tecla se imprime em
/// duas linhas (a PrtSc), as duas ao mesmo tamanho; senão, só para uma tecla do SISTEMA, o símbolo ISO dela.
pub(crate) fn choose(
    ts: &mut TextSystem,
    key: Key,
    (level, sys_icon): (Level, bool),
    w: f32,
    h: f32,
) -> Option<Legend> {
    let one = legend_at(key, level);
    if let Legend::Text(s, _) = &one
        && is_punct(s)
        && let Some(big) = PUNCT_PX.into_iter().find_map(|px| {
            let l = Legend::Text(s.clone(), px);
            (px * ph2d_tokens::LineHeight::Tight.ratio() <= h && fits(ts, &l, w)).then_some(l)
        })
    {
        return Some(big);
    }
    if fits(ts, &one, w) {
        return Some(one);
    }
    if let (Some((a, b)), Legend::Text(_, px)) = (key.cap_lines(), &one) {
        let two = Legend::Two(a.to_owned(), b.to_owned(), *px);
        if two_lines_h(*px) <= h && fits(ts, &two, w) {
            return Some(two);
        }
    }
    let icon = (iso_icon(key))
        .filter(|_| sys_icon && key.system_owned().is_some())
        .map(Legend::Icon);
    icon.filter(|l| fits(ts, l, w))
}

/// O símbolo ISO/IEC 9995-7 de uma tecla, o último degrau da legenda (a fonte não tem os glifos ⇱ ⇲ ⇞ ⇟ ↵ ⎙).
pub(crate) fn iso_icon(key: Key) -> Option<IconId> {
    Some(match key.0 {
        0xF729 => IconId::KbdHome,
        0xF72B => IconId::KbdEnd,
        0xF72C => IconId::KbdPageUp,
        0xF72D => IconId::KbdPageDown,
        0xF72E => IconId::KbdPrintScreen,
        0xF717 => IconId::KbdNumLock,
        _ if key.is_enter_like() => IconId::KbdEnter,
        _ => return None,
    })
}

/// O degrau do grupo `grp` no arranjo `a` ao passo `p`: o primeiro em que TODA tecla do grupo cabe. O ícone só
/// conta se toda palavra do grupo tem símbolo (senão o grupo misturaria palavra e ícone); `None` = nenhum cabe.
fn group_level(ts: &mut TextSystem, a: Arrangement, p: f32, grp: Group) -> Option<Level> {
    let caps: Vec<KeyCap> = g::keyboard(a)
        .into_iter()
        .filter(|c| group_of(c.key) == grp)
        .collect();
    let words =
        |c: &KeyCap| word(c.key, true).chars().count() > 1 && g::arrow_icon(c.key).is_none();
    let icons_all = caps
        .iter()
        .filter(|c| words(c))
        .all(|c| iso_icon(c.key).is_some());
    // Primeiro sem o símbolo da tecla do sistema (palavras em todo o grupo); só se nenhum degrau serve, com ele.
    [false, true].into_iter().find_map(|sys_icon| {
        LEVELS
            .into_iter()
            // O símbolo só quando toda palavra do grupo tem o seu ISO/IEC 9995-7 (o numérico: ⇭ e ↵).
            .filter(|l| *l != Level::Icon || icons_all)
            .find(|l| {
                caps.iter().all(|c| {
                    let h = label_h(key_h(c.rows, p));
                    choose(ts, c.key, (*l, sys_icon), key_w(c.span, p), h).is_some()
                })
            })
    })
}

const GROUPS: [Group; 5] = [
    Group::FnRow,
    Group::Edge,
    Group::Arrows,
    Group::Nav,
    Group::Numpad,
];

/// O degrau de cada grupo para a escolha `f` (o que não cabe em nenhum fica no último degrau de texto, e o censo
/// regista-o como cortado — nunca se esconde).
#[must_use]
pub fn levels(ts: &mut TextSystem, f: Fit) -> Vec<(Group, Level)> {
    GROUPS
        .into_iter()
        .map(|grp| {
            let l = group_level(ts, f.arrangement, f.pitch, grp).unwrap_or(Level::ShortSmall);
            (grp, l)
        })
        .collect()
}

/// O arranjo escolhido e o passo, para uma caixa de largura `w` (e altura `h`, se a caixa já foi dada).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub arrangement: Arrangement,
    pub pitch: f32,
    /// Toda tecla tem uma legenda que cabe.
    pub legible: bool,
}

fn pitch_of(a: Arrangement, w: f32, h: Option<f32>, cap: f32) -> f32 {
    let (cols, rows) = g::extent(a);
    let p = (w / cols).min(m::FIELD_H * cap);
    h.map_or(p, |h| p.min(pitch_for_rows(h, rows)))
}

fn legible(ts: &mut TextSystem, a: Arrangement, p: f32) -> bool {
    GROUPS
        .into_iter()
        .all(|grp| group_level(ts, a, p, grp).is_some())
}

/// ⭐ A escolha (plano §13.2): `Wide` se toda tecla tem legenda que cabe; senão `Narrow`.
pub(crate) fn fit(ts: &mut TextSystem, w: f32, h: Option<f32>, cap: f32) -> Fit {
    let wide = pitch_of(Arrangement::Wide, w, h, cap);
    if legible(ts, Arrangement::Wide, wide) {
        return Fit {
            arrangement: Arrangement::Wide,
            pitch: wide,
            legible: true,
        };
    }
    let p = pitch_of(Arrangement::Narrow, w, h, cap);
    Fit {
        arrangement: Arrangement::Narrow,
        pitch: p,
        legible: legible(ts, Arrangement::Narrow, p),
    }
}

/// A altura do desenho (sem o enchimento) para a escolha `f`.
pub(super) fn drawing_rows_h(f: Fit) -> f32 {
    rows_h(f.pitch, g::extent(f.arrangement).1)
}

/// A escolha para a largura `w` (pública para os testes e o relatório de medida).
#[must_use]
pub fn fit_for_width(ts: &mut TextSystem, w: f32, cap_b: bool) -> Fit {
    let cap = if cap_b {
        g::PITCH_MAX_ROWS_B
    } else {
        g::PITCH_MAX_ROWS
    };
    fit(ts, w, None, cap)
}

/// As teclas sem legenda que caiba no arranjo `a` à largura `w` (tecto da B ou da C): `(rótulo, largura da tecla)`.
/// A medida que decide o arranjo, exposta para o relatório e os testes.
#[must_use]
pub fn misfits(ts: &mut TextSystem, w: f32, a: Arrangement, cap_b: bool) -> Vec<(String, f32)> {
    let cap = if cap_b {
        g::PITCH_MAX_ROWS_B
    } else {
        g::PITCH_MAX_ROWS
    };
    let p = pitch_of(a, w, None, cap);
    let lost: Vec<Group> = GROUPS
        .into_iter()
        .filter(|grp| group_level(ts, a, p, *grp).is_none())
        .collect();
    g::keyboard(a)
        .into_iter()
        .filter(|c| {
            lost.contains(&group_of(c.key))
                && choose(
                    ts,
                    c.key,
                    (Level::ShortSmall, true),
                    key_w(c.span, p),
                    key_h(c.rows, p),
                )
                .is_none()
        })
        .map(|c| (c.key.label(), key_w(c.span, p)))
        .collect()
}

/// As teclas com a sua caixa relativa à origem do desenho: `(tecla, x, y, w, h)`.
pub(super) fn placed(f: Fit) -> Vec<(KeyCap, f32, f32, f32, f32)> {
    let p = f.pitch;
    g::keyboard(f.arrangement)
        .into_iter()
        .map(|c| {
            let (x, y) = (c.col * p, c.row * row_step(p));
            (c, x, y, key_w(c.span, p), key_h(c.rows, p))
        })
        .collect()
}
