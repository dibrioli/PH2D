//! **A W6f — o texel do canto: a mistura das faces** (handoff W6e §4.1). A sonda de cor do `c3_fina`
//! (`Config::sonda`) nos texels de CANTO da caixa do `o_interior_do_opaco`, na lei antiga (`miolo: None`),
//! nas variantes antes da W6d · a lei W6d · a cunha (`VERTICE_PADRAO`), num processo. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w6f -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Por texel de canto: a luz e o gabarito; o gabarito DECOMPOSTO por face (a fração da parte do texel
//! dentro da caixa mais perto de cada face e a média da lei nela — a soma é o gabarito); e as seis
//! sondas (o `sonda_cor` do `c3_direta.wgsl`).

use ph2d_light2d_gpu::{Config, VERTICE_PADRAO};
use vello::kurbo::Point;

use crate::comum::{fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;
use crate::o_interior_do_opaco::{CENTRO, EXTERIOR, LADO, cena, gabarito_dos_cantos};

const SUB: u32 = 24;
const NOMES_DAS_FACES: [&str; 4] = ["esquerda", "direita", "baixo", "cima"];

/// O gabarito do texel `p` (lado `tx`) por face da caixa deslocada `d`: `(face, fração, média da lei)`
/// das faces com parte do texel, a mais perto do centro primeiro.
fn por_face(p: [f64; 2], tx: f64, d: [f32; 2], lei: &dyn Fn([f64; 2]) -> f64) -> Vec<(usize, f64, f64)> {
    let (cx, cy) = (f64::from(CENTRO[0] + d[0]), f64::from(CENTRO[1] + d[1]));
    let (hx, hy) = (0.5 * f64::from(LADO[0]), 0.5 * f64::from(LADO[1]));
    let dist = |q: [f64; 2]| [q[0] - (cx - hx), (cx + hx) - q[0], q[1] - (cy - hy), (cy + hy) - q[1]];
    let perto = |f: [f64; 4]| (0..4).min_by(|&a, &b| f[a].total_cmp(&f[b])).expect("4 faces");
    let mut soma = [(0u32, 0.0_f64); 4];
    let mut n = 0u32;
    for b in 0..SUB {
        for a in 0..SUB {
            let o = |k: u32| (f64::from(k) + 0.5) / f64::from(SUB) - 0.5;
            let q = [p[0] + o(a) * tx, p[1] + o(b) * tx];
            let f = dist(q);
            if f.iter().any(|&x| x <= 0.0) {
                continue;
            }
            let i = perto(f);
            soma[i].0 += 1;
            soma[i].1 += lei(q);
            n += 1;
        }
    }
    let primeira = perto(dist(p));
    let mut r: Vec<(usize, f64, f64)> = (0..4)
        .filter(|&i| soma[i].0 > 0)
        .map(|i| (i, f64::from(soma[i].0) / f64::from(n), soma[i].1 / f64::from(soma[i].0)))
        .collect();
    r.sort_by_key(|x| usize::from(x.0 != primeira));
    r
}

#[test]
#[ignore = "a sonda de cor do canto: corre-se à mão"]
fn medida_luz2d_w6f() {
    use ph2d_app_light2d::{
        DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela,
    };
    let Some(gpu) = gpu() else {
        return;
    };
    eprintln!("# w6f · loadavg antes {}", loadavg());
    let fhd = (1920u32, 1080u32);
    let fabrica = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(fhd.0, fhd.1));
    let dg = DEGRAU_PADRAO;
    let variantes: [(&str, fn(Config) -> Config); 3] = [
        ("antes da W6d", |c| Config {
            borda_continua: false,
            borda_fora: false,
            borda_vertice: None,
            ..c
        }),
        ("lei W6d", |c| Config {
            borda_vertice: None,
            ..c
        }),
        ("cunha", |c| Config {
            borda_vertice: Some(VERTICE_PADRAO),
            ..c
        }),
    ];
    let mut errados = [0usize; 3];
    for zoom in [1.0, 2.5] {
        let a = fabrica.then_scale_about(zoom, Point::new(960.0, 540.0));
        let grade = grade_do_quadro(a, fhd, dg).expect("grade");
        let base = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg
                .holografica
                .then_some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO),
            rebote: false,
            miolo: None,
            ..Config::padrao_para(&gpu.device, grade)
        };
        for k in [0u32, 3, 4, 6] {
            let tx0 = grade.texel_m / f64::from(base.fina);
            let d = [
                (f64::from(k) * tx0 / 8.0) as f32,
                (f64::from(k) * tx0 * 3.0 / 8.0) as f32,
            ];
            let c = cena(d);
            let antiga = ph2d_light2d_ref::Gabarito::com_miolo(&c, EXTERIOR);
            let lei = |p: [f64; 2]| antiga.fluencia(p)[1];
            // [variante][sonda] — a fluência lida.
            let mut lidas = Vec::new();
            for (_, f) in &variantes {
                let mut por_sonda = Vec::new();
                for sonda in 0..=6 {
                    let cfg = Config { sonda, ..f(base) };
                    let mut s = ph2d_light2d_gpu::Solver::novo(&gpu, cfg).expect("solver");
                    let (i, fi) = regiao_da_janela(s.grade_da_fluencia(), a, fhd);
                    s.regiao_fina(i, fi);
                    s.quadro(&gpu, &c);
                    por_sonda.push(fluencia_lida(&gpu, &s));
                }
                lidas.push(por_sonda);
            }
            let g = lidas[0][0].grade;
            let gab = gabarito_dos_cantos(g, d, lei);
            let max = crate::o_interior_do_opaco::gabarito_por(g, d, lei)
                .0
                .iter()
                .map(|x| x.1)
                .fold(0.0, f64::max);
            for &(p, gv) in &gab {
                let (cx, cy) = (f64::from(CENTRO[0] + d[0]), f64::from(CENTRO[1] + d[1]));
                let canto = format!(
                    "{}-{}",
                    if p[1] > cy { "cima" } else { "baixo" },
                    if p[0] > cx { "direita" } else { "esquerda" }
                );
                let faces: String = por_face(p, g.texel_m, d, &lei)
                    .iter()
                    .map(|&(i, fr, m)| format!(" {} {m:.3}×{fr:.2}", NOMES_DAS_FACES[i]))
                    .collect();
                eprintln!(
                    "CANTO · zoom {zoom} · {k}/8 · {canto} ({:.4}; {:.4}) · gabarito {gv:.3} =",
                    p[0], p[1]
                );
                eprintln!("    faces:{faces}");
                for (vi, (nome, _)) in variantes.iter().enumerate() {
                    let v = lidas[vi][0].amostra(p);
                    let e = (v - gv) / max;
                    errados[vi] += usize::from(e.abs() > 0.10);
                    let mut l = format!(
                        "    {nome:<12} luz {v:.3} (e {e:+.3}{})",
                        if e.abs() > 0.10 { " ✗" } else { "" }
                    );
                    for s in 1..=6 {
                        let x: Vec<f64> = (0..3).map(|ch| lidas[vi][s].amostra_canal(p, ch)).collect();
                        l += &format!(" · s{s} ({:+.2},{:+.2},{:+.2})", x[0], x[1], x[2]);
                    }
                    eprintln!("{l}");
                }
            }
        }
    }
    for (vi, (nome, _)) in variantes.iter().enumerate() {
        eprintln!("RESUMO W6f · {nome}: {} texels de canto errados", errados[vi]);
    }
    eprintln!("# loadavg depois {}", loadavg());
}
