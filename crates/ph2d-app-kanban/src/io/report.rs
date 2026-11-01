//! ⭐ **O relatório da importação** — o modal sobre a aba do Kanban importado: o que entrou (as
//! contagens) e o que ficou de fora (cada perda agrupada pelo porquê, com o objecto do Trello por
//! baixo). Pinta-se como uma folha da W6, centrada, sobre o véu; «Done», `Esc`, `Enter` ou um clique
//! fora fecham-no.

use crate::docs::KanbanDocs;
use crate::w6::Act;
use crate::w6::sheet::{self, Item, Sheet};
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::widget::{AxisBar, BarGrab};
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
    /// A barra do relatório que rola e o agarrar dela (a das folhas da W6).
    bar: Option<AxisBar>,
    grab: Option<BarGrab>,
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
        bar: None,
        grab: None,
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
            // O porquê no texto do corpo; cada objecto do Trello por baixo, pequeno e com marca.
            let what = tr(super::loss_key(kind));
            sh.items.push((
                Item::Entry {
                    text: tr_with(
                        "kanban.import.loss_group",
                        &[("what", &what), ("n", &details.len())],
                    ),
                    icon: None,
                    on: false,
                },
                Act::Inside,
            ));
            for d in details.into_iter().filter(|d| !d.is_empty()) {
                sh.items
                    .push(note(tr_with("kanban.import.loss_item", &[("what", &d)])));
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
        scene.fill_rect(
            rect_to_vello(area),
            crate::paint::c(theme, T::KanbanShadowFlight),
        );
        let area = Rect::new(area.x, area.y, area.w, (floor - area.y).max(0.0));
        paint_open(s, area, scene, ts, theme);
    });
}

/// O relatório centrado em `area`, com a barra se rola.
fn paint_open(
    s: &mut Open,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let w = W.min(area.w - 2.0 * sheet::PAD);
    let (sh, rect) = sheet::lay(&build(s), ts, w, |sh| {
        let h = sh.content_h().min(area.h - 2.0 * sheet::PAD);
        Rect::new(
            area.x + (area.w - w) / 2.0,
            area.y + (area.h - h) / 2.0,
            w,
            h,
        )
    });
    s.max = (sh.content_h() - rect.h).max(0.0);
    s.scroll = s.scroll.clamp(0.0, s.max);
    s.bar = sheet::bar(&sh, rect).map(|(_, b)| b);
    let hover = s.grab.map_or(s.hover, |_| Some(Act::Bar));
    s.hits = sheet::paint(scene, ts, theme, &sh, rect, s.scroll, hover);
}

/// Cada quadro: a seta segura do relatório repete. `true` = há uma.
pub(crate) fn tick(docs: &mut KanbanDocs, now: u128) -> bool {
    docs.import_report.as_mut().is_some_and(|s| step(s, now))
}

fn step(s: &mut Open, now: u128) -> bool {
    let (Some(b), Some(g)) = (s.bar, s.grab.as_mut()) else {
        return false;
    };
    b.tick(g, s.scroll, now).map(|v| s.scroll = v).is_some()
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
    let now = docs.clock.0;
    let shut = with(docs, tab, |s| on(s, kind, (x, y), now))?;
    if shut {
        docs.import_report = None;
    }
    Some(true)
}

/// O rato sobre o relatório: a barra agarra, segue e solta; senão o clique. `true` = fecha-o.
fn on(s: &mut Open, kind: PointerKind, (x, y): (f32, f32), now: u128) -> bool {
    let at = hit(s, x, y);
    if let (Some(b), PointerKind::Down, Some(Act::Bar)) = (s.bar, kind, at) {
        let (v, g) = b.grab((x, y), s.scroll, now);
        (s.scroll, s.grab) = (v, Some(g));
        return false;
    }
    if let (Some(b), Some(g)) = (s.bar, s.grab.as_mut()) {
        match kind {
            PointerKind::Move => s.scroll = b.follow(g, (x, y), s.scroll),
            _ => s.grab = None,
        }
        return false;
    }
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
                    | Item::Entry { text: t, .. }
                    | Item::Button { text: t, .. } => Some(t),
                    _ => None,
                })
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_kanban_import::Loss;

    /// O relatório comprido (o smoke 9): «Done» fica por baixo; segurar ▼ da barra rola até ao fim
    /// e pára, e lá «Done» está inteiro à vista e fecha.
    #[test]
    fn the_report_done_is_reached_by_holding_the_bar_arrow() {
        let loss = |i: usize| Loss {
            kind: LossKind::PowerUp,
            detail: format!("Power-Up {i}"),
        };
        let report = ImportReport {
            losses: (0..40).map(loss).collect(),
            ..ImportReport::default()
        };
        let mut s = Open {
            tab: 1,
            name: "Lua".into(),
            report,
            scroll: 0.0,
            max: 0.0,
            hover: None,
            press: None,
            hits: Vec::new(),
            bar: None,
            grab: None,
        };
        let (mut ts, mut scene) = (TextSystem::without_system_fonts(), VectorScene::new());
        let area = Rect::new(0.0, 0.0, 1000.0, 500.0);
        paint_open(&mut s, area, &mut scene, &mut ts, Theme::default());
        let done = |s: &Open| {
            let top = s.hits[0].0.y + sheet::TITLE_H;
            s.hits
                .iter()
                .find(|(r, a)| *a == Act::Close && r.y >= top)
                .map(|h| h.0)
        };
        assert!(done(&s).is_none(), "«Done» começa escondido");
        let b = s.bar.expect("a barra do relatório que rola");
        let down = (b.track.x + 1.0, b.track.y + b.track.h - 1.0);
        assert!(!on(&mut s, PointerKind::Down, down, 0));
        assert!(step(&mut s, 60_000_000_000) && s.scroll == s.max && s.max > 0.0);
        assert!(!on(&mut s, PointerKind::Up, down, 0) && !step(&mut s, 1));
        paint_open(&mut s, area, &mut scene, &mut ts, Theme::default());
        let d = done(&s).expect("«Done» à vista no fim");
        let rect = s.hits[0].0;
        assert!(d.y + d.h <= rect.y + rect.h, "inteiro: {d:?} em {rect:?}");
        let at = (d.x + d.w / 2.0, d.y + d.h / 2.0);
        assert!(!on(&mut s, PointerKind::Down, at, 0) && on(&mut s, PointerKind::Up, at, 0));
    }
}
