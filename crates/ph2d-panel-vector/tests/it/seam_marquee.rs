//! Seam da **FORMA do marquee** (`Box | Lasso`) — os dois chips estão vivos sob o MOUSE, e só no
//! modo em que a região existe.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist e o switch da tool mas **pula a checagem de
//! focabilidade no store** — a lacuna que deixou as 36 células da matriz de física e os dez chips
//! de ferramenta do Painter *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! ⚠️ **A metade da AUSÊNCIA é a que carrega o gate.** Um par de chips pintado em todo modo é um
//! controle que não faz nada em treze deles: só no Node existe um arrasto no vazio para lhes
//! obedecer. E a *presença* sozinha fica verde num painel que os pinte sempre.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::{VectorPanelState, set_current_vector_style};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{DrawMode, MarqueeShape};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn arm(mode: DrawMode, marquee: MarqueeShape) {
    set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        marquee,
        ..VectorStyleSnapshot::default()
    }));
}

/// **Fora do modo Node os chips NÃO existem** — a metade da ausência.
///
/// A região só nasce ao arrastar do vazio no Node; num modo em que ela não existe, um par que diz
/// que forma ela tem é um controle mudo. E o gate a percorre em vários modos, porque um `!= Node`
/// escrito ao contrário passa em qualquer um deles isolado.
#[test]
fn the_marquee_chips_do_not_exist_outside_the_node_mode() {
    for mode in [
        DrawMode::Select,
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Shape,
        DrawMode::Cut,
    ] {
        let mut host = crate::host_aberto();
        let mut panel_state = VectorPanelState;
        arm(mode, MarqueeShape::Lasso);
        for id in [
            ph2d_tool_vector::ids::VECTOR_MARQUEE_BOX,
            ph2d_tool_vector::ids::VECTOR_MARQUEE_LASSO,
        ] {
            assert!(
                host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                    .is_none(),
                "{id:?} foi pintado no modo {mode:?}, onde nao ha' regiao nenhuma para ele \
                 qualificar"
            );
        }
    }
}
