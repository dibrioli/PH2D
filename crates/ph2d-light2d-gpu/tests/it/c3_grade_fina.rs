//! ⭐ **C3 na grade FINA** (W4b) — a luz direta das lâmpadas numa grade `fina` vezes mais fina que a
//! das cascatas: o que o RECEPTOR lê (a bilinear da fluência) contra o gabarito no centro de cada
//! texel fino; o refino pelas bordas contra a visibilidade medida em todo texel; a região; o G10.

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, Refino, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro, faisca, penumbra};
use crate::comum::{gabarito, gpu, grade, ler_bytes, ler_fluencia, rmse_rel};

const FINA: u32 = 4;

fn rodar(gpu: &ph2d_gpu::GpuContext, cfg: Config, cena: &CenaDeLuz) -> (Vec<[f32; 3]>, Grade) {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.quadro(gpu, cena);
    (ler_fluencia(gpu, &s), s.grade_da_fluencia())
}

/// A conta do `receber.wgsl`: a fluência `f` (na grade `g`) bilinear em `p`, bordas presas.
fn bilinear(f: &[[f32; 3]], g: &Grade, p: [f64; 2]) -> [f32; 3] {
    let tx = (p[0] - g.origem_m[0]) / g.texel_m - 0.5;
    let ty = (p[1] - g.origem_m[1]) / g.texel_m - 0.5;
    let (i0, j0) = (tx.floor(), ty.floor());
    let (fx, fy) = ((tx - i0) as f32, (ty - j0) as f32);
    let em = |i: f64, j: f64| {
        let i = (i.max(0.0) as u32).min(g.largura - 1);
        let j = (j.max(0.0) as u32).min(g.altura - 1);
        f[(j * g.largura + i) as usize]
    };
    let mut out = [0.0; 3];
    for (c, o) in out.iter_mut().enumerate() {
        let a = em(i0, j0)[c] * (1.0 - fx) + em(i0 + 1.0, j0)[c] * fx;
        let b = em(i0, j0 + 1.0)[c] * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0)[c] * fx;
        *o = a * (1.0 - fy) + b * fy;
    }
    out
}

/// ⭐⭐⭐ **O que o receptor lê, no texel FINO, contra o gabarito** — no degrau Baixo, as cenas com
/// lâmpadas diretas: a grade fina tem de bater o gabarito muito melhor que a grossa interpolada.
#[test]
fn a_grade_fina_e_o_gabarito_no_texel_fino() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    for (nome, cena, ganho) in [
        ("penumbra", penumbra(), TETO_GANHO[0]),
        ("faísca", faisca(), TETO_GANHO[1]),
        ("céu e vidro", ceu_e_vidro(), TETO_GANHO[2]),
    ] {
        let grossa = Config::padrao(grade(lado, texel));
        let fina = Config {
            fina: FINA,
            ..grossa
        };
        let (ff, gf) = rodar(&gpu, fina, &cena);
        let (fg, gg) = rodar(&gpu, grossa, &cena);
        let r = gabarito(&cena, &gf);
        let lida_da_grossa: Vec<[f32; 3]> = (0..gf.altura)
            .flat_map(|j| (0..gf.largura).map(move |i| (i, j)))
            .map(|(i, j)| bilinear(&fg, &gg, gf.centro(i, j)))
            .collect();
        let (e_fina, e_grossa) = (
            rmse_rel(&ff, &r, |_| true),
            rmse_rel(&lida_da_grossa, &r, |_| true),
        );
        eprintln!(
            "C3 fina · {nome}: RMSE relativo no texel fino {e_fina:.4} (a grossa interpolada {e_grossa:.4}; razão {:.3}, teto {ganho})",
            e_fina / e_grossa
        );
        assert!(
            e_fina <= ganho * e_grossa,
            "C3 fina · {nome}: {e_fina:.4} não melhora a grossa {e_grossa:.4} (teto ×{ganho})"
        );
    }
}

/// O teto da razão fina ÷ grossa por cena = o MEDIDO + 10 % (W4b, RTX 5060 Ti, `fina` ×4 no Baixo):
/// penumbra 0,0551 ÷ 0,5226 = 0,105 · faísca 0,0807 ÷ 1,8661 = 0,043 · céu e vidro 0,0275 ÷ 0,0568 =
/// 0,484 (as cascatas pesam mais nela: o céu e os vidros não são luz direta).
const TETO_GANHO: [f64; 3] = [0.116, 0.048, 0.533];

