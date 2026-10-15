//! **O preço das rampas de cor** (prompt 36, Parte 2 — doc 37): o custo de um traço com cada rampa
//! desligada · em B&W · em cor, por meio e por rota, e o custo por texel de N grãos. Régua INTERCALADA
//! ([MEDIR_VELOCIDADE](../../../../../../docs/DevOps/MEDIR_VELOCIDADE.md)): as variantes no mesmo
//! processo, blocos curtos em ordem rodada, o MÍNIMO das rodadas com a mediana ao lado. Corra no perfil
//! `smoke`: `cargo test -p ph2d-tool-painter --profile smoke --lib custo_das_rampas -- --ignored --nocapture`.

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};
use std::time::Instant;

const LADO: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Rampa {
    Off,
    ShapeBw,
    ShapeCor,
    GrainBw,
    GrainCor,
}

#[derive(Clone, Copy, Debug)]
struct Variante {
    meio: &'static str,
    grao: TextureMapping,
    rampa: Rampa,
}

fn ferramenta(v: Variante) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    match v.meio {
        "Impasto" => t.set_paint_media(PaintMedia::Impasto),
        "Watercolor" => t.set_paint_media(PaintMedia::Watercolor),
        "WetPaint" => {
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
        }
        "Composite" => {
            // Duas camadas de pincel: cada uma deposita pela mesma porta de rotas do Digital.
            t.paint.composite_enabled = true;
            for pos in 0..2 {
                t.paint.composite[pos] = composite::CompositeLayer {
                    op: composite::CompositeOp::Brush,
                    strength: 1.0,
                    ..composite::CompositeLayer::default()
                };
            }
        }
        _ => {}
    }
    t.paint.brush.radius_px = 24.0;
    t.set_brush_shape_kind(TextureKind::Checker.to_u8());
    t.paint.brush.shape.size = [1.5, 1.5];
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t.set_brush_texture_mapping(v.grao.to_u8());
    // `set_brush_texture_kind` liga o B&W da Shape (a guarda de 26/06): o estado de cada variante é posto
    // DEPOIS dele.
    t.set_shape_ramp_enabled(matches!(v.rampa, Rampa::ShapeBw | Rampa::ShapeCor));
    t.paint.shape_color_ramp_bw = v.rampa == Rampa::ShapeBw;
    t.set_texture_ramp_enabled(matches!(v.rampa, Rampa::GrainBw | Rampa::GrainCor));
    t.paint.texture_ramp_bw = v.rampa == Rampa::GrainBw;
    t.paint.texture_ramp_dirty = true;
    t
}

