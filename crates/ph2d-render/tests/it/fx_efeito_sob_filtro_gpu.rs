//! ⭐ **Filtros × efeitos, par a par, NA IMAGEM** (F33, achado do dono no smoke de 26/10: Bevel +
//! Flag = vincos). Plano e medição: `docs/UI_New_and_Simple/F33_plano.md`.
//!
//! Um filtro não sabe que há efeito: ele vê a curva COZIDA. Aqui cada efeito do catálogo cozinha
//! a forma, o Vello a rasteriza (a porta do produto) e o `FxStackPass` corre cada filtro com a
//! silhueta exata em segmentos (o que o `silhouette_segments` entrega no app). A régua é o
//! **VINCO**: o salto de luminância entre dois píxeis vizinhos cujos pés na curva estão colados
//! (a mesma «folha» do campo — atravessar o eixo medial de uma reentrância não conta). Numa forma
//! LISA o vinco só pode vir de uma quebra que o efeito pôs na curva.
//!
//! ⚠️ **É uma SONDA, não um gate** (medido 26/10): a contagem de vincos NÃO separa o antes do depois
//! da cura F33 — o laço de um vale é um gancho escuro, não um salto entre isolinhas, e uma quina
//! preservada vinca por direito (a cura até sobe a contagem). O oráculo do defeito é a FOTO
//! (`docs/UI_New_and_Simple/handoffs/img_F33/`); os gates são da geometria
//! (`ph2d-vec-scene/src/effect_geometry_tests.rs`).
//!
//! ```text
//! PH2D_FX_LOOK_DIR=/tmp/f33 cargo test -p ph2d-render --profile smoke --test it \
//!     fx_efeito_sob_filtro -- --ignored --nocapture
//! ```

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};
use ph2d_vec_scene::effect::{FxEntry, PathEffect, run_stack};
use ph2d_vec_scene::{VecPath, VecVertex};

use crate::fx_stack_common::{op, readback, try_headless_gpu};

const W: u32 = 360;
const H: u32 = 300;
/// O lado maior da forma FONTE, em texels — o resto é margem para sombra e brilho.
const LADO: f64 = 200.0;
/// A tolerância do achatamento — a MESMA do `silhouette.rs` (`FLATTEN_TOL`).
const TOL: f64 = 0.02;
/// Dois vizinhos cujas distâncias à curva diferem menos do que isto (texels) estão na mesma
/// isolinha do filtro — o sombreado ao longo dela só muda com a direção da curva.
const MESMA_DISTANCIA: f64 = 0.15;
/// Dois pés mais próximos do que isto (texels) são a mesma folha do campo.
const MESMA_FOLHA: f64 = 1.5;
/// Um salto acima disto (níveis de 0..255) é um vinco. MEDIDO: a bolha sem efeito dá ≤ 5 em todos
/// os filtros (a rampa de AA e a quantização), e o vinco da foto do dono dá dezenas.
const VINCO: f64 = 12.0;

