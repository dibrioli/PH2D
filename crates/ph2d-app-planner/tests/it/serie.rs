//! Um evento que se repete: arrastar, editar ou apagar UMA ocorrência pergunta «This event · This
//! and following events · All events» (o Google Calendar); a resposta é um passo de desfazer.

use ph2d_app_planner::source::event_key;
use ph2d_app_planner::{PopHit, ScopeHit};
use ph2d_editor_core::widget::DatePickerLayout;
use ph2d_kanban_dates::{Zone, day_of};
use ph2d_kanban_model::{Event, KanbanId, Moment, Scope};

use crate::{Mesa, centro};

const DAY: i64 = 86_400_000;

fn serie(m: &Mesa) -> Event {
    m.set()
        .agenda()
        .events
        .values()
        .find(|e| e.title == "Team review" && e.repeat.is_some())
        .cloned()
        .expect("a reunião semanal da cena")
}

fn avulsos(m: &Mesa) -> Vec<Event> {
    m.set()
        .agenda()
        .events
        .values()
        .filter(|e| e.title == "Team review" && e.repeat.is_none() && !e.meta.deleted)
        .cloned()
        .collect()
}

fn responde(m: &mut Mesa, s: Scope) {
    let r = {
        let q = m.docs().probe_ask().expect("a pergunta está aberta");
        q.hits().into_iter().find(|(h, _)| *h == ScopeHit::Pick(s)).map(|(_, r)| r).expect("pintada")
    };
    m.clica(centro(r));
    assert!(m.docs().probe_ask().is_none(), "responder fecha a pergunta");
}

/// Arrasta a 1.ª ocorrência (amanhã) para o dia seguinte.
fn arrasta_amanha(m: &mut Mesa) -> (Event, Moment) {
    let e = serie(m);
    let z = Zone::utc();
    let alvo = day_of(e.start.0, &z).add_days(1);
    let (de, para) = {
        let cal = m.docs().probe_calendar();
        let de = *cal.item_rects(event_key(e.id, e.start)).first().expect("a ocorrência de amanhã está pintada");
        (centro(de), centro(cal.cell_rect(alvo).expect("o dia à vista")))
    };
    m.arrasta(de, para);
    let novo = Moment(e.start.0 + DAY);
    (e, novo)
}

#[test]
fn arrastar_uma_ocorrencia_pergunta_e_this_event_tira_so_ela() {
    let (mut m, _) = Mesa::cena();
    let (antes, novo) = arrasta_amanha(&mut m);
    assert!(m.docs().probe_ask().is_some(), "o arrasto pergunta o alcance");
    assert_eq!(serie(&m), antes, "nada muda antes da resposta");
    responde(&mut m, Scope::This);
    let s = serie(&m);
    let x = s.exception_at(antes.start).expect("a ocorrência saiu da série");
    let um = avulsos(&m);
    assert_eq!(um.len(), 1);
    assert_eq!((x.becomes, um[0].start), (Some(um[0].id), novo), "e nasceu sozinha no dia seguinte");
    assert!(!s.skips(Moment(antes.start.0 + 7 * DAY)), "a semana seguinte fica na série");
    let ics = m.docs().ics();
    assert!(ics.contains("EXDATE:"), "o .ics tira a ocorrência da série (RFC 5545)");
    assert_eq!(m.desfaz(), Some(true));
    assert!(serie(&m).exceptions.is_empty() && avulsos(&m).is_empty(), "um passo desfaz tudo");
}

#[test]
fn arrastar_e_all_events_anda_a_serie_inteira() {
    let (mut m, _) = Mesa::cena();
    let (antes, novo) = arrasta_amanha(&mut m);
    responde(&mut m, Scope::All);
    let s = serie(&m);
    assert_eq!(s.start, novo);
    assert_eq!(s.end.0 - s.start.0, antes.end.0 - antes.start.0, "a duração fica");
    assert!(avulsos(&m).is_empty());
}

#[test]
fn fechar_a_pergunta_nao_muda_nada() {
    let (mut m, _) = Mesa::cena();
    let (antes, _) = arrasta_amanha(&mut m);
    let r = {
        let q = m.docs().probe_ask().expect("pergunta");
        q.hits().into_iter().find(|(h, _)| *h == ScopeHit::Cancel).map(|(_, r)| r).expect("Cancel")
    };
    m.clica(centro(r));
    assert!(m.docs().probe_ask().is_none());
    assert_eq!(serie(&m), antes);
}

/// Clica na ocorrência de amanhã: abre o popover nela.
fn abre_amanha(m: &mut Mesa) -> Event {
    let e = serie(m);
    let r = *m.docs().probe_calendar().item_rects(event_key(e.id, e.start)).first().expect("pintada");
    m.clica(centro(r));
    let pop = m.docs().probe_pop().expect("o clique num evento abre o popover dele");
    assert_eq!(pop.occurrence, Some(e.start), "na ocorrência clicada");
    assert_eq!(pop.kanban, KanbanId::AGENDA);
    e
}

fn pop_rect(m: &mut Mesa, h: PopHit) -> ph2d_editor_core::zones::Rect {
    let pop = m.docs().probe_pop().expect("popover");
    pop.hits().into_iter().find(|(x, _)| *x == h).map(|(_, r)| r).expect("pintado")
}

#[test]
fn editar_uma_ocorrencia_pelo_popover_pergunta_e_this_event_muda_so_ela() {
    let (mut m, _) = Mesa::cena();
    let e = abre_amanha(&mut m);
    let duas_horas = pop_rect(&mut m, PopHit::Duration(2));
    m.clica(centro(duas_horas));
    let save = DatePickerLayout::of(pop_rect(&mut m, PopHit::Picker)).save;
    m.clica(centro(save));
    assert!(m.docs().probe_pop().is_none(), "Save fecha o popover…");
    assert!(m.docs().probe_ask().is_some(), "…e pergunta o alcance");
    responde(&mut m, Scope::This);
    let um = avulsos(&m);
    assert_eq!(um.len(), 1);
    assert_eq!((um[0].start, um[0].end.0 - um[0].start.0), (e.start, 2 * 3_600_000));
    assert_eq!(serie(&m).end.0 - serie(&m).start.0, e.end.0 - e.start.0, "a série fica de uma hora");
}

#[test]
fn apagar_uma_ocorrencia_pergunta_e_this_event_some_so_ela() {
    let (mut m, _) = Mesa::cena();
    let e = abre_amanha(&mut m);
    let remove = DatePickerLayout::of(pop_rect(&mut m, PopHit::Picker)).remove;
    m.clica(centro(remove));
    responde(&mut m, Scope::This);
    let s = serie(&m);
    assert!(!s.meta.deleted, "a série continua");
    assert_eq!(s.exception_at(e.start).map(|x| x.becomes), Some(None), "a ocorrência apagou-se");
    m.quadro();
    assert!(
        m.docs().probe_calendar().item_rects(event_key(e.id, e.start)).is_empty(),
        "e já não se pinta"
    );
}
