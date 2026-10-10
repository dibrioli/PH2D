//! **Sonda (ignorada, W1g): as opções numa rodada só** (§0.10) — a Água ao vivo e a Borda que pesa
//! (`Sim::sedimenta`, a fracção por passo por unidade de `|g|`), na bancada do doc 06 §2 (Wet Round, as
//! quatro águas, direita · meia · máxima: a orla de baixo ÷ de cima e quanto desce) e no S do oráculo
//! (a régua do recto: orla plana, fios largos, pontas quadradas), no MESMO processo. `PH2D_SONDA_OUT`
//! grava os `.ppm` das pranchas.
use super::sonda_achatamento::{Variante, cena};
use super::sonda_gota::bancada_com;

const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];

fn grava(nome: &str, w: usize, h: usize, px: &[u8]) {
    if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
        let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
        for p in px.chunks(4) {
            out.extend_from_slice(&p[..3]);
        }
        let _ = std::fs::write(format!("{dir}/{nome}.ppm"), out);
    }
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_opcoes_numa_rodada() {
    let ks: Vec<f64> = std::env::var("PH2D_SONDA_KS")
        .ok()
        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_else(|| vec![0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 40.0]);
    let casos: Vec<(bool, f64)> = [false, true]
        .iter()
        .flat_map(|&v| ks.iter().map(move |&k| (v, k)))
        .collect();
    let mut linhas: Vec<String> = std::thread::scope(|sc| {
        let fios: Vec<_> = casos
            .iter()
            .map(|&(viva, k)| sc.spawn(move || um_caso(viva, k)))
            .collect();
        fios.into_iter().map(|f| f.join().expect("caso")).collect()
    });
    linhas.sort();
    for l in linhas {
        println!("{l}");
    }
}

fn um_caso(viva: bool, k: f64) -> String {
    let ajusta = move |s: &mut ph2d_wet_paint::sim::Sim| {
        s.sedimenta = k;
        s.sim_while_painting = viva;
    };
    let mut linha = format!("[opções] viva {viva} k {k:>5}:");
    for (nome, anel) in [("direita", None), ("meia", Some(4)), ("máxima", Some(8))] {
        let px = bancada_com(0.005, 0.0, &[], anel, AGUAS, &ajusta);
        grava(
            &format!("bancada_v{}_k{k}_{nome}", u8::from(viva)),
            1000,
            640,
            &px,
        );
        let l: Vec<(f64, usize)> = (0..4).map(|i| super::gota_tests::le(&px, i)).collect();
        linha += &format!(
            " | bancada {nome}: orla {:.2} {:.2} {:.2} {:.2} · desce {}",
            l[0].0,
            l[1].0,
            l[2].0,
            l[3].0,
            l.iter().map(|x| x.1).max().unwrap_or(0)
        );
    }
    for (nome, anel) in [("meia", 4u8), ("máxima", 8)] {
        let v = Variante {
            sedimenta: Some(k),
            agua_no_traco: Some(viva),
            ..Variante::default()
        };
        let px = cena(&v, Some((3, anel)), 30, &mut |_, _| {}).0;
        grava(
            &format!("s_v{}_k{k}_{nome}", u8::from(viva)),
            1000,
            640,
            &px,
        );
        linha += &format!(" | S {nome}: recto {:?}", super::achatamento_tests::le(&px));
    }
    linha
}

/// A Live Water vê-se A PINTAR: o S no instante do Up (0 s de água depois), desligada × ligada, nas três
/// inclinações — as fotos da prancha.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_live_water_no_up() {
    for (nome, tilt) in [
        ("direita", None),
        ("meia", Some((3, 4))),
        ("máxima", Some((3, 8))),
    ] {
        for viva in [false, true] {
            let v = Variante {
                live_water: Some(viva),
                ..Variante::default()
            };
            let px = cena(&v, tilt, 0, &mut |_, _| {}).0;
            grava(&format!("up_v{}_{nome}", u8::from(viva)), 1000, 640, &px);
            println!(
                "[up] {nome} viva {viva}: tinta abaixo da orla {:?}",
                super::achatamento_tests::le(&px)
            );
        }
    }
}

/// O Smooth Edges no S do dono (Wet Round 100 px, água 1, tela a meia, 30 s): 0 · ¼ · ½ · 1 — as fotos
/// da prancha que escolhe o `RAIO_MAX`.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_smooth_edges_no_s() {
    for s in [0.0f32, 0.25, 0.5, 1.0] {
        let v = Variante {
            smooth: Some(s),
            ..Variante::default()
        };
        let px = cena(&v, Some((3, 4)), 30, &mut |_, _| {}).1;
        grava(&format!("smooth_{s}"), 1000, 640, &px);
    }
}

/// **Sonda (dono 10/10: «o smooth edge não funciona quando a tinta escorre sobre área já pintada»):** um
/// traço horizontal vermelho, depois um S azul por cima com a tela no máximo — os fios descem SOBRE o
/// vermelho. Smooth Edges 0 e 1, `PH2D_SONDA_OUT/levanta_<s>.ppm`.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_smooth_edges_sobre_tinta() {
    use crate::tool::PainterTool;
    use ph2d_brush_presets::{Biblioteca, Volumes};
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Wash")
        .expect("o pincel de fábrica")
        .preset;
    for s in [0.0f32, 1.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 1000 * 640 * 4], 1000, 640);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
        t.wet_preset_aplicar_volumes(&Volumes {
            size_px: Some(120.0),
            water: Some(1.0),
            ..Volumes::default()
        });
        t.paint.wetpaint.smooth_edges = s;
        // `PH2D_SONDA_BORDA=<v>`: a Heavy Edge (5.º smoke do dono: as linhas escuras dela sobre a tinta).
        if let Some(v) = std::env::var("PH2D_SONDA_BORDA")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            t.set_wet_borda_que_pesa(v);
        }
        let cp = |x: f32, y: f32, phase| CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        // O vermelho, com a tela direita, e seco.
        t.paint.wetpaint.tilt_on = false;
        t.set_brush_color_srgb8([200, 30, 30]);
        t.on_canvas_pointer(cp(100.0, 380.0, PointerPhase::Down));
        for k in 1..=266 {
            t.on_canvas_pointer(cp(100.0 + 3.0 * k as f32, 380.0, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(900.0, 380.0, PointerPhase::Up));
        for _ in 0..60 * 40 {
            t.on_tick(1000.0 / 60.0);
        }
        // O azul por cima, a tela no máximo: os fios descem sobre o vermelho.
        t.paint.wetpaint.tilt_on = true;
        t.paint.wetpaint.tilt_ring = 8;
        t.set_brush_color_srgb8([30, 60, 210]);
        let pts: Vec<[f32; 2]> = super::sonda_achatamento::o_s();
        t.on_canvas_pointer(cp(pts[0][0], pts[0][1], PointerPhase::Down));
        for q in &pts[1..] {
            t.on_canvas_pointer(cp(q[0], q[1], PointerPhase::Move));
            t.on_tick(8.0);
        }
        let u = pts[pts.len() - 1];
        t.on_canvas_pointer(cp(u[0], u[1], PointerPhase::Up));
        for _ in 0..60 * 40 {
            t.on_tick(1000.0 / 60.0);
        }
        let px = t.run_full().0;
        grava(&format!("levanta_{s}"), 1000, 640, &px);
    }
}
