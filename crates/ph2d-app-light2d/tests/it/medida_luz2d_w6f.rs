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

use ph2d_light2d_gpu::{Canto, Config, VERTICE_PADRAO};
use vello::kurbo::Point;

use crate::comum::{fluencia_lida, gpu, ler};
use crate::medida_luz2d_w4b::loadavg;
use crate::o_interior_do_opaco::{CENTRO, EXTERIOR, LADO, cena, gabarito_dos_cantos};

const SUB: u32 = 24;
const NOMES_DAS_FACES: [&str; 4] = ["esquerda", "direita", "baixo", "cima"];

/// O gabarito do texel `p` (lado `tx`) por face da caixa deslocada `d`: `(face, fração, média da lei)`
/// das faces com parte do texel, a mais perto do centro primeiro.
fn por_face(
    p: [f64; 2],
    tx: f64,
    d: [f32; 2],
    lei: &dyn Fn([f64; 2]) -> f64,
) -> Vec<(usize, f64, f64)> {
    let (cx, cy) = (f64::from(CENTRO[0] + d[0]), f64::from(CENTRO[1] + d[1]));
    let (hx, hy) = (0.5 * f64::from(LADO[0]), 0.5 * f64::from(LADO[1]));
    let dist = |q: [f64; 2]| {
        [
            q[0] - (cx - hx),
            (cx + hx) - q[0],
            q[1] - (cy - hy),
            (cy + hy) - q[1],
        ]
    };
    let perto = |f: [f64; 4]| {
        (0..4)
            .min_by(|&a, &b| f[a].total_cmp(&f[b]))
            .expect("4 faces")
    };
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
        .map(|i| {
            (
                i,
                f64::from(soma[i].0) / f64::from(n),
                soma[i].1 / f64::from(soma[i].0),
            )
        })
        .collect();
    r.sort_by_key(|x| usize::from(x.0 != primeira));
    r
}

/// O campo lido: a borda (o 4.º canal, a distância com sinal em texels da captura), as cunhas espalhadas
/// (com `borda_cantos`) e a grade da captura.
type Campo = (Vec<u8>, Option<Vec<u8>>, ph2d_light2d_gpu::Grade);

/// `p` dentro pelo campo — a réplica do `meio` do `c2_rc.wgsl`: a bilinear dos centros e, com as cunhas,
/// junto ao vértice (|d| < 1, cunha no texel) as duas retas sem interpolar.
fn dentro_pelo_campo((campo, cantos, gc): &Campo, p: [f64; 2]) -> bool {
    let h = |b: &[u8], k: usize| f64::from(half::f16::from_le_bytes([b[k], b[k + 1]]).to_f32());
    let px = (p[0] - gc.origem_m[0]) / gc.texel_m;
    let py = (p[1] - gc.origem_m[1]) / gc.texel_m;
    let (x, y) = (px - 0.5, py - 0.5);
    let (i0, j0) = (x.floor(), y.floor());
    let (fx, fy) = (x - i0, y - j0);
    let texel = |i: f64, j: f64| {
        let i = (i.max(0.0) as u32).min(gc.largura - 1);
        let j = (j.max(0.0) as u32).min(gc.altura - 1);
        (j * gc.largura + i) as usize * 8
    };
    let em = |i: f64, j: f64| h(campo, texel(i, j) + 6);
    let a0 = em(i0, j0) * (1.0 - fx) + em(i0 + 1.0, j0) * fx;
    let a1 = em(i0, j0 + 1.0) * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0) * fx;
    let d = a0 * (1.0 - fy) + a1 * fy;
    if let Some(w) = cantos
        && d.abs() < 1.0
    {
        let k = texel(px.floor(), py.floor());
        let v = [h(w, k), h(w, k + 2), h(w, k + 4), h(w, k + 6)];
        // ⭐ W6g: o bloco 8×8 sem vértice à volta só tem a marca na origem (`bloco_valido`).
        let ci = px.floor().clamp(0.0, f64::from(gc.largura - 1));
        let cj = py.floor().clamp(0.0, f64::from(gc.altura - 1));
        let valido = h(w, texel(ci - ci % 8.0, cj - cj % 8.0)) < 150.0;
        if v[0] < 50.0 && valido {
            let o = [px - px.floor() - 0.5, py - py.floor() - 0.5];
            let concavo = v[2] > 5.0;
            let t2 = if concavo { v[2] - 10.0 } else { v[2] };
            let a = o[0] * v[0].cos() + o[1] * v[0].sin() - v[1];
            let b = o[0] * t2.cos() + o[1] * t2.sin() - v[3];
            return if concavo { a.max(b) } else { a.min(b) } >= 0.0;
        }
    }
    d >= 0.0
}

