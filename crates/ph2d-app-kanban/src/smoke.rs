//! ⭐ **As cenas de smoke do Kanban** — `PH2D_KANBAN_SMOKE=<n>` abre, no arranque, um Kanban montado
//! numa aba. Roteador PRÓPRIO (o do Quadro é o `PH2D_BOARD_SMOKE`); a próxima cena é a `CENAS + 1`.
//!
//! - **1** — o quadro da maquete aprovada (27/10): «Short film — production», cinco listas, a
//!   «Doing» azul e cheia (limite `3 / 3`), a «Review» recolhida, e cartões com todas as variantes
//!   da frente: uma capa de IMAGEM («Fox concept sheet», atrasada), uma entrega em menos de 24 h
//!   («Clean up shot 08») e uma concluída («Lock the script»). ⚠️ Sem o cartão envelhecido (W6).
//! - **2** — um quadro grande (8 listas × 250 cartões) para sentir o arrasto a 60 fps.
//! - **3** — o quadro da cena 1 com a «To Do» NA MÃO, a meio do caminho entre a «Doing» e a
//!   «Review»: inclinada, com a sombra de voo e os cartões dentro; as outras abriram-lhe o espaço.
//!   Mexer o rato leva-a; um clique larga-a ali.
//! - **4** — o quadro da cena 1 com o VERSO de «Animate shot 12» aberto (a maquete `Back.dc.html`):
//!   capa, membros, etiquetas, prazo atrasado, descrição rica, campos, a checklist «Animation pass»
//!   com responsável e data, um comentário com menção e reacção, e o histórico.
//! - **5** — W6: o FILTRO aberto (a maquete `Filter.dc.html`) e o envelhecimento regular.
//! - **6** — W6: o envelhecimento PIRATA, um MODELO e um ESPELHO ([`crate::smoke_w6`]).
//! - **7** — J1: o Kanban da cena 1 PARTILHADO, o «Share» aberto e um colega («Rui») a ver
//!   ([`crate::share::smoke`]).
//! - **8** — J2: o BUTLER (regras, botões, uma sugestão) e o verso com botões e ligações ao
//!   projecto ([`crate::smoke_j2`]).
//! - **9** — W10: o quadro sintético do Trello IMPORTADO (a fixture do `ph2d-kanban-import`), com
//!   o RELATÓRIO da importação aberto por cima da aba: o que entrou e o que ficou de fora.

use crate::docs::KanbanDocs;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_kanban_model::{
    Activity, ActivityId, CardId, Comment, CommentId, Cover, CoverFill, Due, FieldDef, FieldId,
    FieldKind, FieldOption, FieldValue, Happened, Hue, KanbanId, KanbanSet, ListId, Member, Meta,
    Moment, OptionId, Reaction, Record, Shade, Subject,
};
use ph2d_kanban_scenes::{Scene, people, review_list, write};

/// As cenas que existem.
pub const CENAS: u8 = 9;
pub const ENV: &str = "PH2D_KANBAN_SMOKE";

/// O roteador que a família declara ao registo (o gate de colisão lê-o).
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: ENV,
    max_level: CENAS as u32,
}];

/// O nível pedido pela env (`None` = nenhum, ou fora de `1..=CENAS`).
#[must_use]
pub fn armed_level() -> Option<u8> {
    std::env::var(ENV)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=CENAS).contains(n))
}

/// Monta a cena pedida pela env (inerte sem ela). Chamar depois de instalar a família.
pub fn stage_armed_smokes(hero: &mut HeroScreen) {
    if let Some(n) = armed_level() {
        stage(hero, n);
    } else if code_smoke_wants_kanban(std::env::var(CODE_ENV).ok().as_deref()) {
        stage_in_project(hero);
    }
}

/// A cena 1 só no projecto, sem aba: o foco fica na aba do script.
fn stage_in_project(hero: &mut HeroScreen) {
    let shared = hero.documents.kanbans_shared();
    short_film(&mut shared.borrow_mut());
    if let Some(docs) = hero.documents.surfaces_mut().find_mut::<KanbanDocs>() {
        docs.follow_project(shared);
    }
}

/// O roteador do editor de scripts: a cena 10 é um script sobre o Kanban da cena 1, que é esta
/// família quem põe no projecto (lido pelo nome, sem depender do `ph2d-app-code`).
const CODE_ENV: &str = "PH2D_CODE_SMOKE";
const CODE_KANBAN_LEVEL: &str = "10";

fn code_smoke_wants_kanban(v: Option<&str>) -> bool {
    v.is_some_and(|v| v.trim() == CODE_KANBAN_LEVEL)
}

