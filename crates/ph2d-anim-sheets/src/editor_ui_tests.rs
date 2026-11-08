use super::*;

#[test]
fn a_mistura_conta_so_quadros_de_verdade() {
    let a = AnimFrameRef { sheet: 0, cell: 1 };
    let b = AnimFrameRef { sheet: 1, cell: 0 };
    assert!(!mixes_sheets(&[a, AnimFrameRef::EMPTY, a]));
    assert!(mixes_sheets(&[a, b]));
}