/// Um traço horizontal de 49 movimentos, um quadro por movimento e dez depois — o relógio inteiro.
fn traco(t: &mut PainterTool) -> f64 {
    let y = LADO as f32 / 2.0;
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
    let mut x = 30.0;
    while x < LADO as f32 - 30.0 {
        x += 4.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    for _ in 0..10 {
        frame(t);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

#[test]
#[ignore = "régua do prompt 36 — perfil smoke, --ignored --nocapture"]
fn custo_das_rampas_por_meio() {
    println!(
        "loadavg início: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let meios = ["Digital", "Impasto", "Composite", "Watercolor", "WetPaint"];
    let rampas = [
        Rampa::Off,
        Rampa::ShapeBw,
        Rampa::ShapeCor,
        Rampa::GrainBw,
        Rampa::GrainCor,
    ];
    let mut vs = Vec::new();
    for meio in meios {
        for grao in [TextureMapping::Tiled, TextureMapping::ViewPlane] {
            for rampa in rampas {
                vs.push(Variante { meio, grao, rampa });
            }
        }
    }
    // Aquecimento fora da régua.
    for v in &vs {
        traco(&mut ferramenta(*v));
    }
    let (rodadas, bloco) = (7, 2);
    let mut tempos = vec![Vec::new(); vs.len()];
    for r in 0..rodadas {
        for k in 0..vs.len() {
            let i = (k + r * 7) % vs.len();
            for _ in 0..bloco {
                let mut t = ferramenta(vs[i]);
                tempos[i].push(traco(&mut t));
            }
        }
    }
    println!(
        "\n| Meio | Grain | Off | Shape B&W | Shape Cor | Grain B&W | Grain Cor |\n|---|---|---|---|---|---|---|"
    );
    for (c, chunk) in vs.chunks(rampas.len()).enumerate() {
        let linha: Vec<String> = (0..rampas.len())
            .map(|j| {
                let (mi, me) = min_med(&mut tempos[c * rampas.len() + j]);
                format!("{mi:.1} ({me:.1})")
            })
            .collect();
        println!(
            "| {} | {:?} | {} |",
            chunk[0].meio,
            chunk[0].grao,
            linha.join(" | ")
        );
    }
    println!("\n(ms por traço: mínimo (mediana) de {rodadas}×{bloco})");
    println!(
        "loadavg fim: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// O custo por texel de N grãos: N amostras do Grain multiplicadas sobre a pegada de um dab de raio 48 —
/// a conta que um núcleo por pixel com N slots faria. Mede-se a amostra, não um núcleo inventado.
#[test]
#[ignore = "régua do prompt 36 — perfil smoke, --ignored --nocapture"]
fn custo_de_n_graos_por_texel() {
    use ph2d_painter_brush::texture::{TextureSettings, dab_basis, sample};
    let r = 48.0_f32;
    let c = [128.0_f32, 128.0];
    let mk = |kind: TextureKind, mapping: TextureMapping| TextureSettings {
        kind,
        mapping,
        size: [4.0, 4.0],
        ..Default::default()
    };
    let kinds = [
        ("Noise", TextureKind::Noise),
        ("Grain", TextureKind::Grain),
        ("Voronoi", TextureKind::Voronoi),
    ];
    let mut vs = Vec::new();
    for (nome, k) in kinds {
        for m in [TextureMapping::Tiled, TextureMapping::ViewPlane] {
            for n in [0usize, 1, 2, 4] {
                vs.push((nome, m, n, mk(k, m)));
            }
        }
    }
    let um_dab = |s: &TextureSettings, n: usize| -> (f64, f32) {
        let b = dab_basis(
            s,
            &mut 0u64,
            [256.0, 256.0],
            ph2d_painter_brush::FootprintDeform::identity(),
        );
        let t0 = Instant::now();
        let mut acc = 0.0_f32;
        for py in (c[1] - r) as i64..(c[1] + r) as i64 {
            for px in (c[0] - r) as i64..(c[0] + r) as i64 {
                let (dx, dy) = (px as f32 + 0.5 - c[0], py as f32 + 0.5 - c[1]);
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let mut g = 1.0_f32;
                for j in 0..n {
                    // Cada slot com o seu deslocamento: N amostras DISTINTAS, como N grãos.
                    g *= sample(s, &b, px + 17 * j as i64, py, c, r, None);
                }
                acc += g;
            }
        }
        (t0.elapsed().as_secs_f64() * 1e6, acc)
    };
    for v in &vs {
        um_dab(&v.3, v.2);
    }
    let mut tempos = vec![Vec::new(); vs.len()];
    let mut guarda = 0.0_f32;
    for rodada in 0..9 {
        for k in 0..vs.len() {
            let i = (k + rodada * 5) % vs.len();
            for _ in 0..5 {
                let (us, a) = um_dab(&vs[i].3, vs[i].2);
                guarda += a;
                tempos[i].push(us);
            }
        }
    }
    println!("\n| Grain | Mapping | N=0 | N=1 | N=2 | N=4 |\n|---|---|---|---|---|---|");
    for (c, chunk) in vs.chunks(4).enumerate() {
        let linha: Vec<String> = (0..4)
            .map(|j| {
                let (mi, me) = min_med(&mut tempos[c * 4 + j]);
                format!("{mi:.0} ({me:.0})")
            })
            .collect();
        println!(
            "| {} | {:?} | {} |",
            chunk[0].0,
            chunk[0].1,
            linha.join(" | ")
        );
    }
    println!(
        "\n(µs por dab de raio 48 ≈ 7 238 texels: mínimo (mediana) de 9×5; guarda {guarda:.0})"
    );
}
