//! As barras do quadro para FACILITAR (W7, módulo filho de [`super`] — `docs/MiroClone/05_facilitacao.md`):
//! na barra curta o Cronómetro e a Votação (abrem o cartão e o painel do canto); na barra de contexto
//! de notas o Carimbo (a galeria dos oito, S2) e Agrupar (por cor, etiqueta ou palavra, C1); na de
//! frames, Agrupar as notas de dentro.
//!
//! ⚠️ Um controlo que não faria nada à selecção de agora NÃO se pinta: Agrupar só com duas notas ou
//! mais (as dos frames contam).

use super::{HeroScreen, Item, paint_item, paint_panel};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_icon as icon, paint_text_centered};
use crate::zones::Rect;
use ph2d_board_edit::{ClusterBy, ClusterNames, Editor};
use ph2d_board_model::{BoardDoc, Stamp};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Color, VectorScene};

/// Um controlo de facilitar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FacItem {
    /// Abre ou fecha o cartão do cronómetro.
    Timer,
    /// Abre ou fecha o painel da votação.
    Vote,
    /// Abre ou fecha a galeria dos carimbos.
    Stamps,
    /// O carimbo `i` de [`Stamp::ALL`] (na galeria).
    Stamp(usize),
    /// Abre ou fecha as opções de agrupar.
    Cluster,
    /// A opção `i` de [`CLUSTER_BY`].
    ClusterBy(usize),
    /// W8 (K1) — a ferramenta de comentar (`C`).
    Comment,
}

pub(super) const TIMER: Item = Item::Fac(FacItem::Timer);
pub(super) const VOTE: Item = Item::Fac(FacItem::Vote);
pub(super) const COMMENT: Item = Item::Fac(FacItem::Comment);
const STAMPS: Item = Item::Fac(FacItem::Stamps);
const CLUSTER: Item = Item::Fac(FacItem::Cluster);

/// As opções de agrupar, pela ordem do Miro (Color · Tag · Keywords).
const CLUSTER_BY: [(ClusterBy, &str); 3] = [
    (ClusterBy::Color, "board.fac.cluster.color"),
    (ClusterBy::Tag, "board.fac.cluster.tag"),
    (ClusterBy::Keyword, "board.fac.cluster.keyword"),
];

/// A largura de uma opção de agrupar, em botões da barra.
const CLUSTER_W: f32 = 4.0; // LITERAL-PX-OK: largura em botões («By keyword» a `Sm`)

/// Os itens das janelas (registados na tabela da barra).
pub(super) fn flyout_items() -> Vec<Item> {
    (0..Stamp::ALL.len())
        .map(|i| Item::Fac(FacItem::Stamp(i)))
        .chain((0..CLUSTER_BY.len()).map(|i| Item::Fac(FacItem::ClusterBy(i))))
        .collect()
}

/// O grupo da barra de contexto de notas.
pub(super) fn note_group() -> Vec<Item> {
    vec![STAMPS, CLUSTER]
}

/// O grupo da barra de contexto de frames: agrupar as notas de dentro (C1).
pub(super) fn frame_group() -> Vec<Item> {
    vec![CLUSTER]
}

/// As notas seleccionadas que um carimbo mudaria (as travadas não — a mesma pergunta do
/// `Editor::stamp`, auditoria W7).
fn notes(ed: &Editor, doc: &BoardDoc) -> usize {
    ed.selection()
        .iter()
        .filter_map(|id| doc.get(*id))
        .filter(|e| e.shape().is_some_and(|s| s.kind.is_note()) && !e.locked())
        .count()
}

/// O controlo faz alguma coisa à selecção de agora? `None` = não é daqui.
pub(super) fn shows(it: Item, ed: &Editor, doc: &BoardDoc) -> Option<bool> {
    match it {
        CLUSTER => Some(ed.clusterable(doc).len() >= 2),
        STAMPS => Some(notes(ed, doc) >= 1),
        _ => None,
    }
}