/// Uma bolha LISA com âncoras em arcos irregulares — o caso em que amostras da grade e âncoras
/// caem quase coladas, e onde qualquer vinco é defeito (não há quina na fonte).
fn bolha() -> VecPath {
    let pts: [(f64, f64); 7] = [
        (0.0, 1.0),
        (0.37, 0.8),
        (1.1, 0.95),
        (2.03, 0.7),
        (3.0, 1.05),
        (4.4, 0.85),
        (5.3, 1.0),
    ];
    let p = |(a, r): (f64, f64)| [r * a.cos(), 0.75 * r * a.sin()];
    let n = pts.len();
    let verts = (0..n)
        .map(|i| {
            let (a, b, c) = (p(pts[(i + n - 1) % n]), p(pts[i]), p(pts[(i + 1) % n]));
            let t = [(c[0] - a[0]) / 6.0, (c[1] - a[1]) / 6.0];
            VecVertex::smooth(b, [b[0] - t[0], b[1] - t[1]], [b[0] + t[0], b[1] + t[1]])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// ⭐ A Nuvem DO APP (`ShapeKind::Cloud`, valores de fábrica) — a forma da foto do dono, com os vales
/// em quina (que vincam por direito: o gate é a bolha, a nuvem é medida e fotografada).
fn nuvem() -> VecPath {
    let k = ph2d_vec_scene::ShapeKind::Cloud;
    ph2d_vec_scene::cook(k, [0.0, 0.0], [120.0, 80.0], &k.defaults())
}

/// Os parâmetros típicos de cada efeito — no Flag, os da foto do dono.
fn ajustado(kind: usize) -> Option<PathEffect> {
    let mut e = PathEffect::from_kind(kind)?;
    let nomes: Vec<&str> = e.params().iter().map(|p| p.name).collect();
    for (i, n) in nomes.iter().enumerate() {
        let v = match *n {
            "Size" => 5.0,
            "Ridges" => 10.0,
            "Smooth" => 1.0,
            "Copies X" => 2.0,
            "Move X" => 15.0,
            "Amount" => 40.0,
            "Bend" => 17.45,
            "Horizontal" => 12.21,
            "Vertical" => -11.66,
            "Angle" => 90.0,
            "Gap" => 5.0,
            "Passes" => 1.0,
            "Roughness" => 3.0,
            "Detail" => 8.0,
            "Seed" => 3.0,
            _ => continue,
        };
        e.set(i, v);
    }
    Some(e)
}

/// Os contornos FECHADOS em polilinhas de texel (só eles têm interior — a lei do `build_fill`).
fn polilinhas(p: &VecPath, xf: impl Fn([f64; 2]) -> [f64; 2]) -> Vec<Vec<[f64; 2]>> {
    let contornos = std::iter::once((&p.verts, p.closed))
        .chain(p.subpaths.iter().map(|c| (&c.verts, c.closed)));
    let mut out = Vec::new();
    for (v, closed) in contornos {
        if !closed || v.len() < 2 {
            continue;
        }
        let mut poly = vec![xf(v[0].anchor)];
        for i in 0..v.len() {
            let (a, b) = (&v[i], &v[(i + 1) % v.len()]);
            let c = [xf(a.anchor), xf(a.out_handle), xf(b.in_handle), xf(b.anchor)];
            let hull = (1..4)
                .map(|k| (c[k][0] - c[k - 1][0]).hypot(c[k][1] - c[k - 1][1]))
                .sum::<f64>();
            let n = ((hull / (8.0 * TOL)).sqrt().ceil() as usize).clamp(1, 256);
            for s in 1..=n {
                let t = s as f64 / n as f64;
                let u = 1.0 - t;
                let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                poly.push(std::array::from_fn(|k| {
                    b0 * c[0][k] + b1 * c[1][k] + b2 * c[2][k] + b3 * c[3][k]
                }));
            }
        }
        out.push(poly);
    }
    out
}

fn segmentos(polys: &[Vec<[f64; 2]>]) -> Vec<[f32; 4]> {
    polys
        .iter()
        .flat_map(|p| p.windows(2))
        .filter(|w| w[0] != w[1])
        .map(|w| [w[0][0] as f32, w[0][1] as f32, w[1][0] as f32, w[1][1] as f32])
        .collect()
}

/// Rasteriza pelo Vello (NonZero, a porta do produto) — a fonte que o `FxStackPass` lê.
fn rasteriza(gpu: &ph2d_gpu::GpuContext, polys: &[Vec<[f64; 2]>]) -> ph2d_render::VelloPass {
    let mut pass = ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H))
        .expect("scratch");
    let mut bp = vello::kurbo::BezPath::new();
    for p in polys {
        bp.move_to((p[0][0], p[0][1]));
        for q in &p[1..] {
            bp.line_to((q[0], q[1]));
        }
        bp.close_path();
    }
    let mut cena = vello::Scene::new();
    cena.fill(
        vello::peniko::Fill::NonZero,
        vello::kurbo::Affine::IDENTITY,
        vello::peniko::Color::from_rgba8(120, 170, 100, 255),
        None,
        &bp,
    );
    pass.render_to_intermediate(gpu, &cena, (W, H), vello::peniko::Color::TRANSPARENT)
        .expect("render");
    pass
}

/// O pé de cada texel na curva (o ponto mais próximo) e a distância, por grelha de baldes.
fn pes(segs: &[[f32; 4]]) -> Vec<([f64; 2], f64)> {
    const C: f64 = 8.0;
    let (gw, gh) = ((f64::from(W) / C).ceil() as usize, (f64::from(H) / C).ceil() as usize);
    let mut baldes: Vec<Vec<usize>> = vec![Vec::new(); gw * gh];
    for (i, s) in segs.iter().enumerate() {
        let (x0, x1) = (f64::from(s[0].min(s[2])), f64::from(s[0].max(s[2])));
        let (y0, y1) = (f64::from(s[1].min(s[3])), f64::from(s[1].max(s[3])));
        let cx = |x: f64| ((x / C).floor().max(0.0) as usize).min(gw - 1);
        let cy = |y: f64| ((y / C).floor().max(0.0) as usize).min(gh - 1);
        for gy in cy(y0)..=cy(y1) {
            for gx in cx(x0)..=cx(x1) {
                baldes[gy * gw + gx].push(i);
            }
        }
    }
    let alcance = 3_usize; // 24 texels: a banda mais larga que a régua lê.
    let mut out = vec![([0.0; 2], f64::MAX); (W * H) as usize];
    for y in 0..H as usize {
        for x in 0..W as usize {
            let p = [x as f64 + 0.5, y as f64 + 0.5];
            let (gx, gy) = (x / C as usize, y / C as usize);
            let mut melhor = ([0.0; 2], f64::MAX);
            for by in gy.saturating_sub(alcance)..(gy + alcance + 1).min(gh) {
                for bx in gx.saturating_sub(alcance)..(gx + alcance + 1).min(gw) {
                    for &i in &baldes[by * gw + bx] {
                        let s = segs[i];
                        let a = [f64::from(s[0]), f64::from(s[1])];
                        let d = [f64::from(s[2]) - a[0], f64::from(s[3]) - a[1]];
                        let l2 = d[0] * d[0] + d[1] * d[1];
                        let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0);
                        let q = [a[0] + t * d[0], a[1] + t * d[1]];
                        let dist = (p[0] - q[0]).hypot(p[1] - q[1]);
                        if dist < melhor.1 {
                            melhor = (q, dist);
                        }
                    }
                }
            }
            out[y * W as usize + x] = melhor;
        }
    }
    out
}

