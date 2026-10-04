//! ⭐⭐⭐ **O GAUSSIANO BORRA O RELEVO DA PEÇA** (`docs/3D/30` §20) — pelo painel real, lido da placa.

use super::cena_52;
use super::painel::{Painel, painter_vermelho};
use crate::painter_na_malha::quadro;
use crate::pilha_da_peca::PilhaDaPeca;
use ph2d_tool_painter::ids::{
    PAINTER_LAYERS_ADD, PAINTER_LAYERS_ADD_ADJUSTMENT, PainterLayerWidget,
    painter_adjustment_kind_option_id, painter_layer_widget_id,
};
use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, GaussianBlurParams, LayerId, LayerKind, PaintMedia,
    PainterTool,
};

fn pilha(s: &crate::Sculpt3dScene) -> &PilhaDaPeca {
    s.objects[s.active]
        .pilha
        .as_ref()
        .expect("a peça tem pilha")
}

fn painter_impasto() -> PainterTool {
    let mut p = painter_vermelho();
    p.set_paint_media(PaintMedia::Impasto);
    p.set_brush_size_px(28.0);
    p
}

/// Uma camada de cima com relevo em TODAS as amostras (o pior caso: cada passo muda a peça inteira)
/// e um Gaussiano por cima dela, pelo menu.
fn camada_e_desfoque(
    s: &mut crate::Sculpt3dScene,
    p: &mut PainterTool,
    painel: &mut Painel,
) -> (LayerId, LayerId) {
    painel.clica(p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut *s), Some(&mut *p));
    let cima = pilha(s).pilha().active().expect("cima");
    let n = pilha(s).amostras();
    let todas: Vec<u32> = (0..n as u32).collect();
    let r: Vec<[f32; 2]> = (0..n).map(|i| [((i / 7) % 5) as f32 * 4e-3, 1.0]).collect();
    let o = &mut s.objects[s.active];
    o.pilha
        .as_mut()
        .and_then(|pl| pl.troca_relevo(cima, &todas, &r))
        .expect("relevo em todas");
    crate::tinta_da_peca::pilha::recompoe(o);
    painel.abre(p, PAINTER_LAYERS_ADD_ADJUSTMENT);
    let i = AdjustmentKind::ALL
        .iter()
        .position(|k| *k == AdjustmentKind::GaussianBlur)
        .expect("o desfoque está no menu");
    painel.clica(p, painter_adjustment_kind_option_id(i as u8));
    quadro(Some(&mut *s), Some(&mut *p));
    let desfoque = pilha(s).pilha().root()[0];
    assert!(matches!(
        pilha(s).pilha().get(desfoque).map(|c| &c.kind),
        Some(LayerKind::Adjustment(_))
    ));
    (cima, desfoque)
}

/// 🔎 O preço de um passo do arrasto do RAIO do Gaussiano com o relevo a passar por ele, a `8x…64x`
/// (critério da W6: `100 ms` a `32x`). Perfil `smoke`; diga a carga ao lado.
#[test]
#[ignore = "sonda: precisa de adaptador"]
fn diag_o_preco_de_arrastar_o_raio_com_relevo() {
    use ph2d_tool_painter::PieceLayerOp;
    use std::time::Instant;
    let gpu = gpu_or_skip!();
    for k in 3u8..=6 {
        let mut s = cena_52(&gpu.device);
        s.tinta_nivel = Some(k);
        s.sync_mesh(&gpu);
        let mut p = painter_impasto();
        let mut painel = Painel::novo();
        quadro(Some(&mut s), Some(&mut p));
        let (_, desfoque) = camada_e_desfoque(&mut s, &mut p, &mut painel);
        s.sync_mesh(&gpu);
        let ph2d_tool_painter::SpatialUnits::Surface { size } = p.panel_spatial_units() else {
            panic!("na peça o raio é das unidades dela")
        };
        let curso = ph2d_tool_painter::SURFACE_RADIUS_MAX * size;
        let gesto = painter_layer_widget_id(desfoque.0, PainterLayerWidget::AdjParam0);
        let n = pilha(&s).amostras();
        for frac in [0.4f32, 1.0] {
            let (mut cpu, mut placa) = (Vec::new(), Vec::new());
            for q in 0..10u32 {
                let mut nova = p.panel_layers().expect("pilha").clone();
                nova.adjustment_mut(desfoque).expect("ajuste").params =
                    AdjustmentParams::GaussianBlur(GaussianBlurParams {
                        radius: curso * frac * (0.8 + 0.02 * q as f32),
                    });
                let t = Instant::now();
                let recusa = s.aplica_pedidos_da_pilha(vec![PieceLayerOp::Metadata {
                    stack: nova,
                    gesture: Some(gesto),
                }]);
                cpu.push(t.elapsed().as_secs_f64() * 1e3);
                assert!(recusa.is_none());
                assert!(s.objects[s.active].relevo_sujo, "o passo redobrou o relevo");
                quadro(Some(&mut s), Some(&mut p));
                let t = Instant::now();
                s.sync_mesh(&gpu);
                gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
                placa.push(t.elapsed().as_secs_f64() * 1e3);
            }
            for v in [&mut cpu, &mut placa] {
                v.sort_by(f64::total_cmp);
            }
            eprintln!(
                "degrau {k} ({}x) · {n} amostras · raio {:.0} % do curso · um passo: porta+redobra \
                 (CPU) mediana {:.1} · pior {:.1} ms | sync_mesh mediana {:.1} · pior {:.1} ms",
                1u32 << k,
                frac * 100.0,
                cpu[5],
                cpu[9],
                placa[5],
                placa[9]
            );
        }
    }
}

