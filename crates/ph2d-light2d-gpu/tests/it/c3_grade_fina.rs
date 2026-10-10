//! ⭐ **C3 na grade FINA** (W4b) — a luz direta das lâmpadas numa grade `fina` vezes mais fina que a
//! das cascatas: o que o RECEPTOR lê (a bilinear da fluência) contra o gabarito no centro de cada
//! texel fino; o refino pelas bordas contra a visibilidade medida em todo texel; a região; o G10.

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, REFINO_PADRAO, Refino, Solver};

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
        // Fora dos opacos: dentro a lei é a do preenchimento (gate próprio, abaixo).
        let dentro = crate::comum::dentro_de_opaco(&cena, &gf);
        let (e_fina, e_grossa) = (
            rmse_rel(&ff, &r, |k| !dentro(k)),
            rmse_rel(&lida_da_grossa, &r, |k| !dentro(k)),
        );
        eprintln!(
            "C3 fina · {nome}: RMSE relativo no texel fino, fora dos opacos, {e_fina:.4} (a grossa interpolada {e_grossa:.4}; razão {:.3}, teto {ganho})",
            e_fina / e_grossa,
        );
        assert!(
            e_fina <= ganho * e_grossa,
            "C3 fina · {nome}: {e_fina:.4} não melhora a grossa {e_grossa:.4} (teto ×{ganho})"
        );
    }
}

/// O teto da razão fina ÷ grossa por cena = o MEDIDO + 10 % (W4b, RTX 5060 Ti, `fina` ×4 no Baixo):
/// penumbra 0,0533 ÷ 0,5226 = 0,102 · faísca 0,0804 ÷ 1,8661 = 0,043 · céu e vidro 0,0275 ÷ 0,0568 =
/// 0,484 (as cascatas pesam mais nela: o céu e os vidros não são luz direta). ⚠️ W5a: a população é
/// a de FORA dos opacos (dentro, o gabarito deixou de ser 0 — o F3; gate próprio): penumbra 0,0446 ÷
/// 0,5207 = 0,086 com o passo junto à borda e a curvatura.
const TETO_GANHO: [f64; 3] = [0.112, 0.048, 0.533];

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

/// O MEDIDO + 10 % (W4b, o `REFINO_PADRAO`): penumbra 0,00127 · faísca 0,00026 — o que a interpolação
/// das sondas deixa no miolo da penumbra (com as sondas no passo grosso e sem o joelho: 0,0060 · 0,0030).
const TETO_BORDAS: f64 = 0.0014;

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

/// O fio de sombra de um oclusor-PONTO de 0,8 texel (centrado num texel: a captura vê-o) diante de
/// uma lâmpada pontual na diagonal: a sombra é mais estreita que o espaço entre as sondas e corre de
/// viés, então passa ENTRE elas. A fração das amostras da linha do meio dela, de 0,2 a 1,2 m atrás,
/// que a placa põe na sombra (com ÷ sem < ½).
fn fio_na_sombra(gpu: &ph2d_gpu::GpuContext, refino: Refino) -> f64 {
    let (lado, texel) = (128, 0.04);
    let cfg = Config {
        fina: FINA,
        refino,
        ..Config::padrao(grade(lado, texel))
    };
    let c = [60.5 * texel as f32, 70.5 * texel as f32];
    let m = 0.4 * texel as f32;
    let dir = [0.8_f32, 0.6];
    let lampada = ph2d_light2d_ecs::Emissor::Ponto {
        centro: [c[0] - 1.5 * dir[0], c[1] - 1.5 * dir[1]],
        intensidade: [5.0; 3],
        alcance_m: f32::INFINITY,
    };
    let com = CenaDeLuz {
        emissores: vec![lampada.clone()],
        oclusores: vec![crate::comum::oclusor(
            vec![
                [c[0] - m, c[1] - m],
                [c[0] + m, c[1] - m],
                [c[0] + m, c[1] + m],
                [c[0] - m, c[1] + m],
            ],
            1.0,
        )],
        ceu: [0.0; 3],
        ..Default::default()
    };
    let sem = CenaDeLuz {
        oclusores: vec![],
        ..com.clone()
    };
    let (fc, g) = rodar(gpu, cfg, &com);
    let (fs, _) = rodar(gpu, cfg, &sem);
    let n = 200;
    let escuras = (0..n)
        .filter(|&k| {
            let t = 0.2 + f64::from(k) / f64::from(n);
            let p = [
                f64::from(c[0]) + t * f64::from(dir[0]),
                f64::from(c[1]) + t * f64::from(dir[1]),
            ];
            bilinear(&fc, &g, p)[0] < 0.5 * bilinear(&fs, &g, p)[0]
        })
        .count();
    escuras as f64 / f64::from(n)
}

