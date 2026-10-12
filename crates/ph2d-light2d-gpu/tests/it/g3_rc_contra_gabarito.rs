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
// ⭐ W5c — a fusão junto a uma face (`MascaraDaFusao::PeloFim` + `fronteira_na_face`, o ABERTO 1):
// céu e vidro 0,0121 · 0,0198 · 0,0374 → 0,0074 · 0,0087 · 0,0117; anel bilinear 0,0393 → 0,0365
// (clássica 0,1251); penumbra e faísca (luz direta) iguais: 0,0162 · 0,0264 · 0,0594 e 0,0322 ·
// 0,0583 · 0,1005. Os tetos herdados largos (W2) apertam-se ao medido de hoje + 10 %.
const TOL_PENUMBRA: [f64; 3] = [0.0179, 0.0291, 0.0654];
const TOL_FAISCA: [f64; 3] = [0.0355, 0.0642, 0.1106];
// ⭐ W5m — a sonda de dentro de um translúcido deixa de entrar no campo de fora (peso 0 na C3): céu e
// vidro 0,0074 · 0,0087 · 0,0117 (W5c) → 0,0100 · 0,0102 · 0,0122 (W5k, o peso τ: o fundo dos vidros −9 %,
// `sonda_o_g3_ceu_e_vidro_por_regiao`) → 0,0065 · 0,0074 · 0,0098.
const TOL_CEU_E_VIDRO: [f64; 3] = [0.0072, 0.0081, 0.0108];
const TOL_ANEL: f64 = 0.0402;
/// Os caminhos de precisão na penumbra, sobre TODOS os texels (dentro dos opacos também): W5c
/// F32/F32 0,0218 · F16/F16 0,0217.
const TOL_PRECISAO: f64 = 0.024;

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
    for (k, (degrau, lado, texel)) in DEGRAUS.into_iter().enumerate() {
        let g = grade(lado, texel);
        let fora = o_que_fica_fora(&cena, &g).fora;
        assert!(
            fora.is_empty(),
            "{degrau}: a faísca direta não é «fora»: {fora:?}"
        );
        let (gf, r) = rodar(&gpu, Config::padrao(g), &cena);
        let e = rmse_rel(&gf, &r, |_| true);
        eprintln!(
            "G3 faísca · {degrau}: RMSE relativo {e:.4} (tolerância {})",
            TOL_FAISCA[k]
        );
        assert!(e <= TOL_FAISCA[k], "G3 faísca · {degrau}: {e:.4}");
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
            e <= TOL_PRECISAO,
            "{aritmetica:?}/{armazenamento:?}: {e:.4}"
        );
    }
}

/// ⭐ **A queda de alcance FINITO entra nas cascatas** (mutação P9 da W5c — nenhum portão a via: a
/// lâmpada de alcance finito do «céu e vidro» vai pela luz direta). Um corpo que brilha com alcance
/// de 0,8 m: a fluência das cascatas contra o gabarito, na clássica e na holográfica.
#[test]
fn g3_a_queda_de_alcance_finito_nas_cascatas() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[1];
    let g = grade(lado, texel);
    let cena = CenaDeLuz {
        emissores: vec![ph2d_light2d_ecs::Emissor::Poligono {
            vertices: crate::comum::regular([2.56, 2.56], 0.15, 12, 0.0),
            radiancia: [1.0; 3],
            alcance_m: 0.8,
        }],
        luzes: 1,
        ..Default::default()
    };
    let r = gabarito(&cena, &g);
    let dentro = crate::comum::dentro_de_opaco(&cena, &g);
    let (flu, _) = rodar(&gpu, Config::padrao(g), &cena);
    let e = rmse_rel(&flu, &r, |k| !dentro(k));
    // ⚠️ A holográfica conta a queda do início de cada segmento fundido (a composição dos traços não
    // carrega a distância ao recetor): 4,21 nesta cena — ABERTO da W5c, impresso, sem teto.
    let (holo, _) = rodar(
        &gpu,
        Config {
            holografica: Some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO),
            ..Config::padrao(g)
        },
        &cena,
    );
    let eh = rmse_rel(&holo, &r, |k| !dentro(k));
    eprintln!("G3 queda finita: clássica {e:.4} · holográfica {eh:.4} (ABERTO)");
    assert!(e <= TOL_QUEDA, "{e:.4}");
}

/// Medido (W5c) na clássica: 0,3601 + 10 %.
const TOL_QUEDA: f64 = 0.40;

