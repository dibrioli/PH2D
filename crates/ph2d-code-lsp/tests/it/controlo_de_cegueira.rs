//! Gate (c), the blindness control: a script with deliberate API misuse yields EXACTLY the expected
//! diagnostics. A clean `bob.luau` alone proves nothing — an empty definitions file or a checker that
//! never ran would also be «clean». Expectations live on the fixture lines (`-- ERR <code> <needle>`).

use ph2d_code_lsp::Solver;

use crate::support::{
    assert_exactly, diagnostics_of, expected_from_markers, fixture, ph2d_definitions, start,
};

#[test]
fn deliberate_errors_yield_exactly_the_expected_diagnostics() {
    let text = std::fs::read_to_string(fixture("erros_de_proposito.luau")).expect("fixture");
    let expected = expected_from_markers(&text);
    assert_eq!(expected.len(), 2, "fixture lost its ERR markers");
    let mut c = start(&ph2d_definitions(), Solver::Old);
    let diags = diagnostics_of(&mut c, "erros_de_proposito.luau", &text);
    c.shutdown().expect("shutdown");
    assert_exactly(&diags, &expected);
}
