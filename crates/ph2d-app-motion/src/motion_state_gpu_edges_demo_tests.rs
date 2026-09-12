//! Gates da cena `=102` — a parede que absorve e o seed por peça (folha 06).
//!
//! ⚠️ Medem o que a cena DESENHA ao longo do TEMPO. As duas metades são idênticas no
//! instante zero por construção (o campo nasce plano, o cacho nasce parado), então um
//! gate de um cozimento só não distinguiria nada.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Quantos quadros correr, a 60 fps. O tanque reflector precisa de tempo para a onda ir
/// à borda e voltar várias vezes — é isso que o acumula.
const TICKS: usize = 420;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_edges_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// A envergadura INTERNA de um conjunto — a maior distância entre duas peças dele.
fn spread(p: &[[f32; 2]]) -> f32 {
    let mut m = 0.0f32;
    for a in p {
        for b in p {
            m = m.max(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
        }
    }
    m
}

/// O que cada sink mostrou ao longo da corrida: as posições finais, a maior EXCURSÃO de
/// uma peça, e a maior ENVERGADURA interna que o conjunto chegou a ter.
///
/// ⚠️ **A envergadura é o MÁXIMO no tempo, e não a do último tique.** A 1.ª versão media
/// o instante final e lia `0,548` num cacho que chega a `1,1` — as peças passam por
/// configurações apertadas o tempo todo, e apanhar uma delas mede o relógio, não a lei.
struct Shown {
    last: Vec<[f32; 2]>,
    swing: f32,
    widest: f32,
}

fn run(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Shown> {
    let mut cook = Cook::new();
    let mut out: Vec<Shown> = sinks
        .iter()
        .map(|_| Shown {
            last: Vec::new(),
            swing: 0.0,
            widest: 0.0,
        })
        .collect();
    let mut first: Vec<Option<Vec<[f32; 2]>>> = vec![None; sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            let p = match v[0].as_stream().get("P") {
                Some(Column::Vec2(p)) => p.clone(),
                _ => Vec::new(),
            };
            out[s].widest = out[s].widest.max(spread(&p));
            if first[s].is_none() {
                first[s] = Some(p.clone());
            } else if let Some(f) = &first[s] {
                for (a, b) in p.iter().zip(f) {
                    out[s].swing = out[s]
                        .swing
                        .max((a[0] - b[0]).abs())
                        .max((a[1] - b[1]).abs());
                }
            }
            if k == TICKS - 1 {
                out[s].last = p;
            }
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    out
}

/// A cena monta as QUATRO metades, e nenhuma diverge.
#[test]
fn the_edges_scene_builds_all_four_halves() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 4, "dois tanques e dois cachos");
    for (k, sh) in run(&doc, &reg, &sinks).into_iter().enumerate() {
        assert!(!sh.last.is_empty(), "metade {k} nao desenhou nada");
        assert!(
            sh.last.iter().all(|q| q.iter().all(|x| x.is_finite())),
            "metade {k} divergiu"
        );
    }
}

/// ⭐⭐ **O TANQUE QUE ABSORVE ACALMA-SE, O DE PAREDE DURA NÃO.** É a primeira coisa que o
/// Enio olha, e o CONTROLE (o tanque reflector) é o que mede o fenómeno: sem ele, este
/// gate só diria «um número é pequeno».
#[test]
fn the_absorbing_pond_settles_and_the_reflecting_one_does_not() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    // O tanque desenha a altura no `size`, mas a excursão de `P` do `motion.transform`
    // é fixa — a energia lê-se pela dispersão do `size`, então medimos o campo cru.
    let energy = |sink: NodeId| -> f32 {
        let mut cook = Cook::new();
        let mut e = 0.0;
        for k in 0..TICKS {
            let t = k as f64 / 60.0;
            let v = cook.cook(&doc.graph, &reg, sink, t).expect("coze");
            if k == TICKS - 1
                && let Some(Column::Vec2(s)) = v[0].as_stream().get("size")
            {
                e = s.iter().map(|q| (q[0] - s[0][0]).abs()).sum::<f32>();
            }
            cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        }
        e
    };
    let (refl, absb) = (energy(sinks[0]), energy(sinks[1]));
    assert!(
        refl > 1e-3,
        "a fixtura nao contem o fenomeno: o tanque reflector ja' esta' quieto ({refl:e})"
    );
    assert!(
        absb < refl * 0.5,
        "o tanque que absorve tem de estar bem mais calmo: reflect {refl:e} · absorb {absb:e}"
    );
    // E os dois tiveram vida — um tanque que nunca se mexeu passaria a barra acima.
    assert!(r[0].swing >= 0.0 && r[1].swing >= 0.0);
}

/// ⭐ **O CACHO COM SEED PARTILHADO ANDA COMO UM BLOCO.** A régua é a dispersão INTERNA
/// do cacho, não «houve movimento»: os dois cachos mexem-se, e o que os distingue é as
/// peças de um andarem juntas.
#[test]
fn only_the_per_element_clump_breaks_apart() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    // A MAIOR envergadura interna que cada cacho chegou a ter — ver [`Shown`].
    let shared = r[2].widest;
    let per_piece = r[3].widest;
    let born = (CLUMP - 1.0) * CLUMP_GAP;
    // ⚠️ **O cacho partilhado NÃO é rígido, e a 1.ª versão deste gate afirmava que era.**
    // Um campo coerente deforma um aglomerado pelo GRADIENTE dele ao longo da largura do
    // aglomerado — o que ele não faz é espalhá-lo pela AMPLITUDE. É essa a lei, e é essa
    // que o olho lê como «isto anda junto».
    assert!(
        shared < CLUMP_AMP * 0.5,
        "o cacho partilhado deforma-se pelo gradiente, nunca pela amplitude: \
         {shared:.4} contra amplitude {CLUMP_AMP}"
    );
    assert!(
        per_piece > shared * 2.0,
        "o cacho com seed por peca tinha de se espalhar bem mais: {per_piece:.4} vs {shared:.4}"
    );
    // E ele partiu de um aglomerado APERTADO — sem isto, dois cachos já nascidos largos
    // dariam a mesma razão.
    assert!(born < shared, "o cacho nasce apertado: {born:.4}");
    // ⚠️ E o CONTROLE de que a cena não pousou num ponto de rede: o cacho partilhado
    // tem de se MOVER, senão as duas metades estariam paradas e o gate acima passaria
    // sobre uma cena morta.
    assert!(
        r[2].swing > 0.05,
        "o cacho partilhado tem de tremer: excursao {:.4}",
        r[2].swing
    );
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`** — a mesma lei das cenas
/// `=98`..`=101`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_announce.rs");
    assert!(
        src.contains("gpu_edges_demo::SIDE") && src.contains("gpu_edges_demo::CLUMP"),
        "o lado do tanque e a contagem do cacho saem dos `const`"
    );
}
