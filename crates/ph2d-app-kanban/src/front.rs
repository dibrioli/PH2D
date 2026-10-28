//! ⭐ **A frente de um cartão** — o que ela mostra (capa de cor, rótulos, título, distintivos,
//! membros) e a altura que isso ocupa. A altura mede o título com o MESMO texto que o pintor pinta e
//! guarda-se por cartão até mudar o que pesa nela: um quadro de milhares de cartões não molda texto a
//! cada imagem.
//!
//! ⚠️ A data de entrega e a capa de IMAGEM entram com a W3: a primeira pede o fuso e o «atrasado à
//! meia-noite» do `ph2d-kanban-dates`, a segunda os ficheiros do projecto.

use crate::metrics as m;
use ph2d_kanban_model::{CardId, CoverFill, Hue, Kanban, LabelColor, Member};
use ph2d_text::TextSystem;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// O que o cartão mostra na frente.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Front {
    pub title: String,
    pub cover: Option<Hue>,
    /// `(cor, nome)` de cada rótulo, pela ordem do cartão; sem cor = não se mostra.
    pub labels: Vec<(LabelColor, String)>,
    pub description: bool,
    pub comments: u32,
    pub attachments: u32,
    /// Itens feitos e itens no total.
    pub check: Option<(u32, u32)>,
    pub votes: u32,
    pub location: bool,
    pub members: Vec<Member>,
}

impl Front {
    pub fn badges(&self) -> bool {
        self.description
            || self.comments > 0
            || self.attachments > 0
            || self.check.is_some()
            || self.votes > 0
            || self.location
            || !self.members.is_empty()
    }
}

/// As contagens que moram FORA do cartão (comentários, anexos, itens), juntas numa passagem.
#[derive(Clone, Debug, Default)]
pub(crate) struct Counts {
    comments: BTreeMap<CardId, u32>,
    attachments: BTreeMap<CardId, u32>,
    check: BTreeMap<CardId, (u32, u32)>,
}

impl Counts {
    pub fn of(k: &Kanban) -> Self {
        let mut c = Self::default();
        for x in k.comments.values().filter(|x| !x.meta.deleted) {
            *c.comments.entry(x.card).or_default() += 1;
        }
        for x in k.attachments.values().filter(|x| !x.meta.deleted) {
            *c.attachments.entry(x.card).or_default() += 1;
        }
        for i in k.items.values().filter(|i| !i.meta.deleted) {
            let Some(list) = k.checklists.get(&i.checklist).filter(|l| !l.meta.deleted) else {
                continue;
            };
            let e = c.check.entry(list.card).or_default();
            e.0 += u32::from(i.done);
            e.1 += 1;
        }
        c
    }
}

/// A frente do cartão `id`.
pub(crate) fn front(k: &Kanban, counts: &Counts, id: CardId) -> Option<Front> {
    let c = k.cards.get(&id)?;
    let labels = c
        .labels
        .iter()
        .filter_map(|l| k.labels.get(l).filter(|l| !l.meta.deleted))
        .filter_map(|l| l.color.map(|col| (col, l.name.clone())))
        .collect();
    Some(Front {
        title: c.title.clone(),
        cover: c.cover.and_then(|cv| match cv.fill {
            CoverFill::Color(h) => Some(h),
            CoverFill::Image(_) => None,
        }),
        labels,
        description: !c.description.trim().is_empty(),
        comments: counts.comments.get(&id).copied().unwrap_or(0),
        attachments: counts.attachments.get(&id).copied().unwrap_or(0),
        check: counts.check.get(&id).copied(),
        votes: u32::try_from(c.votes.len()).unwrap_or(u32::MAX),
        location: !c.location.trim().is_empty(),
        members: c.members.clone(),
    })
}

/// A largura de um chip de rótulo com texto.
pub(crate) fn chip_w(ts: &mut TextSystem, name: &str) -> f32 {
    let w = ts.prefix_width(name, m::SMALL_PX) + 2.0 * m::LABEL_PAD_X;
    w.max(m::LABEL_MIN_W)
}

