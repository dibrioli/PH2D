//! O `.ics` exportado, lido por um leitor INDEPENDENTE do escritor (as regras do RFC 5545 escritas de
//! novo aqui: `CRLF`, dobra a 75 octetos, desdobrar, escapes, `BEGIN`/`END` aninhados).

use ph2d_kanban_model::{Event, EventId, KanbanId, KanbanOp, Moment, Record, Repeat};

use crate::Mesa;

/// Uma componente lida: o nome, as propriedades `(nome com parâmetros, valor)` e as filhas.
#[derive(Debug, Default)]
struct Comp {
    name: String,
    props: Vec<(String, String)>,
    kids: Vec<Comp>,
}

impl Comp {
    fn get(&self, name: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|(n, _)| n.split(';').next() == Some(name))
            .map(|(_, v)| v.as_str())
    }
}

fn unescape(v: &str) -> String {
    let mut out = String::new();
    let mut it = v.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n' | 'N') => out.push('\n'),
                Some(x) => out.push(x),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn read(text: &str) -> Comp {
    assert!(text.ends_with("\r\n"), "cada linha termina em CRLF");
    let raw: Vec<&str> = text.split("\r\n").filter(|l| !l.is_empty()).collect();
    for l in &raw {
        assert!(l.len() <= 75, "linha com mais de 75 octetos: {l:?}");
        assert!(!l.contains('\n'), "LF solto");
    }
    let mut lines: Vec<String> = Vec::new();
    for l in raw {
        if let Some(rest) = l.strip_prefix(' ') {
            lines.last_mut().expect("continuação sem linha").push_str(rest);
        } else {
            lines.push(l.to_owned());
        }
    }
    let mut stack: Vec<Comp> = vec![Comp::default()];
    for l in lines {
        let (name, value) = l.split_once(':').expect("NOME:valor");
        match name {
            "BEGIN" => stack.push(Comp {
                name: value.to_owned(),
                ..Comp::default()
            }),
            "END" => {
                let c = stack.pop().expect("END sem BEGIN");
                assert_eq!(c.name, value, "END fecha a componente aberta");
                stack.last_mut().expect("raiz").kids.push(c);
            }
            _ => stack
                .last_mut()
                .expect("raiz")
                .props
                .push((name.to_owned(), unescape(value))),
        }
    }
    assert_eq!(stack.len(), 1, "componentes por fechar");
    let mut root = stack.pop().expect("raiz");
    assert_eq!(root.kids.len(), 1);
    root.kids.pop().expect("VCALENDAR")
}

#[test]
fn o_ics_do_projecto_le_se_por_um_leitor_independente() {
    let (mut m, _) = Mesa::cena();
    {
        // Um título com o que se escapa e longo bastante para dobrar.
        let shared = m.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let id = EventId(set.agenda().mint());
        let mut e = Event::new(
            id,
            "Review, notes; and a very long title that certainly needs folding — ção ção ção",
            Moment(1_767_225_600_000),
            Moment(1_767_312_000_000),
            true,
        );
        e.repeat = Some(Repeat::Yearly);
        set.apply(KanbanId::AGENDA, vec![KanbanOp::put(Record::Event(e))]);
    }
    let text = m.docs().ics();
    let cal = read(&text);
    assert_eq!(cal.name, "VCALENDAR");
    assert_eq!(cal.get("VERSION"), Some("2.0"));
    assert!(cal.get("PRODID").is_some());
    let events: Vec<&_> = cal.kids.iter().filter(|c| c.name == "VEVENT").collect();
    assert!(events.len() >= 8, "eventos e datas de cartões: {}", events.len());
    let mut uids: Vec<&str> = Vec::new();
    for e in &events {
        for p in ["UID", "DTSTAMP", "DTSTART", "SUMMARY"] {
            assert!(e.get(p).is_some(), "{p} em {e:?}");
        }
        uids.push(e.get("UID").expect("uid"));
    }
    uids.sort_unstable();
    uids.dedup();
    assert_eq!(uids.len(), events.len(), "UIDs únicos");
    let long = events
        .iter()
        .find(|e| e.get("SUMMARY").is_some_and(|s| s.starts_with("Review, notes; and")))
        .expect("o título escapado volta igual");
    assert!(long.get("SUMMARY").is_some_and(|s| s.ends_with("ção ção ção")));
    assert_eq!(long.get("DTSTART"), Some("20260101"));
    assert!(long.props.iter().any(|(n, _)| n == "DTSTART;VALUE=DATE"));
    assert_eq!(long.get("RRULE"), Some("FREQ=YEARLY"));
    let review = events
        .iter()
        .find(|e| e.get("SUMMARY") == Some("Team review"))
        .expect("a reunião semanal");
    assert_eq!(review.get("RRULE"), Some("FREQ=WEEKLY"));
    let alarm = review.kids.iter().find(|c| c.name == "VALARM").expect("o lembrete");
    assert_eq!(alarm.get("TRIGGER"), Some("-PT15M"));
    assert!(
        review.get("DTSTART").is_some_and(|s| s.len() == 16 && s.ends_with('Z')),
        "hora em UTC"
    );
    assert!(events.iter().any(|e| e.get("SUMMARY") == Some("Design the fox")), "datas de cartões");
}
