//! O teclado de notebook medido: o passo para a caixa, a legenda que cabe em cada tecla (`cap` → `cap_short`, as
//! setas em ícone) e o passo mínimo, abaixo do qual o desenho rola na horizontal (plano §13.2, dono 10/11). Funções
//! puras da caixa e do texto: a altura pedida e a pintura chegam ao MESMO passo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_input::Key;
use ph2d_text::TextSystem;

use super::geometry::{self as g, KeyCap};
use super::keys::{ARROW_BOX, BAR_GAP, BAR_H};

/// A face do hover: a de repouso com o véu do hover do cartão por cima (`KanbanShadowHover`) — mais clara que o
/// repouso no tema escuro, mais funda no claro: sempre MAIS ênfase que o repouso, nunca um contorno.
pub const HOVER_FACE: ph2d_tokens::ColorToken = ph2d_tokens::ColorToken::KanbanShadowHover;

/// O corpo da tecla APERTADA (premida ou acesa no Try it): o de REPOUSO — nunca a cor de uma acção nem um token de
/// dado de outra família; a plaquinha acesa marca o apertado. Medido (`onda3_tracos::
/// nenhum_neutro_de_cromo_cumpre_a_lei_do_premido`): nenhum neutro de cromo deixa cada barra ≥ max(3:1, repouso)
/// nos 12 temas × 12 cores (o melhor, `PanelBg`, falha 12 de 144).
pub const PRESS_BODY: ph2d_tokens::ColorToken = ph2d_tokens::ColorToken::KanbanCardRaised;

/// O vão do corpo da tecla entre as barras e a plaquinha ACESA (a mesma folga da plaquinha em repouso).
pub const LIT_GAP: f32 = m::SEG_PAD;

/// A plaquinha ACESA `plate` (a da legenda `l`): `(halo, aceso)` — a parte acesa recua dos lados até [`LIT_GAP`] sem
/// entrar na legenda, e o que faltar o halo (do corpo da tecla, por cima das barras) cresce para fora: entre as barras
/// da mesma cor e a acesa fica sempre o vão.
#[must_use]
pub fn lit_plate(plate: Rect, l: Rect) -> (Rect, Rect) {
    let x0 = (plate.x + LIT_GAP).min(l.x);
    let x1 = (plate.x + plate.w - LIT_GAP).max(l.x + l.w);
    let (out0, out1) = (LIT_GAP - (x0 - plate.x), LIT_GAP - (plate.x + plate.w - x1));
    let halo = Rect::new(plate.x - out0, plate.y, plate.w + out0 + out1, plate.h);
    (halo, Rect::new(x0, plate.y, (x1 - x0).max(0.0), plate.h))
}

/// A tecla mais baixa: a letra na plaquinha com folga para a pilha de barras (o valor de antes, que a 1280 já cabia).
pub const KEY_MIN_H: f32 = m::VALUE_PX + m::SEG_PAD + (BAR_H + BAR_GAP) * 2.0 + m::SEG_INSET * 0.5;

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
pub const BAR_PEEK: f32 = m::SEG_INSET;

/// Quanto de cada barra tem de ficar à vista de cada lado da plaquinha: duas arestas (2 px) — abaixo disso a cor da
/// acção some atrás do adesivo (as capturas do dono, 10/11). Com 3 px o piso medido subia a 33,5 e a área real a 1280
/// (630 px de desenho) já rolava.
pub const BAR_VISIBLE_MIN: f32 = m::FIELD_BORDER * 2.0;

/// ⭐ O passo mínimo da tecla (dono 10/11: «dê um limite ao achatamento»). MEDIDO por [`measured_floor`]
/// (`teclado_piso::o_passo_minimo_e_o_medido_nos_dois_modos_de_texto`, fontes embutidas): `Default` 30,5 px,
/// `CrispHeavyPlus` 31,5 px (barra à vista 2,16 px) — o maior vale. Nele todo grupo tem um degrau de legenda que cabe
/// com [`BAR_VISIBLE_MIN`] de barra de cada lado, a tecla tem `KEY_MIN_H` e a proporção dela; o desenho mede
/// 603 px (cabe nos 630 da área real a 1280). Acima, o teclado enche a caixa; abaixo, fica neste passo e a caixa ROLA.
pub const PITCH_FLOOR: f32 = 31.5; // LITERAL-PX-OK: medido por measured_floor, ver o doc