/// As linhas que os chips ocupam, quebrando em `width`.
pub(crate) fn chip_rows(widths: &[f32], width: f32) -> usize {
    let mut rows = usize::from(!widths.is_empty());
    let mut x = 0.0_f32;
    for &w in widths {
        if x > 0.0 && x + w > width {
            rows += 1;
            x = 0.0;
        }
        x += w + m::LABEL_GAP;
    }
    rows
}

/// As linhas que o título ocupa a `width` (a quebra do pintor).
pub(crate) fn title_lines(ts: &mut TextSystem, title: &str, width: f32) -> usize {
    if title.is_empty() {
        return 1;
    }
    ts.layout(title, m::CARD_TITLE_PX, width).len().max(1)
}

fn print(f: &Front, label_text: bool) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    f.title.hash(&mut h);
    f.cover.hash(&mut h);
    label_text.hash(&mut h);
    for (c, n) in &f.labels {
        c.hash(&mut h);
        n.hash(&mut h);
    }
    f.badges().hash(&mut h);
    h.finish()
}

/// A altura da faixa dos rótulos (chips com texto, ou barras).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn labels_h(ts: &mut TextSystem, f: &Front, label_text: bool) -> f32 {
    let inner = m::CARD_W - 2.0 * m::CARD_PAD_X;
    let rows = if label_text {
        let w: Vec<f32> = f.labels.iter().map(|(_, n)| chip_w(ts, n)).collect();
        chip_rows(&w, inner)
    } else {
        chip_rows(&vec![m::LABEL_MIN_W; f.labels.len()], inner)
    };
    let row_h = if label_text { m::LABEL_H } else { m::LABEL_H / 2.0 };
    rows as f32 * row_h + rows.saturating_sub(1) as f32 * m::LABEL_GAP
}

/// ⭐ A altura do cartão (guardada em `cache` até mudar o que pesa nela).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn height(
    ts: &mut TextSystem,
    cache: &mut BTreeMap<CardId, (u64, f32)>,
    id: CardId,
    f: &Front,
    label_text: bool,
) -> f32 {
    let key = print(f, label_text);
    if let Some((k, h)) = cache.get(&id)
        && *k == key
    {
        return *h;
    }
    let inner = m::CARD_W - 2.0 * m::CARD_PAD_X;
    let mut h = 2.0 * m::CARD_PAD_Y;
    if f.cover.is_some() {
        h += m::COVER_COLOR_H;
    }
    if !f.labels.is_empty() {
        h += labels_h(ts, f, label_text) + m::CARD_ROW_GAP;
    }
    h += title_lines(ts, &f.title, inner) as f32 * m::LINE_H;
    if f.badges() {
        h += m::CARD_ROW_GAP + m::BADGE_ROW_H;
    }
    cache.insert(id, (key, h));
    h
}

/// As iniciais de um membro («Enio Dibrioli» → «ED», «enio» → «E»).
pub(crate) fn initials(m: &Member) -> String {
    m.0.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

/// A cor do avatar de um membro: uma das dez, sempre a mesma para o mesmo nome.
pub(crate) fn avatar_hue(m: &Member) -> Hue {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    m.0.hash(&mut h);
    #[allow(clippy::cast_possible_truncation)]
    let i = (h.finish() % 9) as usize;
    // O preto fica de fora: um avatar cinzento lê-se como «desactivado».
    Hue::ALL[i]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chips_wrap_when_the_next_one_does_not_fit() {
        assert_eq!(chip_rows(&[], 232.0), 0);
        assert_eq!(chip_rows(&[100.0, 100.0], 232.0), 1);
        assert_eq!(chip_rows(&[100.0, 100.0, 100.0], 232.0), 2);
    }

    #[test]
    fn initials_take_the_first_letter_of_two_words() {
        assert_eq!(initials(&Member("enio dibrioli".into())), "ED");
        assert_eq!(initials(&Member("Mia".into())), "M");
    }
}
