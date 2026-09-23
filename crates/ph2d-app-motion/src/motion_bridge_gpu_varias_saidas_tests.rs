//! ⭐⭐⭐ **AS CENAS DE VÁRIAS SAÍDAS, PELA PLACA, DÃO O QUE A CPU DÁ** (doc 119 W3/W4).
//!
//! O ciclo 11 levantou a cerca do multi-sink: `cook_gpu` planeia a UNIÃO das saídas e o
//! `cook_many` baixa-as no mesmo buffer. Este gate corre a MESMA ponte que o quadro chama sobre
//! **toda cena de demo com mais de uma saída**, e compara o que a placa entrega — as linhas, pela
//! ORDEM em que o desenho as pinta — com o que a bomba da CPU entrega, ordenado como o passe de
//! sprites ordena.
//!
//! ⚠️ **Dois estados GÉMEOS** (montados da mesma cena): a rota híbrida marcha a bomba até às
//! fronteiras, e cozinhar a CPU no mesmo estado seria perguntar-lhe um tique que ela já deu.
//!
//! ⚠️ **Só se julga o que a ponte mandou para a placa** — uma cena que ela recusa (forma vectorial
//! viva, colisor, mistura em grupo…) é contada e nomeada, nunca comparada: a CPU consigo mesma não
//! prova nada. E há um PISO: se nenhuma cena de várias saídas fosse à placa, o gate seria vácuo.
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo test -p ph2d-app-motion --lib -- --ignored --nocapture as_cenas_de_varias_saidas

use super::{GpuOutcome, cook_gpu};
use crate::motion_state::MotionState;
use crate::motion_state::demo_router::MAX_DEMO_LEVEL;
use ph2d_gpu::GpuContext;
use ph2d_render::RenderInstance;

/// O tique a comparar — longe do `0`, onde um laço ainda não tem estado.
const TIQUE: u64 = 3;
const DT: f64 = 1.0 / 60.0;
/// A barra das posições. ⚠️ É a de um SIMULADOR, não a de um nó puro: os dois motores integram
/// em `f32` com somas em ordens diferentes (a paridade de simulação da `ph2d-gpu-cook` mede
/// `~1e-4` ao fim de poucos tiques); um nó puro fica em `~1e-6`.
const EPS: f32 = 1e-3;

/// ⭐ **A barra de cada SAÍDA sai do que a corrente dela TEM, e é a que o gate irmão já DECLAROU**
/// — nunca uma barra folgada para a cena inteira (ela engoliria um defeito numa saída limpa).
///
/// - Um nó que LÊ UMA TABELA (a onda/ease `Custom`, LUT de 512): o device reconstrói a curva
///   entre dois nós e a CPU avalia-a exacta ⇒ `EPS_REL_LUT` da `gpu_cpu_parity_curve`, **relativa
///   à amplitude**. Medido aqui: `6,65e-4` nas duas saídas `Custom` da `=85`.
/// - Um ALGORITMO de GPU (o Voronoi por inundação, ADR-0139): a divergência é DECLARADA no módulo
///   dele (a inundação perde texels de empate e uma colisão de semente esconde um ponto por uma
///   ronda) ⇒ a banda da `gpu_voronoi::full_relax`, medida num domínio `5×5` e aqui relativa ao
///   lado: máximo `0,55/5`, média `0,04/5`.
/// - O resto: [`EPS`], absoluta.
const EPS_REL_LUT: f32 = 2.0e-3;
const JFA_MAX_REL: f32 = 0.55 / 5.0;
const JFA_MEDIA_REL: f32 = 0.04 / 5.0;

/// Que barra a corrente desta saída pede — ver [`EPS_REL_LUT`].
enum Barra {
    Absoluta,
    Tabela,
    Inundacao,
}

fn barra_da_saida(m: &MotionState, sink: ph2d_nodegraph::graph::NodeId) -> Barra {
    use ph2d_nodegraph::gpu::KernelResolver;
    let tipos: Vec<ph2d_nodegraph::node::NodeTypeId> =
        ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, sink)
            .iter()
            .filter_map(|n| m.doc.graph.node(*n))
            .map(|i| ph2d_nodegraph::node::NodeTypeId::of(i.type_name.as_str()))
            .collect();
    if tipos.iter().any(|t| m.registry.algorithm(*t).is_some()) {
        Barra::Inundacao
    } else if tipos.iter().any(|t| !m.registry.luts(*t).is_empty()) {
        Barra::Tabela
    } else {
        Barra::Absoluta
    }
}

fn estado(level: u32) -> Option<MotionState> {
    let mut m = MotionState::new();
    let montadas = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry).0;
    if montadas.is_empty() {
        return None;
    }
    // As saídas como o QUADRO as calcula (`dispatch`), não as que a cena devolve.
    m.sinks = super::super::remove::output_nodes(&m.doc.graph);
    (m.sinks.len() > 1).then_some(m)
}

