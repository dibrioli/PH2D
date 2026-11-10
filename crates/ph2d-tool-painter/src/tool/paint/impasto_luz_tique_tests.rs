//! B15.3 — o tique de um knob da lâmpada recompõe só a LUZ, e o que ele mostra é a recomposição inteira ao bit.

use super::Region;
use crate::tool::{DrainBranch, PainterTool};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Papel Textured com relevo, um fundo de blocos de cor e duas cristas de Impasto (uma por camada).
fn tela(w: u32, h: u32) -> PainterTool {
    let mut px = vec![255u8; (w * h * 4) as usize];
    for (i, p) in px.chunks_exact_mut(4).enumerate() {
        let (x, y) = ((i as u32 % w) * 4 / w, (i as u32 / w) * 2 / h);
        p[..3].copy_from_slice(
            &[[230, 60, 40], [40, 160, 90], [50, 70, 200], [240, 200, 40]][((x + y) % 4) as usize],
        );
    }
    let mut t = PainterTool::default();
    t.set_source(px, w, h);
    t.set_substrate_depth(1.0);
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.3);
    t.paint.brush.color = [0.8, 0.2, 0.2];
    t.paint.brush.impasto_wax = 0.9;
    crista(&mut t, 0.3);
    t.add_raster_layer("Layer 2");
    crista(&mut t, 0.7);
    t
}

