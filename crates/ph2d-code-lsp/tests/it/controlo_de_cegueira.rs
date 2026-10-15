//! Gate (c), the blindness control: a script with deliberate API misuse yields EXACTLY the expected
//! diagnostics. A clean `bob.luau` alone proves nothing — an empty definitions file or a checker that
//! never ran would also be «clean». Expectations live on the fixture lines (`-- ERR <code> <needle>`).

use ph2d_code_lsp::Solver;

use crate::support::{diagnostics_of, fixture, ph2d_definitions, render, start};

#[test]
fn deliberate_errors_yield_exactly_the_expected_diagnostics() {
    let text = std::fs::read_to_string(fixture("erros_de_proposito.luau")).expect("fixture");
    let mut expected: Vec<(u32, String, String)> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with("--"))
        .filter_map(|(i, l)| {
            let (_, tail) = l.split_once("-- ERR ")?;
            let (code, needle) = tail.split_once(' ')?;
            Some((i as u32 + 1, code.to_owned(), needle.trim().to_owned()))
        })
        .collect();
    assert_eq!(expected.len(), 2, "fixture lost its ERR markers");
    expected.sort();

    let mut c = start(&ph2d_definitions(), Solver::Old);
    let diags = diagnostics_of(&mut c, "erros_de_proposito.luau", &text);
    c.shutdown().expect("shutdown");

    let mut got: Vec<(u32, String)> = diags
        .iter()
        .map(|d| (d.range.start.line + 1, d.code.clone().unwrap_or_default()))
        .collect();
    got.sort();
    let want: Vec<(u32, String)> = expected.iter().map(|(l, c, _)| (*l, c.clone())).collect();
    assert_eq!(got, want, "diagnostics (line, code) differ; got:{}", render(&diags));
    for (line, code, needle) in &expected {
        let d = diags
            .iter()
            .find(|d| d.range.start.line + 1 == *line && d.code.as_deref() == Some(code))
            .unwrap();
        assert!(d.message.contains(needle.as_str()), "line {line}: «{}» lacks {needle}", d.message);
    }
}
