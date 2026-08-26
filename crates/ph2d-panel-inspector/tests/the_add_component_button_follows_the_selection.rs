//! ⭐ **O `+` do cabeçalho aparece se, e só se, houver um objeto sob o Inspector** (ADR-0166 / F3).
//!
//! ⚠️ **A metade de AUSÊNCIA é a que interessa, e ela foi apanhada a caminho do smoke.** A 1.ª
//! versão pintava o `+` **sempre**: sem seleção, o handler recusa o clique (não há a quem anexar) e
//! o botão ficava a ser um controlo que se desenha e não faz nada.
//!
//! *Um controlo morto sob o dedo e um ausente dão o MESMO report*, e é essa a doença que a F3
//! inteira existe para apagar — reproduzi-la no botão que a cura seria a piada errada. A resposta
//! certa é ele **não estar lá**, e a pergunta é a mesma que o `apply_event` faz: há `Transform`?

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_transform};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn transform() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

fn painted(sel: Option<InspectorTransformInfo>) -> bool {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_transform(sel);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_transform(None);
    rects.iter().any(|(n, _)| *n == ids::INSP_ADD_COMPONENT)
}

/// As duas metades, num gate só.
///
/// (Mutação: tirar o `if …is_some()` do `paint_head` ⇒ a metade de ausência reprova.)
#[test]
fn the_plus_is_painted_with_a_selection_and_absent_without_one() {
    assert!(
        painted(Some(transform())),
        "com um objeto selecionado o + tem de estar la' — senao nao ha' porta nenhuma para anexar"
    );
    assert!(
        !painted(None),
        "sem selecao o + foi pintado, e o clique nele nao faz nada: um controlo morto sob o dedo"
    );
}
