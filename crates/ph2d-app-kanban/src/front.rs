//! ⭐ **A frente de um cartão** — o que ela mostra (capa de cor ou de imagem, metade ou inteira;
//! rótulos, título, campos «na frente», distintivos, membros) e a altura que isso ocupa. A altura
//! mede com as MESMAS fontes, pesos e quebras que o pintor ([`crate::paint_card`]) usa, e guarda-se
//! por cartão até mudar o que pesa nela: um quadro de milhares de cartões não molda texto a cada
//! imagem.
//!
//! As datas leem-se no fuso do sistema e no relógio de [`Counts::of`]; os bytes das capas de imagem
//! chegam por [`Counts::with_files`] e descodificam-se fora do fio do desenho ([`cover`]).

pub(crate) mod badges;
pub(crate) mod cover;

use crate::metrics as m;
use cover::ThumbRef;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{DueLabel, DueTone, Zone, date_label, day_of, due_label, due_tone};
use ph2d_kanban_model::{
    CardId, CoverFill, FieldKind, FieldValue, FileId, Hue, Kanban, KanbanSet, LabelColor, Member,
};
use ph2d_text::{FontWeight, TextSystem};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A capa de imagem metade: a faixa no topo (a da maquete).
pub(crate) const COVER_IMAGE_H: f32 = 128.0;
/// A capa inteira: a altura mínima, o recuo do título (acima, abaixo) e o título.
pub(crate) const FULL_MIN_H: f32 = 152.0;
pub(crate) const FULL_PAD_TOP: f32 = 36.0;
pub(crate) const FULL_PAD_BOTTOM: f32 = 10.0;
pub(crate) const FULL_TITLE_PX: f32 = 16.0;
pub(crate) const FULL_TITLE_WEIGHT: FontWeight = FontWeight::SEMI_BOLD;

/// O que enche a capa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Fill {
    Color(Hue),
    Image(FileId),
}

/// A capa: o enchimento e se é inteira.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Cover {
    pub fill: Fill,
    pub full: bool,
}

/// O distintivo das datas: o texto e o tom.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DueBadge {
    pub text: String,
    pub tone: DueTone,
}

/// Um campo personalizado mostrado na frente (`Nome: valor`; a caixa marcada leva o visto).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FieldChip {
    pub text: String,
    pub hue: Option<Hue>,
    pub check: bool,
}

/// O que o cartão mostra na frente.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Front {
    pub title: String,
    pub cover: Option<Cover>,
    /// A imagem da capa já descodificada (`None` = sem imagem, ou ainda a caminho).
    pub cover_px: Option<ThumbRef>,
    /// `(cor, nome)` de cada rótulo, pela ordem do cartão; sem cor = não se mostra.
    pub labels: Vec<(LabelColor, String)>,
    pub fields: Vec<FieldChip>,
    pub due: Option<DueBadge>,
    /// Eu sigo o cartão (o olho do Trello).
    pub watching: bool,
    pub description: bool,
    pub comments: u32,
    pub attachments: u32,
    /// Itens feitos e itens no total.
    pub check: Option<(u32, u32)>,
    pub votes: u32,
    pub location: String,
    pub members: Vec<Member>,
    /// W6 — um cartão-MODELO (o selo «This card is a template»; sem distintivos de datas).
    pub template: bool,
    /// W6 — um ESPELHO: o nome do Kanban do original (vazio = o mesmo Kanban).
    pub mirror: Option<String>,
}

impl Front {
    /// A capa ocupa o cartão todo (só o título, por cima).
    pub fn full(&self) -> bool {
        self.cover.is_some_and(|c| c.full)
    }
}

/// A altura da capa metade (`0` sem capa ou com a inteira).
pub(crate) fn cover_h(f: &Front) -> f32 {
    match f.cover {
        Some(Cover { full: true, .. }) | None => 0.0,
        Some(Cover {
            fill: Fill::Color(_),
            ..
        }) => m::COVER_COLOR_H,
        Some(Cover {
            fill: Fill::Image(_),
            ..
        }) => COVER_IMAGE_H,
    }
}

