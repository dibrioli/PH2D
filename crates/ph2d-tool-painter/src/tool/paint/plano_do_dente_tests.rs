//! ⭐ B10.2 — o plano do dente do papel em cache dá o dente amostrado por píxel AO BIT, e nunca serve velho.

use super::*;
use crate::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Papel com relevo (Roughness a meio) e uma crista de impasto com cera por cima, numa tela que não é quadrada.
fn papel_e_crista(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.3);
    t.paint.brush.color = [0.8, 0.2, 0.2];
    let at = |x: f32, phase| CanvasPointer {
        pos: [x, h as f32 * 0.5],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(w as f32 * 0.2, PointerPhase::Down));
    t.on_canvas_pointer(at(w as f32 * 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(w as f32 * 0.8, PointerPhase::Up));
    t
}

/// O dente de cada texel pelo plano == a amostra, ao bit.
fn plano_igual_a_amostra(t: &PainterTool) {
    let s = t.substrate().expect("papel ligado");
    let (cargas, w) = s.plano.expect("o reconcile deixou o plano do dente");
    let (tw, th) = t.source_size;
    assert_eq!((w, cargas.len()), (tw as usize, (tw * th) as usize));
    for y in 0..i64::from(th) {
        for x in 0..i64::from(tw) {
            let (p, a) = (s.dente_na_tela(x, y), s.tooth_loads(x, y));
            assert_eq!(p.to_bits(), a.to_bits(), "dente em ({x},{y}): {p} × {a}");
        }
    }
}

/// O passe de luz da CPU e os planos da placa com o plano do dente == sem ele (a amostra por píxel de antes).
fn luz_e_planos_sem_o_plano_iguais(t: &mut PainterTool) {
    let (w, h) = t.source_size;
    let r = Region { x: 0, y: 0, w, h };
    let mut com = t.canvas_rgba.to_vec();
    t.apply_impasto_light(&mut com, r);
    let pc = t.impasto_gpu_planes_in((0, 0, w, h)).expect("planos");
    let guardado = t.plano_do_dente.take();
    assert!(t.substrate().expect("papel").plano.is_none());
    let mut sem = t.canvas_rgba.to_vec();
    t.apply_impasto_light(&mut sem, r);
    let ps = t.impasto_gpu_planes_in((0, 0, w, h)).expect("planos");
    t.plano_do_dente = guardado;
    assert!(com != t.canvas_rgba.to_vec(), "a fixture não acendeu nada");
    assert!(
        com == sem,
        "a luz com o plano do dente difere da amostra por píxel"
    );
    let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&pc.relief), bits(&ps.relief), "relevo dos planos");
    assert!(pc.cover == ps.cover && pc.mat0 == ps.mat0 && pc.mat1 == ps.mat1);
}

/// ⭐ **O plano do dente é o dente amostrado AO BIT** — e um knob do papel ou um resize nunca leem o plano velho
/// (antes do próximo reconcile caem na amostra; depois dele, o plano novo). *Mutações: tirar a chave ou o tamanho do
/// `plano_do_dente_em_dia` ⇒ o plano velho serve o papel novo; tirar o `reconcilia_o_plano_do_dente` ⇒ sem plano.*
#[test]
fn o_plano_do_dente_e_o_dente_amostrado_ao_bit() {
    let mut t = papel_e_crista(150, 101);
    t.reconcile_substrate();
    plano_igual_a_amostra(&t);
    luz_e_planos_sem_o_plano_iguais(&mut t);
    // Um knob do papel: velho até o reconcile, nunca servido; o novo ao bit.
    t.set_substrate_roughness(1.0);
    assert!(
        t.substrate().expect("papel").plano.is_none(),
        "o plano velho serviu a Roughness nova"
    );
    t.reconcile_substrate();
    plano_igual_a_amostra(&t);
    // Outro papel (Noise, ladrilho rodado e maior).
    t.set_brush_paper_kind(TextureKind::Noise as u8);
    t.paint.brush.paper.angle_deg = 30;
    t.paint.brush.paper.size = [2.0, 2.0];
    t.sync_paper_across_slots();
    assert!(t.substrate().expect("papel").plano.is_none());
    t.reconcile_substrate();
    plano_igual_a_amostra(&t);
    luz_e_planos_sem_o_plano_iguais(&mut t);
    // Outro tamanho de tela.
    t.set_source(vec![255u8; 97 * 64 * 4], 97, 64);
    assert!(
        t.substrate().expect("papel").plano.is_none(),
        "o plano velho serviu a tela nova"
    );
    t.reconcile_substrate();
    plano_igual_a_amostra(&t);
    // Sem relevo, o plano larga-se.
    t.set_substrate_depth(0.0);
    t.reconcile_substrate();
    assert!(t.plano_do_dente.is_none());
}

/// Sonda: a luz da CPU e os planos da placa numa tela 4K com papel, sem o plano do dente (a amostra de antes) × com
/// ele, intercalados, o MÍNIMO de 3. `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke --
/// --ignored sonda_plano_do_dente_4k --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_plano_do_dente_4k() {
    let (w, h) = (3840, 2160);
    let mut t = papel_e_crista(w, h);
    let t0 = std::time::Instant::now();
    t.reconcile_substrate();
    let monta = t0.elapsed().as_secs_f64() * 1e3;
    let r = Region { x: 0, y: 0, w, h };
    let guardado = t.plano_do_dente.clone();
    let mut m = [[f64::MAX; 2]; 2];
    for _ in 0..3 {
        for (k, plano) in [None, guardado.clone()].into_iter().enumerate() {
            t.plano_do_dente = plano;
            let mut px = t.canvas_rgba.to_vec();
            let t0 = std::time::Instant::now();
            t.apply_impasto_light(&mut px, r);
            m[k][0] = m[k][0].min(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = std::time::Instant::now();
            let _ = t.impasto_gpu_planes_in((0, 0, w, h));
            m[k][1] = m[k][1].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    eprintln!(
        "[dente 4K] montar o plano {monta:.1} ms · luz CPU {:.1} → {:.1} ms · planos da placa {:.1} → {:.1} ms",
        m[0][0], m[1][0], m[0][1], m[1][1]
    );
}

/// Sonda B10.A2: a 1.ª chamada das alturas do papel de fábrica (a gaguez do 1.º toque Wet sem Kind) e a 2.ª.
/// `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke -- --ignored sonda_alturas_de_fabrica --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_alturas_de_fabrica_1a_chamada() {
    let t0 = std::time::Instant::now();
    let q = alturas_do_papel_de_fabrica();
    let primeira = t0.elapsed().as_secs_f64() * 1e3;
    let t0 = std::time::Instant::now();
    let _ = alturas_do_papel_de_fabrica();
    let segunda = t0.elapsed().as_secs_f64() * 1e3;
    eprintln!(
        "[alturas de fábrica] 1.ª {primeira:.2} ms · 2.ª {segunda:.4} ms · {} quantis",
        q.len()
    );
}
