//! As cenas de smoke da W6 (o roteador é o [`crate::smoke`]):
//!
//! - **5** — o quadro da cena 1 com o FILTRO aberto (a maquete `Filter.dc.html`): etiquetas
//!   «Animation» e «Audio» marcadas (o botão diz «Filters 2»), as listas mostram «n of m» e a que
//!   ficou vazia diz «No cards match the filter.»; o envelhecimento REGULAR ligado — «Voice casting»
//!   parado há 9 dias desbota um degrau, «Pick the music» (16 dias) dois.
//! - **6** — o mesmo quadro sem filtro, no envelhecimento PIRATA (papel amarelo, bordas rasgadas,
//!   rachas: 9, 16 e 30 dias), com um MODELO («Shot template», o selo «This card is a template») e
//!   um ESPELHO de «Animate shot 12» na «Review» (o selo do espelho).

use ph2d_kanban_model::{
    Aging, CardRef, KanbanId, KanbanOp, KanbanSet, Moment, Record,
};

const DAY: i64 = 24 * 3_600_000;

/// Os dias parados de cada cartão envelhecido (o resto mexeu-se ontem).
const IDLE: [(&str, i64); 4] = [
    ("Voice casting", 9),
    ("Pick the music", 16),
    ("Collect references for the forest", 30),
    ("Rough the opening shot", 30),
];

/// Liga o envelhecimento `aging` e dá a cada cartão a idade da cena.
pub(crate) fn age(set: &mut KanbanSet, id: KanbanId, aging: Aging) {
    let now = crate::actions::now().0;
    let Some(k) = set.doc_mut(id) else {
        return;
    };
    let mut ops = Vec::new();
    let mut h = k.header.clone();
    h.aging = Some(aging);
    ops.push(KanbanOp::Header(h));
    for c in k.cards.values() {
        let days = IDLE
            .iter()
            .find(|(t, _)| *t == c.title)
            .map_or(1, |(_, d)| *d);
        let mut c = c.clone();
        c.created = Moment(now - days * DAY);
        ops.push(KanbanOp::put(Record::Card(c.into())));
    }
    ph2d_kanban_model::apply_batch(k, ops);
}

/// A cena 6: um modelo na «Backlog» e um espelho de «Animate shot 12» na «Review».
pub(crate) fn template_and_mirror(set: &mut KanbanSet, id: KanbanId) {
    let Some(k) = set.get(id) else {
        return;
    };
    let by_title = |t: &str| k.cards.values().find(|c| c.title == t).map(|c| c.id);
    let list = |t: &str| k.lists.values().find(|l| l.title == t).map(|l| l.id);
    let (Some(shot), Some(review), Some(backlog), Some(logline)) = (
        by_title("Animate shot 12"),
        list("Review"),
        list("Backlog"),
        by_title("Write the logline"),
    ) else {
        return;
    };
    let mut tpl = k.cards[&logline].clone();
    tpl.id = ph2d_kanban_model::CardId(k.mint());
    tpl.title = "Shot template".into();
    tpl.template = true;
    tpl.list = backlog;
    tpl.pos = k.pos_after_cards(backlog);
    tpl.created = crate::actions::now();
    // O `mint` só avança com o `Put`: o modelo entra ANTES de o espelho pedir o id dele.
    if let Some(k) = set.doc_mut(id) {
        ph2d_kanban_model::apply_batch(k, vec![KanbanOp::put(Record::Card(tpl.into()))]);
    }
    let mirror = set.place_at(id, review, 0, None).and_then(|p| {
        ph2d_kanban_model::mirror_card(set, CardRef { kanban: id, card: shot }, &p)
    });
    if let Some(plan) = mirror {
        for (k, ops) in plan.into_steps() {
            if let Some(doc) = set.doc_mut(k) {
                ph2d_kanban_model::apply_batch(doc, ops);
            }
        }
    }
}

/// A cena 5: o filtro aberto com «Animation» e «Audio».
pub(crate) fn open_filter(docs: &mut crate::docs::KanbanDocs, id: KanbanId) {
    let labels: Vec<_> = docs
        .set
        .borrow()
        .get(id)
        .map(|k| {
            k.labels
                .values()
                .filter(|l| l.name == "Animation" || l.name == "Audio")
                .map(|l| l.id)
                .collect()
        })
        .unwrap_or_default();
    crate::w6::toggle_over(docs, id.0, crate::w6::Over::Filter);
    if let Some(t) = docs.tab_mut(id.0) {
        t.w6.filter.labels.extend(labels);
        // O retrato mostra o painel sem o cursor a piscar na caixa, rolado ao fim: as etiquetas
        // marcadas e o «Match» (a régua da maquete).
        t.w6.input = None;
        t.w6.scroll = f32::MAX;
    }
}
