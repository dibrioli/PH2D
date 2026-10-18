//! ⭐ W6e — **o campo da borda é o da geometria**: a distância com sinal que o `c3_borda` grava no
//! centro de cada texel da captura (o 4.º canal, a lei da W6d) contra a distância EXATA ao polígono do
//! corpo, nos texels a menos de um texel da borda. O canto (o parcial do vértice com a sua reta
//! diagonal arredondava-o: o texel do canto convexo no ambiente, handoff W6e §1) e o sinal do cheio
//! junto às retas inclinadas de um vértice girado (a mutação M3 da W6d, que só o F3 da crate gpu via)
//! não se veem na luz de nenhum outro portão do app.
//!
//! A régua: a caixa do `o_interior_do_opaco` direita e girada [`GIRO`], nos casos do portão B (zoom 1 e
//! 2,5 × fase 0, 3, 4, 6/8), pela configuração do produto.

use ph2d_app_light2d::{
    DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela,
};
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::Camera2d;
use vello::kurbo::Point;

use crate::comum::{gpu, ler};
use crate::o_interior_do_opaco::{CENTRO, cena_girada};

const FHD: (u32, u32) = (1920, 1080);
/// O giro da caixa girada (rad): longe de 45°, as faces saem com o Sobel inclinado junto ao vértice.
pub(crate) const GIRO: f32 = 0.5;
/// A zona do CANTO: a menos disto (texels da captura) de um vértice.
const ZONA_DO_CANTO: f64 = 1.5;

/// A distância de `p` à borda do polígono `v` (metros) e se está dentro.
pub(crate) fn ao_poligono(v: &[[f32; 2]], p: [f64; 2]) -> (f64, bool) {
    let mut d = f64::INFINITY;
    let mut dentro = false;
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let (a, b) = (v[j].map(f64::from), v[i].map(f64::from));
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            dentro = !dentro;
        }
        let ab = [b[0] - a[0], b[1] - a[1]];
        let t = (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1])
            / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18))
        .clamp(0.0, 1.0);
        d = d.min((p[0] - a[0] - t * ab[0]).hypot(p[1] - a[1] - t * ab[1]));
        j = i;
    }
    (d, dentro)
}

/// O que a régua achou numa leitura: por zona (`[canto, face]`), quantos texels, o pior `campo −
/// exata` (texels da captura) e quantos de SINAL trocado a mais de ¼ texel da borda.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Campo {
    pub n: [usize; 2],
    pub pior: [f64; 2],
    pub sinal: [usize; 2],
    /// Quantos erram mais que `tol` (o argumento de [`campo_lido`]).
    pub errados: [usize; 2],
}

/// O polígono da caixa (o oclusor de 4 vértices com o centro em [`CENTRO`] + `d`).
fn caixa(c: &CenaDeLuz, d: [f32; 2]) -> Vec<[f32; 2]> {
    let alvo = [f64::from(CENTRO[0] + d[0]), f64::from(CENTRO[1] + d[1])];
    c.oclusores
        .iter()
        .find(|o| {
            let n = o.vertices.len() as f64;
            let m = o.vertices.iter().fold([0.0; 2], |s, v| {
                [s[0] + f64::from(v[0]) / n, s[1] + f64::from(v[1]) / n]
            });
            o.vertices.len() == 4 && (m[0] - alvo[0]).hypot(m[1] - alvo[1]) < 1e-3
        })
        .expect("a caixa")
        .vertices
        .clone()
}

/// O campo da borda de `s` (já com um quadro) contra a distância exata ao polígono `v`.
pub(crate) fn campo_lido(
    gpu: &ph2d_gpu::GpuContext,
    s: &Solver,
    v: &[[f32; 2]],
    tol: f64,
) -> Campo {
    let g = s.config().grade_da_captura();
    let t = s.borda_da_captura().expect("a borda exata");
    assert_eq!((t.width(), t.height()), (g.largura, g.altura));
    let bytes = ler(gpu, t, 8);
    let mut r = Campo::default();
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = g.centro(i, j);
            let (d, dentro) = ao_poligono(v, p);
            let exata = if dentro { d } else { -d } / g.texel_m;
            if exata.abs() > 1.0 {
                continue;
            }
            let k = (j * g.largura + i) as usize * 8;
            let lido = f64::from(half::f16::from_le_bytes([bytes[k + 6], bytes[k + 7]]).to_f32());
            let canto = v
                .iter()
                .any(|q| (f64::from(q[0]) - p[0]).hypot(f64::from(q[1]) - p[1]) < ZONA_DO_CANTO * g.texel_m);
            let z = usize::from(!canto);
            let e = lido - exata;
            if canto && std::env::var_os("PH2D_W6E_LISTA").is_some() {
                let a = f64::from(half::f16::from_le_bytes([bytes[k], bytes[k + 1]]).to_f32());
                eprintln!(
                    "    texel ({i}, {j}) centro ({:.4}; {:.4}) cobertura {a:.3} · campo {lido:+.3} · exata {exata:+.3}",
                    p[0], p[1]
                );
            }
            r.n[z] += 1;
            if e.abs() > r.pior[z].abs() {
                r.pior[z] = e;
            }
            r.errados[z] += usize::from(e.abs() > tol);
            r.sinal[z] += usize::from(exata.abs() > 0.25 && (lido >= 0.0) != (exata >= 0.0));
        }
    }
    r
}