/// Um filtro da pilha, com o LADO da curva que ele pinta (`true` = dentro).
struct Filtro {
    nome: &'static str,
    op: FxOpGpu,
    dentro: bool,
}

fn filtros() -> Vec<Filtro> {
    let preto = [0.0, 0.0, 0.0, 1.0];
    let branco = [1.0, 1.0, 1.0, 1.0];
    let com = |mut o: FxOpGpu, off: [f32; 2]| {
        o.offset_px = off;
        if !FxOp::spec(o.kind).modes.is_empty() {
            o.mode = FxOp::new(o.kind).mode;
        }
        o
    };
    vec![
        Filtro { nome: "Bevel", op: com(op(FxOp::BEVEL, 12.0, preto), [-8.0, 8.0]), dentro: true },
        Filtro { nome: "Inner Shadow", op: com(op(FxOp::INNER_SHADOW, 8.0, preto), [4.0, 4.0]), dentro: true },
        Filtro { nome: "Inner Glow", op: com(op(FxOp::INNER_GLOW, 8.0, branco), [0.0, 0.0]), dentro: true },
        Filtro { nome: "Feather", op: com(op(FxOp::FEATHER, 8.0, preto), [0.0, 0.0]), dentro: true },
        Filtro { nome: "Color Overlay", op: com(op(FxOp::COLOR_OVERLAY, 0.0, branco), [0.0, 0.0]), dentro: true },
        Filtro { nome: "Outer Glow", op: com(op(FxOp::GLOW, 10.0, branco), [0.0, 0.0]), dentro: false },
        Filtro { nome: "Drop Shadow", op: com(op(FxOp::DROP_SHADOW, 6.0, preto), [0.0, 0.0]), dentro: false },
        Filtro { nome: "Outline", op: com(op(FxOp::OUTLINE, 6.0, preto), [0.0, 0.0]), dentro: false },
        Filtro { nome: "Blur", op: com(op(FxOp::BLUR, 4.0, preto), [0.0, 0.0]), dentro: false },
    ]
}

