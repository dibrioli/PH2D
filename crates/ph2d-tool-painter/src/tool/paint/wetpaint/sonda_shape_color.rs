//! As SONDAS da Shape Color no Wet Paint (prompt 41): a régua do passo 1 (a cor sobrevive à água? segue
//! o Digital? quanto custa?), os números das leis e a prancha Digital × Wet Paint.
use super::shape_color_tests::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::texture::TextureKind;

/// **A RÉGUA DO PROMPT 41, num lote** — (1) a rampa SOBREVIVE à água? (o doc 46 §1 recusou-a por
/// análise: «a mistura K–M homogeneíza o carimbo no primeiro passo»): G−R no soltar, a 1 s e a 10 s;
/// (2) a cor segue a do Digital no mesmo traço (correlação de G−R); (3) o custo por traço com e sem a
/// cor, intercalados no mesmo processo, o MÍNIMO de 9.
#[test]
#[ignore = "sonda: a régua do prompt 41"]
fn sonda_shape_color_no_wet_paint() {
    let camadas: [(&str, Option<(bool, bool)>); 4] = [
        ("desligada", None),
        ("Texture", Some((true, false))),
        ("Falloff", Some((false, true))),
        ("as duas", Some((true, true))),
    ];
    for (nome, c) in camadas {
        let mut dig = cena_da_regua(PaintMedia::Digital, c, 14.0);
        traco(&mut dig, 64.0, 16.0, 112.0, 0);
        let mut t = cena_da_regua(PaintMedia::WetPaint, c, 14.0);
        let mut linha = format!("{nome:>9}:");
        let mut assentou = 0;
        for (rotulo, ate) in [("soltar", 0u32), ("1 s", 60), ("10 s", 600)] {
            if assentou == 0 {
                traco(&mut t, 64.0, 16.0, 112.0, ate);
            } else {
                for _ in assentou..ate {
                    t.on_tick(1000.0 / 60.0);
                }
            }
            assentou = ate.max(1);
            let (n, m, s) = g_menos_r(&t.canvas_rgba);
            let r = correlacao(&dig.canvas_rgba, &t.canvas_rgba);
            linha += &format!("  {rotulo}: n {n} G−R {m:+.1}±{s:.1} ρ {r:.2}");
        }
        let (_, dm, ds) = g_menos_r(&dig.canvas_rgba);
        linha += &format!("  | Digital G−R {dm:+.1}±{ds:.1}");
        eprintln!("{linha}");
    }
    // Os 12 pincéis de fábrica: o desvio de G−R com a Texture Color ÷ o mesmo traço sem rampa.
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    // Só os do Wet Paint (os outros meios têm presets irmãos desde a J3).
    for e in bib
        .entradas
        .iter()
        .filter(|e| e.preset.parametros.media == crate::MEIO_DO_PRESET)
    {
        let corre = |c: Option<(bool, bool)>| {
            let mut t = cena_da_regua(PaintMedia::WetPaint, c, 14.0);
            assert!(t.wet_preset_aplicar(&e.preset).is_empty());
            t.wet_preset_aplicar_volumes(&e.preset.parametros.volumes);
            t.paint.wetpaint.tilt_on = false;
            t.set_brush_size_px(14.0);
            t.set_brush_color_srgb8([30, 60, 220]);
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            if c.is_some() {
                arma_rampa(&mut t, c);
            }
            let seed = t.paint.brush;
            t.paint.brush_by_mode.fill(seed);
            traco(&mut t, 64.0, 16.0, 112.0, 0);
            let a = g_menos_r(&t.canvas_rgba);
            for _ in 0..600 {
                t.on_tick(1000.0 / 60.0);
            }
            (a, g_menos_r(&t.canvas_rgba))
        };
        let (o0, o1) = corre(None);
        let (t0, t1) = corre(Some((true, false)));
        eprintln!(
            "{:>22}: sem rampa ±{:.1}/±{:.1} · Texture ±{:.1}/±{:.1} (soltar/10 s) · ×{:.1}/×{:.1}",
            e.preset.parametros.name,
            o0.2,
            o1.2,
            t0.2,
            t1.2,
            t0.2 / o0.2.max(0.1),
            t1.2 / o1.2.max(0.1)
        );
    }
    // O custo: raio 48 (o depósito domina), só os eventos do ponteiro (depósito + composição).
    let mede = |c: Option<(bool, bool)>| {
        let mut t = cena_da_regua(PaintMedia::WetPaint, c, 48.0);
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down));
        for k in 1..=24 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([16.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([112.0, 64.0], PointerPhase::Up));
        t0.elapsed().as_secs_f64() * 1e3
    };
    let (mut off, mut on) = (f64::MAX, f64::MAX);
    for k in 0..9 {
        if k % 2 == 0 {
            off = off.min(mede(None));
            on = on.min(mede(Some((true, true))));
        } else {
            on = on.min(mede(Some((true, true))));
            off = off.min(mede(None));
        }
    }
    eprintln!(
        "custo do traço (raio 48, 26 eventos, mín de 9): sem {off:.2} ms · com as duas {on:.2} ms · ×{:.2}",
        on / off
    );
}

