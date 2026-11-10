//! B15-cont — os planos da placa escritos nos buffers do quadro anterior são os planos novos AO BIT.

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

/// Os buffers do quadro anterior cheios de lixo: um byte que a dobra não escreva aparece na comparação.
fn com_lixo(mut p: ImpastoPlanes) -> ImpastoPlanes {
    p.relief.fill(f32::from_bits(0x7fc0_dead));
    p.cover.fill(0xA5);
    p.mat0.fill(0x5A);
    p.mat1.fill(0xC3);
    p
}

fn iguais(a: &ImpastoPlanes, b: &ImpastoPlanes, caso: &str) {
    assert_eq!(a.region, b.region, "{caso}: janela");
    let bits = |p: &ImpastoPlanes| p.relief.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(a), bits(b), "{caso}: relevo");
    assert_eq!(a.cover, b.cover, "{caso}: cobertura");
    assert_eq!(a.mat0, b.mat0, "{caso}: mat0");
    assert_eq!(a.mat1, b.mat1, "{caso}: mat1");
    assert_eq!(a.paper_body.to_bits(), b.paper_body.to_bits(), "{caso}: papel");
    assert_eq!(a.lamps.len(), b.lamps.len(), "{caso}: lâmpadas");
}

/// **Ao bit:** janela igual, maior, menor e de outra forma que a do quadro anterior, sobre buffers com lixo.
/// E na janela igual (o Ctrl+Z, a tela inteira de novo) os buffers são OS MESMOS: zero alocação.
///
/// **Mutações que sangram:** alocar de novo dentro de `impasto_gpu_planes_reusing` (o ponteiro muda) · `resize`
/// trocado por `reserve` (o comprimento fica o velho: janela menor reprova) · uma linha da dobra por escrever (o lixo).
#[test]
fn os_planos_reusados_sao_os_novos_ao_bit() {
    let (w, h) = (96, 64);
    let t = papel_e_crista(w, h);
    let tela = (0, 0, w, h);
    let janelas = [tela, (10, 7, 40, 30), (0, 0, 3, 64), (50, 20, 200, 200), tela];
    for antes in janelas {
        for agora in janelas {
            let velho = com_lixo(t.impasto_gpu_planes_in(antes).expect("planos"));
            let ptr = velho.relief.as_ptr();
            let reusado = t
                .impasto_gpu_planes_reusing(agora, Some(velho))
                .expect("planos");
            let novo = t.impasto_gpu_planes_in(agora).expect("planos");
            iguais(&reusado, &novo, &format!("{antes:?} → {agora:?}"));
            if antes == agora {
                assert_eq!(reusado.relief.as_ptr(), ptr, "a mesma janela alocou de novo");
            }
        }
    }
}

/// Sonda: a dobra da tela 4K com papel, buffers novos × reusados, intercalados, o MÍNIMO de 5.
/// `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke -- --ignored sonda_planos_reusados_4k --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_planos_reusados_4k() {
    let (w, h) = (3840, 2160);
    let t = papel_e_crista(w, h);
    let tela = (0, 0, w, h);
    let mut guardado = t.impasto_gpu_planes_in(tela);
    let mut m = [f64::MAX; 2];
    let ms = |t0: std::time::Instant| t0.elapsed().as_secs_f64() * 1e3;
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        let p = t.impasto_gpu_planes_in(tela);
        drop(p);
        m[0] = m[0].min(ms(t0));
        let t0 = std::time::Instant::now();
        guardado = t.impasto_gpu_planes_reusing(tela, guardado.take());
        m[1] = m[1].min(ms(t0));
    }
    eprintln!("[planos 4K] novos {:.1} → reusados {:.1} ms", m[0], m[1]);
}