/// O que está aberto agora (lido antes de pintar).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Now {
    timer: bool,
    vote: bool,
    stamps: bool,
    cluster: bool,
    comment: bool,
}

impl Now {
    pub(super) fn of(hero: &HeroScreen) -> Self {
        let f = &hero.documents.live.fac;
        Self {
            timer: super::super::board_nav::facilitate::timer_open(hero),
            vote: super::super::board_nav::facilitate::vote_open(hero),
            stamps: f.stamps_open,
            cluster: f.cluster_open,
            comment: hero.documents.live.team.commenting,
        }
    }

    /// Aceso? `None` = não é daqui.
    pub(super) fn is_on(self, it: Item) -> Option<bool> {
        Some(match it {
            TIMER => self.timer,
            VOTE => self.vote,
            STAMPS => self.stamps,
            CLUSTER => self.cluster,
            COMMENT => self.comment,
            Item::Fac(_) => false,
            _ => return None,
        })
    }
}

/// A chave i18n da dica.
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    let Item::Fac(f) = it else {
        return None;
    };
    Some(match f {
        FacItem::Timer => "board.tool.timer",
        FacItem::Vote => "board.tool.vote",
        FacItem::Stamps => "board.fac.stamp",
        FacItem::Stamp(i) => stamp_key(*Stamp::ALL.get(i)?),
        FacItem::Cluster => "board.fac.cluster",
        FacItem::ClusterBy(i) => CLUSTER_BY.get(i)?.1,
        FacItem::Comment => "board.tool.comment",
    })
}

fn stamp_key(s: Stamp) -> &'static str {
    match s {
        Stamp::ThumbsUp => "board.stamp.thumbs_up",
        Stamp::ThumbsDown => "board.stamp.thumbs_down",
        Stamp::Heart => "board.stamp.heart",
        Stamp::Star => "board.stamp.star",
        Stamp::Flame => "board.stamp.flame",
        Stamp::Lightbulb => "board.stamp.lightbulb",
        Stamp::Question => "board.stamp.question",
        Stamp::Party => "board.stamp.party",
    }
}

/// O desenho de um controlo daqui. `false` = não é daqui.
pub(super) fn paint_icon(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    it: Item,
    r: Rect,
    inner: Rect,
    fg: Color,
    line: f64,
) -> bool {
    let Item::Fac(f) = it else {
        return false;
    };
    let w = line as f32;
    let at = [f64::from(inner.x), f64::from(inner.y)];
    match f {
        FacItem::Timer => icon(scene, IconId::Timer, inner, fg, w),
        FacItem::Vote => icon(scene, IconId::Vote, inner, fg, w),
        FacItem::Cluster => icon(scene, IconId::LayoutGrid, inner, fg, w),
        FacItem::Comment => icon(scene, IconId::MessageSquare, inner, fg, w),
        // O carimbo é o MESMO desenho do quadro (`ph2d_board_render::paint_stamp`), nunca um segundo.
        FacItem::Stamps => {
            ph2d_board_render::paint_stamp(
                scene,
                Stamp::ThumbsUp,
                at,
                f64::from(inner.w),
                Affine::IDENTITY,
            );
        }
        FacItem::Stamp(i) => {
            let Some(s) = Stamp::ALL.get(i) else {
                return false;
            };
            ph2d_board_render::paint_stamp(scene, *s, at, f64::from(inner.w), Affine::IDENTITY);
        }
        FacItem::ClusterBy(i) => {
            let Some((_, key)) = CLUSTER_BY.get(i) else {
                return false;
            };
            paint_text_centered(text_system, scene, tr(key), r, TypeToken::Sm.px(), fg);
        }
    }
    true
}