/// ⭐⭐ **O fio de sombra mais estreito que o espaço entre as sondas** sai como no exato em todo texel
/// (o refino pelas bordas não o perde). Medido (W4b): 0,515 das amostras na sombra pelas bordas e pelo
/// exato — e 0,515 também com a sonda «certificada» por uma célula de folga (por isso ela saiu).
#[test]
fn o_fio_de_sombra_entre_as_sondas_e_o_do_exato() {
    let Some(gpu) = gpu() else { return };
    let bordas = fio_na_sombra(&gpu, REFINO_PADRAO);
    let tudo = fio_na_sombra(&gpu, Refino::Tudo);
    eprintln!("fio de sombra: na sombra {bordas:.3} (exato em todo texel {tudo:.3})");
    assert!(
        tudo > 0.3,
        "controlo: o fio tem de existir na captura ({tudo:.3})"
    );
    assert!(
        (bordas - tudo).abs() <= 0.02,
        "o refino perde o fio: {bordas:.3} × {tudo:.3}"
    );
}

/// ⭐⭐⭐ **F3 — o INTERIOR de um opaco é o exterior mais próximo** (W5a; a regra do gabarito,
/// `ph2d-light2d-ref`): nos texels finos dentro dos opacos a até [`FAIXA_INTERIOR`] texels da borda,
/// a placa contra o gabarito. Sem o preenchimento o interior é 0. ⚠️ Mais fundo, no EIXO MEDIAL do
/// opaco, «a face mais próxima» troca de lado num salto (acesa ↔ sombra) e qualquer discretização
/// erra por inteiro num fio de texels (medido: os 6 piores da penumbra todos na diagonal, placa ~0,27
/// × gabarito 0) — fica fora da régua. ⚠️ Só luz DIRETA: com céu (a «céu e vidro») o EXTERIOR junto à
/// parede já sai baixo pelas cascatas (0,17 → 0,01 contra 0,25 em ~2 texels grossos, sem o
/// preenchimento — ABERTO da W5a, território das cascatas/HRC) e o interior copia-o fielmente.
#[test]
fn o_interior_de_um_opaco_e_o_exterior_mais_proximo() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    for (nome, cena, teto) in [
        ("penumbra", penumbra(), TETO_INTERIOR[0]),
        ("faísca", faisca(), TETO_INTERIOR[1]),
    ] {
        // A captura do PRODUTO: duas vezes mais fina que a grade fina (a grossa arredonda o canto em
        // ~1 texel fino e troca a face mais próxima junto a ele).
        let com = Config {
            fina: FINA,
            captura: 2 * FINA,
            ..Config::padrao(grade(lado, texel))
        };
        let sem = Config {
            preenchimento: None,
            ..com
        };
        let (fc, g) = rodar(&gpu, com, &cena);
        let (fs, _) = rodar(&gpu, sem, &cena);
        let r = gabarito(&cena, &g);
        let dentro = crate::comum::dentro_de_opaco_a_menos_de(&cena, &g, FAIXA_INTERIOR);
        let (ec, es) = (rmse_rel(&fc, &r, &dentro), rmse_rel(&fs, &r, &dentro));
        let mut piores: Vec<(f64, usize)> = (0..r.len())
            .filter(|&k| dentro(k))
            .map(|k| ((f64::from(fc[k][1]) - r[k][1]).abs(), k))
            .collect();
        piores.sort_by(|a, b| b.0.total_cmp(&a.0));
        let w = g.largura as usize;
        for &(d, k) in piores.iter().take(6) {
            eprintln!(
                "  pior ({}, {}): placa {:.4} · gabarito {:.4} · |Δ| {d:.4} · de {} texels de dentro",
                k % w,
                k / w,
                fc[k][1],
                r[k][1],
                piores.len()
            );
        }
        eprintln!(
            "F3 interior · {nome}: RMSE relativo {ec:.4} (sem o preenchimento {es:.4}; teto {teto})"
        );
        assert!(
            es > 0.9,
            "controlo: sem o preenchimento o interior é 0 ({es:.4})"
        );
        assert!(ec <= teto, "F3 interior · {nome}: {ec:.4} > {teto}");
    }
}

/// Texels finos de dentro, a partir da borda, que a régua mede.
const FAIXA_INTERIOR: f64 = 2.0;
/// O MEDIDO + 10 % (W5a, captura 2 × fina, a faixa de 2 texels): penumbra 0,3135 · faísca 0,0032;
/// sem o preenchimento 1,67 · 2,00. Com a captura GROSSA a penumbra dava 0,80 (o canto arredondado
/// troca a face mais próxima); escolhendo o texel exterior mais perto em vez do ponto de FRONTEIRA
/// mais perto, 0,83; só com as sementes do JFA de células de 4, 1,15.
const TETO_INTERIOR: [f64; 2] = [0.345, 0.0036];
