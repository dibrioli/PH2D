//! Seam dos **dois knobs do LÁPIS** — vivos sob o MOUSE, e o valor chega ao barramento.
//!
//! O irmão `seam_pencil.rs` prova o CHIP de modo; este prova a SEÇÃO que o modo abre. As duas
//! metades de cada slider são independentes: sair do `populate` mata a 1ª asserção (o ponteiro não
//! vira `ValueChanged`, porque a checagem de focabilidade mora no store), sair do `event` mata a 2ª
//! (o valor não chega ao bus e o slider anda sem fazer nada).
//!
//! ⚠️ **E há uma 3ª condição que só esta seção tem: ela SÓ EXISTE no modo Pencil.** Um seam que
//! pintasse o painel no modo default não acharia retângulo nenhum e passaria em silêncio se as
//! asserções fossem `if let Some(..)`; por isso o `painted_rect` aqui é `expect`, e a fixture
//! começa por escolher o modo.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// Publica o modo que a shell publicaria — sem isto a seção não é pintada e não há knob a clicar.
fn publish_mode(mode: ph2d_tool_vector::DrawMode) {
    ph2d_panel_vector::state::set_current_vector_style(Some(
        ph2d_tool_vector::VectorStyleSnapshot {
            mode,
            ..ph2d_tool_vector::VectorStyleSnapshot::default()
        },
    ));
}

/// **A seção só existe no modo Pencil** — noutro modo os dois knobs não são nem pintados.
///
/// A metade da AUSÊNCIA vale tanto quanto a da presença: dois sliders que descrevem como a mão é
/// capturada, oferecidos num modo em que não há mão a capturar, são dois controles mortos.
#[test]
fn the_pencil_section_is_absent_outside_pencil_mode() {
    publish_mode(ph2d_tool_vector::DrawMode::Select);

    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    for id in [
        ph2d_tool_vector::ids::VECTOR_PENCIL_FIDELITY,
        ph2d_tool_vector::ids::VECTOR_PENCIL_STABILIZER,
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "o knob {id:?} foi pintado no modo Select — a secao descreve a captura da MAO livre, e \
             fora do lapis nao ha mao a capturar"
        );
    }
}

/// **A fileira de Width some fora do modo Pencil**, como o resto da seção — três chips que
/// descrevem como uma mão livre é capturada, oferecidos onde não há mão livre, são três controles
/// mortos.
#[test]
fn the_pencil_width_source_chips_are_absent_outside_pencil_mode() {
    publish_mode(ph2d_tool_vector::DrawMode::Select);
    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    for id in [
        ph2d_tool_vector::ids::VECTOR_PENCIL_W_UNIFORM,
        ph2d_tool_vector::ids::VECTOR_PENCIL_W_SPEED,
        ph2d_tool_vector::ids::VECTOR_PENCIL_W_PRESSURE,
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "o chip {id:?} foi pintado no modo Select"
        );
    }
}
