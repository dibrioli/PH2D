//! ⭐ **A VISTA de um Kanban** (W4, lote V) — dispor (alvos → molas → ecrã), seguir o arrasto e pintar,
//! numa área em px da VISTA levada ao ecrã por uma `Affine`. A aba é um chamador (identidade); o
//! elemento do Quadro é outro (a câmara do canvas × o zoom interno, [`crate::canvas`]). O texto e o
//! clique leem a MESMA disposição vista, nas coordenadas da vista.

use crate::front::{self, Counts, Front};
use crate::layout::{self, Heights, Layout, Lifted};
use crate::metrics as m;
use crate::paint_card::CardLook;
use crate::tab::{Lift, Tab};
use ph2d_board_render::KanbanLod;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, Kanban, KanbanSet, Member};
use ph2d_spring::Spring;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};
use std::collections::BTreeMap;

/// O que todas as vistas de um quadro de imagem partilham.
#[derive(Clone, Debug)]
pub(crate) struct ViewCtx {
    pub dt: f64,
    pub law: Option<Spring>,
    pub me: Member,
    pub team: Vec<String>,
    /// O chrome do hero por cima (o HUD de baixo): as listas param acima dele (só a aba).
    pub floor: Option<f32>,
    /// J1 — a partilha ao vivo: o papel e quem vê cada Kanban.
    pub share: crate::share::Look,
}

impl Default for ViewCtx {
    fn default() -> Self {
        Self {
            dt: 0.0,
            law: None,
            me: Member(ph2d_i18n::tr("kanban.back.me").to_owned()),
            team: Vec::new(),
            floor: None,
            share: crate::share::Look::default(),
        }
    }
}

/// Onde e como se pinta uma vista.
#[derive(Clone, Copy, Debug)]
pub(crate) struct At {
    /// Px da vista → ecrã.
    pub to_screen: Affine,
    /// A área da vista, em px da vista.
    pub area: Rect,
    /// `High` = tudo; `Mid` = títulos e cartões só com título e capa (sem distintivos nem menus).
    pub lod: KanbanLod,
    /// Pinta o verso aberto aqui (a aba); o elemento pinta-o por cima do quadro, em ecrã.
    pub modal: bool,
}

/// A frente de um cartão ao nível médio: o título e a capa.
fn titles_only(f: Front) -> Front {
    Front {
        title: f.title,
        cover: f.cover,
        cover_px: f.cover_px,
        labels: Vec::new(),
        fields: Vec::new(),
        due: None,
        watching: false,
        description: false,
        comments: 0,
        attachments: 0,
        check: None,
        votes: 0,
        location: String::new(),
        members: Vec::new(),
        template: f.template,
        mirror: f.mirror,
    }
}

/// A frente de uma linha ao nível `lod` (um espelho mostra o original, com o selo).
struct Fronts<'a> {
    set: &'a KanbanSet,
    k: &'a Kanban,
    counts: &'a Counts,
    others: &'a BTreeMap<ph2d_kanban_model::KanbanId, Counts>,
    lod: KanbanLod,
}

impl Fronts<'_> {
    fn get(&self, rows: &crate::rows::Rows, id: CardId) -> Option<Front> {
        let f = crate::rows::front_of(self.set, self.k, self.counts, self.others, rows, id)?;
        Some(if self.lod == KanbanLod::High {
            f
        } else {
            titles_only(f)
        })
    }
}

