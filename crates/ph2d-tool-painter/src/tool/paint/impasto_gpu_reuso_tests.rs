//! A dobra entregue por linhas aos destinos de envio da GPU (stride, padding, buffer maior) é a dos `Vec` AO BIT.

use crate::tool::PainterTool;
use crate::tool::paint::impasto_gpu::ImpastoPlanes;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Papel com relevo + uma crista com cera fechada + uma por soltar, à escala `w × h`.
fn papel_e_crista(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.3);
    t.paint.brush.impasto_wax = 0.9;
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x * w as f32, y * h as f32],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(0.2, 0.5, PointerPhase::Down));
    t.on_canvas_pointer(at(0.8, 0.5, PointerPhase::Up));
    t.add_raster_layer("cima").expect("camada de cima");
    t.on_canvas_pointer(at(0.5, 0.1, PointerPhase::Down));
    t.on_canvas_pointer(at(0.5, 0.6, PointerPhase::Move));
    t.reconcile_substrate();
    t
}

/// O lixo que enche os destinos: um byte que a dobra não escreva (ou um de padding que escreva) aparece.
const LIXO: u8 = 0xA5;

/// **Ao bit:** a dobra entregue por linhas a destinos com `stride` alinhado a 256 (o envio da GPU), cheios de lixo e
/// MAIORES do que a janela pede (o buffer de envio reaproveitado de uma janela maior), é a dos `Vec` da
/// [`PainterTool::impasto_gpu_planes_in`] — e nenhum byte de padding, nem das linhas a mais, é tocado. Larguras
/// não múltiplas de 64 (o `stride` tem padding de verdade) e a tela inteira.
///
/// **Mutações que sangram:** a linha `j` da faixa entregue a outra (`j` sem o início da faixa) · o `y` da linha sem o
/// `ry` da janela · o papel por entregar.
#[test]
fn a_dobra_por_linhas_nos_destinos_de_envio_e_a_dos_vec_ao_bit() {
    let (w, h) = (96, 64);
    let t = papel_e_crista(w, h);
    let alinha = |b: usize| b.div_ceil(256) * 256;
    for janela in [
        (0, 0, w, h),
        (10, 7, 41, 30),
        (0, 0, 3, 64),
        (50, 20, 200, 200),
    ] {
        let velho: ImpastoPlanes = t.impasto_gpu_planes_in(janela).expect("planos");
        let dobra = t.impasto_dobra(janela).expect("dobra");
        assert_eq!(dobra.region, velho.region, "{janela:?}: janela");
        assert!(dobra.com_papel(), "a fixtura tem papel");
        let (_, _, rw, rh) = dobra.region;
        let (rw, rh) = (rw as usize, rh as usize);
        // `[relevo, cobertura, mat0, mat1, papel]`, cada um com `stride` alinhado e 3 linhas a mais, cheios de lixo.
        let row = [rw * 4, rw, rw * 4, rw * 4, rw * 4];
        let stride = row.map(alinha);
        let mut planos: Vec<Vec<u8>> = (0..5).map(|k| vec![LIXO; stride[k] * (rh + 3)]).collect();
        {
            let mut linhas: Vec<Vec<&mut [u8]>> = (0..rh).map(|_| Vec::new()).collect();
            for (k, p) in planos.iter_mut().enumerate() {
                for (j, l) in p.chunks_mut(stride[k]).take(rh).enumerate() {
                    linhas[j].push(&mut l[..row[k]]);
                }
            }
            let linhas: Vec<_> = linhas
                .into_iter()
                .map(|l| std::sync::Mutex::new(Some(l)))
                .collect();
            dobra.dobrar_faixas(
                |rows| {
                    rows.map(|j| {
                        linhas[j as usize]
                            .lock()
                            .expect("linha")
                            .take()
                            .expect("abre-se uma vez")
                    })
                    .collect::<Vec<_>>()
                },
                |faixa, j, l| {
                    let d = &mut faixa[j];
                    d[0].copy_from_slice(bytemuck::cast_slice(l.relief));
                    d[1].copy_from_slice(l.cover);
                    d[2].copy_from_slice(l.mat0);
                    d[3].copy_from_slice(l.mat1);
                    d[4].copy_from_slice(bytemuck::cast_slice(l.papel.expect("papel")));
                },
            );
            assert!(
                linhas.iter().all(|l| l.lock().expect("linha").is_none()),
                "cada linha foi aberta"
            );
        }
        let esperado: [&[u8]; 5] = [
            bytemuck::cast_slice(&velho.relief),
            &velho.cover,
            &velho.mat0,
            &velho.mat1,
            bytemuck::cast_slice(&velho.papel),
        ];
        for (k, nome) in ["relevo", "cobertura", "mat0", "mat1", "papel"]
            .into_iter()
            .enumerate()
        {
            for (y, linha) in planos[k].chunks(stride[k]).enumerate() {
                let (dados, pad) = linha.split_at(row[k]);
                if y < rh {
                    assert_eq!(
                        dados,
                        &esperado[k][y * row[k]..(y + 1) * row[k]],
                        "{janela:?}: {nome}, linha {y}"
                    );
                } else {
                    assert!(
                        dados.iter().all(|&b| b == LIXO),
                        "{janela:?}: {nome} escreveu a linha {y} a mais"
                    );
                }
                assert!(
                    pad.iter().all(|&b| b == LIXO),
                    "{janela:?}: {nome} escreveu o padding da linha {y}"
                );
            }
        }
        assert_eq!(
            dobra.paper_body.to_bits(),
            velho.paper_body.to_bits(),
            "{janela:?}: papel"
        );
        assert_eq!(dobra.lamps, velho.lamps, "{janela:?}: lâmpadas");
    }
}
