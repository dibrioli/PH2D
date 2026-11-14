//! ⭐ **A paleta em LISTA** (dono 12/11, versão C): uma linha por comando — nome, o caminho de onde
//! ele mora (apagado) e o atalho numa coluna à direita —, «Recent» em cima e as dicas no rodapé.
//! É um MODO do modelo ([`PaletteLayout::List`]); os outros consumidores ficam em nuvem.
//!
//! O caminho de um item é o título do grupo e o da fatia dele (`File · Export`): nenhuma tabela.
//! A ordem que se pinta é a do modelo que [`arrange`] devolve — a mesma que as setas percorrem.

use super::layout::{badge_font, badge_size};
use super::{
    HitIndex, PaletteGroup, PaletteItem, PaletteModel, PaletteSub, Scoped, fuzzy_score,
    label_matches, match_ranges, split_shortcut,
};
use crate::paint::{fill_rounded_rect, paint_text_elided, paint_text_elided_in_weight, resolve};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::Color;
use ph2d_vector::VectorScene;
use std::borrow::Cow;

mod legivel;
use legivel::legivel;
#[cfg(test)]
use legivel::{TEXTO_MIN, pousa};

/// Como a paleta arruma os itens.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PaletteLayout {
    /// Os cartões de pílulas de sempre.
    #[default]
    Cloud,
    /// Uma linha por item; `recent` = os últimos escolhidos, o mais novo primeiro.
    List { recent: Vec<NodeId> },
}

/// Quantos recentes a lista mostra — escolha de desenho (Raycast, VS Code), não tecto de recurso.
pub const RECENT_SHOWN: usize = 5;

/// O que separa os degraus de um caminho («File · Export»).
pub const PATH_SEP: &str = " \u{00b7} ";

/// A largura máxima da janela em lista.
pub(super) const LIST_MAX_W: f32 = 720.0; // LITERAL-PX-OK: geometria do modal (Command Center do VS Code)

/// O caminho de onde os itens de `sub` moram: grupo e fatia.
#[must_use]
pub fn path_of(group: &PaletteGroup, sub: &PaletteSub) -> String {
    match sub.title.as_deref() {
        Some(t) if group.title.is_empty() => t.to_owned(),
        Some(t) => format!("{}{PATH_SEP}{t}", group.title),
        None => group.title.clone(),
    }
}

fn single(path: String, item: PaletteItem) -> PaletteSub {
    PaletteSub {
        title: (!path.is_empty()).then_some(path),
        items: vec![item],
    }
}

/// ⭐ **A vista em lista** de uma busca já filtrada: sem texto, «Recent» em cima e os grupos (sem os
/// recentes, sem repetir id); com texto, UMA lista plana — os recentes que casam primeiro, depois o
/// melhor casamento. Os prefixos `@` e `:` passam como estão.
pub(super) fn arrange<'a>(recent: &[NodeId], s: Scoped<'a>, query: &str) -> Scoped<'a> {
    if query.starts_with('@') || query.starts_with(':') {
        return s;
    }
    let q = query.strip_prefix('>').unwrap_or(query).trim();
    let src: &PaletteModel = &s.model;
    let mut all: Vec<(String, PaletteItem)> = Vec::new();
    for g in &src.groups {
        for sub in &g.subs {
            let path = path_of(g, sub);
            for it in &sub.items {
                if !all.iter().any(|(_, seen)| seen.id == it.id) {
                    all.push((path.clone(), it.clone()));
                }
            }
        }
    }
    let age = |id: NodeId| recent.iter().position(|r| *r == id);
    let mut groups: Vec<PaletteGroup> = Vec::new();
    let color = src.groups.first().map_or(ColorToken::Text3, |g| g.color);
    if q.is_empty() {
        let mut shown: Vec<(usize, String, PaletteItem)> = all
            .iter()
            .filter_map(|(p, it)| age(it.id).map(|a| (a, p.clone(), it.clone())))
            .collect();
        shown.sort_by_key(|(a, _, _)| *a);
        shown.truncate(RECENT_SHOWN);
        let mut taken: Vec<NodeId> = shown.iter().map(|(_, _, it)| it.id).collect();
        if !shown.is_empty() {
            groups.push(PaletteGroup {
                title: ph2d_i18n::tr("chrome.palette.recent").to_owned(),
                color,
                subs: shown.into_iter().map(|(_, p, it)| single(p, it)).collect(),
            });
        }
        for g in &src.groups {
            let subs: Vec<PaletteSub> = g
                .subs
                .iter()
                .filter_map(|sub| {
                    let mut items = Vec::new();
                    for it in &sub.items {
                        if !taken.contains(&it.id) {
                            taken.push(it.id);
                            items.push(it.clone());
                        }
                    }
                    // A linha diz sempre de onde o comando mora, também sem busca.
                    let path = path_of(g, sub);
                    (!items.is_empty()).then(|| PaletteSub {
                        title: (!path.is_empty()).then_some(path),
                        items,
                    })
                })
                .collect();
            if !subs.is_empty() {
                groups.push(PaletteGroup {
                    title: g.title.clone(),
                    color: g.color,
                    subs,
                });
            }
        }
    } else {
        // Estável: dentro do mesmo degrau fica a ordem de exibição.
        all.sort_by_key(|(_, it)| {
            let name = split_shortcut(&it.label).0;
            (
                age(it.id).unwrap_or(usize::MAX),
                !label_matches(q, name),
                std::cmp::Reverse(fuzzy_score(q, name).unwrap_or(i32::MIN)),
            )
        });
        if !all.is_empty() {
            groups.push(PaletteGroup {
                title: String::new(),
                color,
                subs: all.into_iter().map(|(p, it)| single(p, it)).collect(),
            });
        }
    }
    let top = if q.is_empty() {
        s.top
    } else {
        groups
            .first()
            .and_then(|g| g.subs.first())
            .and_then(|sub| sub.items.first())
            .map(|it| it.id)
    };
    let model = PaletteModel {
        groups,
        ..src.clone()
    };
    Scoped {
        model: Cow::Owned(model),
        top,
        note: s.note,
    }
}