/// ⭐ W6f — o modelo (handoff W6f §3): o ARREDONDADO que o campo bilinear tem em cada vértice convexo. Por
/// vértice e face, o iso-0 do campo ao longo da normal (texels finos `h`, `+` = para FORA da face
/// verdadeira) a 0 · ¼ · ½ · 1 · 2 texels do vértice; o arredondado é o desvio junto ao vértice menos o de
/// longe.
fn arredondado(
    zoom: f64,
    k: u32,
    d: [f32; 2],
    h: f64,
    variantes: &[(&str, fn(Config) -> Config)],
    campos: &[Campo],
) {
    let (cx, cy) = (f64::from(CENTRO[0] + d[0]), f64::from(CENTRO[1] + d[1]));
    let (hx, hy) = (0.5 * f64::from(LADO[0]), 0.5 * f64::from(LADO[1]));
    // A cunha e a 10 (o campo não depende da `Canto`).
    for vi in [2usize, 4] {
        let campo = &campos[vi];
        for (sx, sy) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            let v = [cx + sx * hx, cy + sy * hy];
            // O iso-0 ao longo da normal `n` pelo ponto `q` da face: bissecção em [−1, +1] texel.
            let iso = |q: [f64; 2], n: [f64; 2]| {
                let em =
                    |t: f64| dentro_pelo_campo(campo, [q[0] + n[0] * t * h, q[1] + n[1] * t * h]);
                let (mut a, mut b) = (-1.0, 1.0);
                if !em(a) || em(b) {
                    return f64::NAN;
                }
                for _ in 0..30 {
                    let m = 0.5 * (a + b);
                    if em(m) {
                        a = m;
                    } else {
                        b = m;
                    }
                }
                0.5 * (a + b)
            };
            let perfil = |ponto: &dyn Fn(f64) -> [f64; 2], n: [f64; 2]| {
                [0.0, 0.25, 0.5, 1.0, 2.0]
                    .iter()
                    .map(|&s| format!("{:+.2}", iso(ponto(s * h), n)))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            eprintln!(
                "ARREDONDADO · zoom {zoom} · {k}/8 · {} · {}-{} · vertical [{}] · horizontal [{}]",
                variantes[vi].0,
                if sy > 0.0 { "cima" } else { "baixo" },
                if sx > 0.0 { "direita" } else { "esquerda" },
                perfil(&|s| [v[0], v[1] - sy * s], [sx, 0.0]),
                perfil(&|s| [v[0] - sx * s, v[1]], [0.0, sy]),
            );
        }
    }
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
    // A 9, a 9′ e a 10 sem a 9 foram medidas e recusadas (handoff W6f §3, §9); W6g: a 10 + 9 leve, sem e
    // com a normal de um lado.
    let variantes: [(&str, fn(Config) -> Config); 5] = [
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
        ("cunha + 10 + 9", |c| Config {
            borda_vertice: Some(VERTICE_PADRAO),
            borda_cantos: true,
            canto: Canto {
                um_lado: false,
                ..Canto::LIGADO
            },
            ..c
        }),
        ("10 + 9 um lado", |c| Config {
            borda_vertice: Some(VERTICE_PADRAO),
            borda_cantos: true,
            canto: Canto::LIGADO,
            ..c
        }),
    ];
    let mut errados = [0usize; 5];
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
            let mut campos = Vec::new();
            for (_, f) in &variantes {
                let mut por_sonda = Vec::new();
                for sonda in 0..=6 {
                    let cfg = Config { sonda, ..f(base) };
                    let mut s = ph2d_light2d_gpu::Solver::novo(&gpu, cfg).expect("solver");
                    let (i, fi) = regiao_da_janela(s.grade_da_fluencia(), a, fhd);
                    s.regiao_fina(i, fi);
                    s.quadro(&gpu, &c);
                    por_sonda.push(fluencia_lida(&gpu, &s));
                    if sonda == 0 {
                        let t = s.borda_da_captura().expect("o campo da borda");
                        let w = s.cantos_da_captura().map(|w| ler(&gpu, w, 8));
                        campos.push((ler(&gpu, t, 8), w, cfg.grade_da_captura()));
                    }
                }
                lidas.push(por_sonda);
            }
            let g = lidas[0][0].grade;
            arredondado(zoom, k, d, g.texel_m, &variantes, &campos);
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
                        let x: Vec<f64> =
                            (0..3).map(|ch| lidas[vi][s].amostra_canal(p, ch)).collect();
                        l += &format!(" · s{s} ({:+.2},{:+.2},{:+.2})", x[0], x[1], x[2]);
                    }
                    eprintln!("{l}");
                }
            }
        }
    }
    for (vi, (nome, _)) in variantes.iter().enumerate() {
        eprintln!(
            "RESUMO W6f · {nome}: {} texels de canto errados",
            errados[vi]
        );
    }
    eprintln!("# loadavg depois {}", loadavg());
}