fn crista(t: &mut PainterTool, fy: f32) {
    let (w, h) = (t.source_size.0 as f32, t.source_size.1 as f32);
    let at = |x: f32, phase| CanvasPointer {
        pos: [x, h * fy],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(w * 0.2, PointerPhase::Down));
    t.on_canvas_pointer(at(w * 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(w * 0.8, PointerPhase::Up));
}

/// Os knobs da lâmpada que o tique arrasta, na ordem: 10 arrastos (ângulo, elevação, intensidade, cor, Show).
fn arrasto(t: &mut PainterTool, i: usize) {
    match i % 5 {
        0 => t.set_impasto_light_angle(30.0 + 23.0 * i as f32),
        1 => t.set_impasto_light_elevation(20.0 + 4.0 * i as f32),
        2 => t.set_impasto_light_intensity(0.4 + 0.1 * i as f32),
        3 => t.set_impasto_light_color([1.0, 0.7, 0.4 + 0.05 * i as f32]),
        _ => t.toggle_impasto_show(),
    }
}

fn drena(t: &mut PainterTool) -> Vec<u8> {
    t.take_preview_arc()
        .expect("o tique suja a prévia")
        .0
        .to_vec()
}

/// O tique da lâmpada = a recomposição inteira ao bit, nos 10 arrastos; pintar entre eles larga a composição guardada.
#[test]
fn o_tique_de_luz_pelo_gbuffer_e_a_luz_inteira() {
    let (w, h) = (240, 160);
    let (mut t, mut gemeo) = (tela(w, h), tela(w, h));
    drena(&mut t);
    let mut confere = |t: &mut PainterTool,
                       gemeo: &mut PainterTool,
                       i: usize,
                       ramo: DrainBranch,
                       quando: &str| {
        arrasto(t, i);
        arrasto(gemeo, i);
        let tique = drena(t);
        assert_eq!(
            t.last_drain_branch, ramo,
            "{quando}, arrasto {i}: o ramo da drenagem"
        );
        gemeo.pede_a_composicao_inteira();
        assert!(
            tique == drena(gemeo),
            "{quando}, arrasto {i}: o tique difere da recomposição inteira"
        );
    };
    for i in 0..10 {
        confere(&mut t, &mut gemeo, i, DrainBranch::Luz, "1.ª volta");
    }
    // Um traço drenado (o ramo parcial) larga-a: o 1.º tique depois dele recompõe; os seguintes reacendem.
    crista(&mut t, 0.5);
    crista(&mut gemeo, 0.5);
    drena(&mut t);
    assert_eq!(t.last_drain_branch, DrainBranch::PartialComposite);
    confere(
        &mut t,
        &mut gemeo,
        0,
        DrainBranch::FullComposite,
        "depois do traço",
    );
    for i in 1..10 {
        confere(&mut t, &mut gemeo, i, DrainBranch::Luz, "2.ª volta");
    }
    // A profundidade da camada muda a DOBRA e não a composição não acesa: o G-buffer cai com ela.
    for (k, depth) in [(0, 0.35), (1, -0.6)] {
        for tool in [&mut t, &mut gemeo] {
            let id = tool.layers.active().expect("camada activa");
            tool.set_layer_impasto_depth(id, depth);
        }
        let ramo = DrainBranch::FullComposite;
        confere(&mut t, &mut gemeo, 1 + k, ramo, "depois da profundidade");
        confere(&mut t, &mut gemeo, 2 + k, DrainBranch::Luz, "G-buffer novo");
    }
    // Traço e tique no MESMO quadro (o `mark_dirty` antes do knob): a drenagem compõe o traço, nunca a luz velha.
    crista(&mut t, 0.4);
    crista(&mut gemeo, 0.4);
    confere(
        &mut t,
        &mut gemeo,
        3,
        DrainBranch::FullComposite,
        "traço e tique juntos",
    );
}

/// A pista da PLACA (`compose_light_premul`): as texturas dos planos persistem e cada quadro escreve só a janela
/// dobrada; um quadro sem planos (Show desligado) obriga o seguinte à tela inteira.
#[derive(Default)]
struct Placa {
    relevo: Vec<u32>,
    cob: Vec<u8>,
    m0: Vec<u8>,
    m1: Vec<u8>,
    acesa: bool,
}

impl Placa {
    /// Um quadro da pista: a área dobrada (0 = sem luz); confere planos e lâmpadas contra a dobra inteira.
    fn quadro(&mut self, t: &mut PainterTool, caso: &str) -> u64 {
        assert!(t.take_preview_dirty(), "{caso}: o tique suja a prévia");
        let (w, h) = t.source_size;
        let janela = t
            .preview_gpu_region()
            .filter(|_| self.acesa)
            .unwrap_or((0, 0, w, h));
        let Some(p) = t.impasto_gpu_planes_in(janela) else {
            self.acesa = false;
            return 0;
        };
        let n = (w * h) as usize;
        if self.relevo.len() != n {
            (self.relevo, self.cob) = (vec![0; n], vec![0; n]);
            (self.m0, self.m1) = (vec![0; n * 4], vec![0; n * 4]);
        }
        let (x, y, pw, ph) = p.region;
        for r in 0..ph as usize {
            let (de, para, k) = (
                r * pw as usize,
                (y as usize + r) * w as usize + x as usize,
                pw as usize,
            );
            for (d, s) in self.relevo[para..para + k]
                .iter_mut()
                .zip(&p.relief[de..de + k])
            {
                *d = s.to_bits();
            }
            self.cob[para..para + k].copy_from_slice(&p.cover[de..de + k]);
            self.m0[para * 4..(para + k) * 4].copy_from_slice(&p.mat0[de * 4..(de + k) * 4]);
            self.m1[para * 4..(para + k) * 4].copy_from_slice(&p.mat1[de * 4..(de + k) * 4]);
        }
        self.acesa = true;
        let inteira = t.impasto_gpu_planes().expect("a dobra inteira");
        let bits: Vec<u32> = inteira.relief.iter().map(|v| v.to_bits()).collect();
        assert!(
            bits == self.relevo,
            "{caso}: relevo da placa ≠ dobra inteira"
        );
        assert!(
            inteira.cover == self.cob,
            "{caso}: cobertura ≠ dobra inteira"
        );
        assert!(
            inteira.mat0 == self.m0 && inteira.mat1 == self.m1,
            "{caso}: material ≠ dobra inteira"
        );
        assert_eq!(inteira.lamps, p.lamps, "{caso}: as lâmpadas da janela");
        u64::from(pw) * u64::from(ph)
    }
}

/// B15.3 (placa): o tique da lâmpada na pista GPU não redobra a tela — os planos persistem, só as lâmpadas
/// mudam; traço, profundidade da camada e Show desligado continuam a dobrar o que mudou.
#[test]
fn o_tique_da_lampada_na_placa_nao_redobra_a_tela() {
    let (w, h) = (240, 160);
    let tela_toda = u64::from(w) * u64::from(h);
    let mut t = tela(w, h);
    let mut placa = Placa::default();
    assert_eq!(placa.quadro(&mut t, "semente"), tela_toda);
    let mut tique = |t: &mut PainterTool, placa: &mut Placa, i: usize, caso: &str| {
        let acesa = placa.acesa;
        arrasto(t, i);
        let area = placa.quadro(t, &format!("{caso}, arrasto {i}"));
        if acesa && area > 0 {
            assert!(
                area <= 1,
                "{caso}, arrasto {i}: a placa redobrou {area} texels"
            );
        }
    };
    for i in 0..20 {
        tique(&mut t, &mut placa, i, "arrastos");
    }
    crista(&mut t, 0.5);
    let area = placa.quadro(&mut t, "traço");
    assert!(
        area > 1 && area < tela_toda,
        "o traço dobra a sua janela ({area})"
    );
    tique(&mut t, &mut placa, 1, "depois do traço");
    // A profundidade e o knob no MESMO quadro: a dobra muda na tela inteira.
    let id = t.layers.active().expect("camada activa");
    t.set_layer_impasto_depth(id, 0.35);
    arrasto(&mut t, 2);
    assert_eq!(placa.quadro(&mut t, "profundidade + knob"), tela_toda);
    // Traço e knob no MESMO quadro: a janela do traço.
    crista(&mut t, 0.4);
    arrasto(&mut t, 3);
    assert!(placa.quadro(&mut t, "traço + knob") > 1);
    tique(&mut t, &mut placa, 0, "no fim");
    // Uma tela nova do MESMO tamanho depois do knob (sai pela testemunha, não pela lista branca).
    arrasto(&mut t, 1);
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    assert_eq!(placa.quadro(&mut t, "tela nova"), tela_toda);
}

/// Sonda (B15.3, placa): a dobra do tique da lâmpada na pista GPU a 4K — a tela inteira (antes) × a janela que a
/// drenagem reivindica (agora), no MESMO processo, intercaladas, o mínimo de 10 arrastos.
#[test]
#[ignore = "sonda: corra com --ignored --nocapture em --profile smoke"]
fn sonda_tique_da_placa_4k() {
    let (w, h) = (3840, 2160);
    let mut t = tela(w, h);
    let _ = t.take_preview_dirty();
    let (mut tela_ms, mut janela_ms) = (f64::MAX, f64::MAX);
    for i in 0..10 {
        t.set_impasto_light_angle(30.0 + 17.0 * i as f32);
        let t0 = std::time::Instant::now();
        assert!(t.take_preview_dirty());
        let janela = t.preview_gpu_region().unwrap_or((0, 0, w, h));
        let p = t.impasto_gpu_planes_in(janela).expect("planos");
        janela_ms = janela_ms.min(t0.elapsed().as_secs_f64() * 1e3);
        let t0 = std::time::Instant::now();
        let q = t.impasto_gpu_planes_in((0, 0, w, h)).expect("planos");
        tela_ms = tela_ms.min(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(p.lamps, q.lamps);
        eprintln!("[placa 4K] {i}: janela {:?}", p.region);
    }
    eprintln!("[placa 4K] dobra do tique: tela inteira {tela_ms:.1} ms · janela {janela_ms:.2} ms");
}

/// Sonda (B15.3): 10 arrastos do ângulo a 4K, com papel e duas camadas de Impasto; ms por tique (o mínimo).
#[test]
#[ignore = "sonda: corra com --ignored --nocapture em --profile smoke"]
fn sonda_tique_de_luz_4k() {
    let (w, h) = (3840, 2160);
    let mut t = tela(w, h);
    drena(&mut t);
    let full = Region { x: 0, y: 0, w, h };
    let mut px = vec![128u8; (w * h * 4) as usize];
    let mut luz = f64::MAX;
    for _ in 0..3 {
        let t0 = std::time::Instant::now();
        t.apply_impasto_light(&mut px, full);
        luz = luz.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    let (mut min, mut soma) = (f64::MAX, 0.0);
    for i in 0..10 {
        t.set_impasto_light_angle(30.0 + 17.0 * i as f32);
        let t0 = std::time::Instant::now();
        drena(&mut t);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        (min, soma) = (min.min(ms), soma + ms);
        eprintln!(
            "[tique 4K] {i}: {ms:.1} ms ({})",
            t.last_drain_branch.name()
        );
    }
    eprintln!(
        "[tique 4K] mín {min:.1} · média {:.1} ms · só a luz (apply_impasto_light) {luz:.1} ms",
        soma / 10.0
    );
}

/// Sonda (B15.3 G-buffer): o passe inteiro (dobra + sombra, o tique de antes) × o sombreado pelo G-buffer, no MESMO
/// processo, intercalados, o mínimo de 7; e a montagem do G-buffer (o 1.º tique).
#[test]
#[ignore = "sonda: corra com --ignored --nocapture em --profile smoke"]
fn sonda_gbuffer_da_dobra_4k() {
    use crate::tool::paint::impasto_luz_dobra::LuzGuardada;
    let (w, h) = (3840, 2160);
    let mut t = tela(w, h);
    drena(&mut t);
    let full = Region { x: 0, y: 0, w, h };
    let base = vec![128u8; (w * h * 4) as usize];
    let g = LuzGuardada::new(base.clone());
    let (mut px, t0) = (base.clone(), std::time::Instant::now());
    assert!(t.acende_pela_dobra(&g, &mut px));
    let montagem = t0.elapsed().as_secs_f64() * 1e3;
    let (mut inteira, mut gbuf) = (f64::MAX, f64::MAX);
    for _ in 0..7 {
        let mut a = base.clone();
        let t0 = std::time::Instant::now();
        t.apply_impasto_light(&mut a, full);
        inteira = inteira.min(t0.elapsed().as_secs_f64() * 1e3);
        let mut b = base.clone();
        let t0 = std::time::Instant::now();
        assert!(t.acende_pela_dobra(&g, &mut b));
        gbuf = gbuf.min(t0.elapsed().as_secs_f64() * 1e3);
        assert!(a == b, "o G-buffer acendeu outro bit");
    }
    // B15.1: a faixa de 64 linhas contra a de 32 (o mesmo bit; o que muda é a moldura e o L2).
    let mut faixa = [f64::MAX; 2];
    for _ in 0..7 {
        for (k, linhas) in [32, 64].into_iter().enumerate() {
            let mut a = base.clone();
            let t0 = std::time::Instant::now();
            t.acende_em_faixas(&mut a, full, linhas);
            faixa[k] = faixa[k].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    eprintln!(
        "[gbuffer 4K] passe inteiro {inteira:.1} ms · pelo G-buffer {gbuf:.1} ms · montagem (1.º tique) {montagem:.1} ms"
    );
    eprintln!(
        "[faixa 4K] 32 linhas {:.1} ms · 64 linhas {:.1} ms",
        faixa[0], faixa[1]
    );
}
