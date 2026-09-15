//! ⭐⭐⭐ **O TREMOR DA PILHA** — o 4.º report do dono sobre a `=114` (2026-09-15: *«as shapes que
//! ficam embaixo no centro vibram muito após serem apertadas pelas shapes acima»*, com três setas
//! no fundo do monte). Mecanismo, a ordem em que as hipóteses caíram e a tabela: **doc 109 §8**.
//!
//! Irmão do [`super::tests`] pelo tecto de LOC (HR-18) e por ASSUNTO: lá mede-se a GEOMETRIA da
//! pilha (o vão típico, quem colide com quem, o que o cartão alcança); aqui mede-se o MOVIMENTO
//! dela depois de assentar.
//!
//! ⚠️⚠️ **E é exactamente essa fronteira que explica por que os quatro gates de lá estavam verdes
//! sobre o defeito:** um vão é uma FOTOGRAFIA, e *uma pilha a tremer e uma pilha parada com o mesmo
//! espaçamento leem-se iguais nele*.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

// ---------------------------------------------------------------------------------------------
// SONDA — *«as shapes que ficam embaixo no centro vibram muito após serem apertadas pelas shapes
// acima»* (report do dono, 2026-09-15, com foto e três setas no fundo do monte).
// ---------------------------------------------------------------------------------------------

/// **Quanto cada peça ainda MEXE depois de a pilha assentar, e ONDE ela está.**
///
/// ⚠️ A régua da cena (`vizinho_mediano`) mede o VÃO entre peças, que é uma fotografia: uma pilha a
/// tremer e uma pilha parada com o mesmo espaçamento leem-se **iguais** nela. O que o dono fotografou
/// é MOVIMENTO, e nenhuma régua desta cena o via — é por isso que os gates dela estão todos verdes
/// sobre o defeito.
///
/// A grandeza é o deslocamento por tique **ao longo da última meia janela**, em fracção do LADO da
/// peça (adimensional ⇒ comparável entre densidades). Uma peça assente lê `~0`.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_que_ainda_treme -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_que_ainda_treme() {
    let (tremor, fim, vel) = tremor_da_direita(2.0, 2.9);
    let lado = LADO;
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    eprintln!("\n  peca |      x      y | tremor/lado | |vel|");
    eprintln!("  -----|---------------|-------------|-------");
    for i in ordem {
        eprintln!(
            "  {i:>4} | {:>6.3} {:>6.3} | {:>11.4} | {:>5.3}",
            fim[i][0],
            fim[i][1],
            tremor[i] / lado,
            vel[i]
        );
    }
    // A leitura que a foto pede: o tremor contra a ALTURA da peça no monte.
    let (mut baixo, mut alto) = (Vec::new(), Vec::new());
    let y_med = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
    for (i, p) in fim.iter().enumerate() {
        if p[1] < y_med {
            baixo.push(tremor[i] / lado);
        } else {
            alto.push(tremor[i] / lado);
        }
    }
    eprintln!(
        "\n  metade de BAIXO: mediana {:.4} · pior {:.4}",
        mediana(&baixo),
        baixo.iter().copied().fold(0.0_f32, f32::max)
    );
    eprintln!(
        "  metade de CIMA : mediana {:.4} · pior {:.4}",
        mediana(&alto),
        alto.iter().copied().fold(0.0_f32, f32::max)
    );
}

