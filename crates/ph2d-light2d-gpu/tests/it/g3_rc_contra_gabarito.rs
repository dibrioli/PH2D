//! **G3** — a fluência do RC contra o gabarito (`ph2d-light2d-ref::fluencia`), por degrau, no
//! centro de cada texel da grade. Erro = `√(média (gpu − ref)²) / média(ref)` nos três canais.
//!
//! Cenas (`cenas.rs`): a réplica da HRC Fig. 8, a faísca (luz de 2 px) e «céu e vidro» (céu,
//! vidros sobrepostos de α 0,5, parede opaca, luz de alcance finito). O ANEL: na região onde a luz
//! se vê sem obstáculo, o RC sem *bilinear fix* tem de reprovar e com ele tem de passar.

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Fusao, Precisao, Solver, o_que_fica_fora};

use crate::cenas::{DEGRAUS, ceu_e_vidro, faisca, penumbra, regiao_livre_longe};
use crate::comum::{gabarito, gpu, grade, ler_fluencia, rmse_rel, tem_f16};

// Tolerância = medido + 10 % (o RC é determinístico — G10 —; a folga cobre outro driver, não uma
// regressão de algoritmo). Medido na RTX 5060 Ti/Vulkan com `Config::padrao`, W2 (handoff):
//
// | cena            | Alto   | Médio  | Baixo  |
// |-----------------|--------|--------|--------|
// | penumbra        | 0,0902 | 0,1393 | 0,2333 |
// | céu e vidro     | 0,0126 | 0,0198 | 0,0374 |
// | faísca          | 0,2821 | fora (C3) | fora (C3) |
// | anel (livre)    | bilinear 0,0395 · clássica 0,1262 |
//
// O piso da CAPTURA (força bruta pela mesma grade, `diagnostico.rs`) na penumbra: 0,018 · 0,052 ·
// 0,134 — o resto é das cascatas (o vazamento na sombra de luz pequena, o que o HRC ataca).
// ⚠️ W5a: a população é a de FORA dos opacos (dentro, o gabarito deixou de ser 0 — o F3, com gate
// próprio em `c3_grade_fina`). Fora, com a W5a: penumbra 0,0162 · 0,0264 · 0,0594 · céu e vidro
// 0,0121 (Alto) — dentro dos tetos da W2.
const TOL_PENUMBRA: [f64; 3] = [0.100, 0.155, 0.257];
const TOL_FAISCA_ALTO: f64 = 0.310;
const TOL_CEU_E_VIDRO: [f64; 3] = [0.014, 0.022, 0.042];
const TOL_ANEL: f64 = 0.0435;

fn rodar(
    gpu: &ph2d_gpu::GpuContext,
    cfg: Config,
    cena: &CenaDeLuz,
) -> (Vec<[f32; 3]>, Vec<[f64; 3]>) {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let rel = s.quadro(gpu, cena);
    assert!(
        rel.fora.is_empty(),
        "a cena entra inteira na C0: {:?}",
        rel.fora
    );
    (ler_fluencia(gpu, &s), gabarito(cena, &s.config().grade))
}

fn por_degrau(nome: &str, cena: &CenaDeLuz, tol: [f64; 3]) {
    let Some(gpu) = gpu() else { return };
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let (g, r) = rodar(&gpu, Config::padrao(grade(lado, texel)), cena);
        // Fora dos opacos: dentro, a lei é a do preenchimento (W5a, F3 — gate em `c3_grade_fina`).
        let dentro = crate::comum::dentro_de_opaco(cena, &grade(lado, texel));
        let e = rmse_rel(&g, &r, |k| !dentro(k));
        eprintln!(
            "G3 {nome} · {degrau}: RMSE relativo {e:.4} fora dos opacos (tolerância {})",
            tol[k]
        );
        assert!(e <= tol[k], "G3 {nome} · {degrau}: {e:.4} > {}", tol[k]);
    }
}

#[test]
fn g3_penumbra_da_hrc_fig8_por_degrau() {
    por_degrau("penumbra", &penumbra(), TOL_PENUMBRA);
}