/// `(vincos, maior salto)` — pares de vizinhos na banda do filtro, na mesma folha do campo.
fn vincos(px: &[u8], cob: &[u8], pe: &[([f64; 2], f64)], dentro: bool, banda: f64) -> (usize, f64) {
    let lum = |i: usize| {
        let o = i * 4;
        let a = f64::from(px[o + 3]) / 255.0;
        a * (0.299 * f64::from(px[o]) + 0.587 * f64::from(px[o + 1]) + 0.114 * f64::from(px[o + 2]))
    };
    let na_banda = |i: usize| {
        let lado = (cob[i * 4 + 3] > 127) == dentro;
        lado && pe[i].1 > 1.5 && pe[i].1 < banda
    };
    let (mut n, mut m) = (0, 0.0_f64);
    for y in 0..H as usize - 1 {
        for x in 0..W as usize - 1 {
            let i = y * W as usize + x;
            if !na_banda(i) {
                continue;
            }
            for j in [i + 1, i + W as usize] {
                if !na_banda(j) {
                    continue;
                }
                let (a, b) = (pe[i].0, pe[j].0);
                // A MESMA distância (a rampa do filtro é ao longo da normal, e muda por direito) e
                // a mesma folha (atravessar o eixo medial de um vale não conta).
                if (pe[i].1 - pe[j].1).abs() > MESMA_DISTANCIA
                    || (a[0] - b[0]).hypot(a[1] - b[1]) > MESMA_FOLHA
                {
                    continue;
                }
                let s = (lum(i) - lum(j)).abs();
                m = m.max(s);
                n += usize::from(s > VINCO);
            }
        }
    }
    (n, m)
}

/// Os efeitos cujo VINCO é o desenho: as pétalas com bico do Pucker & Bloat (a lei do Illustrator)
/// e a interseção das cópias do Repeater (uma reentrância nova). Medidos e fotografados, sem gate.
fn por_desenho(pilha: &[PathEffect]) -> bool {
    pilha
        .iter()
        .any(|e| matches!(e, PathEffect::Bloat(_) | PathEffect::Repeat(_)))
}

fn ppm(dir: &str, nome: &str, px: &[u8]) {
    let bg = [0x2c_u8, 0x2e, 0x33];
    let mut body = Vec::with_capacity((W * H * 3) as usize);
    for i in 0..(W * H) as usize {
        let a = f32::from(px[i * 4 + 3]) / 255.0;
        for c in 0..3 {
            body.push((a * f32::from(px[i * 4 + c]) + (1.0 - a) * f32::from(bg[c])) as u8);
        }
    }
    let mut f = std::fs::File::create(format!("{dir}/{nome}.ppm")).expect("ppm");
    use std::io::Write;
    write!(f, "P6\n{W} {H}\n255\n").expect("cabeçalho");
    f.write_all(&body).expect("corpo");
}

