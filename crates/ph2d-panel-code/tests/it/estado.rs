//! A barra de estado: posição 1-based (tabs até à paragem), problemas e o selo do salvar.

use crate::{Deco, frame, spans_de};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{Badge, Problem, Severity, StatusLine, Tone};

fn status(buf: &Buffer, d: &Deco) -> StatusLine {
    let mut f = |r: std::ops::Range<usize>| spans_de(buf, r);
    let mut m = crate::marcas(d);
    StatusLine::of(&frame(buf, d, &mut f, &mut m))
}

fn p(severity: Severity) -> Problem {
    Problem {
        line: 0,
        severity,
        culprit: None,
        message: String::new(),
    }
}

#[test]
fn posicao_problemas_e_selo() {
    let mut buf = Buffer::new("a\n\tif x\n");
    buf.set_selection(Selection::single(Range::cursor(
        buf.text().byte_of_line(1) + 1,
    )));
    let s = status(&buf, &Deco::default());
    assert_eq!(s.position, "Ln 2, Col 5");
    assert_eq!(s.problems, ("No problems".to_owned(), Tone::Plain));
    assert_eq!(s.badge, None);
    let d = Deco {
        problems: vec![
            p(Severity::Error),
            p(Severity::Warning),
            p(Severity::Warning),
        ],
        badge: Some(Badge::Reloaded { objects: 3 }),
        ..Deco::default()
    };
    let s = status(&buf, &d);
    assert_eq!(s.problems, ("1 error · 2 warnings".to_owned(), Tone::Error));
    assert_eq!(
        s.badge,
        Some(("Reloaded · 3 objects".to_owned(), Tone::Success))
    );
    let d = Deco {
        problems: vec![p(Severity::Warning)],
        badge: Some(Badge::Refused { line: Some(7) }),
        ..Deco::default()
    };
    let s = status(&buf, &d);
    assert_eq!(s.problems.1, Tone::Warning);
    let (texto, tom) = s.badge.expect("selo");
    assert!(texto.contains("line 7"), "{texto}");
    assert_eq!(tom, Tone::Error);
}

/// O assistente de código só fala quando NÃO está pronto, e diz que a edição continua.
#[test]
fn o_assistente_so_aparece_quando_nao_esta_pronto() {
    use ph2d_panel_code::Assist;
    let buf = Buffer::new("a\n");
    assert_eq!(status(&buf, &Deco::default()).assist, None);
    let mut textos = std::collections::BTreeSet::new();
    for (a, tom) in [
        (Assist::Starting, Tone::Plain),
        (Assist::Restarting, Tone::Warning),
        (Assist::Missing, Tone::Warning),
        (Assist::Stopped, Tone::Warning),
    ] {
        let d = Deco {
            assist: Some(a),
            ..Deco::default()
        };
        let (texto, t) = status(&buf, &d).assist.expect("peça do assistente");
        if matches!(a, Assist::Missing | Assist::Stopped) {
            assert!(texto.contains("editing works"), "{a:?}: {texto}");
        }
        assert_eq!(t, tom, "{a:?}");
        textos.insert(texto);
    }
    // Cada estado diz uma coisa sua (uma chave trocada daria dois textos iguais).
    assert_eq!(textos.len(), 4, "{textos:?}");
}

/// ⛔ «1 errors · 0 warnings» e «Console · 1 errors» (foto `code_smoke_2.png`): a forma de UM.
#[test]
fn uma_contagem_de_um_fala_no_singular() {
    let buf = Buffer::new("a\n");
    let com = |problems: Vec<Problem>| {
        status(
            &buf,
            &Deco {
                problems,
                ..Deco::default()
            },
        )
        .problems
        .0
    };
    assert_eq!(com(vec![p(Severity::Error)]), "1 error · 0 warnings");
    assert_eq!(
        com(vec![p(Severity::Error), p(Severity::Error)]),
        "2 errors · 0 warnings"
    );
    assert_eq!(com(vec![p(Severity::Warning)]), "0 errors · 1 warning");
    assert_eq!(
        com(vec![
            p(Severity::Error),
            p(Severity::Warning),
            p(Severity::Warning)
        ]),
        "1 error · 2 warnings"
    );
    assert_eq!(ph2d_panel_code::console_title_text(0), "Console");
    assert_eq!(ph2d_panel_code::console_title_text(1), "Console · 1 error");
    assert_eq!(ph2d_panel_code::console_title_text(2), "Console · 2 errors");
    let um = ph2d_panel_code::FindCount {
        current: None,
        total: 1,
    };
    assert_eq!(
        ph2d_panel_code::count_text(Some(um)).as_deref(),
        Some("1 result")
    );
}