#[test]
fn g3_ceu_vidros_sobrepostos_e_alcance_finito_por_degrau() {
    por_degrau("céu e vidro", &ceu_e_vidro(), TOL_CEU_E_VIDRO);
}

/// ⭐ A faísca (luz de 2 px) ACENDE em TODO degrau pela luz direta (C3, W4) — antes só no Alto,
/// e nos outros saía da C0. O tamanho da lâmpada já não decide se ela existe.
#[test]
fn g3_a_faisca_acende_em_todo_degrau_pela_luz_direta() {
    let Some(gpu) = gpu() else { return };
    let cena = faisca();
    for (degrau, lado, texel) in DEGRAUS {
        let g = grade(lado, texel);
        let fora = o_que_fica_fora(&cena, &g).fora;
        assert!(
            fora.is_empty(),
            "{degrau}: a faísca direta não é «fora»: {fora:?}"
        );
        let (gf, r) = rodar(&gpu, Config::padrao(g), &cena);
        let e = rmse_rel(&gf, &r, |_| true);
        eprintln!("G3 faísca · {degrau}: RMSE relativo {e:.4} (tolerância {TOL_FAISCA_ALTO})");
        assert!(e <= TOL_FAISCA_ALTO, "G3 faísca · {degrau}: {e:.4}");
    }
}

#[test]
fn g3_o_anel_reprova_sem_a_correcao_bilinear() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    // ⚠️ O anel é das CASCATAS: a lâmpada da penumbra vai pela luz direta (C3), então o emissor aqui
    // é o MESMO disco como polígono (o que um sprite que brilha é) — ele fica nas cascatas.
    let cena = crate::cenas::penumbra_pelas_cascatas();
    // A região LIVRE: onde o gabarito é o da mesma cena sem obstáculo (e longe da luz).
    let sem = gabarito(
        &CenaDeLuz {
            oclusores: vec![],
            ..cena.clone()
        },
        &grade(lado, texel),
    );
    let livre = |r: &[[f64; 3]]| {
        let sem = &sem;
        let longe = regiao_livre_longe(lado, texel);
        let r = r.to_vec();
        move |k: usize| longe(k) && (r[k][0] - sem[k][0]).abs() <= 1e-9 * sem[k][0]
    };
    let mut erros = Vec::new();
    for fusao in [Fusao::CorrecaoBilinear, Fusao::Classica] {
        let cfg = Config {
            fusao,
            ..Config::padrao(grade(lado, texel))
        };
        let (g, r) = rodar(&gpu, cfg, &cena);
        let e = rmse_rel(&g, &r, livre(&r));
        eprintln!(
            "G3 anel · {fusao:?}: RMSE relativo na região livre {e:.4} (tolerância {TOL_ANEL})"
        );
        erros.push(e);
    }
    assert!(
        erros[0] <= TOL_ANEL,
        "com a correção bilinear o anel não fecha: {:.4}",
        erros[0]
    );
    assert!(
        erros[1] > TOL_ANEL,
        "sem a correção o anel tinha de reprovar: {:.4}",
        erros[1]
    );
}

/// O caminho f16 (onde a placa tem `SHADER_F16`) e o armazenamento f32 respondem à MESMA pergunta.
#[test]
fn g3_os_caminhos_de_precisao_respondem_igual() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let cena = penumbra();
    let mut variantes = vec![(Precisao::F32, Precisao::F32)];
    if tem_f16(&gpu) {
        variantes.push((Precisao::F16, Precisao::F16));
    } else {
        eprintln!("G3 precisão: a placa não tem SHADER_F16 — só o caminho f32");
    }
    for (aritmetica, armazenamento) in variantes {
        let cfg = Config {
            aritmetica,
            armazenamento,
            ..Config::padrao(grade(lado, texel))
        };
        let (g, r) = rodar(&gpu, cfg, &cena);
        let e = rmse_rel(&g, &r, |_| true);
        eprintln!("G3 precisão · aritmética {aritmetica:?} · cascatas {armazenamento:?}: {e:.4}");
        assert!(
            e <= TOL_PENUMBRA[0],
            "{aritmetica:?}/{armazenamento:?}: {e:.4}"
        );
    }
}