/// A faixa por baixo do desenho que segura a barra horizontal quando o teclado rola.
pub(super) const HBAR_BAND: f32 = m::BAR_TRACK + m::SEG_INSET;

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
pub fn bar_stack(r: Rect, n: usize) -> Vec<Rect> {
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
        .map(|i| Rect::new(r.x + BAR_INSET_X, top + (h + gap) * i as f32, w, h))
        .collect()
}

/// A caixa das barras de um alvo de raio `radius` (o comando, o chip do «Also:»): o raio da tecla já sai da faixa
/// de [`bar_stack`]; um raio maior (pílula, redondo) recua ao retângulo inscrito — as barras nunca saem da curva.
#[must_use]
pub fn bar_box(r: Rect, radius: f32) -> Rect {
    if radius <= m::SEG_RADIUS {
        return r;
    }
    let d = radius * (1.0 - std::f32::consts::FRAC_1_SQRT_2);
    Rect::new(
        r.x + d,
        r.y + d,
        (r.w - d * 2.0).max(0.0),
        (r.h - d * 2.0).max(0.0),
    )
}

/// A plaquinha da legenda `l` (a caixa da legenda) na tecla `kr`: a legenda mais a folga, nunca além de
/// [`LEGEND_PAD`] das arestas (as barras espreitam de cada lado) nem fora da tecla em altura. `icon`: a caixa do
/// ícone já traz a folga dele ([`super::keys::ARROW_BOX`]), a de lado pode ir a zero.
#[must_use]
pub fn plate_of(kr: Rect, l: Rect, icon: bool) -> Rect {
    // A folga de lado cede para as barras espreitarem `BAR_PEEK` de cada lado: nas setas e nas letras a 1280 a
    // folga cheia deixava 1–3 px de barra (fotos b1_1600/b1_1280, 10/11).
    let room = (kr.w - BAR_INSET_X * 2.0 - BAR_PEEK * 2.0 - l.w) * 0.5;
    // Sem lugar, a folga cede até zero: a barra à vista vence a folga do adesivo.
    let floor = if icon {
        0.0
    } else {
        PLATE_PAD_Y.min(room.max(0.0))
    };
    let pad = room.clamp(floor, PLATE_PAD_X);
    let x0 = (l.x - pad).max(kr.x + LEGEND_PAD).min(l.x);
    let x1 = (l.x + l.w + pad)
        .min(kr.x + kr.w - LEGEND_PAD)
        .max(l.x + l.w);
    let y0 = (l.y - PLATE_PAD_Y).max(kr.y);
    let y1 = (l.y + l.h + PLATE_PAD_Y).min(kr.y + kr.h);
    Rect::new(x0, y0, x1 - x0, (y1 - y0).max(0.0))
}

/// ⭐ A fila de cima (Esc F1–F12 Insert PrtSc Delete) e a Home End PgUp PgDn em MEIA altura, como na foto do dono:
/// a altura da tecla de uma fila vezes este fator. MEDIDO por [`measured_fn_factor`] (`onda3_tracos`): o menor
/// fator em que, nos dois modos de texto e em todo passo desde o [`PITCH_FLOOR`], a legenda cabe em altura e
/// largura com [`BAR_VISIBLE_MIN`] de barra de cada lado e uma barra cheia cabe por trás. Tabela no plano §14.
pub const FN_ROW_FACTOR: f32 = 0.64; // LITERAL-PX-OK: medido por measured_fn_factor, ver o doc

/// A tecla é da fila de meia altura (a de cima: [`Group::FnRow`] e [`Group::Nav`])?
#[must_use]
pub fn half_row(key: Key) -> bool {
    matches!(group_of(key), Group::FnRow | Group::Nav)
}

/// A altura de uma tecla de uma fila para o passo `p`.
pub(super) fn key_height(p: f32) -> f32 {
    (p * g::KEY_H).max(KEY_MIN_H)
}