/// A tabela par a par (todo o catálogo × os nove filtros que leem a forma) e a folha do Bevel.
#[test]
#[ignore = "sonda de placa; roda com --ignored (PH2D_FX_LOOK_DIR=<dir> grava as fotos)"]
fn fx_efeito_sob_filtro_sonda() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[f33] sem adapter — skip");
        return;
    };
    let dir = std::env::var("PH2D_FX_LOOK_DIR").ok();
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("dir");
    }
    let mut pass = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, W, H);
    let mut efeitos: Vec<(String, Vec<PathEffect>)> = vec![("Nenhum".into(), Vec::new())];
    for k in 0..PathEffect::KINDS.len() {
        let Some(e) = ajustado(k) else { continue };
        if !matches!(e, PathEffect::Falloff(_)) {
            efeitos.push((PathEffect::KINDS[k].into(), vec![e]));
        }
    }
    let mut falhas = Vec::new();
    // A «zoom» é a escala da foto do dono: a nuvem com ~1 600 px de largo, a janela num vale e na
    // borda direita. A 200 px um laço ou uma quina comida medem menos de um texel e não se veem.
    for (forma_nome, forma, gate, zoom) in [
        ("bolha", bolha(), true, None),
        ("nuvem", nuvem(), false, None),
        ("zoom", nuvem(), true, Some(([92.0, 66.0], 8.0))),
    ] {
        // O enquadramento é o da FONTE: o mesmo para todos os efeitos.
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in &forma.verts {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        let mut esc = LADO / (hi[0] - lo[0]).max(hi[1] - lo[1]);
        let mut centro = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        if let Some((c, k)) = zoom {
            (centro, esc) = (c, esc * k);
        }
        let xf = |p: [f64; 2]| {
            [
                (p[0] - centro[0]) * esc + f64::from(W) / 2.0,
                (p[1] - centro[1]) * esc + f64::from(H) / 2.0,
            ]
        };
        let mut base: Vec<f64> = Vec::new();
        for (efeito_nome, pilha) in &efeitos {
            let entradas: Vec<FxEntry> = pilha.iter().cloned().map(FxEntry::new).collect();
            let cozida = run_stack(&forma, &entradas).unwrap_or_else(|| forma.clone());
            let polys = polilinhas(&cozida, xf);
            if polys.is_empty() {
                eprintln!("[f33] {forma_nome:6} {efeito_nome:16} sem área (contornos abertos)");
                continue;
            }
            let segs = segmentos(&polys);
            let fonte = rasteriza(&gpu, &polys);
            let cob = readback(&gpu, fonte.intermediate_texture(), W, H);
            let pe = pes(&segs);
            let mut linha = format!("[f33] {forma_nome:6} {efeito_nome:16}");
            for (fi, f) in filtros().iter().enumerate() {
                pass.run(&gpu, fonte.intermediate_texture(), &dst, W, H, &[f.op.clone()], &segs);
                let px = readback(&gpu, &dst, W, H);
                let banda = f64::from(f.op.sigma_px.max(4.0)) + 2.0;
                let (n, m) = vincos(&px, &cob, &pe, f.dentro, banda);
                linha += &format!(" {}={n}/{m:.0}", f.nome);
                if efeito_nome == "Nenhum" {
                    base.push(m);
                } else if gate && n > 0 && !por_desenho(pilha) {
                    falhas.push(format!("{forma_nome}/{efeito_nome}/{}: {n} vincos (salto {m:.0}, sem efeito {:.0})", f.nome, base[fi]));
                }
                if let Some(d) = &dir {
                    if f.nome == "Bevel" {
                        ppm(d, &format!("{forma_nome}_{}", efeito_nome.replace([' ', '&'], "_")), &px);
                    }
                }
            }
            eprintln!("{linha}");
        }
    }
    if !falhas.is_empty() {
        eprintln!("[f33] pares acima do controlo: {falhas:#?}");
    }
}
