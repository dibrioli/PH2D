//! **Nota adesiva ↔ cartão** no Quadro (o Miro). Converter SUBSTITUI: as notas saem do Quadro e os
//! cartões nascem na lista escolhida (e o inverso), sem cópia que divirja. O gesto é UM passo da
//! história do Quadro; o lado do Kanban acompanha-o em [`serve`]: desfeito o passo, os cartões que
//! nasceram e ficaram intocados saem (a nota volta); refeito, voltam. Um cartão já mexido fica —
//! nunca se apaga trabalho (a lei do `board_kanban_link::after_history`).

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ph2d_board_model::{BoardId, BoardOp, ElementId, MAX_STEPS, ShapeType};
use ph2d_kanban_links::notes::{card_from_note, label_for_note, note_color_for, note_from_card};
use ph2d_kanban_model::{
    Card, CardId, CardRef, FracKey, KanbanId, KanbanOp, KanbanSet, Label, LabelColor, LabelId,
    ListId, Meta, Moment, Record, RecordId,
};

use ph2d_text::TextSystem;

use crate::screens::hero::HeroScreen;
use crate::screens::hero::board_view::editor;

/// Uma lista onde as notas podem ir (o popover da escolha mostra os dois nomes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub kanban: KanbanId,
    pub list: ListId,
    pub kanban_name: String,
    pub list_name: String,
}

/// Uma conversão feita, para o lado do Kanban acompanhar o desfazer do Quadro.
struct Pair {
    set: Weak<RefCell<KanbanSet>>,
    board: BoardId,
    notes: Vec<ElementId>,
    /// `true` = convertido quando as notas EXISTEM (cartão → nota); `false` = quando saíram.
    alive_means_done: bool,
    kanban: KanbanId,
    forward: Vec<KanbanOp>,
    backward: Vec<KanbanOp>,
    /// Os registos do Kanban como o último lado aplicado os deixou (o «intocado»).
    after: Vec<Record>,
    applied: bool,
}

thread_local! {
    static PAIRS: RefCell<Vec<Pair>> = const { RefCell::new(Vec::new()) };
    static TO_NOTE: RefCell<Vec<CardRef>> = const { RefCell::new(Vec::new()) };
}

/// Uma nota que se converte (a pilha não: é a fonte de notas novas).
fn is_note(k: ShapeType) -> bool {
    matches!(k, ShapeType::Sticky | ShapeType::StickyWide)
}

/// As listas onde as notas podem ir: primeiro as dos Kanbans postos no Quadro activo.
#[must_use]
pub fn choices(hero: &HeroScreen) -> Vec<Choice> {
    let on_board = hero
        .documents
        .active()
        .and_then(|b| hero.documents.boards().get(b))
        .map(|b| crate::screens::hero::board_video::kanban::link::refs(&b.doc))
        .unwrap_or_default();
    let set = hero.documents.kanbans();
    let mut ks: Vec<_> = set.iter().collect();
    ks.sort_by_key(|k| !on_board.contains(&k.id));
    ks.into_iter()
        .flat_map(|k| {
            k.lists_in_order().into_iter().map(|l| Choice {
                kanban: k.id,
                list: l.id,
                kanban_name: k.header.name.clone(),
                list_name: l.title.clone(),
            })
        })
        .collect()
}

/// As notas seleccionadas no Quadro activo, por ordem de leitura (de cima para baixo, da esquerda
/// para a direita) — vazio = o menu não oferece «Convert to card».
#[must_use]
pub fn selected_notes(hero: &mut HeroScreen) -> Vec<ElementId> {
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return Vec::new();
    };
    let ed = editor(&mut live.editor, theme);
    let mut v: Vec<_> = ed
        .selection()
        .iter()
        .filter_map(|id| board.doc.get(*id))
        .filter(|el| el.shape().is_some_and(|s| is_note(s.kind)))
        .map(|el| (el.y, el.x, el.id))
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    v.into_iter().map(|(.., id)| id).collect()
}

/// Os registos que `ops` escrevem, como estão agora.
fn records(set: &KanbanSet, k: KanbanId, ops: &[KanbanOp]) -> Vec<Record> {
    let Some(doc) = set.get(k) else {
        return Vec::new();
    };
    ops.iter()
        .filter_map(KanbanOp::target)
        .filter_map(|id| doc.record(id))
        .collect()
}