/// A ordem em que a placa DESENHA: os runs pela ordem da lista, ou o buffer inteiro por ordem.
fn da_placa(gpu: &GpuContext, m: &MotionState) -> Vec<[f32; 2]> {
    let inst = m
        .gpu_cook
        .instances()
        .map(|i| ph2d_gpu_cook::read_instances(gpu, i))
        .unwrap_or_default();
    let runs = m.gpu_cook.texture_runs();
    if runs.is_empty() {
        return inst.iter().map(|i| i.world_pos).collect();
    }
    runs.iter()
        .flat_map(|r| {
            inst[r.start as usize..r.end as usize]
                .iter()
                .map(|i| i.world_pos)
        })
        .collect()
}

/// A coluna `P` de cada saída, pela placa — a corrente que o plano da UNIÃO produziu.
fn p_da_placa(gpu: &GpuContext, m: &MotionState) -> Vec<Vec<[f32; 2]>> {
    m.sinks
        .iter()
        .map(|&s| m.gpu_cook.read_column_vec2(gpu, s, "P").unwrap_or_default())
        .collect()
}

/// A coluna `P` de cada saída, pela CPU — as tomadas da bomba, que ela coze na marcha.
fn p_da_cpu(m: &MotionState) -> Vec<Vec<[f32; 2]>> {
    m.sinks
        .iter()
        .map(|s| {
            m.pump
                .tap_streams()
                .iter()
                .find(|(n, _)| n == s)
                .and_then(|(_, st)| match st.get("P") {
                    Some(ph2d_nodegraph::attr::Column::Vec2(v)) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        })
        .collect()
}

/// A CPU: a bomba marcha os tiques devidos, e o passe de sprites ordena as linhas.
fn da_cpu(m: &mut MotionState) -> Vec<[f32; 2]> {
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let sinks = m.sinks.clone();
    m.pump.set_taps(&sinks);
    for tick in 0..=TIQUE {
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &m.sinks,
            tick,
            |t| t as f64 * DT,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    let mut inst: Vec<RenderInstance> = m.pump.instances.clone();
    ph2d_render::sort_render_order(&mut inst);
    inst.iter().map(|i| i.world_pos).collect()
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let mut julgadas = Vec::new();
    let mut recusadas = Vec::new();
    let mut erradas: Vec<String> = Vec::new();
    let mut linhas_p = 0usize;
    for level in 1..=MAX_DEMO_LEVEL {
        let Some(mut m) = estado(level) else { continue };
        let Some(mut cpu) = estado(level) else {
            continue;
        };
        cpu.gpu_enabled = false;
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        crate::motion_externals::publish_all(&mut cpu, TIQUE as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        m.gpu_cook.retain_streams_for_debug(true);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        if !matches!(saida, GpuOutcome::Handled) {
            recusadas.push((level, m.route_said.unwrap_or("?")));
            continue;
        }
        let p = da_placa(&gpu, &m);
        let vec_cpu_antes = cpu.pump.vector_instances.len();
        let c = da_cpu(&mut cpu);
        let vec_cpu = cpu.pump.vector_instances.len().max(vec_cpu_antes);
        let dif = |a: &[f32; 2], b: &[f32; 2]| (a[0] - b[0]).abs().max((a[1] - b[1]).abs());
        let pior = p
            .iter()
            .zip(&c)
            .map(|(a, b)| dif(a, b))
            .fold(0.0f32, f32::max);
        // ⭐ A corrente `P` de cada saída — o que o plano da união cozeu. A maioria das demos são
        // só POSIÇÕES (a lei do dono cala-as no desenho), e sem esta metade o gate julgava zero
        // linhas dos dois lados em quase toda cena.
        let (pp, pc) = (p_da_placa(&gpu, &m), p_da_cpu(&cpu));
        let mut p_iguais = true;
        let mut fora_da_barra = false;
        let mut pior_p = 0.0f32;
        for ((a, b), &sk) in pp.iter().zip(&pc).zip(&m.sinks) {
            p_iguais &= a.len() == b.len();
            let difs: Vec<f32> = a.iter().zip(b).map(|(x, y)| dif(x, y)).collect();
            let max = difs.iter().copied().fold(0.0f32, f32::max);
            let media = difs.iter().sum::<f32>() / difs.len().max(1) as f32;
            let (lo, hi) = b
                .iter()
                .flat_map(|p| [p[0], p[1]])
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                });
            let amp = (hi - lo).max(0.0);
            pior_p = pior_p.max(max);
            if max > EPS {
                eprintln!(
                    "  ={level} saída {sk:?}: max {max:e} média {media:e} amplitude {amp:.3} barra {}",
                    match barra_da_saida(&m, sk) {
                        Barra::Absoluta => "absoluta",
                        Barra::Tabela => "tabela",
                        Barra::Inundacao => "inundação",
                    }
                );
            }
            fora_da_barra |= match barra_da_saida(&m, sk) {
                Barra::Absoluta => max > EPS,
                Barra::Tabela => max > EPS.max(EPS_REL_LUT * amp),
                Barra::Inundacao => max > JFA_MAX_REL * amp || media > JFA_MEDIA_REL * amp,
            };
            linhas_p += a.len();
        }
        let por_saida: Vec<String> = pp
            .iter()
            .zip(&pc)
            .zip(&m.sinks)
            .map(|((a, b), &sk)| {
                let d = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| dif(x, y))
                    .fold(0.0f32, f32::max);
                let cadeia: Vec<&str> = ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, sk)
                    .iter()
                    .filter_map(|n| m.doc.graph.node(*n).map(|i| i.type_name.as_str()))
                    .collect();
                let (lo, hi) = b
                    .iter()
                    .flat_map(|p| [p[0], p[1]])
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                        (lo.min(v), hi.max(v))
                    });
                format!(
                    "{d:.2e} amplitude {:.3} rel {:.2e} {cadeia:?}",
                    hi - lo,
                    d / (hi - lo)
                )
            })
            .collect();
        if p.len() != c.len() || pior > EPS || fora_da_barra || vec_cpu > 0 || !p_iguais {
            eprintln!("  ={level} por saída: {por_saida:#?}");
            erradas.push(format!(
                "={level}: placa {} linhas · cpu {} linhas + {vec_cpu} vectoriais · P por saída placa {:?} cpu {:?} · pior |Δ| desenho {pior:e} · P {pior_p:e} · {:?}",
                p.len(),
                c.len(),
                pp.iter().map(Vec::len).collect::<Vec<_>>(),
                pc.iter().map(Vec::len).collect::<Vec<_>>(),
                m.route_said
            ));
            continue;
        }
        julgadas.push((level, p.len(), pior.max(pior_p)));
    }
    eprintln!(
        "=== VÁRIAS SAÍDAS PELA PLACA · {} julgadas ===",
        julgadas.len()
    );
    for (l, n, d) in &julgadas {
        eprintln!("  ={l:<4} {n:>6} linhas   pior |Δ| {d:e}");
    }
    eprintln!("  recusadas pela ponte ({}):", recusadas.len());
    for (l, porque) in &recusadas {
        eprintln!("  ={l:<4} {porque}");
    }
    assert!(
        erradas.is_empty(),
        "{} cena(s) desenham diferente pela placa:\n  {}",
        erradas.len(),
        erradas.join("\n  ")
    );
    // O PISO, nas duas grandezas: se nenhuma cena de várias saídas fosse à placa, ou se as que
    // vão não tivessem posições, o gate seria verde a medir nada.
    eprintln!("  posições P comparadas: {linhas_p}");
    assert!(
        julgadas.len() >= 20 && linhas_p >= 1000,
        "só {} cenas de várias saídas foram à placa, com {linhas_p} posições — o gate deixou de medir",
        julgadas.len()
    );
}

