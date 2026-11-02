//! ⭐ **F47 — «Above Effects» no ⚙ do traço** (dono 02/11: o traço por cima dos Filtros). Cada gate
//! CLICA pelo ponteiro: a caixa pinta-se no «Stroke settings» da base e no painel de uma camada de
//! traço, e o clique sai como `Click` do id dela.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{self, Appearance, PaintRow, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};
const CAIXA: NodeId = ids::VECTOR_STROKE_ABOVE_EFFECTS;

fn pintado(h: &mut MockPanelHost) -> Option<Rect> {
    h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, CAIXA)
}

fn tracos() -> PaintRow {
    PaintRow {
        align: ph2d_vec_scene::StrokeAlign::Centre,
        ramp: None,
        is_fill: false,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: 2.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

fn forma(above: Option<bool>) {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![tracos()],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    state::set_base_paints(Some([state::BasePaint::Shown; 2]));
    state::set_stroke_above_effects(above);
}

fn limpa() {
    state::set_current_appearance(None);
    state::set_base_paints(None);
    state::set_stroke_above_effects(None);
    state::close_open_layer();
}

fn clicou(h: &mut MockPanelHost) -> bool {
    h.drained_actions()
        .into_iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if id == CAIXA))
}

/// ⭐ A caixa mora no «Stroke settings» da BASE e responde ao clique. *Mutação: tirar o
/// `checkbox_row` do `stroke_settings_body`, ou o id do `is_button` ⇒ vermelho.*
#[test]
fn a_caixa_do_traco_de_base_pinta_se_e_clica() {
    forma(Some(false));
    let mut h = crate::host_aberto();
    assert!(pintado(&mut h).is_none(), "à mostra com o ⚙ fechado");
    crate::clica(&mut h, ids::VECTOR_STROKE_SETTINGS);
    assert!(pintado(&mut h).is_some(), "falta no «Stroke settings»");
    crate::clica(&mut h, CAIXA);
    assert!(clicou(&mut h), "pintada e morta");
    limpa();
}

/// ⭐ E no painel de uma CAMADA de traço — o mesmo id, o canal dela decide-o a app (`set_chip`).
#[test]
fn a_caixa_da_camada_de_traco_pinta_se_e_clica() {
    forma(Some(true));
    let mut h = crate::host_aberto();
    crate::clica(&mut h, ids::vector_paint_row_id(0));
    assert!(pintado(&mut h).is_some(), "falta no painel da camada");
    // A aberta deriva-se do `Dropdown` na pintura (a de cima já a fez).
    assert_eq!(state::open_layer_index(), Some(0));
    crate::clica(&mut h, CAIXA);
    assert!(clicou(&mut h), "pintada e morta");
    limpa();
}

/// ⛔ CONTROLO: sem resposta (a linha aberta não é um traço) a caixa não se pinta.
#[test]
fn sem_traco_aberto_a_caixa_nao_aparece() {
    forma(None);
    let mut h = crate::host_aberto();
    crate::clica(&mut h, ids::VECTOR_STROKE_SETTINGS);
    assert!(pintado(&mut h).is_none());
    limpa();
}
