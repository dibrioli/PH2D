//! SONDA W12 (`docs/Macros/handoffs/PROMPT_W12_cruzamentos_em_wet_paint_2026-10-20.md`): no Wet Paint a
//! vertical do artista funde-se com a tinta gravada na MESMA sessão. Mede, na coluna da vertical, o que o
//! Play mostraria antes de as gravadas nascerem (o palco branco) com cada candidato a cobertura `c`.

use super::roteiro::grava_duas_horizontais_ligado_em;
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};

const L: u32 = 96;
const N: usize = (L * L) as usize;

fn tique(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(16.667);
}

fn assenta(t: &mut PainterTool) {
    for _ in 0..40_000 {
        if t.agua_parada() {
            return;
        }
        tique(t);
    }
    panic!("a água não assentou");
}

fn acerta(t: &mut PainterTool) {
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([0, 0, 0]);
}

fn vertical(t: &mut PainterTool, x: f32) {
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = L as f32;
    for i in 0..14 {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let f = i as f32 / 13.0;
        let phase = match i {
            0 => PointerPhase::Down,
            13 => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, (0.12 + 0.76 * f) * l],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        tique(t);
    }
    tique(t);
    assenta(t);
}

/// Os planos de pigmento da camada ativa do motor: (sett, sett_rgb, susp, susp_rgb).
type Massa = (Vec<f32>, Vec<[f32; 3]>, Vec<f32>, Vec<[f32; 3]>);

fn massa(t: &mut PainterTool) -> Massa {
    let mut m = None;
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        m = Some((
            g.sett.clone(),
            g.sett_rgb.clone(),
            g.susp.clone(),
            g.susp_rgb.clone(),
        ));
    });
    m.expect("motor")
}

/// H: o pigmento do artista estimado pela DIFERENÇA de massa por célula desde o 1.º toque, desenhado pela
/// mesma porta do render (RGBA recto, a alfa = a cobertura dele sozinho).
fn render_do_artista(t: &mut PainterTool, antes: &Massa) -> Vec<u8> {
    let mut out = vec![0u8; N * 4];
    t.wet_sonda_motor(|e| {
        assert_eq!((e.width, e.height), (L as usize, L as usize), "razão 1");
        let mut g = e.active_grid().clone();
        for i in 0..g.sett.len() {
            let diff = |agora: f32, rgb: [f32; 3], m0: f32, rgb0: [f32; 3]| -> (f32, [f32; 3]) {
                let d = (agora - m0).max(0.0);
                if d <= 0.0 {
                    return (0.0, rgb);
                }
                let c = std::array::from_fn(|k| {
                    ((agora * rgb[k] - m0 * rgb0[k]) / d).clamp(0.0, 255.0)
                });
                (d, c)
            };
            let (s, sc) = diff(g.sett[i], g.sett_rgb[i], antes.0[i], antes.1[i]);
            let (u, uc) = diff(g.susp[i], g.susp_rgb[i], antes.2[i], antes.3[i]);
            g.sett[i] = s;
            g.sett_rgb[i] = sc;
            g.susp[i] = u;
            g.susp_rgb[i] = uc;
        }
        let p = e.sim.gather_params(&e.tuning);
        let op = e.layers[e.active_layer].opacity;
        let layers = [ph2d_wet_paint::render::RenderLayer {
            grid: &g,
            opacity: op,
            visible: true,
        }];
        let visual = ph2d_wet_paint::render::PigmentVisual {
            paper: false,
            relevo: true,
        };
        let w = L as usize;
        for cy in 1..=L as usize {
            ph2d_wet_paint::render::render_pigment_row_visual(
                Some(&p),
                &layers,
                visual,
                cy,
                1,
                w,
                &mut out[(cy - 1) * w * 4..cy * w * 4],
            );
        }
    });
    out
}

#[test]
#[ignore = "SONDA W12: a fusão no Wet Paint e os candidatos a cobertura"]
fn sonda_w12_candidatos_no_wet_paint() {
    let x = 48.3;
    // A régua «sozinha»: a vertical numa tela branca nova.
    let mut r = PainterTool::default();
    r.set_source(vec![255; N * 4], L, L);
    r.set_wet_relogio_fixo(true);
    acerta(&mut r);
    let m0r = massa(&mut r);
    vertical(&mut r, x);
    let branco = r.imagem_aplicada().expect("doc").0;
    let so_r = render_do_artista(&mut r, &m0r);

    let (mut t, _, _) =
        grava_duas_horizontais_ligado_em(L, 7, PaintMedia::WetPaint).expect("grava");
    assenta(&mut t);
    acerta(&mut t);
    let b = t.imagem_aplicada().expect("doc").0;
    let m0 = massa(&mut t);
    vertical(&mut t, x);
    let atual = t.imagem_aplicada().expect("doc").0;
    let so = render_do_artista(&mut t, &m0);

    // O Play antes de nascerem (palco = branco) por candidato; o canal R (cinzento).
    let w = 255.0f32;
    let mut pior = [0f32; 6];
    eprintln!("y: B atual | branco S_H(régua) S_H | now c=a_true c=a_H fit_H");
    for y in 0..L as usize {
        for xx in 0..L as usize {
            let i = (y * L as usize + xx) * 4;
            let (bb, at, br) = (f32::from(b[i]), f32::from(atual[i]), f32::from(branco[i]));
            let a_true = 1.0 - br / w;
            let ah = f32::from(so[i + 3]) / 255.0;
            let sh = f32::from(so[i]) * ah + (1.0 - ah) * w;
            let ahr = f32::from(so_r[i + 3]) / 255.0;
            let shr = f32::from(so_r[i]) * ahr + (1.0 - ahr) * w;
            let now = at;
            let c_true = at + (1.0 - a_true) * (w - bb);
            let c_h = at + (1.0 - ah) * (w - bb);
            let fit = sh; // o ajuste a dois pontos mostra S_H no branco por construção
            let errs = [
                (now - br).abs(),
                (c_true - br).abs(),
                (c_h.min(255.0) - br).abs(),
                (fit - br).abs(),
                (shr - br).abs(),
                (sh - br).abs(),
            ];
            for (p, e) in pior.iter_mut().zip(errs) {
                *p = p.max(e);
            }
            if xx == 48 && (y % 3 == 0 || (bb < 250.0)) {
                eprintln!(
                    "{y:2}: {bb:3.0} {at:3.0} | {br:3.0} {shr:3.0} {sh:3.0} | {now:3.0} {c_true:3.0} {:3.0} {fit:3.0}",
                    c_h.min(255.0)
                );
            }
        }
    }
    eprintln!(
        "pior contra a régua (níveis): now {:.0} · c=a_true {:.0} · c=a_H {:.0} · fit_H {:.0} · S_H na régua {:.0} · S_H {:.0}",
        pior[0], pior[1], pior[2], pior[3], pior[4], pior[5]
    );
}