/// Os rectângulos de uma janela aberta, por baixo do botão que a abriu (`bar` = a barra de contexto
/// pintada agora).
pub(super) fn flyout_rects(bar: &[(Item, Rect)], now: Now) -> Vec<(Item, Rect)> {
    let (s, g) = (super::btn_px(), super::gap());
    let below = |it: Item| {
        bar.iter()
            .find(|(x, _)| *x == it)
            .map(|(_, r)| (r.x, r.y + r.h + Spacing::Sm.px()))
    };
    if now.stamps
        && let Some((x, y)) = below(STAMPS)
    {
        return (0..Stamp::ALL.len())
            .map(|i| {
                let r = Rect::new(x + i as f32 * (s + g), y, s, s);
                (Item::Fac(FacItem::Stamp(i)), r)
            })
            .collect();
    }
    if now.cluster
        && let Some((x, y)) = below(CLUSTER)
    {
        return (0..CLUSTER_BY.len())
            .map(|i| {
                let r = Rect::new(x, y + i as f32 * (s + g), s * CLUSTER_W, s);
                (Item::Fac(FacItem::ClusterBy(i)), r)
            })
            .collect();
    }
    Vec::new()
}

/// Pinta a janela aberta.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_flyout(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    bar: &[(Item, Rect)],
    now: Now,
) {
    let rects = flyout_rects(bar, now);
    paint_panel(scene, &rects, theme);
    for (it, r) in rects {
        paint_item(scene, text_system, theme, hit_index, store, it, r, false);
    }
}

/// Um clique na barra: os dos Carimbos e de Agrupar abrem ou fecham a janela deles (abrir uma fecha as
/// outras); qualquer outro fecha-as. Corre em TODO clique da barra.
pub(super) fn toggle_flyouts(live: &mut crate::documents::BoardLive, it: Item) {
    let f = &mut live.fac;
    f.stamps_open = it == STAMPS && !f.stamps_open;
    f.cluster_open = it == CLUSTER && !f.cluster_open;
    if f.stamps_open || f.cluster_open {
        live.shapes_open = false;
        live.templates_open = false;
    }
}

/// As janelas fecham-se quando a selecção deixa de ter notas.
pub(super) fn sync_flyouts(f: &mut crate::documents::fac::Fac, has_notes: bool) {
    f.stamps_open &= has_notes;
    f.cluster_open &= has_notes;
}

/// ⭐ Os cliques daqui. `false` = não é daqui.
pub(super) fn apply(hero: &mut HeroScreen, it: Item) -> bool {
    let Item::Fac(f) = it else {
        return false;
    };
    match f {
        FacItem::Timer => super::super::board_nav::facilitate::toggle_timer(hero),
        FacItem::Vote => super::super::board_nav::facilitate::toggle_vote(hero),
        FacItem::Comment => super::super::board_nav::team::toggle_comment_tool(hero),
        FacItem::Stamps | FacItem::Cluster => {}
        FacItem::Stamp(i) => {
            let theme = hero.theme;
            if let (Some(s), Some((board, live))) =
                (Stamp::ALL.get(i), hero.documents.active_parts())
            {
                let history = live.histories.entry(board.id).or_default();
                let ed = super::super::board_view::editor(&mut live.editor, theme);
                ed.stamp(&mut board.doc, history, *s);
            }
        }
        FacItem::ClusterBy(i) => {
            let theme = hero.theme;
            if let (Some((by, _)), Some((board, live))) =
                (CLUSTER_BY.get(i), hero.documents.active_parts())
            {
                let history = live.histories.entry(board.id).or_default();
                let ed = super::super::board_view::editor(&mut live.editor, theme);
                let names = ClusterNames {
                    no_tag: tr("board.fac.cluster.no_tag").to_owned(),
                    uncategorized: tr("board.fac.cluster.uncategorized").to_owned(),
                };
                ed.cluster(&mut board.doc, history, *by, &names);
                if let Some(n) = ed.take_notice() {
                    live.notice = Some((n.key(), std::time::Instant::now()));
                }
            }
        }
    }
    true
}