/// ⭐ Um quadro de imagem de uma vista: dispõe, segue o arrasto e pinta em `scene` (por `at.to_screen`).
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    at: At,
    tab: &mut Tab,
    set: &KanbanSet,
    cx: &ViewCtx,
) {
    let Some(k) = set.get(tab.view.kanban) else {
        return;
    };
    let identity = at.to_screen == Affine::IDENTITY;
    let mut local = VectorScene::new();
    let out = if identity { &mut *scene } else { &mut local };
    let area = at.area;
    edge_scroll(tab, cx.dt);
    *tab.painted.borrow_mut() = crate::tab::Painted::default();
    let counts = Counts::of(k)
        .seen_by(Some(cx.me.clone()))
        .with_files(set, k);
    let zone = ph2d_kanban_dates::Zone::system();
    tab.rows = {
        let ask = crate::rows::Ask {
            filter: &tab.w6.filter,
            now: crate::actions::now(),
            zone: &zone,
            me: &cx.me,
            aging: k.header.aging.is_some(),
        };
        crate::rows::build(set, k, &ask)
    };
    let others = crate::rows::sources(set, k, &tab.rows, &cx.me);
    let fronts = Fronts {
        set,
        k,
        counts: &counts,
        others: &others,
        lod: at.lod,
    };
    let title_w =
        crate::docs::title_width(ts, &k.header.name, m::HEADER_TITLE_PX) + 2.0 * m::FIELD_PAD_X;
    let lifted = lifted(tab, &fronts, ts);
    let mut cache = std::mem::take(&mut tab.heights);
    let target = {
        let mut measure = Measure {
            ts,
            cache: &mut cache,
            fronts: &fronts,
            rows: &tab.rows,
            label_text: tab.label_text,
        };
        layout::targets(
            k,
            &tab.rows,
            &tab.view,
            area,
            (title_w, cx.floor),
            tab.field.as_ref(),
            lifted,
            &mut measure,
        )
    };
    tab.heights = cache;
    clamp_scroll(tab, &target);
    let seen = crate::tab::place(tab, target.clone(), cx.dt, cx.law);
    tab.layout = seen;
    retarget(tab, &target);
    let lod = at.lod;
    tab.to_screen = at.to_screen;
    if tab.other.is_some() {
        crate::paint::chrome(out, ts, theme, k, tab);
        if lod == KanbanLod::High {
            crate::header::other::paint(out, ts, theme, tab, set, (&cx.me, &cx.team, cx.floor));
        }
    } else {
        let t: &Tab = tab;
        let mut looks = |cid: CardId| {
            fronts.get(&t.rows, cid).map(|f| CardLook {
                front: f,
                hover: false,
                focus: false,
                age: t.rows.ages.get(&cid).copied().unwrap_or(0),
            })
        };
        crate::paint::board(out, ts, theme, k, t, &mut looks);
    }
    if let Some(seen) = cx.share.seen.get(&k.id) {
        crate::presence::paint(out, ts, theme, tab, seen);
    }
    if lod == KanbanLod::High {
        crate::overlay::paint(out, ts, theme, k, tab);
    }
    let people = (&cx.me, cx.team.as_slice(), cx.floor);
    crate::w6::paint(out, ts, theme, (k, set), tab, people, lod, &cx.share);
    if at.modal {
        crate::back::paint(
            out,
            ts,
            theme,
            k,
            tab,
            set,
            &cx.me,
            &cx.team,
            crate::actions::now(),
        );
    }
    if at.modal || tab.back.is_none() {
        crate::w6::paint_dialog(
            out,
            ts,
            theme,
            set,
            tab,
            crate::w6::above(tab.layout.area, cx.floor),
        );
    }
    if !identity {
        scene.append_transformed(&local, at.to_screen);
    }
}

/// O medidor de alturas que a disposição pede, sobre o texto e a cache da aba.
struct Measure<'a> {
    ts: &'a mut TextSystem,
    cache: &'a mut BTreeMap<CardId, (u64, f32)>,
    fronts: &'a Fronts<'a>,
    rows: &'a crate::rows::Rows,
    label_text: bool,
}

impl Heights for Measure<'_> {
    fn card(&mut self, _k: &Kanban, id: CardId) -> f32 {
        self.fronts.get(self.rows, id).map_or(m::LINE_H, |f| {
            front::height(self.ts, self.cache, id, &f, self.label_text)
        })
    }

    #[allow(clippy::cast_precision_loss)]
    fn field(&mut self, text: &str, width: f32) -> f32 {
        front::title_lines(self.ts, text, width) as f32 * m::LINE_H
    }

    fn title_w(&mut self, text: &str) -> f32 {
        crate::list_title::width(self.ts, text)
    }

    fn limit_w(&mut self, text: &str) -> f32 {
        crate::docs::title_width(self.ts, text, m::SMALL_PX) + 2.0 * m::COUNT_PAD_X
    }
}

/// O que voa sai do fluxo da disposição e abre o espaço dele onde cai.
fn lifted(tab: &mut Tab, fronts: &Fronts<'_>, ts: &mut TextSystem) -> Option<Lifted> {
    let d = tab.drag?;
    match d.lift {
        Lift::Card(id) => {
            let h = fronts.get(&tab.rows, id).map_or(d.size.1, |f| {
                front::height(ts, &mut tab.heights, id, &f, tab.label_text)
            });
            Some(Lifted::Card {
                id,
                to: d.to_card,
                h,
            })
        }
        Lift::List(id) => Some(Lifted::List {
            id,
            to: d.to_list,
            w: d.size.0,
            h: d.size.1,
        }),
        Lift::Pan { .. } => None,
    }
}

