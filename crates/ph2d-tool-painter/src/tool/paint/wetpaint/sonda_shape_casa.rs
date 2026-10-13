//! SONDAS da W0 do plano 27 (`docs/Painter/NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md` §6):
//! a tinta de cada tipo do Shape no Wet Paint e no Digital (a tabela do Bug #47 inteira) e o pigmento que
//! POUSA sob as leis candidatas do #47, por preset de fábrica.
//!
//! `cargo test -p ph2d-tool-painter --profile smoke --lib sonda_shape_casa -- --ignored --nocapture`
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::{self, ShapeFrame, TextureKind};
use ph2d_wet_paint::brush::{BristleKnobs, build_bristle_texture, sample_bristle};
use ph2d_wet_paint::tuning::Knob;
use std::sync::{Arc, Mutex};

const W: u32 = 320;
const H: u32 = 96;
/// O traço: 220 px na horizontal, de `x = 50` a `270`, a meia altura.
const X0: f32 = 50.0;
const COMPRIMENTO: f32 = 220.0;
const AMOSTRAS: usize = 44;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(media);
    t.set_wet_relogio_fixo(true);
    t
}

/// O traço recto de 220 px, um quadro por evento, e um segundo de quadros parados (a água assenta).
fn traco(t: &mut PainterTool) {
    let y = H as f32 / 2.0;
    t.on_canvas_pointer(cp([X0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=AMOSTRAS {
        let x = X0 + COMPRIMENTO * k as f32 / AMOSTRAS as f32;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([X0 + COMPRIMENTO, y], PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// `Σ(255 − min(rgb))` sobre a tela inteira — a régua do #47.
fn tinta(t: &PainterTool) -> u64 {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

fn nome(k: TextureKind) -> String {
    format!("{k:?}")
}

/// **A tabela do #47 inteira:** cada tipo do Shape (menos o Image, que pede píxeis) × Wet Paint e
/// Digital, o pincel de fábrica, o mesmo traço. A linha `None` do Wet Paint são as cerdas de hoje.
#[test]
#[ignore = "sonda W0 do plano 27: corre-se à mão"]
fn sonda_shape_casa_tinta_por_tipo() {
    let tipos: Vec<TextureKind> = (0..TextureKind::COUNT)
        .map(TextureKind::from_u8)
        .filter(|k| *k != TextureKind::Image)
        .collect();
    let mede = |media: PaintMedia, k: TextureKind| {
        let mut t = tela(media);
        t.set_brush_shape_kind(k.to_u8());
        traco(&mut t);
        tinta(&t)
    };
    let base_w = mede(PaintMedia::WetPaint, TextureKind::None);
    let base_d = mede(PaintMedia::Digital, TextureKind::None);
    println!("| tipo | Wet Paint | ÷ None | Digital | ÷ None |");
    println!("|---|---:|---:|---:|---:|");
    for k in tipos {
        let (w, d) = (mede(PaintMedia::WetPaint, k), mede(PaintMedia::Digital, k));
        println!(
            "| {} | {w} | {:.3} | {d} | {:.3} |",
            nome(k),
            w as f64 / base_w as f64,
            d as f64 / base_d as f64
        );
    }
}

/// As três leis da célula, sobre `stamp = env · sv · texv · I` e o portão `g = (1 − dente)·gate`:
/// `[hoje, A, B]` — A = o portão proporcional à envolvente, B = o padrão do Shape DEPOIS do portão.
fn leis(env: f64, sv: f64, texv: f64, i: f64, g: f64) -> [f64; 3] {
    let stamp = (env * sv * texv * i).min(1.0);
    let hoje = (stamp - g).max(0.0);
    let a = (stamp - g * env).max(0.0);
    let b = ((env * texv * i).min(1.0) - g).max(0.0) * sv;
    [hoje, a, b]
}

/// **As leis candidatas do #47 (D6), por preset de fábrica**, no pigmento que POUSA (sem a água): o dab
/// real do motor (a intensidade e o `dry_gate` pelo `on_dab`), o portão e o papel do preset, as cerdas
/// dele (ou o Grain, que as substitui), a silhueta e o padrão pelas portas do produto. Para cada tipo:
/// `Σ depósito` em cada lei e a razão contra o `None` do mesmo preset; para o `None`, quanto a lei A
/// muda o traço redondo (B é `× 1` exacto — o mesmo por construção).
#[test]
#[ignore = "sonda W0 do plano 27: corre-se à mão"]
fn sonda_shape_casa_leis_do_dente() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = Biblioteca::carregar(&raiz, None);
    let tipos = [
        TextureKind::None,
        TextureKind::Noise,
        TextureKind::Clouds,
        TextureKind::Voronoi,
        TextureKind::Checker,
        TextureKind::Stripes,
        TextureKind::Dots,
        TextureKind::Grain,
        TextureKind::PaperCold,
    ];
    println!(
        "| preset | tipo | Σ hoje | Σ A | Σ B | hoje ÷ None | A ÷ None | B ÷ None | px hoje | px A |"
    );
    println!("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    for e in bib
        .entradas
        .iter()
        .filter(|e| e.relativo.starts_with("WetPainter"))
    {
        let mut t = tela(PaintMedia::WetPaint);
        assert!(t.wet_preset_aplicar(&e.preset).is_empty());
        t.wet_preset_aplicar_volumes(&e.preset.parametros.volumes);
        t.paint.wetpaint.tilt_on = false;
        // O dab REAL: um traço com o gancho do motor ligado.
        let dabs: Arc<Mutex<Vec<(f64, f64)>>> = Arc::default();
        assert!(t.ensure_wet_session());
        {
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            let d2 = Arc::clone(&dabs);
            s.engine.on_dab = Some(Box::new(move |_, d| {
                d2.lock().unwrap().push((d.intensity, d.dry_gate));
            }));
        }
        traco(&mut t);
        t.wet_bring_home();
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        let tun = &s.engine.tuning;
        let gate = tun.get(Knob::PaperGate);
        let ext = if s.engine.sim.ext_bypass {
            0.0
        } else {
            tun.get(Knob::ExtDryBrush)
        };
        let cerdas = build_bristle_texture(BristleKnobs {
            felt: tun.get(Knob::Felt),
            bristle_count: tun.get(Knob::BristleCount),
            bristle_strength: tun.get(Knob::BristleStrength),
            bristle_size: tun.get(Knob::BristleSize),
        });
        let papel = s.engine.paper_tile.clone();
        let lado = (papel.len() as f64).sqrt() as usize;
        let dabs = dabs.lock().unwrap().clone();
        let (inten, dry_gate) = dabs[dabs.len() / 2];
        let brush = t.paint.brush;
        let r = brush.radius_px;
        let passo = (brush.spacing * 2.0 * r).max(1.0);
        let grain_on = brush.grain_samples(false);
        let mut linha_none = [0.0f64; 3];
        let mut px_none = [0usize; 2];
        for k in tipos {
            let mut spec = brush;
            spec.shape.kind = k;
            spec.shape.params = [0.5; texture::MAX_TEX_PARAMS];
            for (i, p) in texture::param_specs(k).iter().enumerate() {
                spec.shape.params[i] = p.default;
            }
            let mut soma = [0.0f64; 3];
            let mut cobre = [0usize; 2];
            let y = H as f32 / 2.0;
            let n = (COMPRIMENTO / passo).floor() as usize;
            let mut rng = 0u64;
            let mut vistos = vec![[false; 2]; (W * H) as usize];
            for j in 0..=n {
                let c = [X0 + j as f32 * passo, y];
                let fp = ph2d_painter_brush::FootprintDeform::new(0.0, 0);
                let b = texture::shape_basis(
                    &spec.shape,
                    &mut rng,
                    [W as f32, H as f32],
                    fp,
                    ShapeFrame::Stroke {
                        arc_len: j as f32 * passo,
                        unit_px: r,
                    },
                );
                let gb = texture::dab_basis(&spec.texture, &mut rng, [W as f32, H as f32], fp);
                let reach = r.ceil() as i64 + 1;
                for py in (c[1] as i64 - reach)..=(c[1] as i64 + reach) {
                    for px in (c[0] as i64 - reach)..=(c[0] as i64 + reach) {
                        if px < 0 || py < 0 || px >= i64::from(W) || py >= i64::from(H) {
                            continue;
                        }
                        let (sx, sy) = (px as f32 + 0.5, py as f32 + 0.5);
                        let tt = fp.falloff_t((sx - c[0]) / r, (sy - c[1]) / r);
                        let env = f64::from(spec.falloff_weight(tt));
                        if env <= 0.0 {
                            continue;
                        }
                        let sv = if k == TextureKind::None {
                            1.0
                        } else {
                            f64::from(texture::sample_shape_silhouette(
                                &spec.shape,
                                &b,
                                px,
                                py,
                                c,
                                r,
                                None,
                            ))
                        };
                        let texv = if grain_on {
                            f64::from(ph2d_painter_brush::dab::grain_at(
                                &spec, &gb, None, px, py, c, r,
                            ))
                        } else {
                            sample_bristle(&cerdas, f64::from(sx - c[0]), f64::from(sy - c[1]))
                        };
                        let tooth =
                            f64::from(papel[(py as usize % lado) * lado + px as usize % lado]);
                        let g = (1.0 - tooth) * gate + dry_gate * ext * 0.6;
                        let l = leis(env, sv, texv, inten, g);
                        for q in 0..3 {
                            soma[q] += l[q];
                        }
                        let idx = (py as u32 * W + px as u32) as usize;
                        for (q, lei) in [l[0], l[1]].into_iter().enumerate() {
                            if lei > 0.0 && !vistos[idx][q] {
                                vistos[idx][q] = true;
                                cobre[q] += 1;
                            }
                        }
                    }
                }
            }
            if k == TextureKind::None {
                linha_none = soma;
                px_none = cobre;
            }
            let rz = |q: usize| soma[q] / linha_none[q].max(1e-9);
            println!(
                "| {} | {} | {:.1} | {:.1} | {:.1} | {:.3} | {:.3} | {:.3} | {} | {} |",
                e.preset.parametros.name,
                nome(k),
                soma[0],
                soma[1],
                soma[2],
                rz(0),
                rz(1),
                rz(2),
                cobre[0],
                cobre[1]
            );
        }
        println!(
            "| {} | redondo: A ÷ hoje | {:.3} | px {:.3} | grão={grain_on} gate={gate} I={inten:.3} |||||",
            e.preset.parametros.name,
            linha_none[1] / linha_none[0].max(1e-9),
            px_none[1] as f64 / px_none[0].max(1) as f64
        );
    }
}

/// **O Spacing por meio** (dono 13/10: *«Spacing em wet paint produz traços mais uniformes que em digital
/// (que fica pontilhado)»*): o mesmo traço recto, raio 10, cada Spacing, nos quatro meios. Por traço: os
/// dabs que o MOTOR recebeu (Wet Paint, pelo `on_dab`), e o perfil da tinta no EIXO do traço (x 70–250):
/// a ondulação `(máx − mín)/máx` e a fração de vales abaixo de metade do máximo — o «pontilhado».
#[test]
#[ignore = "sonda: o Spacing por meio (dono 13/10)"]
fn sonda_spacing_por_meio() {
    println!("| meio | spacing | dabs no motor | ondulação | vales < ½ | Σ tinta |");
    println!("|---|---:|---:|---:|---:|---:|");
    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
        PaintMedia::WetPaint,
    ] {
        for sp in [0.05f32, 0.25, 0.5, 1.0, 1.5] {
            let mut t = tela(media);
            t.set_brush_size_px(20.0);
            t.set_brush_spacing(sp);
            let dabs: Arc<Mutex<usize>> = Arc::default();
            if media == PaintMedia::WetPaint {
                assert!(t.ensure_wet_session());
                let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                let d2 = Arc::clone(&dabs);
                s.engine.on_dab = Some(Box::new(move |_, _| *d2.lock().unwrap() += 1));
            }
            traco(&mut t);
            let y = (H / 2) as usize;
            let perfil: Vec<f64> = (70..250usize)
                .map(|x| {
                    let p = &t.canvas_rgba[(y * W as usize + x) * 4..][..3];
                    f64::from(255 - p[0].min(p[1]).min(p[2]))
                })
                .collect();
            let max = perfil.iter().copied().fold(0.0, f64::max).max(1.0);
            let min = perfil.iter().copied().fold(f64::MAX, f64::min);
            let vales = perfil.iter().filter(|v| **v < max / 2.0).count() as f64 / perfil.len() as f64;
            let n = *dabs.lock().unwrap();
            println!(
                "| {media:?} | {sp} | {} | {:.3} | {:.3} | {} |",
                if media == PaintMedia::WetPaint { n.to_string() } else { "—".into() },
                (max - min) / max,
                vales,
                tinta(&t)
            );
        }
    }
}

/// **O pontilhado das cerdas** (o relato do Spacing, 13/10): Bristles e a redonda × Digital e Wet Paint ×
/// Spacing, raio 20. A régua: o coeficiente de variação da tinta AO LONGO de cada linha do miolo do traço
/// (`|y − eixo| < r/2`, x 90–230), médio — riscas contínuas dão pouco, filas de pontos dão muito. Com
/// `PH2D_SONDA_OUT=<pasta>` grava cada tela em PPM.
#[test]
#[ignore = "sonda: o pontilhado das cerdas por Spacing"]
fn sonda_pontilhado_das_cerdas() {
    println!("| meio | ponta | spacing | dabs no motor | CV ao longo | Σ tinta |");
    println!("|---|---|---:|---:|---:|---:|");
    for media in [PaintMedia::Digital, PaintMedia::WetPaint] {
        for kind in [TextureKind::None, TextureKind::Bristles] {
            for sp in [0.025f32, 0.1, 0.25, 0.5] {
                let mut t = tela(media);
                t.set_brush_size_px(40.0);
                t.set_brush_spacing(sp);
                if kind == TextureKind::Bristles || media == PaintMedia::Digital {
                    t.set_brush_shape_kind(kind.to_u8());
                }
                if kind == TextureKind::Bristles {
                    // As cerdas do Wet Round nos dois meios (Strength 1), para a comparação ser a mesma ponta.
                    t.paint.brush.shape = crate::tool::paint::WetKnobs::DEFAULT.cerdas_no_shape(t.paint.brush.shape);
                }
                let dabs: Arc<Mutex<usize>> = Arc::default();
                if media == PaintMedia::WetPaint {
                    assert!(t.ensure_wet_session());
                    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                    let d2 = Arc::clone(&dabs);
                    s.engine.on_dab = Some(Box::new(move |_, _| *d2.lock().unwrap() += 1));
                }
                traco(&mut t);
                let eixo = (H / 2) as i64;
                let mut cvs = Vec::new();
                for y in (eixo - 9)..=(eixo + 9) {
                    let linha: Vec<f64> = (90..230usize)
                        .map(|x| {
                            let p = &t.canvas_rgba[(y as usize * W as usize + x) * 4..][..3];
                            f64::from(255 - p[0].min(p[1]).min(p[2]))
                        })
                        .collect();
                    let m = linha.iter().sum::<f64>() / linha.len() as f64;
                    if m > 1.0 {
                        let v = linha.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / linha.len() as f64;
                        cvs.push(v.sqrt() / m);
                    }
                }
                let cv = cvs.iter().sum::<f64>() / cvs.len().max(1) as f64;
                println!(
                    "| {media:?} | {kind:?} | {sp} | {} | {cv:.3} | {} |",
                    if media == PaintMedia::WetPaint { dabs.lock().unwrap().to_string() } else { "—".into() },
                    tinta(&t)
                );
                if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
                    for k in 0..(W * H) as usize {
                        ppm.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
                    }
                    std::fs::write(format!("{dir}/pont_{media:?}_{kind:?}_{sp}.ppm"), ppm).expect("grava");
                }
            }
        }
    }
}
