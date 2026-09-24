//! Os portões da cena `=17` (a escada dos tectos) — a rota que ela mede, a regra do dono, e as
//! duas portas de linha de comando.

use super::{N_OMISSAO, build, forma_por, populacao_por};
use crate::motion_bridge::gpu::{GpuRoute, gpu_route};
use crate::motion_state::MotionState;

/// A rota e os NOMES da fronteira do plano da escada.
fn rota(estrela: bool) -> (GpuRoute, Vec<String>) {
    let mut m = MotionState::new();
    let sink = build(&mut m.doc.graph, "Particle", N_OMISSAO, estrela);
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    let plan = ph2d_gpu_cook::plan_driven_many(
        &m.doc.graph,
        &m.registry,
        &m.registry,
        &[sink],
        &ph2d_gpu_cook::DrivenParams::new(),
    );
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let fronteira = plan
        .boundaries
        .iter()
        .filter_map(|&(id, _)| m.doc.graph.node(id).map(|n| n.type_name.clone()))
        .collect();
    (
        gpu_route(
            true,
            1,
            scopes.is_empty(),
            &plan.boundaries,
            plan.dispatching_stages(&m.registry),
        ),
        fronteira,
    )
}

/// ⭐⭐⭐ **A ESCADA PÕE A SIMULAÇÃO NA PLACA: a única fronteira é a FONTE da forma.**
///
/// ⛔ **A premissa deste gate MORREU no ciclo 12 (doc 120 §8), à vista:** ele chamava-se
/// `a_escada_mede_a_rota_hibrida_com_as_duas_formas` e afirmava que o `motion.duplicator` era a
/// fronteira — o que arrastava o emissor, o integrador e as três forças para a CPU (`~80 %` do
/// cozimento, medido pela `sonda_onde_a_escada_gasta`). Com o carimbo no dispositivo a rota continua
/// HÍBRIDA, mas o prefixo da CPU é **só a fonte**: `source.object` (o objecto que a app publica) ou
/// `source.shape` (a estrela). ⚠️ A estrela continua recusada UMA camada acima, pela ponte (o vector
/// vivo, ADR-0154) — isso é o desenho, e não este plano.
#[test]
fn a_escada_poe_a_simulacao_na_placa_e_so_a_fonte_fica_na_cpu() {
    for (estrela, fonte) in [(false, "source.object"), (true, "source.shape")] {
        let (r, fronteira) = rota(estrela);
        assert!(
            matches!(r, GpuRoute::Hybrid),
            "estrela={estrela}: a fonte e' CPU, logo a rota e' HIBRIDA; rota {r:?}"
        );
        assert_eq!(
            fronteira,
            vec![fonte.to_string()],
            "estrela={estrela}: a unica fronteira e' a fonte — o carimbo e a simulacao vao a' placa"
        );
    }
}

/// ⭐ **A regra do dono, afirmada na própria cena:** uma FORMA e SIMULAÇÃO com CAMPOS.
#[test]
fn a_escada_tem_forma_e_simulacao_com_campos() {
    for estrela in [false, true] {
        let mut m = MotionState::new();
        build(&mut m.doc.graph, "Particle", N_OMISSAO, estrela);
        let tipos: Vec<String> = m
            .doc
            .graph
            .nodes()
            .iter()
            .map(|n| n.type_name.clone())
            .collect();
        let conta = |t: &str| tipos.iter().filter(|x| x.as_str() == t).count();
        assert_eq!(
            conta("source.object") + conta("source.shape"),
            1,
            "uma FORMA"
        );
        assert_eq!(conta("motion.integrate"), 1, "uma simulação");
        assert!(
            tipos.iter().filter(|t| t.starts_with("force.")).count() >= 2,
            "a simulação tem campos de força"
        );
    }
}

/// As portas de linha de comando: ausente/lixo/zero ⇒ a omissão; um número ⇒ ele próprio.
#[test]
fn as_portas_da_escada() {
    for v in [None, Some(""), Some("abc"), Some("0"), Some("-5")] {
        assert_eq!(populacao_por(v), N_OMISSAO, "{v:?}");
    }
    assert_eq!(populacao_por(Some(" 262144 ")), 262_144);
    assert!(!forma_por(None) && !forma_por(Some("0")) && forma_por(Some("1")));
}

