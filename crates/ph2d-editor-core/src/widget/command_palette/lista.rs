//! ⭐ **A paleta em LISTA** (dono 12/11, versão C): uma linha por comando — nome, o caminho de onde
//! ele mora (apagado) e o atalho numa coluna à direita —, «Recent» em cima e as dicas no rodapé.
//! É um MODO do modelo ([`PaletteLayout::List`]); os outros consumidores ficam em nuvem.
//!
//! O caminho de um item é o título do grupo e o da fatia dele (`File · Export`): nenhuma tabela.
//! A ordem que se pinta é a do modelo que [`arrange`] devolve — a mesma que as setas percorrem.

use super::{
    HitIndex, PaletteGroup, PaletteItem, PaletteModel, PaletteSub, Scoped, fuzzy_score,
    label_matches, split_shortcut,
};
use crate::paint::{fill_rounded_rect, paint_text_elided};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;
use std::borrow::Cow;

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
        Row::Item(..) => ROW_H_PX,
    }
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

/// Pinta as linhas a partir de `(x, top)` (já rolado) e regista o alvo de cada item. Só as que
/// tocam `body` se pintam.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_rows(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    (x, top, w): (f32, f32, f32),
    body: Rect,
    [hot, chosen]: [Option<NodeId>; 2],
) {
    let superficie = Superficie::de(theme);
    let (sm, xs) = (TypeToken::Sm.px(), TypeToken::Xs.px());
    let pad = Spacing::Sm.px();
    let gap = Spacing::Md.px();
    let faded = superficie.cor(ColorToken::Text3);
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
                paint_text_elided(ts, scene, title, x + pad, ty, xs, inner_w, faded);
            }
            Row::Item(it, path) => {
                let r = Rect::new(x, row_y, w, h);
                let on = chosen == Some(it.id);
                let (fg, dim) = if on {
                    let fg = superficie.botao(scene, r, true, false);
                    (fg, fg)
                } else {
                    if hot == Some(it.id) {
                        superficie.realce(scene, r);
                    }
                    (superficie.cor(ColorToken::Text1), faded)
                };
                let ty = r.y + superficie.folga(h, sm);
                let (name, key) = split_shortcut(&it.label);
                // O atalho, na coluna da direita.
                let key_w = key.map_or(0.0, |k| ts.prefix_width(k, sm));
                if let Some(k) = key {
                    let kx = r.x + r.w - pad - key_w;
                    paint_text_elided(ts, scene, k, kx, ty, sm, key_w + pad, dim);
                }
                let room = (inner_w - key.map_or(0.0, |_| key_w + gap)).max(0.0);
                let name_w = ts.prefix_width(name, sm).min(room);
                paint_text_elided(ts, scene, name, r.x + pad, ty, sm, name_w + pad, fg);
                // O caminho de onde o comando mora, apagado, logo depois do nome.
                let path_room = room - name_w - gap;
                if let Some(p) = path.filter(|_| path_room > m::FIELD_H) {
                    let px = r.x + pad + name_w + gap;
                    let py = r.y + superficie.folga(h, xs);
                    paint_text_elided(ts, scene, p, px, py, xs, path_room, dim);
                }
                hit_index.register(it.id, r);
            }
        }
    }
}

/// As dicas do rodapé numa linha, separadas por «·».
#[must_use]
pub fn footer_text() -> String {
    [
        "chrome.palette.hint_run",
        "chrome.palette.hint_where",
        "chrome.palette.hint_move",
        "chrome.palette.hint_close",
    ]
    .map(ph2d_i18n::tr)
    .join(PATH_SEP)
}

/// Pinta as dicas do rodapé ([`footer_text`]).
pub(super) fn paint_footer(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, at: Rect) {
    let superficie = Superficie::de(theme);
    let xs = TypeToken::Xs.px();
    let color = superficie.cor(ColorToken::Text3);
    let y = at.y + superficie.folga(at.h, xs);
    let x = at.x + Spacing::Sm.px();
    let room = (at.x + at.w - x).max(0.0);
    paint_text_elided(ts, scene, &footer_text(), x, y, xs, room, color);
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