/// Onde começam os rótulos, a contar do topo do cartão (`None` = a capa inteira não os mostra).
pub(crate) fn labels_top(f: &Front) -> Option<f32> {
    (!f.full()).then(|| cover_h(f) + m::CARD_PAD_Y)
}

/// O que mora FORA do cartão e a frente lê: as contagens (comentários, anexos, itens), o relógio e
/// o fuso das datas, quem eu sou (o olho) e os bytes das capas de imagem.
#[derive(Clone, Debug, Default)]
pub(crate) struct Counts {
    comments: BTreeMap<CardId, u32>,
    attachments: BTreeMap<CardId, u32>,
    check: BTreeMap<CardId, (u32, u32)>,
    /// Agora, em ms UTC, e o fuso (`None` = UTC).
    now: i64,
    zone: Option<Zone>,
    me: Option<Member>,
    files: BTreeMap<FileId, Arc<[u8]>>,
}

/// Agora, em ms desde 1970 UTC.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

impl Counts {
    /// As contagens de `k`, o relógio e o fuso do sistema.
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
        c.at(now_ms(), Zone::system())
    }

    /// Outro relógio e outro fuso (os testes param-nos).
    #[must_use]
    pub fn at(mut self, now_ms: i64, zone: Zone) -> Self {
        self.now = now_ms;
        self.zone = Some(zone);
        self
    }

    /// Quem eu sou: os cartões que sigo mostram o olho.
    #[must_use]
    pub fn seen_by(mut self, me: Option<Member>) -> Self {
        self.me = me;
        self
    }

    /// Os bytes das capas de imagem de `k` que ainda não estão descodificadas (partilhados, sem
    /// cópia).
    #[must_use]
    pub fn with_files(mut self, set: &KanbanSet, k: &Kanban) -> Self {
        for c in k.cards.values().filter(|c| !c.meta.deleted) {
            if let Some(CoverFill::Image(id)) = c.cover.map(|c| c.fill)
                && let Some(f) = set.file(id)
            {
                self.files.insert(id, f.bytes.0.clone());
            }
        }
        self
    }
}

const MONTHS: [&str; 12] = [
    "kanban.front.month.1",
    "kanban.front.month.2",
    "kanban.front.month.3",
    "kanban.front.month.4",
    "kanban.front.month.5",
    "kanban.front.month.6",
    "kanban.front.month.7",
    "kanban.front.month.8",
    "kanban.front.month.9",
    "kanban.front.month.10",
    "kanban.front.month.11",
    "kanban.front.month.12",
];

/// O texto de um [`DueLabel`] («Today 14:00», «Oct 28», «Oct 28, 2027»).
pub(crate) fn label_text(l: DueLabel) -> String {
    let time = |hour: u8, minute: u8| {
        let minute = format!("{minute:02}");
        tr_with("kanban.front.time", &[("hour", &hour), ("minute", &minute)])
    };
    let month = |n: u8| tr(MONTHS[usize::from(n.clamp(1, 12)) - 1]);
    match l {
        DueLabel::Yesterday { hour, minute } => {
            tr_with("kanban.front.yesterday", &[("time", &time(hour, minute))])
        }
        DueLabel::Today { hour, minute } => {
            tr_with("kanban.front.today", &[("time", &time(hour, minute))])
        }
        DueLabel::Tomorrow { hour, minute } => {
            tr_with("kanban.front.tomorrow", &[("time", &time(hour, minute))])
        }
        DueLabel::Date { month: n, day } => {
            tr_with("kanban.front.date", &[("month", &month(n)), ("day", &day)])
        }
        DueLabel::DateWithYear {
            year,
            month: n,
            day,
        } => tr_with(
            "kanban.front.date_year",
            &[("month", &month(n)), ("day", &day), ("year", &year)],
        ),
    }
}