/// Os casos do portão B: `(zoom, fase k, a vista, a configuração do produto, o deslocamento)`.
pub(crate) fn casos(gpu: &ph2d_gpu::GpuContext) -> Vec<(f64, u32, vello::kurbo::Affine, Config, [f32; 2])> {
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let dg = DEGRAU_PADRAO;
    let mut v = Vec::new();
    for zoom in [1.0, 2.5] {
        let a = fabrica.then_scale_about(zoom, centro);
        let grade = grade_do_quadro(a, FHD, dg).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
            rebote: false,
            ..Config::padrao_para(&gpu.device, grade)
        };
        let tx = grade.texel_m / f64::from(cfg.fina);
        for k in [0u32, 3, 4, 6] {
            let d = [
                (f64::from(k) * tx / 8.0) as f32,
                (f64::from(k) * tx * 3.0 / 8.0) as f32,
            ];
            v.push((zoom, k, a, cfg, d));
        }
    }
    v
}

/// O campo da caixa girada `rot` no caso `(a, cfg, d)`.
pub(crate) fn ler_campo(
    gpu: &ph2d_gpu::GpuContext,
    a: vello::kurbo::Affine,
    cfg: Config,
    d: [f32; 2],
    rot: f32,
    tol: f64,
) -> Campo {
    let c = cena_girada(d, rot);
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
    s.regiao_fina(i, f);
    s.quadro(gpu, &c);
    campo_lido(gpu, &s, &caixa(&c, d), tol)
}

/// ⭐ W6e — a régua do campo nas variantes a lei da W6d · a cunha (padrão e larga).
/// À mão (cabeçalho do handoff W6e, nome `medida_luz2d_w6e_campo`).
#[test]
#[ignore = "a régua do campo da W6e: corre-se à mão"]
fn medida_luz2d_w6e_campo() {
    let Some(gpu) = gpu() else { return };
    // (Antes da W6d o 4.º canal é o recuo da reta, não a distância: só a W6d em diante.)
    let variantes: [(&str, fn(Config) -> Config); 3] = [
        ("lei W6d", |c| Config {
            borda_vertice: None,
            ..c
        }),
        ("cunha", |c| Config {
            borda_vertice: Some(ph2d_light2d_gpu::VERTICE_PADRAO),
            ..c
        }),
        ("cunha larga", |c| Config {
            borda_vertice: Some(ph2d_light2d_gpu::Vertice {
                pre: 0.02,
                area: 0.05,
            }),
            ..c
        }),
    ];
    for (zoom, k, a, cfg, d) in casos(&gpu) {
        for rot in [0.0, GIRO] {
            for (nome, f) in variantes {
                let r = ler_campo(&gpu, a, f(cfg), d, rot, 0.2);
                eprintln!(
                    "CAMPO · zoom {zoom} · {k}/8 · giro {rot} · {nome} · canto {} texels pior {:+.3} sinal {} >0,2 {} · face {} pior {:+.3} sinal {} >0,2 {}",
                    r.n[0], r.pior[0], r.sinal[0], r.errados[0], r.n[1], r.pior[1], r.sinal[1], r.errados[1]
                );
            }
        }
    }
}

/// Texels de FACE da caixa girada com o campo a mais de [`TOL_CAMPO`] da distância exata, por caso — MEDIDO
/// (RTX 5060 Ti, `medida_luz2d_w6e_campo`, handoff W6e §2): no máximo 4 com a cunha (a lei da W6d 4); com
/// o sinal do cheio trocado (a M3 da W6d) 171 a 701. O teto é o dobro do medido.
const TETO_FACE: usize = 8;
const TOL_CAMPO: f64 = 0.2;

/// ⭐⭐ **GATE — o campo da borda é o da geometria** (W6e): na caixa girada [`GIRO`], nos casos do portão
/// B, nenhum texel de sinal trocado a mais de ¼ texel da borda e no máximo [`TETO_FACE`] texels de face
/// a mais de [`TOL_CAMPO`] da distância exata; o controlo (antes da W6d o 4.º canal é o recuo da reta,
/// não a distância) mostra que a régua vê um campo errado.
#[test]
fn o_campo_da_borda_e_o_da_geometria() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut falhas = Vec::new();
    let cs = casos(&gpu);
    for &(zoom, k, a, cfg, d) in &cs {
        let r = ler_campo(&gpu, a, cfg, d, GIRO, TOL_CAMPO);
        eprintln!(
            "campo · zoom {zoom} · {k}/8 · canto {} texels pior {:+.3} sinal {} · face {} pior {:+.3} sinal {} errados {}",
            r.n[0], r.pior[0], r.sinal[0], r.n[1], r.pior[1], r.sinal[1], r.errados[1]
        );
        if r.sinal != [0, 0] || r.errados[1] > TETO_FACE {
            falhas.push(format!(
                "zoom {zoom} · {k}/8: sinal {:?} · face {} errados (pior {:+.3})",
                r.sinal, r.errados[1], r.pior[1]
            ));
        }
    }
    let (_, _, a, cfg, d) = cs[1];
    let antes = ler_campo(
        &gpu,
        a,
        Config {
            borda_continua: false,
            borda_fora: false,
            ..cfg
        },
        d,
        GIRO,
        TOL_CAMPO,
    );
    assert!(
        antes.errados[1] > 10 * TETO_FACE,
        "controlo: o recuo lido como distância dá só {} texels de face errados — a régua não vê um campo errado",
        antes.errados[1]
    );
    assert!(
        falhas.is_empty(),
        "o campo da borda não é o da geometria (teto {TETO_FACE} de face a mais de {TOL_CAMPO}): {}",
        falhas.join(" · ")
    );
}