/// Monta a cena `n` no projecto aberto e abre-a numa aba.
pub fn stage(hero: &mut HeroScreen, n: u8) -> Option<KanbanId> {
    let shared = hero.documents.kanbans_shared();
    if n == 9 {
        return trello_import(hero, shared);
    }
    let id = {
        let mut set = shared.borrow_mut();
        match n {
            1 | 3 | 4 | 7 | 8 => short_film(&mut set),
            5 | 6 => {
                let id = short_film(&mut set);
                let aging = if n == 5 {
                    ph2d_kanban_model::Aging::Regular
                } else {
                    ph2d_kanban_model::Aging::Pirate
                };
                crate::smoke_w6::age(&mut set, id, aging);
                if n == 6 {
                    crate::smoke_w6::template_and_mirror(&mut set, id);
                }
                id
            }
            2 => big(&mut set),
            _ => return None,
        }
    };
    // J2 — o Quadro a que o cartão da cena 8 se liga, e quem é «eu» (o autor das sugestões).
    let board = (n == 8).then(|| hero.documents.create("Storyboard".into()).0);
    let me = match hero.documents.author().trim() {
        "" => Member(ph2d_i18n::tr("kanban.back.me").to_owned()),
        a => Member(a.to_owned()),
    };
    let docs = hero.documents.surfaces_mut().find_mut::<KanbanDocs>()?;
    docs.follow_project(shared);
    docs.open(id);
    let review = review_list(&docs.set.borrow(), id);
    if matches!(n, 1 | 3 | 4)
        && let Some(review) = review
        && let Some(t) = docs.tab_mut(id.0)
    {
        t.view.collapsed.insert(review);
    }
    let todo = docs
        .set
        .borrow()
        .get(id)
        .and_then(|k| k.lists.values().find(|l| l.title == "To Do").map(|l| l.id));
    if n == 3
        && let Some(todo) = todo
        && let Some(t) = docs.tab_mut(id.0)
    {
        // Um passo e meio de lista para a direita, um pouco levantada.
        let step = crate::metrics::LIST_W + crate::metrics::LIST_GAP;
        t.smoke_hold = Some((todo, (1.55 * step, 2.0 * crate::metrics::LIST_HEAD_BOTTOM)));
    }
    if n == 5 {
        crate::smoke_w6::open_filter(docs, id);
    }
    if n == 7 {
        crate::share::smoke::stage(docs, id);
    }
    if n == 4 || n == 8 {
        let shot = rich_back(&mut docs.set.borrow_mut(), id);
        if let Some(c) = shot {
            if let Some(b) = board {
                crate::smoke_j2::butler(&mut docs.set.borrow_mut(), id, &me, c, b);
            }
            docs.open_back(id.0, c);
        }
    }
    Some(id)
}

/// A fixture sintética do Trello (a mesma dos testes do importador), lida do repositório.
const TRELLO_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ph2d-kanban-import/tests/fixtures/trello_board_synthetic.json"
);

/// Cena 9: «Import from Trello…» da fixture, sem pasta de anexos (entram como links).
fn trello_import(
    hero: &mut HeroScreen,
    shared: ph2d_editor_core::documents::SharedKanbans,
) -> Option<KanbanId> {
    let json = std::fs::read(TRELLO_FIXTURE).ok()?;
    let docs = hero.documents.surfaces_mut().find_mut::<KanbanDocs>()?;
    docs.follow_project(shared);
    docs.import_json(&json, None, TRELLO_FIXTURE)
}

/// ⭐ O Kanban da cena 1 («Short film — production») em `set`, e a «Review» (que a cena recolhe) — a
/// cena 30 do Quadro põe-no no canvas.
pub fn short_film_into(set: &mut KanbanSet) -> (KanbanId, Option<ListId>) {
    ph2d_kanban_scenes::short_film_into(set, crate::actions::now())
}

/// A cena 1 (o quadro da maquete), com os prazos contados a partir de agora.
fn short_film(set: &mut KanbanSet) -> KanbanId {
    ph2d_kanban_scenes::short_film(set, crate::actions::now())
}

/// SMOKE 3: põe na mão a lista pedida, assim que a disposição a tem (o arrasto como o do rato), presa
/// à origem das listas a cada quadro (a janela que maximiza move-a) até o rato a mexer.
pub(crate) fn hold(tab: &mut crate::tab::Tab) {
    let Some((list, (dx, dy))) = tab.smoke_hold else {
        return;
    };
    let Some(r) = tab.layout.list(list).map(|lb| lb.rect) else {
        return;
    };
    let b = tab.layout.board;
    let corner = (
        b.x + crate::metrics::BOARD_PAD + dx,
        b.y + crate::metrics::BOARD_PAD + dy,
    );
    if let Some(d) = tab.drag.as_mut() {
        d.at = (corner.0 + d.grab.0, corner.1 + d.grab.1);
        return;
    }
    let grab = (r.w / 2.0, crate::metrics::LINE_H);
    let to_list = tab.layout.lists.iter().position(|l| l.id == list);
    tab.drag = Some(crate::tab::Drag {
        lift: crate::tab::Lift::List(list),
        at: (corner.0 + grab.0, corner.1 + grab.1),
        grab,
        size: (r.w, r.h),
        to_card: None,
        to_list: to_list.unwrap_or(0),
    });
}