#[test]
#[ignore = "SONDA W12: (A) a sessão da gravação assa no 1.º toque do artista"]
fn sonda_w12_assar_no_primeiro_toque() {
    let x = 48.3;
    let mut r = PainterTool::default();
    r.set_source(vec![255; N * 4], L, L);
    r.set_wet_relogio_fixo(true);
    acerta(&mut r);
    vertical(&mut r, x);
    let branco = r.imagem_aplicada().expect("doc").0;

    let (mut f, _, _) =
        grava_duas_horizontais_ligado_em(L, 7, PaintMedia::WetPaint).expect("grava");
    assenta(&mut f);
    acerta(&mut f);
    vertical(&mut f, x);
    let fundida = f.imagem_aplicada().expect("doc").0;

    let (mut t, _, _) =
        grava_duas_horizontais_ligado_em(L, 7, PaintMedia::WetPaint).expect("grava");
    assenta(&mut t);
    let b = t.imagem_aplicada().expect("doc").0;
    t.wetpaint_end_session();
    acerta(&mut t);
    vertical(&mut t, x);
    let atual = t.imagem_aplicada().expect("doc").0;
    let pig = t.wet_pigmento().expect("sessão").to_vec();
    assert_eq!(pig.len(), N * 4, "razão 1");
    let (mut identidade, mut play, mut doc) = (0f32, 0f32, 0u8);
    for i in 0..N {
        let a = f32::from(pig[i * 4 + 3]) / 255.0;
        for k in 0..3 {
            let j = i * 4 + k;
            let bb = f32::from(b[j]);
            let p = f32::from(pig[j]) * a;
            identidade = identidade.max((p + (1.0 - a) * bb - f32::from(atual[j])).abs());
            let mostra = f32::from(atual[j]) + (1.0 - a) * (255.0 - bb);
            play = play.max((mostra - f32::from(branco[j])).abs());
            doc = doc.max(atual[j].abs_diff(fundida[j]));
        }
    }
    eprintln!(
        "(A) identidade atual = P + (1−pa)·B: {identidade:.1} níveis · Play antes de nascerem contra a régua: {play:.1} · o documento muda (fundida × assada): {doc}"
    );
}

/// **O custo da água marcada** (W12): a 1024², a mesma pincelada de Wet Paint com e sem a marca posta, a
/// água no worker real, o `on_tick` do editor (60 Hz) cronometrado em blocos intercalados por ordem rodada;
/// a diferença é o composite a devolver a cobertura e a marca a recompor-se.
///
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib -- --ignored
/// --nocapture sonda_w12_o_custo_da_agua_marcada`
#[test]
#[ignore = "SONDA W12: o custo da água marcada"]
fn sonda_w12_o_custo_da_agua_marcada() {
    use super::marca::MarcaDoArtista;
    const LADO: u32 = 1024;
    let nova = |marcada: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255; (LADO * LADO * 4) as usize], LADO, LADO);
        if marcada {
            t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
        }
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_brush_size_px(24.0);
        for i in 0..=8u16 {
            let phase = match i {
                0 => PointerPhase::Down,
                8 => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: [100.0 + 40.0 * f32::from(i), 900.0],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
            drop(t.take_preview_arc());
            t.on_tick(16.667);
        }
        (t, Vec::<f64>::new())
    };
    let mut vs = [("sem marca", nova(false)), ("com a marca", nova(true))];
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    for rodada in 0..12 {
        for k in 0..vs.len() {
            let (_, (t, ms)) = &mut vs[(rodada + k) % 2];
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(16));
                drop(t.take_preview_arc());
                let t0 = std::time::Instant::now();
                t.on_tick(16.667);
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    eprintln!("load depois: {}", carga().trim());
    for (nome, (t, ms)) in &mut vs {
        ms.sort_by(f64::total_cmp);
        #[expect(clippy::cast_precision_loss, reason = "centenas de quadros")]
        let media = ms.iter().sum::<f64>() / ms.len() as f64;
        eprintln!(
            "[W12] 1024² {nome:<12} on_tick: mín {:.3} · mediana {:.3} · média {media:.3} · máx {:.3} ms ({} quadros) · água a correr: {} · marcados: {}",
            ms[0],
            ms[ms.len() / 2],
            ms[ms.len() - 1],
            ms.len(),
            !t.agua_parada(),
            t.marca_do_artista().map_or(0, MarcaDoArtista::quantos)
        );
    }
}
