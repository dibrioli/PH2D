//! ⭐ **O relatório da importação** — o modal sobre a aba do Kanban importado: o que entrou (as
//! contagens) e o que ficou de fora (cada perda agrupada pelo porquê, com o objecto do Trello por
//! baixo). Pinta-se como uma folha da W6, centrada, sobre o véu; «Done», `Esc`, `Enter` ou um clique
//! fora fecham-no.

use crate::docs::KanbanDocs;
use crate::w6::Act;
use crate::w6::sheet::{self, Item, Sheet};
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_import::{Counts, ImportReport, LossKind};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// A largura do relatório: duas folhas da W6 (as frases das perdas e os nomes do Trello são longos).
const W: f32 = 2.0 * sheet::W;

/// O relatório aberto sobre uma aba (o campo `KanbanDocs::import_report`).
pub(crate) struct Open {
    tab: u64,
    name: String,
    report: ImportReport,
    scroll: f32,
    max: f32,
    hover: Option<Act>,
    press: Option<Option<Act>>,
    hits: Vec<(Rect, Act)>,
}

pub(super) fn show(docs: &mut KanbanDocs, tab: u64, name: String, report: ImportReport) {
    docs.import_report = Some(Open {
        tab,
        name,
        report,
        scroll: 0.0,
        max: 0.0,
        hover: None,
        press: None,
        hits: Vec::new(),
    });
}

fn with<R>(docs: &mut KanbanDocs, tab: u64, f: impl FnOnce(&mut Open) -> R) -> Option<R> {
    docs.import_report.as_mut().filter(|s| s.tab == tab).map(f)
}

fn counts(c: &Counts) -> [(&'static str, usize); 11] {
    [
        ("kanban.import.count.lists", c.lists),
        ("kanban.import.count.cards", c.cards),
        ("kanban.import.count.labels", c.labels),
        ("kanban.import.count.checklists", c.checklists),
        ("kanban.import.count.items", c.items),
        ("kanban.import.count.comments", c.comments),
        ("kanban.import.count.attachments", c.attachments),
        ("kanban.import.count.fields", c.fields),
        ("kanban.import.count.activities", c.activities),
        ("kanban.import.count.members", c.members),
        ("kanban.import.count.files", c.files),
    ]
}

fn build(s: &Open) -> Sheet {
    let mut sh = Sheet::new(W);
    sh.title("kanban.import.report_title");
    let note = |t: String| (Item::Note(t), Act::Inside);
    let caption = |t: String| (Item::Caption(t), Act::Inside);
    sh.items.push(note(tr_with(
        "kanban.import.report_intro",
        &[("name", &s.name)],
    )));
    sh.caption("kanban.import.counts_caption");
    for (key, n) in counts(&s.report.counts) {
        if n > 0 {
            sh.items.push(note(tr_with(key, &[("n", &n)])));
        }
    }
    let losses = &s.report.losses;
    if losses.is_empty() {
        sh.items
            .push(note(tr("kanban.import.losses_none").to_owned()));
    } else {
        sh.items.push(caption(tr_with(
            "kanban.import.losses_caption",
            &[("n", &losses.len())],
        )));
        let mut groups: BTreeMap<LossKind, Vec<&str>> = BTreeMap::new();
        for l in losses {
            groups.entry(l.kind).or_default().push(&l.detail);
        }
        for (kind, details) in groups {
            let what = tr(super::loss_key(kind));
            sh.items.push(note(tr_with(
                "kanban.import.loss_group",
                &[("what", &what), ("n", &details.len())],
            )));
            for d in details.into_iter().filter(|d| !d.is_empty()) {
                sh.items.push((
                    Item::Choice {
                        on: false,
                        text: d.to_owned(),
                    },
                    Act::Inside,
                ));
            }
        }
    }
    sh.push(
        Item::Button {
            text: tr("kanban.import.done").to_owned(),
            primary: true,
        },
        Act::Close,
    );
    sh
}

/// Pinta o relatório sobre a aba `tab`, se ele é dela.
pub(crate) fn paint(
    docs: &mut KanbanDocs,
    tab: u64,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    // O relatório pára acima do chrome do hero (o HUD de baixo), como as listas.
    let floor = docs
        .over
        .map_or(area.y + area.h, |o| o.y.min(area.y + area.h));
    with(docs, tab, |s| {
        let sh = build(s);
        scene.fill_rect(
            rect_to_vello(area),
            crate::paint::c(theme, T::KanbanShadowFlight),
        );
        let area = Rect::new(area.x, area.y, area.w, (floor - area.y).max(0.0));
        let w = sh.w.min(area.w - 2.0 * sheet::PAD);
        let h = sh.content_h().min(area.h - 2.0 * sheet::PAD);
        let rect = Rect::new(
            area.x + (area.w - w) / 2.0,
            area.y + (area.h - h) / 2.0,
            w,
            h,
        );
        s.max = (sh.content_h() - h).max(0.0);
        s.scroll = s.scroll.clamp(0.0, s.max);
        s.hits = sheet::paint(scene, ts, theme, &sh, rect, s.scroll, s.hover);
    });
}

fn hit(s: &Open, x: f32, y: f32) -> Option<Act> {
    s.hits
        .iter()
        .rev()
        .find(|(r, _)| r.contains(x, y))
        .map(|(_, a)| *a)
}

/// O rato com o relatório aberto: é todo dele (`None` = não há relatório nesta aba).
pub(crate) fn pointer(
    docs: &mut KanbanDocs,
    tab: u64,
    kind: PointerKind,
    x: f32,
    y: f32,
) -> Option<bool> {
    let shut = with(docs, tab, |s| {
        let at = hit(s, x, y);
        match kind {
            PointerKind::Move => {
                s.hover = at;
                false
            }
            PointerKind::Down => {
                s.press = Some(at);
                false
            }
            // Um clique em «Done» ou fora do relatório (no véu) fecha-o.
            PointerKind::Up => s.press.take() == Some(at) && matches!(at, None | Some(Act::Close)),
        }
    })?;
    if shut {
        docs.import_report = None;
    }
    Some(true)
}

/// A roda rola o relatório comprido.
pub(crate) fn wheel(docs: &mut KanbanDocs, tab: u64, dy: f32) -> bool {
    with(docs, tab, |s| s.scroll = (s.scroll + dy).clamp(0.0, s.max)).is_some()
}

/// O teclado com o relatório aberto: `Esc` e `Enter` fecham-no, o resto não passa para o Kanban.
pub(crate) fn key(docs: &mut KanbanDocs, tab: u64, input: &KeyInput<'_>) -> Option<KeyOutcome> {
    with(docs, tab, |_| ())?;
    if input.pressed && matches!(input.key, Key::Escape | Key::Enter) {
        docs.import_report = None;
    }
    Some(KeyOutcome {
        consumed: true,
        copy: None,
    })
}

/// Os textos do relatório aberto sobre `tab`, pela ordem em que se pintam.
pub(super) fn texts(docs: &KanbanDocs, tab: u64) -> Option<Vec<String>> {
    docs.import_report
        .as_ref()
        .filter(|s| s.tab == tab)
        .map(|s| {
            build(s)
                .items
                .into_iter()
                .filter_map(|(i, _)| match i {
                    Item::Title(t)
                    | Item::Caption(t)
                    | Item::Note(t)
                    | Item::Choice { text: t, .. }
                    | Item::Button { text: t, .. } => Some(t),
                    _ => None,
                })
                .collect()
        })
}
