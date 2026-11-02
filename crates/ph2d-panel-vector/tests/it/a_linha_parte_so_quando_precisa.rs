//! ⭐⭐ **A linha de tinta parte-se em duas SÓ quando o Weight não leva o número que mostra** (dono
//! 02/11: *«parte só quando precisa»*).
//!
//! No degrau estreito o Weight ficava com 1 px e o número pintava em nada
//! (`nenhum_rotulo_do_app_pinta_nada`). A cura mede o número que a linha mostra AGORA: na largura
//! de fábrica o valor padrão cabe e a linha fica numa fila (F44c); `100` não cabe e a linha passa a
//! duas filas, com o Weight na 2.ª — e o campo nunca mais estreito que o número.
//!
//! *Mutações que sangram: medir o máximo (`WIDTH_MAX_PX`) em vez do valor ⇒ a linha padrão parte;
//! ignorar o valor (nunca partir) ⇒ o `100` fica cortado na 1.ª fila.*

use ph2d_a11y::NodeId;
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

fn pintado(h: &mut MockPanelHost, id: NodeId) -> Option<Rect> {
    h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, id)
}

fn camada(is_fill: bool, width: f64) -> PaintRow {
    PaintRow {
        align: ph2d_vec_scene::StrokeAlign::Centre,
        ramp: None,
        is_fill,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width,
        enabled: true,
        opacity: 0.5,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

/// Uma forma com uma camada de traço de largura `width` (a 0) e a base visível.
fn forma(width: f64) {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada(false, width)],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    state::set_base_paints(Some([state::BasePaint::Shown; 2]));
}

fn limpa() {
    state::set_current_appearance(None);
    state::set_base_paints(None);
    state::close_open_layer();
}

/// O que o pintor do número precisa para `texto` inteiro: o texto em `Sm`, a folga da esquerda e o
/// stepper (`paint_number_input_with_buffer`).
fn precisa(texto: &str, campo: Rect) -> f32 {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ts.prefix_width_weighted(
        texto,
        ph2d_tokens::TypeToken::Sm.px(),
        ph2d_text::FontWeight::MEDIUM,
    ) + ph2d_editor_core::widget::field_pad_x()
        + ph2d_editor_core::widget::stepper_width(campo)
}

/// (cor, Weight) da camada 0.
fn linha_da_camada(h: &mut MockPanelHost) -> (Rect, Rect) {
    let cor = pintado(h, ids::vector_paint_swatch_id(0)).expect("a cor da camada");
    let peso = pintado(h, ids::vector_paint_row_width_id(0)).expect("o Weight da camada");
    (cor, peso)
}

#[test]
fn com_o_valor_padrao_a_linha_fica_numa_fila_e_com_100_parte() {
    // O padrão (2 px): uma fila, o Weight ao lado da cor — e o campo leva o número.
    forma(2.0);
    let mut h = crate::host_aberto();
    let (cor, peso) = linha_da_camada(&mut h);
    assert!(
        (peso.y - cor.y).abs() < 0.5,
        "com o valor padrão a linha partiu: Weight {peso:?} × cor {cor:?}"
    );
    assert!(
        peso.w >= precisa("2", peso),
        "o «2» não cabe no Weight: {peso:?}"
    );

    // `100`: a linha parte, o Weight vai para a 2.ª fila e leva o número inteiro.
    forma(100.0);
    let mut h = crate::host_aberto();
    let (cor, peso) = linha_da_camada(&mut h);
    assert!(
        peso.y > cor.y + cor.h - 0.5,
        "com 100 o Weight ficou na fila da cor (cortado): {peso:?} × {cor:?}"
    );
    assert!(
        peso.w >= precisa("100", peso),
        "o Weight da 2.ª fila não leva o «100»: {peso:?} precisa {}",
        precisa("100", peso)
    );
    // CONTROLO: a régua é a mesma que reprova a 1.ª fila — a fábrica não leva o «100» numa fila.
    forma(2.0);
    let mut h = crate::host_aberto();
    let (_, numa_fila) = linha_da_camada(&mut h);
    assert!(
        numa_fila.w < precisa("100", numa_fila),
        "controlo: na largura de fábrica o «100» caberia numa fila ({numa_fila:?}) — o teste não \
         mediria a partição"
    );
    limpa();
}