/// Uma linha da lista: o cabeçalho de uma secção, ou um item com o caminho dele.
enum Row<'a> {
    Head(&'a str),
    Item(&'a PaletteItem, Option<&'a str>),
}

fn rows(model: &PaletteModel) -> Vec<Row<'_>> {
    let mut out = Vec::new();
    for g in &model.groups {
        if !g.title.is_empty() {
            out.push(Row::Head(&g.title));
        }
        for sub in &g.subs {
            for it in &sub.items {
                out.push(Row::Item(it, sub.title.as_deref()));
            }
        }
    }
    out
}

fn head_h() -> f32 {
    TypeToken::Xs.px() + Spacing::Sm.px() * 2.0
}

fn height(row: &Row<'_>) -> f32 {
    match row {
        Row::Head(_) => head_h(),
        Row::Item(..) => item_h(),
    }
}

/// A altura de uma linha de item (onda 7: 32, a do campo da busca — o campo e as linhas alinham).
#[must_use]
pub fn item_h() -> f32 {
    m::FIELD_H + Spacing::Xs.px()
}

/// A altura que a lista pede.
pub(super) fn content_h(model: &PaletteModel) -> f32 {
    rows(model).iter().map(height).sum()
}

/// A altura da faixa das dicas, no fundo da janela.
pub(super) fn footer_h() -> f32 {
    TypeToken::Xs.px() + Spacing::Sm.px() * 2.0
}

/// `(y, altura)` da linha de `id` no conteúdo da lista (com o cabeçalho logo acima, se há): o que
/// tem de ficar à vista quando a seleção chega nela.
#[must_use]
pub fn row_span(model: &PaletteModel, id: NodeId) -> Option<(f32, f32)> {
    let mut y = 0.0_f32;
    let mut head = 0.0_f32;
    for row in rows(model) {
        let h = height(&row);
        match row {
            Row::Item(it, _) if it.id == id => return Some((y - head, h + head)),
            Row::Item(..) => head = 0.0,
            Row::Head(_) => head = h,
        }
        y += h;
    }
    None
}

/// A rolagem que deixa `span` inteiro dentro de uma janela de altura `view_h` — a de agora, se já
/// está à vista.
#[must_use]
pub fn follow_scroll((y, h): (f32, f32), scroll: f32, view_h: f32) -> f32 {
    if y < scroll {
        y
    } else if y + h > scroll + view_h {
        (y + h - view_h).max(0.0)
    } else {
        scroll
    }
}