/// O deslocamento mediano por tique de cada peça da DIREITA entre `de` e `ate` segundos, a posição
/// final, e o módulo da velocidade final.
fn tremor_da_direita(de: f64, ate: f64) -> (Vec<f32>, Vec<[f32; 2]>, Vec<f32>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<[f32; 2]>::new());
    let (mut fim, mut vel) = (Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        if k == last {
            vel = match s.get("vel") {
                Some(Column::Vec2(v)) => v.iter().map(|v| v[0].hypot(v[1])).collect(),
                _ => vec![f32::NAN; fim.len()],
            };
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let tremor = (0..fim.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect();
    (tremor, fim, vel)
}

/// A mediana de uma amostra (vazia ⇒ `0`).
fn mediana(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(f32::total_cmp);
    s[s.len() / 2]
}

/// **A DIREÇÃO do tremor, e se ele é oscilação ou deriva.**
///
/// ⚠️ As duas hipóteses previram coisas diferentes e é isto que as separa:
/// - **a velocidade só é cancelada ao longo da correcção LÍQUIDA** ⇒ uma peça apertada de LADO tem
///   `d` horizontal, a gravidade nunca é cancelada, e o tremor sai **vertical**;
/// - **oito varreduras de Jacobi não convergem numa ilha densa** ⇒ o tremor não tem direcção
///   preferida e cede a mais varreduras.
///
/// A segunda coluna é `|Σ passos| / Σ |passos|`: `0` = oscila no sítio, `1` = anda sempre para o
/// mesmo lado. *Vibrar e deslizar leem-se iguais num deslocamento por tique.*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_direcao_do_tremor -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_direcao_do_tremor() {
    let (tremor, fim, eixo, deriva) = direcao_da_direita(2.0, 2.9);
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    eprintln!("\n  peca |      x      y | tremor/lado | |dy|/(|dx|+|dy|) | deriva");
    eprintln!("  -----|---------------|-------------|------------------|-------");
    for i in ordem.into_iter().take(10) {
        eprintln!(
            "  {i:>4} | {:>6.3} {:>6.3} | {:>11.4} | {:>16.3} | {:>6.3}",
            fim[i][0],
            fim[i][1],
            tremor[i] / LADO,
            eixo[i],
            deriva[i]
        );
    }
}

/// Por peça: o passo mediano, a posição final, a fracção VERTICAL do movimento, e a deriva.
fn direcao_da_direita(de: f64, ate: f64) -> (Vec<f32>, Vec<[f32; 2]>, Vec<f32>, Vec<f32>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior, mut fim) = (Vec::<Vec<[f32; 2]>>::new(), Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b): (&[f32; 2], &[f32; 2])| [a[0] - b[0], a[1] - b[1]])
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let (mut tremor, mut eixo, mut deriva) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..fim.len() {
        let ds: Vec<[f32; 2]> = passos.iter().map(|linha| linha[i]).collect();
        tremor.push(mediana(
            &ds.iter().map(|d| d[0].hypot(d[1])).collect::<Vec<_>>(),
        ));
        let (ax, ay) = ds.iter().fold((0.0_f32, 0.0_f32), |(x, y), d| {
            (x + d[0].abs(), y + d[1].abs())
        });
        eixo.push(if ax + ay > 0.0 { ay / (ax + ay) } else { 0.0 });
        let soma = ds
            .iter()
            .fold([0.0_f32, 0.0], |a, d| [a[0] + d[0], a[1] + d[1]]);
        let total: f32 = ds.iter().map(|d| d[0].hypot(d[1])).sum();
        deriva.push(if total > 0.0 {
            soma[0].hypot(soma[1]) / total
        } else {
            0.0
        });
    }
    (tremor, fim, eixo, deriva)
}

/// **O A/B das causas candidatas** — cada linha desliga UMA coisa e mede o mesmo tremor.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_que_cura_o_tremor -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_que_cura_o_tremor() {
    eprintln!("\n  caso                        | baixo p50 | baixo pior | cima p50");
    eprintln!("  ----------------------------|-----------|------------|---------");
    for (rotulo, params) in [
        ("como a cena shipa", &[][..]),
        ("rotacao TRAVADA", &[(param::LOCK_ROTATION, 1.0)][..]),
    ] {
        let (tremor, fim) = tremor_com(2.0, 2.9, params);
        let y = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
        let baixo: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] < y)
            .map(|i| tremor[i] / LADO)
            .collect();
        let cima: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] >= y)
            .map(|i| tremor[i] / LADO)
            .collect();
        eprintln!(
            "  {rotulo:<27} | {:>9.4} | {:>10.4} | {:>8.4}",
            mediana(&baixo),
            baixo.iter().copied().fold(0.0_f32, f32::max),
            mediana(&cima)
        );
    }
}

