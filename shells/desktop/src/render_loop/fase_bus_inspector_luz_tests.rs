//! ⭐⭐ **G8 da shell — o dreno da edição da LUZ** (Luz 2D, W4): o que o painel publica vai para a
//! fila da luz e não sobra (a aplicação tem o gate dela em `fase_inspector_commits_luz_tests.rs`).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::luz_edits::LuzFieldEdit;

use super::DrainOut;

#[test]
fn a_edicao_da_luz_vai_para_a_fila_da_luz() {
    let (bits, edit) = (0x2D_u64, LuzFieldEdit::Intensidade(1.5));
    let mut app = crate::App::new();
    let mut pd = DrainOut::default();
    // O dreno: o que o painel publicou vai para a fila da LUZ e não sobra.
    let resto = app.fase_bus_inspector_sections(
        EditorAction::InspectorComponentEdit {
            entity_bits: bits,
            edit: ComponentEdit::Luz(edit.clone()),
        },
        &mut pd,
    );
    assert!(
        resto.is_none(),
        "o dreno devolveu a edição da luz sem a consumir"
    );
    assert_eq!(pd.luz_edits, vec![(bits, edit)]);
}