/// ⭐ **Os tons da lista** (onda 6, maquete C «Tudo numa busca») — o pintor e o portão de contraste
/// leem ESTA função, nunca uma cópia: o que o teste mede é o que se pinta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tons {
    /// O fundo do cartão.
    pub fundo: Color,
    /// O nome, e a parte dele que casou com a busca (a negrito, na cor da chave).
    pub nome: Color,
    pub chave: Color,
    /// «Onde mora» e o atalho, apagados.
    pub onde: Color,
    /// A linha sob o rato.
    pub realce: Color,
    /// A linha escolhida: cheia na cor do foco, o texto invertido.
    pub escolhido: Color,
    pub invertido: Color,
    /// A plaquinha do tipo: o fundo e o texto dela.
    pub chip: Color,
    pub chip_texto: Color,
}

/// Os [`Tons`] da lista em `theme`, no modo vigente (cartões ou o de antes).
#[must_use]
pub fn tons(theme: Theme) -> Tons {
    let s = Superficie::de(theme);
    let (realce, escolhido, invertido) = match s.tinta() {
        Some(ink) => (
            ink.color(ColorToken::CardFieldHover),
            ink.color(ColorToken::KanbanBlueBold),
            ink.color(ColorToken::AccentFg),
        ),
        None => (
            resolve(ColorToken::Bg2, theme),
            resolve(ColorToken::Accent, theme),
            resolve(ColorToken::AccentFg, theme),
        ),
    };
    let (fundo, nome) = (s.fundo(), s.cor(ColorToken::Text1));
    let le = |c| legivel(c, nome, [fundo, realce]);
    Tons {
        fundo,
        nome,
        chave: le(s.cor(ColorToken::Accent)),
        onde: le(s.cor(ColorToken::Text3)),
        realce,
        escolhido,
        invertido,
        chip: s.cor(ColorToken::Border),
        chip_texto: s.cor(ColorToken::Text2),
    }
}

/// O nome com os trechos que casaram com a busca a negrito (`key`); devolve a largura pintada.
#[allow(clippy::too_many_arguments)]
fn paint_name(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    name: &str,
    lit: &[std::ops::Range<usize>],
    [x, y, room, px]: [f32; 4],
    base: Color,
    key: Color,
) -> f32 {
    let mut cuts: Vec<(std::ops::Range<usize>, bool)> = Vec::new();
    let mut at = 0;
    for r in lit {
        if r.start > at {
            cuts.push((at..r.start, false));
        }
        cuts.push((r.clone(), true));
        at = r.end;
    }
    if at < name.len() {
        cuts.push((at..name.len(), false));
    }
    let mut dx = 0.0_f32;
    for (r, on) in cuts {
        let left = room - dx;
        if left <= 0.0 {
            break;
        }
        let (part, weight) = (
            &name[r],
            if on {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            },
        );
        let w = ph2d_text::com_peso_exato(|| ts.prefix_width_weighted(part, px, weight));
        let ink = if on { key } else { base };
        paint_text_elided_in_weight(ts, scene, part, [x + dx, y], px, left, ink, weight);
        dx += w.min(left);
    }
    dx
}