/// ⭐ O distintivo das datas: início → entrega quando há os dois, «Starts:/Started:» só com o
/// início; o tom é o da ENTREGA (verde concluída, amarelo a menos de 24 h, vermelho atrasada).
pub(crate) fn due_badge(
    start: Option<i64>,
    due: Option<(i64, bool)>,
    now: i64,
    zone: &Zone,
) -> Option<DueBadge> {
    let day = |ms: i64| label_text(date_label(day_of(ms, zone), day_of(now, zone)));
    let (text, tone) = match (start, due) {
        (None, None) => return None,
        (Some(s), None) => {
            let key = if s <= now {
                "kanban.front.started"
            } else {
                "kanban.front.starts"
            };
            (tr_with(key, &[("date", &day(s))]), DueTone::Neutral)
        }
        (s, Some((at, done))) => {
            let d = label_text(due_label(at, now, zone));
            let text = match s {
                Some(s) => tr_with("kanban.front.range", &[("start", &day(s)), ("due", &d)]),
                None => d,
            };
            (
                text,
                due_tone(Some(at), done, now).unwrap_or(DueTone::Neutral),
            )
        }
    };
    Some(DueBadge { text, tone })
}

/// Um número como o artista o escreveu: sem `.0` nos inteiros.
fn number(n: f64) -> String {
    #[allow(clippy::cast_possible_truncation)]
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Os campos de `id` marcados «mostrar na frente», pela ordem do Kanban.
fn field_chips(k: &Kanban, id: CardId, now: i64, zone: &Zone) -> Vec<FieldChip> {
    let Some(c) = k.cards.get(&id) else {
        return Vec::new();
    };
    let mut defs: Vec<_> = k
        .fields
        .values()
        .filter(|d| d.on_front && !d.meta.deleted)
        .collect();
    defs.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    let chip = |name: &str, value: &str, hue| FieldChip {
        text: if name.is_empty() {
            value.to_owned()
        } else {
            tr_with("kanban.front.field", &[("name", &name), ("value", &value)])
        },
        hue,
        check: false,
    };
    defs.into_iter()
        .filter_map(|d| {
            Some(match (c.fields.get(&d.id)?, &d.kind) {
                (FieldValue::Text(t), _) if !t.trim().is_empty() => chip(&d.name, t, None),
                (FieldValue::Number(n), _) => chip(&d.name, &number(*n), None),
                (FieldValue::Date(at), _) => {
                    let l = date_label(day_of(at.0, zone), day_of(now, zone));
                    chip(&d.name, &label_text(l), None)
                }
                (FieldValue::Checkbox(true), _) => FieldChip {
                    text: d.name.clone(),
                    hue: None,
                    check: true,
                },
                (FieldValue::Option(o), FieldKind::Dropdown(opts)) => {
                    let o = opts.iter().find(|x| x.id == *o)?;
                    chip(&d.name, &o.text, o.color)
                }
                _ => return None,
            })
        })
        .collect()
}

/// A frente do cartão `id`.
pub(crate) fn front(k: &Kanban, counts: &Counts, id: CardId) -> Option<Front> {
    let c = k.cards.get(&id)?;
    let utc;
    let zone = if let Some(z) = &counts.zone {
        z
    } else {
        utc = Zone::utc();
        &utc
    };
    let labels = c
        .labels
        .iter()
        .filter_map(|l| k.labels.get(l).filter(|l| !l.meta.deleted))
        .filter_map(|l| l.color.map(|col| (col, l.name.clone())))
        .collect();
    let cover = c.cover.map(|cv| Cover {
        fill: match cv.fill {
            CoverFill::Color(h) => Fill::Color(h),
            CoverFill::Image(f) => Fill::Image(f),
        },
        full: cv.full,
    });
    let cover_px = match cover.map(|c| c.fill) {
        Some(Fill::Image(f)) => cover::get(f, || counts.files.get(&f).cloned()),
        _ => None,
    };
    let now = counts.now;
    // O modelo do Trello não mostra datas na frente: é um molde, não um trabalho.
    let dated = !c.template;
    Some(Front {
        title: c.title.clone(),
        cover,
        cover_px,
        labels,
        fields: field_chips(k, id, now, zone),
        due: due_badge(
            c.start.map(|s| s.0),
            c.due.map(|d| (d.at.0, d.done)),
            now,
            zone,
        )
        .filter(|_| dated),
        watching: counts.me.as_ref().is_some_and(|me| c.watchers.contains(me)),
        description: !c.description.trim().is_empty(),
        comments: counts.comments.get(&id).copied().unwrap_or(0),
        attachments: counts.attachments.get(&id).copied().unwrap_or(0),
        check: counts.check.get(&id).copied(),
        votes: u32::try_from(c.votes.len()).unwrap_or(u32::MAX),
        location: c.location.trim().to_owned(),
        members: c.members.clone(),
        template: c.template,
        mirror: None,
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

/// As linhas de `text` a `px`/`weight` quebrado a `width` — o layout que o PINTOR faz (o mesmo
/// modo de desenho do texto, que pode engrossar o peso).
pub(crate) fn lines(
    ts: &mut TextSystem,
    text: &str,
    px: f32,
    weight: FontWeight,
    width: f32,
) -> usize {
    if text.is_empty() {
        return 1;
    }
    let mode = ph2d_text::active_text_rendering();
    ts.layout_for_rendering(text, px, width, weight, mode)
        .len()
        .max(1)
}

/// As linhas que o título ocupa a `width` (a quebra do pintor, `paint_text_block`).
pub(crate) fn title_lines(ts: &mut TextSystem, title: &str, width: f32) -> usize {
    lines(ts, title, m::CARD_TITLE_PX, FontWeight::MEDIUM, width)
}

/// A altura da capa inteira: o título por cima, nunca menos que [`FULL_MIN_H`].
#[allow(clippy::cast_precision_loss)]
pub(crate) fn full_h(ts: &mut TextSystem, title: &str) -> f32 {
    let inner = m::CARD_W - 2.0 * m::CARD_PAD_X;
    let n = lines(ts, title, FULL_TITLE_PX, FULL_TITLE_WEIGHT, inner) as f32;
    (FULL_PAD_TOP + n * m::LINE_H + FULL_PAD_BOTTOM).max(FULL_MIN_H)
}

fn print(f: &Front, label_text: bool) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    f.title.hash(&mut h);
    f.cover.hash(&mut h);
    label_text.hash(&mut h);
    f.labels.hash(&mut h);
    f.fields.hash(&mut h);
    badges::items(f)
        .iter()
        .map(|b| &b.text)
        .collect::<Vec<_>>()
        .hash(&mut h);
    f.members.len().hash(&mut h);
    format!("{:?}", ph2d_text::active_text_rendering()).hash(&mut h);
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
    let row_h = if label_text {
        m::LABEL_H
    } else {
        m::LABEL_H / 2.0
    };
    rows as f32 * row_h + rows.saturating_sub(1) as f32 * m::LABEL_GAP
}

/// ⭐ A altura do cartão, pela MESMA conta que o pintor percorre.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn measure(ts: &mut TextSystem, f: &Front, label_text: bool) -> f32 {
    if f.full() {
        return full_h(ts, &f.title);
    }
    let inner = m::CARD_W - 2.0 * m::CARD_PAD_X;
    let mut h = 2.0 * m::CARD_PAD_Y + cover_h(f);
    if !f.labels.is_empty() {
        h += labels_h(ts, f, label_text) + m::CARD_ROW_GAP;
    }
    h += title_lines(ts, &f.title, inner) as f32 * m::LINE_H;
    if !f.fields.is_empty() {
        h += m::CARD_ROW_GAP + badges::fields_h(ts, f, inner);
    }
    let placed = badges::place(ts, f, inner);
    if placed.h > 0.0 {
        h += m::CARD_ROW_GAP + placed.h;
    }
    h
}

/// ⭐ A altura do cartão (guardada em `cache` até mudar o que pesa nela).
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
    let h = measure(ts, f, label_text);
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
mod tests;