/// ⚡ **ONDE O COZIMENTO DA ESCADA GASTA — a simulação ou o carimbo?** (ciclo 12 → ciclo 10 W1(b)).
///
/// A escada do doc 120 mostra o tecto confortável das IMAGENS preso na coluna «Motion (cozer)» da rota
/// híbrida, e a rota híbrida corre na CPU tudo o que está ANTES da fronteira — a simulação **e** o
/// carimbo. ⇒ antes de escolher a cura, a pergunta é *qual das duas metades pesa*: se é o carimbo,
/// dar-lhe um kernel é a wave; se é a simulação, o carimbo na placa não compra nada.
///
/// Pela porta do QUADRO (`pump`, tique a andar — o memo chaveado pelo tique leria `0,002 ms` num
/// tique parado), com a população CHEIA (`VIDA` segundos de tiques antes de medir). O controlo é a
/// MESMA simulação sem o carimbo: o `integrate` liga directamente ao `move`.
#[test]
#[ignore = "sonda de medição — corra à mão em --release, com a máquina calma"]
fn sonda_onde_a_escada_gasta() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use std::time::Instant;
    eprintln!("\n  ═══ ONDE A ESCADA GASTA (CPU, pump, população cheia) ═══\n");
    eprintln!("  pedidos | variante         | linhas  | melhor tique | fronteira");
    for n in [16_384u32, 32_768, 65_536] {
        for com_carimbo in [false, true] {
            let mut m = MotionState::new();
            // O objecto que o `source.object` lê — uma imagem do atlas, sem geometria viva.
            m.pump.cook.set_external(
                "Particle",
                Stream::new(1)
                    .with("texture_id", Column::Scalar(vec![1.0]))
                    .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]]))
                    .with("size", Column::Vec2(vec![[0.08, 0.08]])),
            );
            let saida = build(&mut m.doc.graph, "Particle", n, false);
            if !com_carimbo {
                // O controlo: a simulação vai direita ao `move`, sem o carimbo.
                let g = &mut m.doc.graph;
                let mv = g
                    .nodes()
                    .iter()
                    .find(|x| x.type_name == "motion.move")
                    .map(|x| x.id);
                let ig = g
                    .nodes()
                    .iter()
                    .find(|x| x.type_name == "motion.integrate")
                    .map(|x| x.id);
                let (mv, ig) = (mv.expect("move"), ig.expect("integrate"));
                g.disconnect(mv, 0).expect("o move tinha entrada");
                g.connect(ph2d_nodegraph::graph::Edge {
                    from: (ig, 0),
                    to: (mv, 0),
                    delayed: false,
                })
                .expect("integrate -> move");
            }
            let plano = ph2d_gpu_cook::plan_driven(
                &m.doc.graph,
                &m.registry,
                &m.registry,
                saida,
                &ph2d_gpu_cook::DrivenParams::new(),
            );
            let fronteira: Vec<String> = plano
                .boundaries
                .iter()
                .filter_map(|&(id, _)| m.doc.graph.node(id).map(|x| x.type_name.clone()))
                .collect();
            let uv = [0.0, 0.0, 1.0, 1.0];
            let tam = [1.0, 1.0];
            let mut t = 0u64;
            let mut marcha = |m: &mut MotionState| {
                let ph = f64::from(u32::try_from(t).unwrap_or(0)) / 60.0;
                m.pump.mark_dirty();
                let ok = m
                    .pump
                    .pump(&m.doc.graph, &m.registry, &[saida], t, ph, uv, tam);
                t += 1;
                ok
            };
            // `VIDA` segundos a 60 Hz: a população enche.
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "segundos pequenos"
            )]
            let encher = (super::VIDA * 60.0) as usize + 10;
            for _ in 0..encher {
                marcha(&mut m);
            }
            let mut melhor = f64::INFINITY;
            for _ in 0..12 {
                let t0 = Instant::now();
                assert!(marcha(&mut m), "o quadro tem de cozinhar");
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            eprintln!(
                "  {n:>7} | {:<16} | {:>7} | {melhor:>9.3} ms | {}",
                if com_carimbo {
                    "sim + carimbo"
                } else {
                    "só a simulação"
                },
                m.pump.instances.len(),
                if fronteira.is_empty() {
                    "-".into()
                } else {
                    fronteira.join(", ")
                },
            );
        }
    }
    eprintln!(
        "\n  load: {}\n",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
    );
}