/// Sonda (W5m, §4.0): o RMSE do «céu e vidro» partido por região — dentro de um vidro (junto à face
/// ≤ 3 texels · fundo · na sobreposição) × fora de tudo (junto a um vidro ≤ 3 texels · resto) — em
/// cinco variantes: os vidros a ½, a 1 e a 0,001, com e sem o miolo (a placa e o gabarito iguais).
/// Cada parte imprime a sua parcela do erro TOTAL (as parcelas ao quadrado somam o total ao
/// quadrado), o erro local e o viés `média(gpu − ref) / média(ref)`. Só a parede e a lâmpada saem.
#[test]
#[ignore = "sonda: imprime, não decide"]
fn sonda_o_g3_ceu_e_vidro_por_regiao() {
    let Some(gpu) = gpu() else { return };
    let base = ceu_e_vidro();
    let vidros: Vec<Vec<[f32; 2]>> = base
        .oclusores
        .iter()
        .filter(|o| o.opacidade < 1.0)
        .map(|o| o.vertices.clone())
        .collect();
    let so_parede = CenaDeLuz {
        oclusores: base
            .oclusores
            .iter()
            .filter(|o| o.opacidade >= 1.0)
            .cloned()
            .collect(),
        ..base.clone()
    };
    let so = std::env::var("PH2D_G3_DEGRAU")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let (degrau, lado, texel) = DEGRAUS[so];
    let gr = grade(lado, texel);
    let fora = crate::comum::dentro_de_opaco(&so_parede, &gr);
    for (opacidade, com_miolo) in [
        (0.5, true),
        (0.5, false),
        (1.0, true),
        (1.0, false),
        (0.001, true),
    ] {
        let mut cena = base.clone();
        for o in cena.oclusores.iter_mut().filter(|o| o.opacidade < 1.0) {
            o.opacidade = opacidade;
        }
        let cfg = Config {
            miolo: com_miolo.then_some(Config::padrao(gr).miolo).flatten(),
            ..Config::padrao(gr)
        };
        let (g, _) = rodar(&gpu, cfg, &cena);
        let op = ph2d_light2d_ref::OpcoesDoMiolo {
            suavidade: if com_miolo { 1.0 } else { 0.0 },
            ..ph2d_light2d_ref::MIOLO_PADRAO
        };
        let gab = ph2d_light2d_ref::Gabarito::com_miolo(&cena, op);
        let w = gr.largura as usize;
        let r: Vec<[f64; 3]> = {
            let n = g.len();
            let fios = std::thread::available_parallelism()
                .map_or(4, |k| k.get())
                .min(16);
            let pedaco = n.div_ceil(fios);
            let mut out = vec![[0.0; 3]; n];
            std::thread::scope(|s| {
                for (k, fatia) in out.chunks_mut(pedaco).enumerate() {
                    let gab = &gab;
                    s.spawn(move || {
                        for (o, v) in fatia.iter_mut().enumerate() {
                            let i = k * pedaco + o;
                            *v = gab.fluencia(gr.centro((i % w) as u32, (i / w) as u32));
                        }
                    });
                }
            });
            out
        };
        const NOMES: [&str; 5] = [
            "vidro·face",
            "vidro·fundo",
            "vidro∩vidro",
            "fora·perto",
            "fora·resto",
        ];
        let mut se = [0.0_f64; 5];
        let mut vies = [0.0_f64; 5];
        let mut soma = [0.0_f64; 5];
        let mut n = [0_usize; 5];
        for i in 0..g.len() {
            if fora(i) {
                continue;
            }
            let p = gr.centro((i % w) as u32, (i / w) as u32);
            let p = [p[0] as f32, p[1] as f32];
            let (mut dentro, mut d) = (0, f64::INFINITY);
            for v in &vidros {
                let (e, dv) = dentro_e_distancia(v, p);
                dentro += usize::from(e);
                d = d.min(dv);
            }
            let perto = d <= 3.0 * texel;
            let parte = match (dentro, perto) {
                (2.., _) => 2,
                (1, true) => 0,
                (1, false) => 1,
                (0, true) => 3,
                _ => 4,
            };
            for c in 0..3 {
                let dif = f64::from(g[i][c]) - r[i][c];
                se[parte] += dif * dif;
                vies[parte] += dif;
                soma[parte] += r[i][c];
                n[parte] += 1;
            }
        }
        let (nt, mt) = (n.iter().sum::<usize>() as f64, soma.iter().sum::<f64>());
        let media = mt / nt;
        let total = (se.iter().sum::<f64>() / nt).sqrt() / media;
        eprintln!(
            "SONDA G3 céu e vidro · {degrau} · vidros a {opacidade} · miolo {com_miolo}: total {total:.5}"
        );
        for p in 0..5 {
            if n[p] == 0 {
                continue;
            }
            let np = n[p] as f64;
            eprintln!(
                "  {:<12} texels {:>6} · parcela {:.5} · local {:.5} · viés {:+.5} · ref média {:.5} · gpu média {:.5}",
                NOMES[p],
                n[p] / 3,
                (se[p] / nt).sqrt() / media,
                (se[p] / np).sqrt() / (soma[p] / np),
                vies[p] / soma[p],
                soma[p] / np,
                (soma[p] + vies[p]) / np
            );
        }
    }
}

fn dentro_e_distancia(v: &[[f32; 2]], p: [f32; 2]) -> (bool, f64) {
    let (mut dentro, mut d) = (false, f64::INFINITY);
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let (a, b) = (v[i], v[j]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            dentro = !dentro;
        }
        let ab = [f64::from(b[0] - a[0]), f64::from(b[1] - a[1])];
        let ap = [f64::from(p[0] - a[0]), f64::from(p[1] - a[1])];
        let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-30))
            .clamp(0.0, 1.0);
        d = d.min((ap[0] - ab[0] * t).hypot(ap[1] - ab[1] * t));
        j = i;
    }
    (dentro, d)
}
