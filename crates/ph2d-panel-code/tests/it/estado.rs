//! A barra de estado: posição 1-based (tabs até à paragem), problemas e o selo do salvar.

use crate::{Deco, frame, spans_de};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{Badge, Problem, Severity, StatusLine, Tone};

fn status(buf: &Buffer, d: &Deco) -> StatusLine {
    let mut f = |r: std::ops::Range<usize>| spans_de(buf, r);
    StatusLine::of(&frame(buf, d, &mut f))
}

fn p(severity: Severity) -> Problem {
    Problem { line: 0, severity, culprit: None, message: String::new() }
}

#[test]
fn posicao_problemas_e_selo() {
    let mut buf = Buffer::new("a\n\tif x\n");
    buf.set_selection(Selection::single(Range::cursor(buf.text().byte_of_line(1) + 1)));
    let s = status(&buf, &Deco::default());
    assert_eq!(s.position, "Ln 2, Col 5");
    assert_eq!(s.problems, ("No problems".to_owned(), Tone::Plain));
    assert_eq!(s.badge, None);
    let d = Deco { problems: vec![p(Severity::Error), p(Severity::Warning), p(Severity::Warning)], badge: Some(Badge::Reloaded { objects: 3 }), ..Deco::default() };
    let s = status(&buf, &d);
    assert_eq!(s.problems, ("1 errors · 2 warnings".to_owned(), Tone::Error));
    assert_eq!(s.badge, Some(("Reloaded · 3 objects".to_owned(), Tone::Success)));
    let d = Deco { problems: vec![p(Severity::Warning)], badge: Some(Badge::Refused { line: Some(7) }), ..Deco::default() };
    let s = status(&buf, &d);
    assert_eq!(s.problems.1, Tone::Warning);
    let (texto, tom) = s.badge.expect("selo");
    assert!(texto.contains("line 7"), "{texto}");
    assert_eq!(tom, Tone::Error);
}
