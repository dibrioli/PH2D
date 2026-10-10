//! Gates das pontas ARMADAS — irmão de `tool_tests.rs` (tecto de 700): selecionar um caminho mostra
//! as pontas dele mas não as arma para o próximo; só o painel arma, campo a campo.

use super::*;
use crate::ids;
use ph2d_a11y::NodeId;

fn set(t: &mut VectorTool, id: NodeId, v: f64) {
    Tool::handle_panel_event(t, PanelEvent::SetValue(id, v));
}

/// ⭐ **Selecionar uma seta não arma a ponta dela** (o dono, 10/10: *«todo traço está herdando uma
/// ponta pequena»*): o painel mostra a `Open` de tamanho 1,11 da seta, o próximo caminho nasce sem.
#[test]
fn adopting_a_paths_heads_shows_them_but_does_not_arm_them() {
    let mut t = VectorTool::new();
    let before = t.armed_markers();
    t.adopt_markers(Marker::None, Marker::Open, 1.114, 0.0);
    assert_eq!(
        t.marker_end(),
        Marker::Open,
        "o painel mostra a seta selecionada"
    );
    assert_eq!(t.armed_markers(), before, "e o próximo caminho não a herda");
}

/// **O painel arma — e só o campo que o artista mexeu**: mudar o tamanho com uma seta selecionada
/// arma o TAMANHO, não a ponta dela; escolher a ponta arma a ponta; Both Ends arma as duas.
#[test]
fn the_panel_arms_only_the_field_it_touched() {
    let mut t = VectorTool::new();
    t.adopt_markers(Marker::None, Marker::Open, 1.114, 0.0);
    set(&mut t, ids::VECTOR_MARKER_SCALE, 2.0);
    let ([start, end], scale, _) = t.armed_markers();
    assert_eq!((start, end), (Marker::None, Marker::None));
    assert!((scale - 2.0).abs() < 1e-9);
    set(&mut t, ids::VECTOR_MARKER_ROUND, 0.5);
    assert!((t.armed_markers().2 - 0.5).abs() < 1e-9);
    set(
        &mut t,
        ids::VECTOR_MARKER_END_DD,
        f64::from(Marker::Triangle.as_u8()),
    );
    assert_eq!(t.armed_markers().0, [Marker::None, Marker::Triangle]);
    set(
        &mut t,
        ids::VECTOR_MARKER_START_DD,
        f64::from(Marker::Bar.as_u8()),
    );
    assert_eq!(t.armed_markers().0, [Marker::Bar, Marker::Triangle]);
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_MARKER_BOTH));
    assert_eq!(
        t.armed_markers().0,
        [t.marker_start(), t.marker_end()],
        "Both Ends arma as duas"
    );
}