/// O vão entre filas para o passo `p`.
fn row_gap(p: f32) -> f32 {
    p * (g::ROW_PITCH - g::KEY_H)
}

/// O passo vertical entre filas para o passo `p` (a tecla e o vão).
pub(super) fn row_step(p: f32) -> f32 {
    key_height(p) + row_gap(p)
}

/// A altura das teclas da fila de cima (meia altura, fator `k`).
fn half_h(p: f32, k: f32) -> f32 {
    key_height(p) * k
}

/// O topo da fila `row` (a 0 é a de meia altura).
fn row_y(row: f32, p: f32) -> f32 {
    if row < 1.0 {
        0.0
    } else {
        half_h(p, FN_ROW_FACTOR) + row_gap(p) + row_step(p) * (row - 1.0)
    }
}

/// A altura pintada da tecla `c` ao passo `p` com o fator `k` da fila de cima.
fn cap_h_at(c: &KeyCap, p: f32, k: f32) -> f32 {
    if half_row(c.key) {
        half_h(p, k)
    } else {
        key_h(c.rows, p)
    }
}

/// A altura de `rows` filas para o passo `p` (a primeira em meia altura).
pub(super) fn rows_h(p: f32, rows: f32) -> f32 {
    half_h(p, FN_ROW_FACTOR) + row_gap(p) + row_step(p) * (rows - 1.0)
}

