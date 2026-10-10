use super::*;

/// ⭐ Sem `fx.filter` no fio os filtros não correm — e o passe fundido leva o Motion como sempre.
#[test]
fn without_a_filter_nothing_changes() {
    let mut f = MotionFiltro::default();
    f.prepara(&Graph::new());
    assert!(!f.liga(true));
    assert!(f.pilha().is_empty());
}

/// A régua de comprimento do nó é o raio em que o passe SATURA na câmera do produto (o `MAX_HALF`
/// do `FxStackPass`, suporte de 3σ, `height_world = 10` sobre os `534` px da cena medida).
#[test]
fn the_length_ruler_is_where_the_pass_saturates() {
    let camera = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 1600,
        height: 972,
    };
    let [a, ..] = mundo_para_pixel(&camera, janela, Some([0.0, 0.0, 1600.0, 534.0]));
    let sigma_max = f64::from(ph2d_render::fx_stack::MAX_HALF) / 3.0;
    let saturacao = sigma_max / a;
    let regua = f64::from(ph2d_node_fx_filter::COMPRIMENTO_MAX);
    assert!(
        (saturacao - regua).abs() < 0.01,
        "o passe satura em {saturacao:.3} unidades e a régua diz {regua}"
    );
}

/// O Y do mundo sobe e o do pixel desce: uma sombra para BAIXO no mundo cai para baixo na tela.
#[test]
fn world_down_is_screen_down() {
    let camera = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let [a, b, c, d] = mundo_para_pixel(&camera, janela, None);
    assert!((a - 60.0).abs() < 1e-9 && b == 0.0 && c == 0.0 && (d + 60.0).abs() < 1e-9);
    let mut op = ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::DROP_SHADOW);
    op.offset = [0.0, -0.5];
    assert_eq!(FxOpGpu::resolve(&op, [a, b, c, d]).offset_px, [0, 30]);
}

/// ⚠️ A camada não atravessa o tonemap. Hoje ele é `clamp` (bypass) e a conta fecha; se o AgX
/// ligar, a camada filtrada sai com outra cor que o resto da cena — este gate reprova primeiro.
#[test]
fn the_layer_skips_a_tonemap_that_is_a_bypass() {
    let wgsl = include_str!("../../ph2d-render/src/shaders/tonemap.wgsl");
    assert!(
        wgsl.contains("const BYPASS_LUT: bool = true;"),
        "o tonemap deixou o bypass: a camada filtrada (motion_filter) tem de passar por ele"
    );
}
