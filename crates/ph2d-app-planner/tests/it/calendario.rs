//! O Calendário do projecto: abrir pelo menu, arrastar um cartão muda a entrega DELE (o mesmo
//! documento que o quadro mostra), criar um evento pelo popover.

use ph2d_app_planner::{Hit, Mode, PopHit};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::widget::DatePickerLayout;
use ph2d_kanban_dates::{Zone, civil, day_of};
use ph2d_kanban_model::{CardRef, KanbanId};
use ph2d_kanban_views::ItemKey;

use crate::{Mesa, centro};

#[test]
fn o_menu_abre_a_aba_no_calendario_e_os_modos_trocam_por_clique() {
    let mut m = Mesa::new();
    m.quadro();
    assert!(m.hero.apply_event(WidgetEvent::Click(ph2d_app_planner::CMD_CALENDAR)));
    m.quadros(2);
    assert!(m.planner_activo(), "a aba activa é a do Planner");
    assert_eq!(m.docs().probe_mode().0, Mode::Calendar);
    m.clica_em(Hit::Mode(Mode::MyCards));
    assert_eq!(m.docs().probe_mode().0, Mode::MyCards);
    m.clica_em(Hit::Mode(Mode::Planner));
    assert_eq!(m.docs().probe_mode().0, Mode::Planner);
    assert!(m.docs().probe_hit(Hit::InboxField).is_some(), "a coluna da Inbox está à vista");
}

#[test]
fn arrastar_um_cartao_no_calendario_muda_a_entrega_dele_e_o_desfazer_volta() {
    let (mut m, k) = Mesa::cena();
    let antes = m.cartao(k, "Design the fox");
    let due = antes.due.expect("a cena dá-lhe entrega").at;
    let key = ItemKey::Card(CardRef {
        kanban: k,
        card: antes.id,
    });
    let z = Zone::utc();
    let dia = day_of(due.0, &z);
    let alvo = dia.add_days(1);
    let (de, para) = {
        let cal = m.docs().probe_calendar();
        let de = *cal.item_rects(key).first().expect("o cartão está pintado no calendário");
        let para = cal.cell_rect(alvo).expect("o dia seguinte está à vista");
        (centro(de), centro(para))
    };
    m.arrasta(de, para);
    let depois = m.cartao(k, "Design the fox").due.expect("entrega").at;
    assert_eq!(day_of(depois.0, &z), alvo, "a entrega passou para o dia onde caiu");
    assert_eq!(
        (civil(depois.0, &z).hour, civil(depois.0, &z).minute),
        (civil(due.0, &z).hour, civil(due.0, &z).minute),
        "a hora fica"
    );
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(m.cartao(k, "Design the fox").due.expect("entrega").at, due);
}

#[test]
fn mais_evento_abre_o_popover_e_save_grava_o_evento_na_agenda() {
    let (mut m, _) = Mesa::cena();
    let antes = m.set().agenda().events.len();
    m.clica_em(Hit::NewEvent);
    assert!(m.docs().probe_pop().is_some(), "o popover abriu");
    assert!(m.docs().probe_pop().is_some_and(|p| p.editing), "o título abre em edição");
    m.escreve("Kickoff");
    let (semanal, save) = {
        let pop = m.docs().probe_pop().expect("popover");
        let hits = pop.hits();
        let rect = |h: PopHit| hits.iter().find(|(x, _)| *x == h).map(|(_, r)| *r).expect("pintado");
        (rect(PopHit::Repeat(2)), DatePickerLayout::of(rect(PopHit::Picker)).save)
    };
    m.clica(centro(semanal));
    m.clica(centro(save));
    assert!(m.docs().probe_pop().is_none(), "Save fecha o popover");
    let set = m.set();
    let a = set.agenda();
    assert_eq!(a.events.len(), antes + 1);
    let e = a.events.values().find(|e| e.title == "Kickoff").expect("o evento novo");
    assert_eq!(e.repeat, Some(ph2d_kanban_model::Repeat::Weekly));
    assert_eq!(e.end.0 - e.start.0, 60 * 60_000, "uma hora por omissão");
    assert!(
        m.docs().probe_hits().iter().all(|(h, _)| *h != Hit::Pop(PopHit::Title)),
        "nada do popover ficou para clicar"
    );
    let _ = KanbanId::AGENDA;
}
