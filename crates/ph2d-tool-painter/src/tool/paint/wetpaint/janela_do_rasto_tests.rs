//! ⭐ **A JANELA DO RASTO NÃO CORTA O DAB** (smoke do dono 16/10: «parecem dois dabs de cada vez, mas o
//! segundo cortado»). A janela ancorava no 1.º dab e dava-lhe a margem `4 × Spacing` do motor (2 células),
//! mas na porta do pincel os dabs da mesma janela estão a uma CORDA (o Spacing do pincel × o diâmetro, ou o
//! sub-passo da cápsula) — e o que passava de `âncora ± half` perdia o pigmento numa recta vertical.
//!
//! O oráculo é o MESMO traço com a janela crescida de antemão (um traço enorme fora da tela: a janela só
//! cresce e vive entre traços), que não pode cortar nada.
use super::shape_color_tests::arma_rampa;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::TextureKind;

fn ponteiro(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A tela branca `w × h` do Wet Paint, Voronoi pequeno (Size 4) com a Shape Color nas `camadas`.
pub(super) fn tela(w: u32, h: u32, tamanho: f32, camadas: (bool, bool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_brush_shape_size(0, 4.0);
    t.set_brush_shape_size(1, 4.0);
    arma_rampa(&mut t, Some(camadas));
    t
}

/// Semeia o pincel autorado em todos os modos (o traço lê o do modo).
pub(super) fn semeia(t: &mut PainterTool) {
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
}

/// O ORÁCULO: as janelas do rasto das lanes já CRESCIDAS para um raio de 700 células (o `fit_to` só
/// cresce e o `start_stroke` guarda a meia-largura) — o mesmo traço, a mesma semente, e nada pode ser
/// cortado. Fora dos cortes uma janela maior pinta ao byte o mesmo (cada texel é independente).
pub(super) fn cresce_a_janela(t: &mut PainterTool) {
    use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
    assert!(t.ensure_wet_session());
    let sess = t.paint.wetpaint.session.as_mut().expect("a sessão");
    sess.bring_home();
    let e = &mut sess.engine;
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    e.lane_trails.clear();
    for _ in 0..4 {
        let mut tr = Trail::default();
        tr.start_stroke(-5000.0, -5000.0, [0.0; 3], TrailMode::Paint);
        let longe = Dab {
            x: -5000.0,
            y: -5000.0,
            r: 700.0,
            hardness: 0.5,
            intensity: 1.0,
            water_amount: 0.0,
            dry_gate: 0.0,
            reach: 1.0,
            window_reach: 1.0,
            opacity: 1.0,
        };
        let _ = tr.accumulate_paint(e.active_grid_mut(), &p, &tex, &longe, false);
        tr.drop_remainder();
        e.lane_trails.push(tr);
    }
}

/// O traço recto da esquerda para a direita, `passo` px por evento, um quadro por evento.
pub(super) fn traco(t: &mut PainterTool, y: f32, de: f32, ate: f32, passo: f32) {
    t.on_canvas_pointer(ponteiro([de, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    let n = ((ate - de) / passo).round() as i32;
    for k in 1..=n {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(ponteiro([de + passo * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro([ate, y], PointerPhase::Up));
}

/// **A borda vertical**: em cada coluna, quantas linhas passam de papel a tinta (ou o inverso); o máximo
/// ÷ a altura pintada. Um disco espalha a borda (~0,1–0,2); um corte junta-a numa coluna.
pub(super) fn borda_vertical(px: &[u8], w: usize, h: usize) -> f64 {
    let pint = |x: usize, y: usize| {
        let p = &px[(y * w + x) * 4..][..3];
        255 - p[0].min(p[1]).min(p[2]) > 25
    };
    let mut maior = 0usize;
    for x in 1..w {
        let (mut e, mut d) = (0, 0);
        for y in 0..h {
            match (pint(x - 1, y), pint(x, y)) {
                (false, true) => e += 1,
                (true, false) => d += 1,
                _ => {}
            }
        }
        maior = maior.max(e).max(d);
    }
    let altura = (0..h).filter(|&y| (0..w).any(|x| pint(x, y))).count();
    maior as f64 / altura.max(1) as f64
}

/// O que um traço deixa: a borda vertical, a tinta e a tela.
type Traco = (f64, f64, Vec<u8>);

/// O traço `monta` + `pinta` com a janela de sempre e com a janela crescida (o oráculo).
fn a_e_b(
    monta: &dyn Fn() -> PainterTool,
    pinta: &dyn Fn(&mut PainterTool),
    (w, h): (usize, usize),
) -> [Traco; 2] {
    [false, true].map(|crescida| {
        let mut t = monta();
        if crescida {
            cresce_a_janela(&mut t);
        }
        semeia(&mut t);
        pinta(&mut t);
        if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
            let mut ppm = format!("P6 {w} {h} 255\n").into_bytes();
            for k in 0..w * h {
                ppm.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
            }
            let _ = std::fs::write(format!("{dir}/janela_{w}x{h}_{crescida}.ppm"), ppm);
        }
        (
            borda_vertical(&t.canvas_rgba, w, h),
            super::shape_color_tests::tinta(&t.canvas_rgba) as f64,
            t.canvas_rgba.to_vec(),
        )
    })
}

/// Os píxeis que diferem entre duas telas.
fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count()
}

/// **O traço ESPAÇADO não é cortado pela janela** — Spacing 0,3, Size 30 (raio 49 células), Voronoi +
/// Texture Color: antes da cura a borda vertical era `0,79` da altura (os dabs 2 e 3 de cada janela cortados
/// a direito) e a tinta `0,932` da do oráculo (medido 16/10, `sonda_corte.rs`).
///
/// **Mutação que sangra:** a margem do `fit_to` sem a corda E sem a rede (`ensure_fits`).
#[test]
fn o_traco_espacado_nao_e_cortado_pela_janela() {
    const T: f32 = 30.0;
    let monta = || {
        let mut t = tela(480, 220, T, (true, false));
        t.set_brush_spacing(0.3);
        t
    };
    let [(borda, tinta, a), (borda_o, tinta_o, o)] =
        a_e_b(&monta, &|t| traco(t, 110.0, 120.0, 400.0, 12.0), (480, 220));
    eprintln!(
        "borda {borda:.2} (oráculo {borda_o:.2}) · tinta {:.3} do oráculo · {} píxeis diferentes",
        tinta / tinta_o,
        diferentes(&a, &o)
    );
    assert!(
        borda <= 0.3,
        "a borda vertical: {borda:.2} da altura (o oráculo: {borda_o:.2})"
    );
    assert!(
        tinta >= 0.99 * tinta_o,
        "a tinta: {:.3} da do oráculo",
        tinta / tinta_o
    );
}

/// **O PINCEL DO DONO, com a CANETA a pousar** (16/10): Size 169, Spacing 0,364, Continuous, Jitter
/// Position 0,1, Water 1, Smooth Edges 0, Shape Color Texture + Falloff; a pressão sobe de 0,05 a 1 nos 3
/// primeiros eventos (1 px cada). A cápsula interpola o raio de 47 a 325 células em sub-dabs a ~16 células:
/// dentro de uma janela o raio cresce dos DOIS lados, e a janela, medida pelo 1.º sub-dab, cortava-os.
/// Antes da cura: `33 655` píxeis diferentes do oráculo (até 4 níveis — a orla, onde a silhueta do pincel
/// do dono já é fraca). A lei é «ao byte o que a janela grande pinta».
///
/// **Mutação que sangra:** tirar a corda do `fit_to` E a rede (`ensure_fits`) — cada metade sozinha cobre
/// este pincel (a rede sozinha sangra no `a_radius_that_grows_inside_a_window_still_fits` do motor).
#[test]
fn o_pincel_do_dono_pinta_o_que_a_janela_grande_pinta() {
    const T: f32 = 169.0;
    let (w, h) = (1400u32, 720u32);
    let monta = || {
        let mut t = tela(w, h, T, (true, true));
        t.set_brush_spacing(0.364);
        t.toggle_brush_continuous_spacing();
        t.set_brush_jitter_norm(0.1);
        t.paint.wetpaint.knobs.water = 1.0;
        t.paint.wetpaint.smooth_edges = 0.0;
        t
    };
    let caneta = |t: &mut PainterTool| {
        let pr = |k: u32| (0.05 + 0.95 * (k as f32 / 3.0)).min(1.0);
        let (mut x, y) = (380.0f32, 360.0f32);
        let mut p = ponteiro([x, y], PointerPhase::Down);
        p.pressure = pr(0);
        t.on_canvas_pointer(p);
        t.on_tick(1000.0 / 60.0);
        let mut k = 0u32;
        while x < 1020.0 {
            k += 1;
            x = (x + if k <= 3 { 1.0 } else { 24.0 }).min(1020.0);
            let mut p = ponteiro([x, y], PointerPhase::Move);
            p.pressure = pr(k);
            t.on_canvas_pointer(p);
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ponteiro([x, y], PointerPhase::Up));
    };
    let [(borda, tinta, a), (borda_o, tinta_o, o)] =
        a_e_b(&monta, &caneta, (w as usize, h as usize));
    let n = diferentes(&a, &o);
    eprintln!(
        "borda {borda:.2} (oráculo {borda_o:.2}) · tinta {:.4} do oráculo · {n} píxeis diferentes",
        tinta / tinta_o
    );
    assert_eq!(n, 0, "píxeis diferentes da janela grande");
}