/// ⭐⭐ **O refino pelas BORDAS dá a visibilidade medida em todo texel** (`Refino::Tudo`), a menos
/// do que a interpolação das sondas deixa onde a visibilidade é suave.
#[test]
fn o_refino_pelas_bordas_e_o_exato_em_todo_texel() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    for (nome, cena) in [("penumbra", penumbra()), ("faísca", faisca())] {
        let bordas = Config {
            fina: FINA,
            ..Config::padrao(grade(lado, texel))
        };
        let tudo = Config {
            refino: Refino::Tudo,
            ..bordas
        };
        let (fb, _) = rodar(&gpu, bordas, &cena);
        let (ft, _) = rodar(&gpu, tudo, &cena);
        let r: Vec<[f64; 3]> = ft.iter().map(|v| v.map(f64::from)).collect();
        let e = rmse_rel(&fb, &r, |_| true);
        eprintln!("C3 bordas × tudo · {nome}: RMSE relativo {e:.5} (teto {TETO_BORDAS})");
        assert!(
            e <= TETO_BORDAS,
            "C3 bordas × tudo · {nome}: {e:.5} > {TETO_BORDAS}"
        );
    }
}

/// O MEDIDO + 10 % (W4b): penumbra 0,00599 · faísca 0,00302 — o que a interpolação das sondas deixa
/// no miolo da penumbra (o limiar 0,1 do `REFINO_PADRAO`).
const TETO_BORDAS: f64 = 0.0066;

/// ⭐⭐ **Só a REGIÃO se calcula, e por dentro dela é o MESMO bit** que a grade inteira (cada
/// texel fino é independente); fora dela o solver novo deixa a textura como nasceu (zeros).
#[test]
fn so_a_regiao_se_calcula_e_por_dentro_e_o_mesmo_bit() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cfg = Config {
        fina: FINA,
        ..Config::padrao(grade(lado, texel))
    };
    let cena = penumbra();
    let mut inteira = Solver::novo(&gpu, cfg).expect("solver");
    inteira.quadro(&gpu, &cena);
    let a = ler_bytes(&gpu, inteira.fluencia());
    let mut parte = Solver::novo(&gpu, cfg).expect("solver");
    let g = parte.grade_da_fluencia();
    let (ini, fim) = ([37, 101], [g.largura - 53, g.altura - 9]);
    parte.regiao_fina(ini, fim);
    parte.quadro(&gpu, &cena);
    let b = ler_bytes(&gpu, parte.fluencia());
    let px = (b.len() / (g.largura * g.altura) as usize) as u32;
    let (mut dentro, mut fora) = (0, 0);
    for j in 0..g.altura {
        for i in 0..g.largura {
            let o = ((j * g.largura + i) * px) as usize;
            let (ta, tb) = (&a[o..o + px as usize], &b[o..o + px as usize]);
            if (ini[0]..fim[0]).contains(&i) && (ini[1]..fim[1]).contains(&j) {
                assert_eq!(ta, tb, "o texel ({i}, {j}) DENTRO da região mudou");
                dentro += 1;
            } else {
                assert!(
                    tb.iter().all(|&x| x == 0),
                    "o texel ({i}, {j}) FORA da região foi calculado"
                );
                fora += 1;
            }
        }
    }
    assert!(
        a.iter().any(|&x| x != 0) && dentro > 0 && fora > 0,
        "controlo: {dentro} dentro, {fora} fora"
    );
}

/// **G10 na grade fina**: dois quadros e dois solvers dão o mesmo bit.
#[test]
fn g10_a_grade_fina_e_bit_identica() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cfg = Config {
        fina: FINA,
        ..Config::padrao(grade(lado, texel))
    };
    let cena = ceu_e_vidro();
    let mut s = Solver::novo(&gpu, cfg).expect("solver");
    s.quadro(&gpu, &cena);
    let a = ler_bytes(&gpu, s.fluencia());
    s.quadro(&gpu, &cena);
    let b = ler_bytes(&gpu, s.fluencia());
    let mut outro = Solver::novo(&gpu, cfg).expect("solver");
    outro.quadro(&gpu, &cena);
    let c = ler_bytes(&gpu, outro.fluencia());
    assert!(a.iter().any(|&x| x != 0), "a fluência não pode ser zero");
    assert!(a == b, "dois quadros da mesma cena diferem");
    assert!(a == c, "dois solvers da mesma configuração diferem");
}