/// Pinta as linhas a partir de `(x, top)` (já rolado) e regista o alvo de cada item. Só as que
/// tocam `body` se pintam. Cada item: o nome (a parte que casou com `query` a negrito, na cor da
/// chave), onde ele mora e o atalho apagados, e a plaquinha do TIPO à direita (maquete C).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_rows(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    query: &str,
    (x, top, w): (f32, f32, f32),
    body: Rect,
    [hot, chosen]: [Option<NodeId>; 2],
) {
    let superficie = Superficie::de(theme);
    let t = tons(theme);
    let (sm, xs) = (TypeToken::Sm.px(), TypeToken::Xs.px());
    let pad = Spacing::Md.px();
    let gap = Spacing::Sm.px();
    let q = query.strip_prefix('>').unwrap_or(query);
    let inner_w = (w - pad * 2.0).max(0.0);
    let mut y = top;
    for row in rows(model) {
        let h = height(&row);
        // Só a linha INTEIRA dentro do corpo se pinta: a borda corta limpo, nunca a meio.
        let slack = ph2d_tokens::StrokeToken::Default.px();
        let (row_y, seen) = (y, y >= body.y - slack && y + h <= body.y + body.h + slack);
        y += h;
        if !seen {
            continue;
        }
        match row {
            Row::Head(title) => {
                let ty = row_y + superficie.folga(h, xs);
                paint_text_elided(ts, scene, title, x + pad, ty, xs, inner_w, t.onde);
            }
            Row::Item(it, path) => {
                let r = Rect::new(x, row_y, w, h);
                let on = chosen == Some(it.id);
                let (fg, dim, key) = if on {
                    fill_rounded_rect(scene, r, m::SEG_RADIUS, t.escolhido);
                    (t.invertido, t.invertido, t.invertido)
                } else {
                    if hot == Some(it.id) {
                        fill_rounded_rect(scene, r, m::SEG_RADIUS, t.realce);
                    }
                    (t.nome, t.onde, t.chave)
                };
                // A plaquinha do tipo, encostada à direita.
                let chip_w = it.badge.as_deref().map_or(0.0, |b| {
                    let (cw, ch) = badge_size(ts, b);
                    let c = Rect::new(r.x + r.w - pad - cw, r.y + (h - ch) * 0.5, cw, ch);
                    // Na linha escolhida, a plaquinha é um furo no tom do cartão (maquete:
                    // `rgba(0,0,0,.2)` sobre o foco), com o texto do nome — o par já medido.
                    let (fundo, tinta) = if on {
                        (t.fundo, t.nome)
                    } else {
                        (t.chip, t.chip_texto)
                    };
                    fill_rounded_rect(scene, c, ch * 0.5, fundo);
                    let (px, weight) = badge_font();
                    let ty = c.y + superficie.folga(ch, px);
                    let tx = c.x + Spacing::Xs.px();
                    paint_text_elided_in_weight(ts, scene, b, [tx, ty], px, cw, tinta, weight);
                    cw + gap
                });
                let room = (inner_w - chip_w).max(0.0);
                let (name, shortcut) = split_shortcut(&it.label);
                let lit = if q.trim().is_empty() {
                    Vec::new()
                } else {
                    match_ranges(q, name)
                };
                let ty = r.y + superficie.folga(h, sm);
                let name_w = paint_name(ts, scene, name, &lit, [r.x + pad, ty, room, sm], fg, key);
                // Onde o item mora e o atalho, apagados, logo depois do nome.
                let onde = match (path, shortcut) {
                    (Some(p), Some(k)) => Cow::Owned(format!("{p}{PATH_SEP}{k}")),
                    (Some(p), None) => Cow::Borrowed(p),
                    (None, Some(k)) => Cow::Borrowed(k),
                    (None, None) => Cow::Borrowed(""),
                };
                let onde_room = room - name_w - gap;
                if !onde.is_empty() && onde_room > m::FIELD_H {
                    let px = r.x + pad + name_w + gap;
                    let py = r.y + superficie.folga(h, xs);
                    let normal = FontWeight::NORMAL;
                    paint_text_elided_in_weight(
                        ts,
                        scene,
                        &onde,
                        [px, py],
                        xs,
                        onde_room,
                        dim,
                        normal,
                    );
                }
                hit_index.register(it.id, r);
            }
        }
    }
}

/// As dicas do rodapé, pela ordem (maquete C: «Enter to open», «Alt+Enter: show me where it lives»).
#[must_use]
pub fn footer_hints() -> [&'static str; 2] {
    ["chrome.palette.hint_run", "chrome.palette.hint_where"].map(ph2d_i18n::tr)
}

/// Pinta as dicas do rodapé ([`footer_hints`]) lado a lado, sob um filete.
pub(super) fn paint_footer(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, at: Rect) {
    let superficie = Superficie::de(theme);
    let xs = TypeToken::Xs.px();
    let color = tons(theme).onde;
    let y = at.y + superficie.folga(at.h, xs);
    let mut x = at.x + Spacing::Sm.px();
    for hint in footer_hints() {
        let room = (at.x + at.w - x).max(0.0);
        let normal = FontWeight::NORMAL;
        paint_text_elided_in_weight(ts, scene, hint, [x, y], xs, room, color, normal);
        x += ph2d_text::com_peso_exato(|| ts.prefix_width_weighted(hint, xs, normal));
        x += Spacing::Lg.px();
    }
    // Um filete separa as dicas da lista.
    let rule = ph2d_tokens::StrokeToken::Default.px();
    fill_rounded_rect(
        scene,
        Rect::new(at.x, at.y, at.w, rule),
        0.0,
        superficie.cor(ColorToken::Border),
    );
}

#[cfg(test)]
#[path = "lista_tests.rs"]
mod tests;