/// **O CENSO DE ROTA PELA PORTA DO PRODUTO** (doc 119 W5) — a sonda irmã do `motion_route_census`.
///
/// ⚠️ Aquela conta pelo PLANO e não conhece as recusas de aparência da ponte (a forma viva, o
/// colisor, a mistura em grupo, a forma condicional): ela dizia `110 de 126` na placa com cenas
/// que o quadro manda para a CPU. Esta corre o MESMO `cook_gpu` do quadro e agrupa pela razão que
/// ele DISSE — *a régua de uma rota é a porta que a decide*.
///
///   cargo test -p ph2d-app-motion --lib -- --ignored --nocapture censo_de_rota_pela_ponte
#[test]
#[ignore = "sonda de rota pela ponte (precisa de adapter), não um gate"]
fn censo_de_rota_pela_ponte() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    let mut porques: std::collections::BTreeMap<String, Vec<u32>> = Default::default();
    let (mut total, mut placa) = (0u32, 0u32);
    for level in 1..=MAX_DEMO_LEVEL {
        let mut m = MotionState::new();
        if crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry)
            .0
            .is_empty()
        {
            continue;
        }
        m.sinks = super::super::remove::output_nodes(&m.doc.graph);
        // ⚠️ O que o quadro faz ao montar a cena (`motion_state_verbos`): a cena que ensina um modo
        // SÓ da CPU pede-a pelo nome. Sem esta linha a `=107` lia-se «na placa» aqui e ia à CPU
        // no app — *uma sonda que salta um passo da montagem mede outra cena*.
        m.cpu_pedida =
            crate::motion_state::demo_router::cena_pede_a_cpu_em(Some(&level.to_string()));
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        total += 1;
        placa += u32::from(matches!(saida, GpuOutcome::Handled));
        let razao = m.route_said.unwrap_or("?").to_string();
        porques.entry(razao).or_default().push(level);
    }
    eprintln!("=== CENSO DE ROTA PELA PONTE · {total} cenas ===");
    for (razao, cenas) in &porques {
        eprintln!("  {:>3} · {razao}", cenas.len());
        eprintln!("        {cenas:?}");
    }
    eprintln!(
        "  na placa │ {placa} de {total} ({:.1}%)",
        f64::from(placa) * 100.0 / f64::from(total.max(1))
    );
}
