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
    t.adopt_markers(Marker::None, Marker::Open, [1.0, 1.114], 0.0);
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
    t.adopt_markers(Marker::None, Marker::Open, [1.0, 1.114], 0.0);
    set(&mut t, ids::VECTOR_MARKER_SCALE, 2.0);
    let ([start, end], scale, _) = t.armed_markers();
    assert_eq!((start, end), (Marker::None, Marker::None));
    // F16c: o Head Size é POR PONTA — a caixa do começo arma só o começo; o fim fica no de
    // nascença (o 1,114 da seta adotada NÃO é armado).
    assert_eq!(scale, [2.0, crate::params::DEFAULT_MARKER_SCALE]);
    set(&mut t, ids::VECTOR_MARKER_SCALE_END, 1.5);
    assert_eq!(t.armed_markers().1, [2.0, 1.5], "a caixa do fim arma o fim");
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

/// ⭐ F16c — **o par 2 do tracejado chega à tool e ao padrão**, sem mexer no par 1; e o tamanho
/// da ponta do FIM muda só o fim.
#[test]
fn the_second_dash_pair_and_the_end_head_size_reach_the_tool() {
    let mut t = VectorTool::new();
    let _ = t.take_authored();
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_DASH_NUM, 4.0));
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_GAP_NUM, 2.0));
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_DASH_2, 1.0));
    use crate::StyleFields as F;
    assert_eq!(
        t.take_authored(),
        F::dash(0, false) | F::dash(0, true) | F::dash(1, false),
        "cada número do tracejado arma o SEU campo (plano 31)"
    );
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_GAP_2, 0.5));
    assert_eq!(t.take_authored(), F::dash(1, true));
    let p = t.dash_pattern().expect("ha' tracejado");
    assert_eq!(p.pairs[0], [4.0, 2.0], "o par 1 ficou");
    assert_eq!(p.pairs[1], [1.0, 0.5], "o par 2 chegou");
    assert_eq!(p.pairs[2], [0.0, 0.0], "o par 3 nao foi tocado");
    assert_eq!(p.active().len(), 2, "o padrao percorre DOIS pares");
    assert_eq!(t.ui_snapshot().dash[1], [1.0, 0.5], "e o snapshot leva-o");

    let antes = t.marker_scale();
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_MARKER_SCALE_END, 2.5),
    );
    assert_eq!(t.marker_scale(), [antes[0], 2.5], "so' o fim mudou");
    assert_eq!(t.take_authored(), F::MARKER_SCALE_END);
    assert_eq!(t.ui_snapshot().marker_scale, [antes[0], 2.5]);
}

/// ⭐ **F22 — a leitura do documento não pisa o gesto deste quadro.** Durante um Show a forma muda
/// a cada quadro (a ponte re-adota «por mão alheia»); a cor escolhida no seletor e a Width do painel
/// estão ARMADAS e por escrever, e a adoção apagava-as antes da escrita.
#[test]
fn adopting_never_overwrites_a_field_armed_this_frame() {
    let laranja =
        ph2d_vec_scene::StrokeSpec::new(ph2d_vec_scene::Rgba8::new(240, 140, 40, 255), 1.0);
    let mut t = VectorTool::new();
    t.pick_fill_rgba([40, 200, 90, 255]);
    t.adopt_fill([240, 140, 40, 255]);
    assert_eq!(
        t.fill_rgba(),
        [40, 200, 90, 255],
        "a adoção apagou a cor escolhida"
    );

    let mut t = VectorTool::new();
    t.pick_stroke_rgba([40, 200, 90, 255]);
    set(&mut t, ids::VECTOR_WIDTH, 1.0);
    let w = t.stroke_width_px();
    t.adopt_stroke(&laranja, 3.0);
    assert_eq!(
        t.stroke_rgba(),
        [40, 200, 90, 255],
        "a adoção apagou a cor do traço"
    );
    assert_eq!(t.stroke_width_px(), w, "a adoção apagou a Width do painel");

    // O controlo: sem gesto armado, a adoção lê o documento.
    let mut t = VectorTool::new();
    t.adopt_fill([240, 140, 40, 255]);
    t.adopt_stroke(&laranja, 3.0);
    assert_eq!(t.fill_rgba(), [240, 140, 40, 255]);
    assert_eq!(t.stroke_rgba(), [240, 140, 40, 255]);
    assert!((t.stroke_width_px() - 3.0).abs() < 1e-9);
}