/// A cena 2: um quadro grande.
fn big(set: &mut KanbanSet) -> KanbanId {
    write(set, "Big board \u{2014} 2 000 cards", |s| {
        let tag = s.label("Shot", Hue::Purple, Shade::Normal);
        for l in 0..8 {
            let list = s.list(&format!("Sequence {}", l + 1), None, None);
            for c in 0..250 {
                s.card(
                    list,
                    &format!("Shot {:03} \u{2014} layout, anim, light", c + 1),
                    |card| {
                        if c % 3 == 0 {
                            card.labels = vec![tag];
                        }
                        if c % 5 == 0 {
                            card.members = people(&["Enio"]);
                        }
                    },
                );
            }
        }
    })
}

/// A cena 4: «Animate shot 12» com tudo o que o verso mostra (a maquete `Back.dc.html`). As horas
/// contam-se a partir de agora (o prazo fica atrasado, o comentário «2 hours ago»).
fn rich_back(set: &mut KanbanSet, id: KanbanId) -> Option<CardId> {
    const HOUR: i64 = 3_600_000;
    let now = crate::actions::now().0;
    let k = set.doc_mut(id)?;
    let card = k
        .cards
        .values()
        .find(|c| c.title == "Animate shot 12")?
        .clone();
    let story = k.labels.values().find(|l| l.name == "Story")?.id;
    let doing = card.list;
    let todo = k.lists.values().find(|l| l.title == "To Do")?.id;
    let mut s = Scene::new(k.mint() - 1);
    let priority = FieldId(s.id());
    let high = OptionId(1);
    let estimate = FieldId(s.id());
    let shot_no = FieldId(s.id());
    for (f, name, kind) in [
        (
            priority,
            "Priority",
            FieldKind::Dropdown(vec![
                FieldOption {
                    id: OptionId(0),
                    text: "Low".into(),
                    color: Some(Hue::Green),
                },
                FieldOption {
                    id: high,
                    text: "High".into(),
                    color: Some(Hue::Orange),
                },
            ]),
        ),
        (estimate, "Estimate (days)", FieldKind::Number),
        (shot_no, "Shot", FieldKind::Text),
    ] {
        let pos = s.pos();
        s.put(Record::Field(FieldDef {
            id: f,
            name: name.into(),
            kind,
            on_front: false,
            pos,
            meta: Meta::default(),
        }));
    }
    let mut c = card.clone();
    c.labels.push(story);
    c.members = people(&["Enio", "Rui"]);
    c.cover = Some(Cover {
        fill: CoverFill::Color(Hue::Sky),
        full: false,
    });
    c.start = Some(Moment(now - 6 * 24 * HOUR));
    c.due = Some(Due {
        at: Moment(now - 3 * 24 * HOUR),
        done: false,
        remind_before_min: None,
    });
    c.description = "Fox jumps the log, lands, looks back. **Hold the look for 12 frames** before \
the cut \u{2014} the music hits there.\n\n- Reference: [fox_jump_ref.mp4](https://example.com/fox_jump_ref.mp4)\n\
- Timing sheet in the Storyboard tab\n"
        .into();
    c.fields.insert(priority, FieldValue::Option(high));
    c.fields.insert(estimate, FieldValue::Number(3.0));
    c.fields.insert(shot_no, FieldValue::Text("12".into()));
    s.put(Record::Card(c.into()));
    let lists: Vec<_> = k
        .checklists
        .values()
        .filter(|x| x.card == card.id)
        .cloned()
        .collect();
    for mut cl in lists {
        cl.title = "Animation pass".into();
        let items: Vec<_> = k
            .items
            .values()
            .filter(|i| i.checklist == cl.id)
            .cloned()
            .collect();
        let names = [
            "Blocking",
            "Key poses",
            "Breakdowns",
            "In-betweens",
            "Polish and overlap",
        ];
        for (mut it, name) in items.into_iter().zip(names) {
            it.text = name.into();
            match name {
                "In-betweens" => {
                    it.due = Some(Moment(now + 24 * HOUR));
                    it.member = Some(Member("Enio".into()));
                }
                "Polish and overlap" => it.member = Some(Member("Rui".into())),
                _ => {}
            }
            s.put(Record::CheckItem(it));
        }
        s.put(Record::Checklist(cl));
    }
    let comment = CommentId(s.id());
    s.put(Record::Comment(Comment {
        id: comment,
        card: card.id,
        author: Member("Rui".into()),
        at: Moment(now - 2 * HOUR),
        text: "@Ana can you check the timing on 12C? The landing feels one frame late.".into(),
        edited: None,
        reactions: vec![Reaction {
            emoji: "\u{1F44D}".into(),
            by: people(&["Ana", "Enio"]),
        }],
        meta: Meta::default(),
    }));
    let moved = ActivityId(s.id());
    s.put(Record::Activity(Activity {
        id: moved,
        subject: Subject::Card(card.id),
        by: Member("Ana".into()),
        at: Moment(now - 20 * HOUR),
        what: Happened::Moved {
            from: todo,
            to: doing,
        },
        meta: Meta::default(),
    }));
    ph2d_kanban_model::apply_batch(k, s.into_ops());
    Some(card.id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn so_a_cena_10_do_editor_de_scripts_pede_o_kanban() {
        use super::code_smoke_wants_kanban as quer;
        assert!(quer(Some("10")) && quer(Some(" 10 ")));
        assert!(!quer(Some("9")) && !quer(Some("1")) && !quer(None));
    }
}