/// O tremor mediano por peça e a posição final, com `params` escritos no cartão da DIREITA.
fn tremor_com(de: f64, ate: f64, params: &[(&'static str, f32)]) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let formas: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .collect();
    for (nome, valor) in params {
        state.doc.graph.set_param(formas[1], *nome, *valor);
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior, mut fim) = (Vec::<Vec<f32>>::new(), Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b): (&[f32; 2], &[f32; 2])| (a[0] - b[0]).hypot(a[1] - b[1]))
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let tremor = (0..fim.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect();
    (tremor, fim)
}

/// **O SINAL do ângulo das piores peças, tique a tique** — 2 ciclos = sobre-correcção.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_angulo_tique_a_tique -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_angulo_tique_a_tique() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let mut serie: Vec<Vec<f32>> = Vec::new();
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if t >= 2.6
            && let Some(Column::Scalar(r)) = s.get("rot")
        {
            serie.push(r.clone());
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    eprintln!("\n  tique |   rot[2] |   rot[3] |   rot[0]");
    eprintln!("  ------|----------|----------|---------");
    for (k, r) in serie.iter().enumerate().take(14) {
        eprintln!("  {k:>5} | {:>8.4} | {:>8.4} | {:>8.4}", r[2], r[3], r[0]);
    }
    // Quantas vezes o passo TROCA de sinal: um ciclo de 2 tiques troca em todos.
    for i in [2_usize, 3, 0] {
        let d: Vec<f32> = serie.windows(2).map(|w| w[1][i] - w[0][i]).collect();
        let trocas = d.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
        eprintln!(
            "  peca {i}: {trocas} trocas de sinal em {} passos · |passo| p50 = {:.5}°",
            d.len(),
            mediana(&d.iter().map(|x| x.abs()).collect::<Vec<_>>())
        );
    }
}

/// **QUEM vibra: as peças pousadas na TAÇA, ou as pousadas noutras peças?**
///
/// ⚠️ É a pergunta que a foto faz e que nenhuma coluna anterior separava — *«em baixo»* e *«apoiada
/// no mundo»* são a mesma região nesta cena, e só uma delas é o mecanismo.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_quem_vibra_toca_a_taca -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_quem_vibra_toca_a_taca() {
    let (tremor, fim, _eixo, deriva) = direcao_da_direita(2.0, 2.9);
    // A taça da DIREITA: centro em (VAO/2 + …, TACA_Y), raio TACA_R. O centro em x sai da própria
    // nuvem — a cena desloca as duas metades, e uma constante aqui envelheceria com ela.
    let cx = fim.iter().map(|p| p[0]).sum::<f32>() / fim.len() as f32;
    let meia = LADO * std::f32::consts::SQRT_2 / 2.0;
    let mut na_taca = Vec::new();
    let mut solta = Vec::new();
    eprintln!("\n  peca | dist. a' parede | tremor/lado | deriva | apoio");
    eprintln!("  -----|-----------------|-------------|--------|-------");
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    for i in ordem {
        let d = TACA_R - (fim[i][0] - cx).hypot(fim[i][1] - TACA_Y) - meia;
        let toca = d.abs() < meia;
        if deriva[i] < 0.5 {
            if toca {
                na_taca.push(tremor[i] / LADO);
            } else {
                solta.push(tremor[i] / LADO);
            }
        }
        eprintln!(
            "  {i:>4} | {d:>15.4} | {:>11.4} | {:>6.3} | {}",
            tremor[i] / LADO,
            deriva[i],
            if toca { "TAÇA" } else { "peças" }
        );
    }
    eprintln!(
        "\n  (só as que OSCILAM, deriva < 0,5)\n  apoiadas na TAÇA  : n={} · p50 {:.4} · pior {:.4}",
        na_taca.len(),
        mediana(&na_taca),
        na_taca.iter().copied().fold(0.0_f32, f32::max)
    );
    eprintln!(
        "  apoiadas em PEÇAS : n={} · p50 {:.4} · pior {:.4}",
        solta.len(),
        mediana(&solta),
        solta.iter().copied().fold(0.0_f32, f32::max)
    );
}

/// **SONDA — o que o ATRITO entre peças faz ao resto do tremor.**
///
/// ⚠️ A cena não escreve material nenhum nas peças, logo elas são **gelo** entre si (`μ = 0`, o
/// `Material::LISO`): um monte sem atrito não assenta, e isso é Física e não um defeito. Esta sonda
/// diz quanto do resíduo é isso.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_atrito_entre_pecas -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_atrito_entre_pecas() {
    eprintln!("\n  atrito das peças | baixo p50 | baixo pior | cima p50");
    eprintln!("  -----------------|-----------|------------|---------");
    for mu in [0.0_f32, 0.3, 0.6, 0.9] {
        let (tremor, fim) = tremor_com(2.0, 2.9, &[(param::FRICTION, mu)]);
        let y = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
        let baixo: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] < y)
            .map(|i| tremor[i] / LADO)
            .collect();
        let cima: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] >= y)
            .map(|i| tremor[i] / LADO)
            .collect();
        eprintln!(
            "  {mu:>16.2} | {:>9.4} | {:>10.4} | {:>8.4}",
            mediana(&baixo),
            baixo.iter().copied().fold(0.0_f32, f32::max),
            mediana(&cima)
        );
    }
}