/// O passo cujas `rows` filas medem `h` (o inverso de [`rows_h`]).
fn pitch_for_rows(h: f32, rows: f32) -> f32 {
    let keys = FN_ROW_FACTOR + rows - 1.0;
    let gap = g::ROW_PITCH - g::KEY_H;
    let linear = h / (g::KEY_H * keys + gap * rows);
    if linear * g::KEY_H >= KEY_MIN_H {
        linear
    } else {
        (h - KEY_MIN_H * keys).max(0.0) / (gap * rows)
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

/// A margem de cada lado da legenda que a faz «caber»: o recuo das barras e a barra à vista ([`BAR_VISIBLE_MIN`]) —
/// a legenda nunca tapa a pilha de barras inteira (dono 10/11).
pub(crate) const LEGEND_MARGIN: f32 = BAR_INSET_X + BAR_VISIBLE_MIN;

fn fits(ts: &mut TextSystem, l: &Legend, w: f32) -> bool {
    match l {
        Legend::Icon(_) => m::VALUE_PX + LEGEND_MARGIN * 2.0 <= w,
        Legend::Text(s, px) => ts.prefix_width(s, *px) + LEGEND_MARGIN * 2.0 <= w,
        Legend::Two(a, b, px) => {
            ts.prefix_width(a, *px).max(ts.prefix_width(b, *px)) + LEGEND_MARGIN * 2.0 <= w
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
    // A legenda cabe também em ALTURA (a fila de meia altura): o texto à sua linha, o símbolo da altura da letra.
    let tall = match &one {
        Legend::Text(_, px) => px * ph2d_tokens::LineHeight::Tight.ratio() <= h,
        Legend::Icon(_) => m::VALUE_PX <= h,
        Legend::Two(..) => true,
    };
    if let Legend::Text(s, _) = &one
        && is_punct(s)
        && let Some(big) = PUNCT_PX.into_iter().find_map(|px| {
            let l = Legend::Text(s.clone(), px);
            (px * ph2d_tokens::LineHeight::Tight.ratio() <= h && fits(ts, &l, w)).then_some(l)
        })
    {
        return Some(big);
    }
    if tall && fits(ts, &one, w) {
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
    icon.filter(|l| m::VALUE_PX <= h && fits(ts, l, w))
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

/// O degrau do grupo `grp` ao passo `p`: o primeiro em que TODA tecla do grupo cabe. O ícone só
/// conta se toda palavra do grupo tem símbolo (senão o grupo misturaria palavra e ícone); `None` = nenhum cabe.
fn group_level(ts: &mut TextSystem, p: f32, grp: Group) -> Option<Level> {
    group_level_at(ts, p, grp, FN_ROW_FACTOR)
}

/// [`group_level`] com o fator `k` da fila de cima (a medida de [`measured_fn_factor`]).
fn group_level_at(ts: &mut TextSystem, p: f32, grp: Group, k: f32) -> Option<Level> {
    let caps: Vec<KeyCap> = g::keyboard()
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
                    let h = label_h(cap_h_at(c, p, k));
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
            let l = group_level(ts, f.pitch, grp).unwrap_or(Level::ShortSmall);
            (grp, l)
        })
        .collect()
}

/// O passo escolhido para uma caixa de largura `w` (e altura `h`, se a caixa já foi dada).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub pitch: f32,
    /// Toda tecla tem uma legenda que cabe.
    pub legible: bool,
    /// A caixa é mais estreita que o desenho ao [`PITCH_FLOOR`]: o teclado rola na horizontal.
    pub scrolls: bool,
}

fn pitch_of(w: f32, h: Option<f32>, cap: f32) -> f32 {
    let (cols, rows) = g::extent();
    let p = (w / cols).min(m::FIELD_H * cap);
    h.map_or(p, |h| p.min(pitch_for_rows(h, rows)))
}

fn legible(ts: &mut TextSystem, p: f32) -> bool {
    GROUPS
        .into_iter()
        .all(|grp| group_level(ts, p, grp).is_some())
}

/// A caixa da legenda `l` centrada na tecla `kr` (o mesmo centro com 0 ou N barras).
pub(super) fn legend_box(ts: &mut TextSystem, kr: Rect, l: &Legend) -> Rect {
    let (cx, cy) = (kr.x + kr.w * 0.5, kr.y + kr.h * 0.5);
    let (w, h) = match l {
        Legend::Icon(_) => {
            let s = ARROW_BOX
                .min(label_h(kr.h))
                .min(kr.w - LEGEND_MARGIN * 2.0)
                .max(0.0);
            (s, s)
        }
        Legend::Text(s, px) => (
            ts.prefix_width(s, *px).min(kr.w),
            px * ph2d_tokens::LineHeight::Tight.ratio(),
        ),
        Legend::Two(a, b, px) => (
            ts.prefix_width(a, *px)
                .max(ts.prefix_width(b, *px))
                .min(kr.w),
            two_lines_h(*px),
        ),
    };
    Rect::new(cx - w * 0.5, cy - h * 0.5, w, h)
}

/// A menor barra à vista ao lado da plaquinha, em toda tecla desenhada ao passo `p` (o que a pintura faria).
#[must_use]
pub fn bar_visible_at(ts: &mut TextSystem, p: f32) -> f32 {
    let f = Fit {
        pitch: p,
        legible: true,
        scrolls: false,
    };
    let lv = levels(ts, f);
    let mut least = f32::MAX;
    for (c, x, y, w, h) in placed(f) {
        let kr = Rect::new(x, y, w, h);
        let level = lv
            .iter()
            .find(|(gr, _)| *gr == group_of(c.key))
            .map_or(Level::ShortSmall, |l| l.1);
        let legend = choose(ts, c.key, (level, true), w, label_h(h))
            .unwrap_or_else(|| legend_at(c.key, level));
        let plate = plate_of(
            kr,
            legend_box(ts, kr, &legend),
            matches!(legend, Legend::Icon(_)),
        );
        least = least.min(plate.x - (kr.x + BAR_INSET_X));
    }
    least
}

/// O passo serve: todo grupo tem legenda e toda tecla deixa [`BAR_VISIBLE_MIN`] de barra de cada lado.
#[must_use]
pub fn fits_at(ts: &mut TextSystem, p: f32) -> bool {
    legible(ts, p) && bar_visible_at(ts, p) >= BAR_VISIBLE_MIN - f32::EPSILON
}

/// A medida do [`PITCH_FLOOR`]: o menor passo (de meio em meio px) a partir do qual todo passo maior serve.
#[must_use]
pub fn measured_floor(ts: &mut TextSystem) -> f32 {
    let top = m::FIELD_H * g::PITCH_MAX_ROWS_B;
    let mut p = top;
    while p - 0.5 >= m::VALUE_PX && fits_at(ts, p - 0.5) {
        p -= 0.5;
    }
    p
}

/// A medida do [`FN_ROW_FACTOR`]: o menor fator (de centésimo em centésimo, desde meia tecla) com que a fila de cima
/// serve em todo passo do [`PITCH_FLOOR`] ao tecto da B (de 2 em 2 px): os grupos dela têm um degrau de legenda que
/// cabe em altura e em largura (com [`BAR_VISIBLE_MIN`] de barra de cada lado) e uma barra cheia cabe por trás.
#[must_use]
pub fn measured_fn_factor(ts: &mut TextSystem) -> f32 {
    let top = m::FIELD_H * g::PITCH_MAX_ROWS_B;
    let mut pitches = Vec::new();
    let mut p = PITCH_FLOOR;
    while p < top {
        pitches.push(p);
        p += m::SEG_PAD;
    }
    pitches.push(top);
    let serves = |ts: &mut TextSystem, k: f32| {
        pitches.iter().all(|&p| {
            bars_full(half_h(p, k)) >= 1
                && [Group::FnRow, Group::Nav]
                    .into_iter()
                    .all(|grp| group_level_at(ts, p, grp, k).is_some())
        })
    };
    (50..=100)
        .map(|i| i as f32 / 100.0) // LITERAL-PX-OK: fator em centésimos
        .find(|k| serves(ts, *k))
        .unwrap_or(1.0)
}

/// ⭐ A escolha (plano §13.2): o passo que enche a caixa, nunca abaixo de [`PITCH_FLOOR`] — abaixo, o desenho fica
/// no passo mínimo e a caixa rola na horizontal (o numérico nunca muda de sítio).
pub(crate) fn fit(ts: &mut TextSystem, w: f32, h: Option<f32>, cap: f32) -> Fit {
    let p = pitch_of(w, h, cap);
    let scrolls = pitch_of(w, None, cap) < PITCH_FLOOR;
    let pitch = p.max(PITCH_FLOOR);
    Fit {
        pitch,
        legible: legible(ts, pitch),
        scrolls,
    }
}

/// A altura do desenho (sem o enchimento) para a escolha `f`, com a faixa da barra quando rola.
pub(super) fn drawing_rows_h(f: Fit) -> f32 {
    rows_h(f.pitch, g::extent().1) + if f.scrolls { HBAR_BAND } else { 0.0 }
}

/// A largura do desenho ao passo `p` (a tinta: sem o vão que segue a última coluna).
#[must_use]
pub fn drawing_w(p: f32) -> f32 {
    (g::extent().0 - (1.0 - g::KEY_FILL)) * p
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

/// As teclas sem legenda que caiba à largura `w` (tecto da B ou da C): `(rótulo, largura da tecla)`.
#[must_use]
pub fn misfits(ts: &mut TextSystem, w: f32, cap_b: bool) -> Vec<(String, f32)> {
    let p = fit_for_width(ts, w, cap_b).pitch;
    let lost: Vec<Group> = GROUPS
        .into_iter()
        .filter(|grp| group_level(ts, p, *grp).is_none())
        .collect();
    g::keyboard()
        .into_iter()
        .filter(|c| {
            lost.contains(&group_of(c.key))
                && choose(
                    ts,
                    c.key,
                    (Level::ShortSmall, true),
                    key_w(c.span, p),
                    label_h(cap_h_at(c, p, FN_ROW_FACTOR)),
                )
                .is_none()
        })
        .map(|c| (c.key.label(), key_w(c.span, p)))
        .collect()
}

/// As teclas com a sua caixa relativa à origem do desenho: `(tecla, x, y, w, h)`.
pub(super) fn placed(f: Fit) -> Vec<(KeyCap, f32, f32, f32, f32)> {
    let p = f.pitch;
    g::keyboard()
        .into_iter()
        .map(|c| {
            let (x, y) = (c.col * p, row_y(c.row, p));
            (c, x, y, key_w(c.span, p), cap_h_at(&c, p, FN_ROW_FACTOR))
        })
        .collect()
}
