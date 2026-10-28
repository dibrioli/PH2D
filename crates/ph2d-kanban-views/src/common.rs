//! O que as quatro vistas partilham: o contexto do quadro (agora, fuso, «eu», filtro), o rato, a
//! resposta, os cartões à vista e as cores das etiquetas, listas e membros pelos tokens `kanban-*`.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::paint::resolve;
use ph2d_kanban_dates::{CivilDate, Weekday, Zone};
use ph2d_kanban_filter::{Filter, matches};
use ph2d_kanban_model::{
    Card, CardRef, Hue, Kanban, KanbanId, KanbanSet, LabelColor, Member, Moment, Plan, Shade,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::Color;

/// O que o quadro diz às vistas a cada quadro de imagem.
#[derive(Clone, Copy, Debug)]
pub struct Ctx<'a> {
    pub now: Moment,
    pub zone: &'a Zone,
    pub me: &'a Member,
    /// As pessoas que se podem pôr num cartão (a Equipa), além das que já estão nos cartões.
    pub team: &'a [Member],
    /// O filtro da aba (o MESMO do quadro): as vistas só mostram o que ele deixa ver.
    pub filter: &'a Filter,
    pub week_start: Weekday,
}

/// O gesto do rato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerKind {
    Down,
    Move,
    Up,
}

/// O rato em px da VISTA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerAt {
    pub kind: PointerKind,
    pub x: f32,
    pub y: f32,
}

/// O que um gesto pede ao chamador.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reply {
    /// As mudanças a aplicar (um lote por Kanban, UM passo de desfazer).
    pub plan: Plan,
    /// Abrir o verso deste cartão (o verso é do app).
    pub open: Option<CardRef>,
    /// O gesto foi da vista (não passa ao que está por baixo).
    pub taken: bool,
    /// A vista mudou e pede outro quadro.
    pub redraw: bool,
}

impl Reply {
    #[must_use]
    pub fn taken() -> Self {
        Self {
            taken: true,
            redraw: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_plan(plan: Plan) -> Self {
        Self {
            plan,
            ..Self::taken()
        }
    }

    #[must_use]
    pub fn open(card: CardRef) -> Self {
        Self {
            open: Some(card),
            ..Self::taken()
        }
    }
}

/// Os cartões de `k` que a vista mostra: vivos, fora do arquivo, não modelos, numa lista viva, e que
/// passam no filtro — pela ordem do quadro (lista, depois posição).
#[must_use]
pub fn visible_cards<'k>(k: &'k Kanban, cx: &Ctx<'_>) -> Vec<&'k Card> {
    let all = cx.filter.is_empty();
    let mut out = Vec::new();
    for l in k.lists_in_order() {
        for c in k.cards_in(l.id) {
            if !c.template && (all || matches(cx.filter, c, k, cx.now, cx.zone, cx.me)) {
                out.push(c);
            }
        }
    }
    out
}

/// O Kanban `id` de `set`, e a referência de um cartão dele.
#[must_use]
pub fn card_ref(kanban: KanbanId, c: &Card) -> CardRef {
    CardRef { kanban, card: c.id }
}

/// O Kanban `id`, se vivo.
#[must_use]
pub fn kanban(set: &KanbanSet, id: KanbanId) -> Option<&Kanban> {
    set.get(id)
}

fn hue_index(h: Hue) -> usize {
    Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)
}

fn shade_index(s: Shade) -> usize {
    Shade::ALL.iter().position(|x| *x == s).unwrap_or(0)
}

/// O token de uma das 30 etiquetas.
#[must_use]
pub fn label_token(l: LabelColor) -> T {
    ph2d_tokens::kanban::LABELS[shade_index(l.shade)][hue_index(l.hue)]
}

/// A cor de uma etiqueta e a tinta que a lê.
#[must_use]
pub fn label_colors(theme: Theme, l: LabelColor) -> (Color, Color) {
    (
        resolve(label_token(l), theme),
        resolve(ph2d_tokens::kanban::label_ink(shade_index(l.shade)), theme),
    )
}

/// A cor de uma lista: a etiqueta NORMAL do matiz dela; sem cor, o azul do quadro (o Trello pinta
/// as barras de uma lista sem cor no azul da marca).
#[must_use]
pub fn list_token(hue: Option<Hue>) -> T {
    label_token(LabelColor {
        hue: hue.unwrap_or(Hue::Blue),
        shade: Shade::Normal,
    })
}

/// A cor do avatar de um membro: uma das nove (sem o preto), sempre a mesma para o mesmo nome — a
/// MESMA lei da frente do cartão (`ph2d-app-kanban`, `front::avatar_hue`).
#[must_use]
pub fn avatar_hue(m: &Member) -> Hue {
    #[allow(deprecated)]
    let mut h = std::collections::hash_map::DefaultHasher::new();
    m.0.hash(&mut h);
    #[allow(clippy::cast_possible_truncation)]
    let i = (h.finish() % 9) as usize;
    Hue::ALL[i]
}

/// As iniciais de um membro (duas, em maiúsculas).
#[must_use]
pub fn initials(m: &Member) -> String {
    m.0.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

/// Todas as pessoas que aparecem: a Equipa, depois quem está num cartão de `k` (sem repetir).
#[must_use]
pub fn people(k: &Kanban, cx: &Ctx<'_>) -> Vec<Member> {
    let mut out: Vec<Member> = cx.team.to_vec();
    for c in k.cards.values().filter(|c| !c.meta.deleted) {
        for m in &c.members {
            if !out.contains(m) {
                out.push(m.clone());
            }
        }
    }
    out
}

/// O topo de uma linha de texto de `px` centrada numa faixa de `row_h` que começa em `y`.
#[must_use]
pub fn text_top(ts: &mut TextSystem, y: f32, row_h: f32, px: f32) -> f32 {
    let h = ts.layout("Ag", px, f32::INFINITY).height();
    y + ((row_h - h) / 2.0).max(0.0)
}

/// A cor de um token no tema.
#[must_use]
pub fn c(theme: Theme, t: T) -> Color {
    resolve(t, theme)
}

/// «Oct 25» (com o ano se não for o de `today`).
#[must_use]
pub fn day_text(d: CivilDate, today: CivilDate) -> String {
    crate::datepick::day_text(d, today)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_hue_is_stable_and_never_black() {
        let m = Member("Ana".into());
        assert_eq!(avatar_hue(&m), avatar_hue(&m));
        for n in ["Ana", "Rui", "Mia", "Enio", "Zé"] {
            assert_ne!(avatar_hue(&Member(n.into())), Hue::Black);
        }
        assert_eq!(initials(&Member("Ana Maria".into())), "AM");
    }
}