/// O `|Δrot|` mediano por tique de CADA peça da direita, na janela assente.
fn balanco_angular(de: f64, ate: f64) -> Vec<f32> {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Scalar(r)) = s.get("rot") {
            if t >= de && anterior.len() == r.len() {
                passos.push(
                    r.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a - b).abs())
                        .collect(),
                );
            }
            anterior = r.clone();
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    (0..anterior.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect()
}

/// **SONDA — a faixa do balanço angular, para a barra do gate sair de um VALE medido.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_faixa_do_balanco -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_faixa_do_balanco() {
    let mut b = balanco_angular(2.0, 2.9);
    b.sort_by(f32::total_cmp);
    eprintln!("\n  |Δrot| mediano por tique, as 25 peças em ordem:");
    for (i, v) in b.iter().enumerate() {
        eprintln!("  {i:>3} | {v:>8.4}°");
    }
}

/// ⭐⭐⭐ **A PILHA ASSENTA EM VEZ DE ZUMBIR** — o gate do 4.º report do dono (2026-09-15:
/// *«as shapes que ficam embaixo no centro vibram muito após serem apertadas pelas shapes acima»*,
/// com três setas no fundo do monte). Mecanismo e tabela: [doc 109 §8].
///
/// ⚠️⚠️ **A régua da cena não podia ver isto:** o `vizinho_mediano` mede o VÃO, que é uma
/// fotografia — *uma pilha a tremer e uma pilha parada com o mesmo espaçamento leem-se iguais nela*,
/// e os quatro gates da `=114` estavam verdes sobre o defeito. Esta mede **movimento**: o `|Δrot|`
/// mediano por tique de cada peça, na janela em que a pilha já devia estar assente.
///
/// ⭐ **A barra sai de um VALE MEDIDO, com os dois lados:** com a lei antiga (a rotação como
/// empurrão de ângulo) as 25 peças leem `… 0,38 · 1,24 · 2,53 · 3,50 · 3,96 · 4,16°` — **cinco**
/// acima de `1,0` —, e com a moeda certa (`spin`) a pior lê **`0,87°`**. ⛔ Não é um número
/// escolhido: é o vazio entre o lado que o dono reprovou e o lado curado.
///
/// ⚠️ **É um gate DETERMINÍSTICO, não de relógio** — ele não divide dois tempos nem conta
/// alocações, logo não é candidato à família de flakes de carga do `CLAUDE.md` §5.0.
#[test]
fn the_pile_settles_instead_of_buzzing() {
    /// Graus por tique. Ver o vale acima.
    const BARRA: f32 = 1.0;
    let balanco = balanco_angular(2.0, 2.9);
    assert!(
        balanco.len() >= 20,
        "piso de populacao: a metade da direita tem de ter peças ({})",
        balanco.len()
    );
    let piores: Vec<(usize, f32)> = balanco
        .iter()
        .enumerate()
        .filter(|(_, v)| **v > BARRA)
        .map(|(i, v)| (i, *v))
        .collect();
    assert!(
        piores.is_empty(),
        "peças a zumbir acima de {BARRA}°/tique: {piores:?} — doc 109 §8"
    );
}