#[test]
#[ignore = "sonda: os números das leis do prompt 41"]
fn sonda_shape_color_leis() {
    // (a) O padrão que vira cor deixa de recortar: a MESMA cor única pela Falloff Color na ponta lisa e
    // pela Texture Color no Voronoi.
    let ink = |kind: TextureKind, textura: bool| {
        let mut t = tela();
        t.set_brush_shape_kind(kind.to_u8());
        let r = rampa_de([0.05, 0.1, 0.7, 1.0]);
        t.set_shape_color_ramp(r.clone());
        t.set_shape_falloff_ramp(r);
        t.set_shape_ramp_enabled(true);
        t.paint.shape_camadas.textura = textura;
        t.paint.shape_camadas.queda = !textura;
        t.set_shape_falloff_ramp(t.paint.shape_camadas.queda_rampa.clone());
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        traco(&mut t, 64.0, 16.0, 112.0, 60);
        tinta(&t.canvas_rgba)
    };
    let lisa = ink(TextureKind::None, false) as f64;
    eprintln!(
        "(a) tinta ÷ ponta lisa (Falloff Color): Voronoi + Falloff {:.3} · Voronoi + Texture {:.3}",
        ink(TextureKind::Voronoi, false) as f64 / lisa,
        ink(TextureKind::Voronoi, true) as f64 / lisa
    );
    // (b) A cápsula: ρ de G−R contra o traço denso.
    let denso = capsula(0.05, false, true);
    for (nome, t) in [
        ("espaçado 1,5", capsula(1.5, false, true)),
        ("espaçado 1,5 sem rampa", capsula(1.5, false, false)),
        ("espaçado 1,5 + Continuous", capsula(1.5, true, true)),
    ] {
        eprintln!(
            "(b) {nome}: ρ contra o denso {:.2} · G−R {:?} · tinta {}",
            correlacao(&denso.canvas_rgba, &t.canvas_rgba),
            g_menos_r(&t.canvas_rgba),
            tinta(&t.canvas_rgba)
        );
    }
    eprintln!("(b) denso: G−R {:?}", g_menos_r(&denso.canvas_rgba));
    // (c) O carimbo (a prévia é o carimbo do Digital, prompt 38): ρ Wet × Digital.
    for c in [Some((true, false)), Some((false, true)), Some((true, true))] {
        let mut d = cena_da_regua(PaintMedia::Digital, c, 24.0);
        carimbo(&mut d);
        let mut w = cena_da_regua(PaintMedia::WetPaint, c, 24.0);
        carimbo(&mut w);
        eprintln!(
            "(c) carimbo {c:?}: ρ {:.2} · Wet {:?} · Digital {:?}",
            correlacao(&d.canvas_rgba, &w.canvas_rgba),
            g_menos_r(&w.canvas_rgba),
            g_menos_r(&d.canvas_rgba)
        );
    }
    // (d) A brocha suja (Wet Round): o traço atravessa uma faixa azul MOLHADA em x = 40; o azul (B −
    // max(R, G)) depois do cruzamento, com a rampa e com a cor única.
    let suja = |sobre_azul: bool, rampa: bool| {
        let mut t = tela();
        wet_round(&mut t);
        if sobre_azul {
            t.set_brush_color_srgb8([20, 30, 230]);
            let seed = t.paint.brush;
            t.paint.brush_by_mode.fill(seed);
            t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
            for k in 1..=17 {
                #[allow(clippy::cast_precision_loss)]
                t.on_canvas_pointer(cp([40.0, 30.0 + 4.0 * k as f32], PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(cp([40.0, 98.0], PointerPhase::Up));
        }
        t.set_brush_color_srgb8([230, 200, 20]);
        if rampa {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            arma_rampa(&mut t, Some((true, false)));
        }
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        traco(&mut t, 64.0, 16.0, 112.0, 0);
        let mut s = 0.0f64;
        for y in 56..72usize {
            for x in 60..80usize {
                let p = &t.canvas_rgba[(y * LADO as usize + x) * 4..][..4];
                s += f64::from(p[2]) - f64::from(p[0]).max(f64::from(p[1]));
            }
        }
        s / 320.0
    };
    for rampa in [false, true] {
        eprintln!(
            "(d) rampa {rampa}: B − max(R,G) depois do cruzamento: sobre branco {:.1} · sobre azul molhado {:.1}",
            suja(false, rampa),
            suja(true, rampa)
        );
    }
    // (e) O tom (B&W) age no Wet Paint.
    eprintln!(
        "(e) B&W ≠ Voronoi: {}",
        impressao(&cena(5).canvas_rgba) != impressao(&cena(1).canvas_rgba)
    );
}

/// **A prancha do prompt 41** — Digital × Wet Paint, sem rampa · Texture Color · Falloff Color · as duas
/// (Voronoi, verde → vermelho, pincel azul), no soltar e (Wet) 10 s depois; o Wet Round de fábrica; e um
/// carimbo de raio 40 nos dois meios (onde as células do padrão se vêem).
/// `PH2D_SONDA_OUT=<pasta>` grava `sc_<linha>_<coluna>.ppm`; `ferramentas/prancha_shape_color.py` monta-a.
#[test]
#[ignore = "sonda: a prancha da Shape Color no Wet Paint"]
fn sonda_prancha_shape_color() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let grava = |t: &crate::tool::PainterTool, nome: &str| {
        let mut ppm = format!("P6 {LADO} {LADO} 255\n").into_bytes();
        for k in 0..(LADO * LADO) as usize {
            ppm.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
        }
        std::fs::write(format!("{dir}/sc_{nome}.ppm"), ppm).expect("grava");
    };
    let linhas: [(&str, Option<(bool, bool)>); 4] = [
        ("desligada", None),
        ("texture", Some((true, false))),
        ("falloff", Some((false, true))),
        ("duas", Some((true, true))),
    ];
    for (nome, c) in linhas {
        let mut d = cena_da_regua(PaintMedia::Digital, c, 14.0);
        traco(&mut d, 64.0, 16.0, 112.0, 0);
        grava(&d, &format!("{nome}_digital"));
        let mut w = cena_da_regua(PaintMedia::WetPaint, c, 14.0);
        traco(&mut w, 64.0, 16.0, 112.0, 0);
        grava(&w, &format!("{nome}_wet"));
        for _ in 0..600 {
            w.on_tick(1000.0 / 60.0);
        }
        grava(&w, &format!("{nome}_wet10"));
        let mut r = cena_da_regua(PaintMedia::WetPaint, c, 14.0);
        wet_round(&mut r);
        r.set_brush_color_srgb8([30, 60, 220]);
        r.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
        if c.is_some() {
            arma_rampa(&mut r, c);
        }
        let seed = r.paint.brush;
        r.paint.brush_by_mode.fill(seed);
        traco(&mut r, 64.0, 16.0, 112.0, 0);
        grava(&r, &format!("{nome}_round"));
        for _ in 0..600 {
            r.on_tick(1000.0 / 60.0);
        }
        grava(&r, &format!("{nome}_round10"));
        for (m, col) in [
            (PaintMedia::Digital, "cdigital"),
            (PaintMedia::WetPaint, "cwet"),
        ] {
            let mut t = cena_da_regua(m, c, 40.0);
            carimbo(&mut t);
            grava(&t, &format!("{nome}_{col}"));
        }
    }
}