fn remember(p: Pair) {
    PAIRS.with(|v| {
        let mut v = v.borrow_mut();
        v.push(p);
        // Mais antigas do que a história do Quadro guarda já não se desfazem.
        let over = v.len().saturating_sub(MAX_STEPS);
        v.drain(..over);
    });
}

/// ⭐ «Convert to card»: as notas seleccionadas viram cartões no fim de `list` (título = 1.ª linha,
/// o resto em markdown, a cor → a etiqueta mais próxima, reusada se o Kanban já a tem) e saem do
/// Quadro num passo de desfazer. Devolve os cartões novos, pela ordem de leitura das notas.
pub fn notes_to_cards(
    hero: &mut HeroScreen,
    kanban: KanbanId,
    list: ListId,
    at: Moment,
) -> Vec<CardId> {
    let notes = selected_notes(hero);
    let shared = hero.documents.kanbans_shared();
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return Vec::new();
    };
    let mut ops = Vec::new();
    let mut cards = Vec::new();
    {
        let set = shared.borrow();
        let Some(k) = set
            .get(kanban)
            .filter(|k| k.lists.get(&list).is_some_and(|l| !l.meta.deleted))
        else {
            return Vec::new();
        };
        if notes.is_empty() {
            return Vec::new();
        }
        let ids = k.mint_n(notes.len() * 2);
        let mut pos = k.pos_after_cards(list);
        let mut made: Vec<(LabelColor, LabelId)> = Vec::new();
        for (i, id) in notes.iter().enumerate() {
            let Some(s) = board.doc.get(*id).and_then(|el| el.shape()) else {
                continue;
            };
            let text = card_from_note(&s.text);
            let mut card = Card::new(CardId(ids[2 * i]), list, pos.clone(), &text.title, at);
            card.description = text.description;
            if let Some(color) = s.style.fill.and_then(label_for_note) {
                let have = k
                    .labels
                    .values()
                    .filter(|l| !l.meta.deleted && l.color == Some(color))
                    .min_by_key(|l| (!l.name.is_empty(), l.id))
                    .map(|l| l.id)
                    .or_else(|| made.iter().find(|m| m.0 == color).map(|m| m.1));
                let label = have.unwrap_or_else(|| {
                    let l = LabelId(ids[2 * i + 1]);
                    made.push((color, l));
                    ops.push(KanbanOp::put(Record::Label(Label {
                        id: l,
                        name: String::new(),
                        color: Some(color),
                        meta: Meta::default(),
                    })));
                    l
                });
                card.labels.push(label);
            }
            cards.push(card.id);
            ops.push(KanbanOp::put(Record::card(card)));
            pos = FracKey::between(Some(&pos), None);
        }
    }
    let backward = shared.borrow_mut().apply(kanban, ops.clone());
    let after = records(&shared.borrow(), kanban, &ops);
    let history = live.histories.entry(board.id).or_default();
    history.apply(
        &mut board.doc,
        notes.iter().map(|id| BoardOp::Delete(*id)).collect(),
    );
    editor(&mut live.editor, theme).select(&board.doc, []);
    remember(Pair {
        set: Rc::downgrade(&shared),
        board: board.id,
        notes,
        alive_means_done: false,
        kanban,
        forward: ops,
        backward,
        after,
        applied: true,
    });
    cards
}

/// «Convert to sticky note» no menu de um cartão (servido no Quadro activo, na pintura seguinte:
/// a nota mede o texto com o moldador).
pub fn request_card_to_note(card: CardRef) {
    TO_NOTE.with(|q| q.borrow_mut().push(card));
}

/// Onde a nota nasce: no Quadro activo, no centro da vista; sem Quadro activo (pedido da aba do
/// Kanban), no Quadro onde o Kanban está posto — o da última vez, ou o primeiro —, que passa a ser o
/// activo, à direita da referência. `None` = o Kanban não está em nenhum Quadro (avisa-se).
/// O passo entre duas notas convertidas de seguida, em lados de nota (unidades do MUNDO do Quadro).
const NOTE_PITCH: f64 = 1.1; // LITERAL-PX-OK: fracção do lado da nota, no mundo do Quadro
/// O vão da referência do Kanban à primeira nota, em lados de nota.
const NOTE_MARGIN: f64 = 0.6; // LITERAL-PX-OK: fracção do lado da nota, no mundo do Quadro

