//! **Exportar `.ics`** (RFC 5545, sem login — a porta do plano §8.1 para o Google/Outlook/Apple): os
//! eventos próprios (com a recorrência e o lembrete) e as datas dos cartões e dos itens de checklist.
//! Linhas `CRLF`, dobradas a 75 octetos, texto escapado; horas em UTC (`Z`), dia inteiro em `DATE`.

use ph2d_kanban_dates::{Zone, civil};
use ph2d_kanban_model::{Event, KanbanSet, Moment, Repeat};

const FOLD: usize = 75;

/// Escapa um TEXT (§3.3.11): `\` `;` `,` e as quebras de linha.
fn text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            ';' => o.push_str("\\;"),
            ',' => o.push_str("\\,"),
            '\n' => o.push_str("\\n"),
            '\r' => {}
            c => o.push(c),
        }
    }
    o
}

/// Uma linha de conteúdo dobrada (§3.1): no máximo 75 octetos, sem partir um carácter.
fn line(out: &mut String, l: &str) {
    let mut used = 0;
    for c in l.chars() {
        let n = c.len_utf8();
        if used + n > FOLD {
            out.push_str("\r\n ");
            used = 1;
        }
        out.push(c);
        used += n;
    }
    out.push_str("\r\n");
}

fn utc(m: Moment) -> String {
    let c = civil(m.0, &Zone::utc());
    let s = m.0.div_euclid(1000).rem_euclid(60);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{s:02}Z",
        c.date.year, c.date.month, c.date.day, c.hour, c.minute
    )
}

fn date(m: Moment) -> String {
    let d = civil(m.0, &Zone::utc()).date;
    format!("{:04}{:02}{:02}", d.year, d.month, d.day)
}

fn freq(r: Repeat) -> &'static str {
    match r {
        Repeat::Daily => "DAILY",
        Repeat::Weekly => "WEEKLY",
        Repeat::Monthly => "MONTHLY",
        Repeat::Yearly => "YEARLY",
    }
}

struct Vevent<'a> {
    uid: String,
    start: Moment,
    end: Moment,
    all_day: bool,
    summary: &'a str,
    description: &'a str,
    repeat: Option<Repeat>,
    /// O fim da série (exclusivo) e as ocorrências tiradas (v7).
    until: Option<Moment>,
    exdates: Vec<Moment>,
    alarm_min: Option<u32>,
}

fn vevent(out: &mut String, e: &Vevent<'_>, stamp: &str) {
    line(out, "BEGIN:VEVENT");
    line(out, &format!("UID:{}", e.uid));
    line(out, &format!("DTSTAMP:{stamp}"));
    if e.all_day {
        line(out, &format!("DTSTART;VALUE=DATE:{}", date(e.start)));
        line(out, &format!("DTEND;VALUE=DATE:{}", date(e.end)));
    } else {
        line(out, &format!("DTSTART:{}", utc(e.start)));
        line(out, &format!("DTEND:{}", utc(e.end.max(e.start))));
    }
    line(out, &format!("SUMMARY:{}", text(e.summary)));
    if !e.description.is_empty() {
        line(out, &format!("DESCRIPTION:{}", text(e.description)));
    }
    if let Some(r) = e.repeat {
        // O `UNTIL` do RFC 5545 é INCLUSIVO; o nosso fim é exclusivo.
        let until = e.until.map_or_else(String::new, |u| {
            let last = if e.all_day { date(Moment(u.0 - 1)) } else { utc(Moment(u.0 - 1000)) };
            format!(";UNTIL={last}")
        });
        line(out, &format!("RRULE:FREQ={}{until}", freq(r)));
        for x in &e.exdates {
            if e.all_day {
                line(out, &format!("EXDATE;VALUE=DATE:{}", date(*x)));
            } else {
                line(out, &format!("EXDATE:{}", utc(*x)));
            }
        }
    }
    if let Some(m) = e.alarm_min {
        line(out, "BEGIN:VALARM");
        line(out, "ACTION:DISPLAY");
        line(out, &format!("DESCRIPTION:{}", text(e.summary)));
        line(out, &format!("TRIGGER:-PT{m}M"));
        line(out, "END:VALARM");
    }
    line(out, "END:VEVENT");
}

fn event(e: &Event) -> Vevent<'_> {
    Vevent {
        uid: format!("event-{:016x}@ph2d", e.id.0),
        start: e.start,
        end: e.end,
        all_day: e.all_day,
        summary: &e.title,
        description: &e.notes,
        repeat: e.repeat,
        until: e.until,
        exdates: e.exceptions.iter().map(|x| x.at).collect(),
        alarm_min: e.remind_before_min,
    }
}

/// ⭐ O calendário do projecto em `.ics`. `now` = o `DTSTAMP` (a hora da exportação).
#[must_use]
pub fn export(set: &KanbanSet, name: &str, now: Moment) -> String {
    let stamp = utc(now);
    let mut out = String::new();
    line(&mut out, "BEGIN:VCALENDAR");
    line(&mut out, "VERSION:2.0");
    line(&mut out, "PRODID:-//PH2D//Project calendar//EN");
    line(&mut out, "CALSCALE:GREGORIAN");
    line(&mut out, &format!("X-WR-CALNAME:{}", text(name)));
    let docs = std::iter::once(set.agenda()).chain(set.iter());
    for k in docs {
        for e in k.events.values().filter(|e| !e.meta.deleted) {
            vevent(&mut out, &event(e), &stamp);
        }
        for c in k.cards.values() {
            if c.meta.deleted || c.archived || c.template {
                continue;
            }
            let Some(end) = c.due.map(|d| d.at).or(c.start) else {
                continue;
            };
            let start = c.start.filter(|s| *s < end).unwrap_or(end);
            vevent(
                &mut out,
                &Vevent {
                    uid: format!("card-{}-{:016x}@ph2d", k.id.0, c.id.0),
                    start,
                    end,
                    all_day: false,
                    summary: &c.title,
                    description: &k.header.name,
                    repeat: None,
                    until: None,
                    exdates: Vec::new(),
                    alarm_min: c.due.and_then(|d| d.remind_before_min),
                },
                &stamp,
            );
        }
        for it in k.items.values().filter(|i| !i.meta.deleted) {
            let Some(at) = it.due else { continue };
            vevent(
                &mut out,
                &Vevent {
                    uid: format!("item-{}-{:016x}@ph2d", k.id.0, it.id.0),
                    start: at,
                    end: at,
                    all_day: false,
                    summary: &it.text,
                    description: &k.header.name,
                    repeat: None,
                    until: None,
                    exdates: Vec::new(),
                    alarm_min: None,
                },
                &stamp,
            );
        }
    }
    line(&mut out, "END:VCALENDAR");
    out
}
