//! O clique da grelha (item 8) vira a MESMA `InspectorComponentEdit` do cartão Script: o número
//! limitado pelas pistas, o texto que não serve ao tipo recusado.

use crate::{SCRIPT, decls, factos, objecto};
use ph2d_code_bancada::{act, read};
use ph2d_code_viewmodel::{Intent, Who};
use ph2d_ecs::World;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::script_edits::ScriptFieldEdit as E;
use ph2d_script::PropDecl;

/// Dois objectos (A, B) a correr o script, e as declarações.
fn grelha() -> (Who, Vec<PropDecl>, [u64; 2]) {
    let mut w = World::new();
    let a = objecto(&mut w, 1, Some("A"), SCRIPT, &[]);
    let b = objecto(&mut w, 2, Some("B"), SCRIPT, &[]);
    let d = decls();
    (
        read(&mut w, SCRIPT, Some(&d), &factos()),
        d,
        [a.to_bits(), b.to_bits()],
    )
}

fn edicao(bits: u64, edit: E) -> Option<EditorAction> {
    Some(EditorAction::InspectorComponentEdit {
        entity_bits: bits,
        edit: ComponentEdit::Script(edit),
    })
}

/// ⭐ O número escrito vira o `SetNumber` do objecto da linha, limitado ao `min`/`max` da declaração
/// (como o campo do Inspector); a vírgula decimal serve; o que não é número finito, nada.
#[test]
fn o_numero_escrito_vira_o_set_number_limitado_pelas_pistas() {
    let (who, d, [a, b]) = grelha();
    let ed = |h: usize, s: &str| act(&who, Some(&d), &Intent::Edit(h, 0, s.into()));
    let speed = |v: f64| E::SetNumber("speed".into(), v);
    assert_eq!(ed(0, "300"), edicao(a, speed(300.0)));
    assert_eq!(ed(1, "300"), edicao(b, speed(300.0)), "o objecto da linha");
    assert_eq!(
        ed(0, "9000"),
        edicao(a, speed(500.0)),
        "acima do max: o max"
    );
    assert_eq!(ed(0, "-5"), edicao(a, speed(0.0)), "abaixo do min: o min");
    assert_eq!(ed(0, " 1,5 "), edicao(a, speed(1.5)), "a vírgula decimal");
    assert_eq!(ed(0, "depressa"), None);
    assert_eq!(ed(0, "inf"), None, "infinito não é número de campo");
    assert_eq!(
        act(&who, None, &Intent::Edit(0, 0, "9000".into())),
        edicao(a, speed(9000.0)),
        "sem declarações à mão, sem pistas: o número como veio"
    );
}

/// Cada tipo pela sua porta: o lógico, a opção da lista, o par; a cor, a coluna sem tipo, o salto e
/// a selecção não são edições.
#[test]
fn cada_tipo_aceita_so_o_que_lhe_serve() {
    let (who, d, [a, _]) = grelha();
    let ed = |c: usize, s: &str| act(&who, Some(&d), &Intent::Edit(0, c, s.into()));
    assert_eq!(
        ed(1, "false"),
        edicao(a, E::SetBool("active".into(), false))
    );
    assert_eq!(ed(1, "talvez"), None);
    assert_eq!(ed(2, "fly"), None, "fora das opções");
    assert_eq!(
        ed(2, "run"),
        edicao(a, E::SetText("mode".into(), "run".into()))
    );
    assert_eq!(
        ed(3, "1, 2"),
        edicao(a, E::SetVec2("aim".into(), [1.0, 2.0]))
    );
    assert_eq!(ed(3, "1"), None, "falta o y");
    assert_eq!(ed(3, "1, 2, 3"), None, "um a mais");
    assert_eq!(ed(4, "1, 0, 0"), None, "a cor edita-se no Inspector");
    assert_eq!(
        ed(5, "ana"),
        edicao(a, E::SetText("label".into(), "ana".into()))
    );
    assert_eq!(ed(6, "3"), None, "a coluna que a cena não conhece");
    assert_eq!(ed(7, "3"), None, "fora da grelha");
    assert_eq!(act(&who, Some(&d), &Intent::Edit(2, 0, "3".into())), None);
    assert_eq!(act(&who, Some(&d), &Intent::Jump(0)), None);
    assert_eq!(act(&who, Some(&d), &Intent::Select(0)), None);
}