/// A rolagem não passa do fim (uma lista que encolheu, o quadro que estreitou).
fn clamp_scroll(tab: &mut Tab, target: &Layout) {
    let max_x = layout::max_scroll_x(target);
    tab.view.scroll_x = tab.view.scroll_x.clamp(0.0, f64::from(max_x));
    for lb in &target.lists {
        let max_y = f64::from(layout::max_scroll_y(lb));
        if let Some(v) = tab.view.scroll_y.get_mut(&lb.id) {
            *v = v.clamp(0.0, max_y);
        }
    }
}

/// A fundura `0 → 1` de `p` na faixa junto à borda `[lo, hi]` (negativa = a de baixo/esquerda).
fn edge(p: f32, lo: f32, hi: f32) -> f32 {
    let band = m::EDGE_SCROLL_BAND.min((hi - lo) / 3.0).max(1.0);
    if p < lo + band {
        -((lo + band - p) / band).min(1.0)
    } else if p > hi - band {
        ((p - (hi - band)) / band).min(1.0)
    } else {
        0.0
    }
}

/// ⭐ A rolagem nas bordas durante um arrasto (o Trello): o quadro na horizontal, a lista por baixo
/// do ponteiro na vertical; mais depressa quanto mais fundo na faixa.
#[allow(clippy::cast_possible_truncation)]
fn edge_scroll(tab: &mut Tab, dt: f64) {
    let Some(d) = tab.drag else {
        return;
    };
    if matches!(d.lift, Lift::Pan { .. }) {
        return;
    }
    let speed = f64::from(m::EDGE_SCROLL_MAX_PX_S) * dt;
    let b = tab.layout.board;
    let ex = edge(d.at.0, b.x, b.x + b.w);
    tab.view.scroll_x += f64::from(ex) * speed;
    if let Lift::Card(_) = d.lift
        && let Some(lb) = tab
            .layout
            .lists
            .iter()
            .find(|l| !l.collapsed && l.rect.x <= d.at.0 && d.at.0 <= l.rect.x + l.rect.w)
    {
        let ey = edge(d.at.1, lb.body.y, lb.body.y + lb.body.h);
        if ey != 0.0 && (lb.body.y..=lb.body.y + lb.body.h).contains(&d.at.1) {
            *tab.view.scroll_y.entry(lb.id).or_default() += f64::from(ey) * speed;
        }
    }
}

/// ⭐ Onde cai o que voa — medido nos ALVOS sem o espaço aberto (senão o espaço que abre empurra a
/// fronteira e o alvo oscila entre dois sítios).
fn retarget(tab: &mut Tab, target: &Layout) {
    let Some(mut d) = tab.drag else {
        return;
    };
    let ox = target.board.x + m::BOARD_PAD;
    #[allow(clippy::cast_possible_truncation)]
    let sx = tab.view.scroll_x as f32;
    let cx = d.at.0 - d.grab.0 + d.size.0 / 2.0 - ox + sx;
    match d.lift {
        Lift::Card(id) => {
            let open: Vec<_> = target.lists.iter().filter(|l| !l.collapsed).collect();
            let Some(lb) = open
                .iter()
                .find(|l| cx < l.rect.x + l.rect.w + m::LIST_GAP / 2.0)
                .or(open.last())
            else {
                d.to_card = None;
                tab.drag = Some(d);
                return;
            };
            let sy = tab.scroll_y(lb.id);
            let cy = d.at.1 - d.grab.1 + d.size.1 / 2.0 - (lb.body.y + m::LIST_BODY_TOP) + sy;
            let mut acc = 0.0_f32;
            let mut index = 0usize;
            for c in lb.cards.iter().filter(|c| c.id != id) {
                if acc + c.rect.h / 2.0 > cy {
                    break;
                }
                acc += c.rect.h + m::CARD_GAP;
                index += 1;
            }
            d.to_card = Some((lb.id, index));
        }
        Lift::List(id) => {
            let mut acc = 0.0_f32;
            let mut index = 0usize;
            for l in target.lists.iter().filter(|l| l.id != id) {
                if acc + l.rect.w / 2.0 > cx {
                    break;
                }
                acc += l.rect.w + m::LIST_GAP;
                index += 1;
            }
            d.to_list = index;
        }
        Lift::Pan { .. } => {}
    }
    tab.drag = Some(d);
}