/// ⭐⭐⭐ **GATE (o DoD da DIRETIVA §5) — O ARRASTO REAL DO RAIO DO GAUSSIANO BORRA O RELEVO DA PEÇA,
/// LIDO DA PLACA, E O `Ctrl+Z` DEVOLVE-O AO BIT** (decisão do dono de 04/10, `docs/3D/30` §20). A
/// camada de cima tem riscas de relevo em todas as amostras; o Gaussiano nasce pelo menu (raio `0`:
/// CONTROLO, nada muda) e o raio arrasta-se pelo painel. A placa tem o relevo da dobra da CPU (a
/// referência, o calor na CPU) — a paridade —, as riscas amaciaram, e as inclinações da placa são as
/// do relevo novo.
#[test]
#[ignore = "precisa de adaptador"]
fn o_raio_do_desfoque_pelo_painel_borra_o_relevo_da_peca_e_o_ctrl_z_o_devolve() {
    use super::relevo_painel::{bits, da_placa, inclinacoes_de, pior};
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let (_, desfoque) = camada_e_desfoque(&mut s, &mut p, &mut painel);
    s.sync_mesh(&gpu);
    let (antes, _) = da_placa(&mut s, &gpu);
    assert_eq!(
        bits(&antes),
        bits(&pilha(&s).relevo_composto().expect("relevo")),
        "CONTROLO: com o raio 0 a placa tem a dobra de sempre"
    );
    painel.arrasta(
        &mut p,
        painter_layer_widget_id(desfoque.0, PainterLayerWidget::AdjParam0),
        0.02,
        0.6,
    );
    quadro(Some(&mut s), Some(&mut p));
    s.sync_mesh(&gpu);
    assert!(
        !pilha(&s).relevo_por_dobrar(),
        "o sync_mesh dobrou o relevo"
    );
    let (depois, inc) = da_placa(&mut s, &gpu);
    let n = depois.len();
    let mudaram = (0..n).filter(|&i| depois[i] != antes[i]).count();
    assert!(
        mudaram * 2 > n,
        "o raio mudou o relevo em {mudaram} de {n} amostras"
    );
    let cpu = pilha(&s).relevo_composto().expect("relevo");
    let desvio = (0..n)
        .flat_map(|i| (0..2).map(move |e| (i, e)))
        .map(|(i, e)| (depois[i][e] - cpu[i][e]).abs() / (1e-3 + cpu[i][e].abs()))
        .fold(0.0f32, f32::max);
    eprintln!("a placa contra a CPU: pior desvio relativo {desvio:.2e}");
    assert!(
        desvio < 1e-3,
        "a placa diverge da dobra da CPU ({desvio:.2e})"
    );
    let energia = |r: &[[f32; 2]]| {
        let m = r.iter().map(|x| f64::from(x[0])).sum::<f64>() / r.len() as f64;
        r.iter().map(|x| (f64::from(x[0]) - m).powi(2)).sum::<f64>()
    };
    assert!(
        energia(&depois) < 0.7 * energia(&antes),
        "as riscas não amaciaram ({} → {})",
        energia(&antes),
        energia(&depois)
    );
    assert!(
        pior(&inc, &inclinacoes_de(&s, &depois)) < 1e-4,
        "as inclinações da placa não são as do relevo borrado"
    );
    assert!(super::tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    quadro(Some(&mut s), Some(&mut p));
    s.sync_mesh(&gpu);
    let (desfeito, _) = da_placa(&mut s, &gpu);
    assert_eq!(
        bits(&desfeito),
        bits(&antes),
        "o Ctrl+Z não devolveu o relevo da placa ao bit"
    );
}

/// O Brilho por cima de uma camada com relevo não o mexe (é uma operação de TOM): a placa fica AO BIT.
#[test]
#[ignore = "precisa de adaptador"]
fn o_brilho_por_cima_nao_mexe_no_relevo_da_peca() {
    use super::relevo_painel::{bits, da_placa};
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_impasto();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let (_, _) = camada_e_desfoque(&mut s, &mut p, &mut painel);
    s.sync_mesh(&gpu);
    let (antes, _) = da_placa(&mut s, &gpu);
    painel.abre(&p, PAINTER_LAYERS_ADD_ADJUSTMENT);
    let i = AdjustmentKind::ALL
        .iter()
        .position(|k| *k == AdjustmentKind::Bloom)
        .expect("o brilho está no menu");
    painel.clica(&mut p, painter_adjustment_kind_option_id(i as u8));
    quadro(Some(&mut s), Some(&mut p));
    s.sync_mesh(&gpu);
    let (depois, _) = da_placa(&mut s, &gpu);
    assert_eq!(bits(&depois), bits(&antes));
}