fn note_place(hero: &mut HeroScreen, kanban: KanbanId, nth: usize) -> Option<[f64; 2]> {
    let side = ph2d_board_model::STICKY_SIDE;
    let step = side * NOTE_PITCH * nth as f64;
    if let Some(b) = hero.documents.active_board() {
        return Some([b.camera.center_x + step, b.camera.center_y]);
    }
    use crate::screens::hero::board_video::kanban::link;
    let all = link::placements(&hero.documents, kanban);
    let last = link::last_board(&hero.documents);
    let Some(p) = all.iter().find(|p| Some(p.board) == last).or(all.first()) else {
        hero.documents
            .tell(ph2d_i18n::tr("kanban.links.to_note_no_board").to_owned());
        return None;
    };
    hero.documents.activate(Some(p.board));
    let b = hero.documents.active_board_mut()?;
    let el = b.doc.get(p.element)?;
    let centre = [el.x + el.w + side * NOTE_MARGIN + step, el.y + side / 2.0];
    b.camera.center_x = centre[0];
    b.camera.center_y = centre[1];
    Some(centre)
}

/// O cartão `r` vira uma nota (cor da 1.ª etiqueta, texto = título e descrição) no sítio de
/// [`note_place`] e sai do Kanban — um passo de desfazer no Quadro.
fn card_to_note(
    hero: &mut HeroScreen,
    ts: &mut TextSystem,
    r: CardRef,
    nth: usize,
) -> Option<ElementId> {
    let shared = hero.documents.kanbans_shared();
    let theme = hero.theme;
    let (fill, text) = {
        let set = shared.borrow();
        let k = set.get(r.kanban)?;
        let c = k.cards.get(&r.card).filter(|c| !c.meta.deleted)?;
        let color = c.labels.iter().find_map(|l| k.labels.get(l)?.color);
        (
            note_color_for(color),
            note_from_card(&c.title, &c.description),
        )
    };
    let centre = note_place(hero, r.kanban, nth)?;
    let (board, live) = hero.documents.active_parts()?;
    let ed = editor(&mut live.editor, theme);
    let el = ed.note_for_card(&mut board.doc, ts, fill, text, centre);
    let id = el.id;
    let history = live.histories.entry(board.id).or_default();
    history.apply(&mut board.doc, vec![BoardOp::Put(el)]);
    ed.select(&board.doc, [id]);
    let forward = vec![KanbanOp::Delete(RecordId::Card(r.card))];
    let backward = shared.borrow_mut().apply(r.kanban, forward.clone());
    let after = records(&shared.borrow(), r.kanban, &forward);
    remember(Pair {
        set: Rc::downgrade(&shared),
        board: board.id,
        notes: vec![id],
        alive_means_done: true,
        kanban: r.kanban,
        forward,
        backward,
        after,
        applied: true,
    });
    Some(id)
}

/// Converte os cartões pedidos em notas (na pintura do Quadro activo, com o moldador).
pub(crate) fn run_pending(hero: &mut HeroScreen, ts: &mut TextSystem) -> bool {
    let asks = TO_NOTE.with(|q| std::mem::take(&mut *q.borrow_mut()));
    let mut any = false;
    for (i, r) in asks.into_iter().enumerate() {
        any |= card_to_note(hero, ts, r, i).is_some();
    }
    any
}

/// Acompanha, no Kanban, o desfazer/refazer das conversões.
pub(crate) fn serve(hero: &mut HeroScreen) -> bool {
    let mut any = false;
    let shared = hero.documents.kanbans_shared();
    let boards = hero.documents.boards();
    PAIRS.with(|v| {
        v.borrow_mut().retain_mut(|p| {
            let Some(set) = p.set.upgrade().filter(|s| Rc::ptr_eq(s, &shared)) else {
                return false;
            };
            let Some(b) = boards.get(p.board) else {
                return false;
            };
            let alive = p.notes.iter().all(|n| b.doc.get(*n).is_some());
            let done = alive == p.alive_means_done;
            if done == p.applied {
                return true;
            }
            let now = records(&set.borrow(), p.kanban, &p.forward);
            let untouched = now.len() == p.after.len()
                && now.iter().zip(&p.after).all(|(a, b)| a.same_content(b));
            if !untouched {
                return false;
            }
            let ops = if done { &p.forward } else { &p.backward };
            set.borrow_mut().apply(p.kanban, ops.clone());
            p.after = records(&set.borrow(), p.kanban, &p.forward);
            p.applied = done;
            any = true;
            true
        });
    });
    any
}
