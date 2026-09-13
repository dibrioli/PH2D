//! Os gates da cena `=114` — e a prova de que ela ensina o que anuncia (doc 109 W3).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu.** As peças são `source.shape`, que lê um EXTERNAL
//! que a shell publica (`motion_shape_gen::publish`): num cook virgem ele emite **zero**, e as duas
//! taças sairiam vazias com os gates a medir nada — o precedente que a cena `=110` já nomeia.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{COLLIDER_COLUMN, Column};

/// As posições de cada metade num instante — uma nuvem por metade.
type PorMetade = Vec<Vec<[f32; 2]>>;

/// O que uma corrida devolve: as nuvens no instante zero e no fim, e o raio de colisão de MUNDO
/// que a metade da direita declarou (lido do stream COZIDO, não de uma constante).
struct Corrida {
    inicio: PorMetade,
    fim: PorMetade,
    raio_direita: Option<f32>,
}

/// Monta a cena num `MotionState`, deixa `mexe` ajustar o grafo, publica as formas e corre.
fn corre(secs: f64, mexe: impl FnOnce(&mut MotionState, &[NodeId])) -> Corrida {
    let mut state = MotionState::new();
    assert!(
        state.doc.graph.nodes().is_empty(),
        "o `MotionState` nasceu com nos: desligue o PH2D_GPU_COOK_DEMO para correr estes gates"
    );
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let formas: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .collect();
    mexe(&mut state, &formas);
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let last = (secs * 60.0) as u64;
    let mut c = Corrida {
        inicio: vec![Vec::new(); sinks.len()],
        fim: vec![Vec::new(); sinks.len()],
        raio_direita: None,
    };
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        for (i, sink) in sinks.iter().enumerate() {
            let s = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, *sink, t)
                .expect("cozinha")[0]
                .as_stream()
                .clone();
            if let Some(Column::Vec2(v)) = s.get("P") {
                if k == 0 {
                    c.inicio[i] = v.clone();
                }
                if k == last {
                    c.fim[i] = v.clone();
                }
            }
            if k == last
                && i == 1
                && let (Some(Column::Scalar(r)), Some(Column::Vec2(sz))) =
                    (s.get(COLLIDER_COLUMN), s.get("size"))
                && let (Some(r0), Some(s0)) = (r.first(), sz.first())
            {
                c.raio_direita = Some(r0 * s0[0].abs().max(s0[1].abs()));
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    c
}

/// A distância de cada peça ao vizinho MAIS PRÓXIMO dela — uma por peça.
fn vizinhos(p: &[[f32; 2]]) -> Vec<f32> {
    p.iter()
        .enumerate()
        .map(|(i, a)| {
            p.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .fold(f32::MAX, f32::min)
        })
        .collect()
}

/// A MEDIANA dessas distâncias — o vão TÍPICO da pilha (a régua que a 1.ª versão desta cena pagou:
/// o par mais próximo é um extremo, o vão típico é uma propriedade).
fn vizinho_mediano(p: &[[f32; 2]]) -> f32 {
    let mut v = vizinhos(p);
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

const PECAS: usize = (ROWS * COLS) as usize;

/// ⭐⭐⭐ **Só a metade com `Collide` mantém as peças separadas** — à distância que ELA declarou.
///
/// ⚠️ O alvo é o diâmetro do colisor lido do STREAM cozido (`collider × max|size|`), e não uma
/// constante desta cena: se a medição do contorno ou a escolha do `Fit` mudarem, o gate segue o
/// produto em vez de o contradizer.
///
/// ⚠️ **E o CONTROLO é a outra metade**, senão o gate ficava verde sobre duas taças a fazer o mesmo.
#[test]
fn only_the_half_whose_shape_collides_keeps_the_pieces_apart() {
    let c = corre(2.6, |_, _| {});
    assert_eq!(c.fim.len(), 2, "a cena tem duas metades");
    for (i, metade) in c.fim.iter().enumerate() {
        assert_eq!(
            metade.len(),
            PECAS,
            "a metade {i} tem de trazer as {PECAS} pecas -- vazia, a forma nao foi publicada"
        );
    }
    let r = c.raio_direita.expect("a metade da direita declara colisor");
    let (sem, com) = (vizinho_mediano(&c.fim[0]), vizinho_mediano(&c.fim[1]));
    eprintln!(
        "  vao tipico │ Collide off {sem:.4} · Collide on {com:.4} ({:.0}% de 2r) · r {r:.4}",
        com / (2.0 * r) * 100.0
    );
    assert!(
        com >= 2.0 * r * 0.85,
        "com Collide o vao tipico mede {com:.4} contra o diametro declarado {:.4}",
        2.0 * r
    );
    assert!(
        sem < com * 0.6,
        "sem Collide o vao tipico mede {sem:.4} contra {com:.4} -- as duas metades tem de contar \
         historias DIFERENTES"
    );
}

/// ⭐⭐ **As duas taças apanham o que cai — medido como DESCIDA de cada peça contra si própria.**
#[test]
fn both_bowls_actually_catch_what_falls() {
    let c = corre(2.6, |_, _| {});
    for (i, (a, b)) in c.inicio.iter().zip(&c.fim).enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas")]
        let n = a.len().max(1) as f32;
        let descida = a.iter().zip(b).map(|(p, q)| p[1] - q[1]).sum::<f32>() / n;
        let dentro = b
            .iter()
            .filter(|p| p[1] < TACA_Y + TACA_R && p[1] > TACA_Y - TACA_R - 0.5)
            .count();
        eprintln!("  metade {i} │ descida media {descida:.3} │ na taca {dentro}");
        assert!(
            descida > 0.5,
            "na metade {i} as pecas desceram {descida:.3}"
        );
        assert!(dentro >= 20, "na metade {i} so' {dentro} ficaram na taca");
    }
}

/// ⭐⭐⭐ **O que a ordem do dono proíbe: NENHUM nó `Collide` na linha da simulação**, e a pergunta
/// da cena é UMA caixa no cartão da forma — ligada só à direita.
#[test]
fn the_scene_has_no_collide_node_and_only_the_right_shape_collides() {
    let mut state = MotionState::new();
    let _ = build(&mut state.doc, &state.registry).expect("a cena monta");
    let g = &state.doc.graph;
    assert_eq!(
        g.nodes()
            .iter()
            .filter(|n| n.type_name == "motion.collide")
            .count(),
        0,
        "o colisor e' da FORMA: nenhum `motion.collide` pode voltar a esta cena"
    );
    let colide: Vec<bool> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| {
            g.node_param_overrides(n.id)
                .and_then(|o| o.get(param::COLLIDE).copied())
                .is_some_and(|v| v >= 0.5)
        })
        .collect();
    assert_eq!(
        colide,
        vec![false, true],
        "esquerda desligada, direita ligada"
    );
}

/// ⭐⭐⭐ **OS CONTROLOS QUE O ANÚNCIO MANDA MEXER FAZEM O QUE ELE DIZ.**
///
/// ⚠️ Um passo de smoke que manda mexer num controlo **afirma que ele tem efeito**. Aqui são três,
/// todos no cartão da forma da direita: desligar `Collide` (a pilha volta a borrão), arrastar
/// `Collider Scale` (a pilha incha, monotonamente) e trocar o `Collider Fit` para `Inside` (os
/// quadrados encostam: vão menor que com `Around`).
#[test]
fn the_controls_the_announcement_names_do_what_it_says() {
    let vao_direita = |collide: f32, fit: f32, escala: f32| {
        let c = corre(2.6, |state, formas| {
            let direita = formas[1];
            state.doc.graph.set_param(direita, param::COLLIDE, collide);
            state.doc.graph.set_param(direita, param::COLLIDER_FIT, fit);
            state
                .doc
                .graph
                .set_param(direita, param::COLLIDER_SCALE, escala);
        });
        vizinho_mediano(&c.fim[1])
    };
    let ligado = vao_direita(1.0, 0.0, 1.0);
    let desligado = vao_direita(0.0, 0.0, 1.0);
    eprintln!("  Collide     on {ligado:.4} · off {desligado:.4}");
    assert!(
        desligado < ligado * 0.6,
        "desligar a caixa tem de desfazer a pilha"
    );

    let escalas = [0.6_f32, 1.0, 1.4];
    let v: Vec<f32> = escalas.iter().map(|k| vao_direita(1.0, 0.0, *k)).collect();
    eprintln!("  Scale       {escalas:?} -> {v:?}");
    assert!(
        v[0] < v[1] && v[1] < v[2],
        "o Collider Scale incha a pilha: {v:?}"
    );

    let dentro = vao_direita(1.0, 1.0, 1.0);
    eprintln!("  Fit         Around {ligado:.4} · Inside {dentro:.4}");
    assert!(
        dentro < ligado * 0.9,
        "com Inside os quadrados encostam pelos lados: {dentro:.4} contra {ligado:.4}"
    );
}

/// O anúncio da cena, lido como TEXTO — para os nomes que o gate abaixo defende e os que o dono lê
/// não poderem divergir em silêncio.
const ANUNCIO: &str = include_str!("motion_state_demo_announce.rs");

/// ⭐⭐⭐ **CADA PASSO DO ANÚNCIO É POSSÍVEL NO APP.**
///
/// ⛔⛔ Um passo que manda clicar numa linha **afirma que ela está no cartão** — e esta casa já pagou
/// por escrever um passo impossível. Montada pelo ROTEADOR (`monta("114")`), que é o que o smoke
/// abre, e com os params carimbados no cartão como o dono os vê.
#[test]
fn every_row_the_announcement_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("114", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let titulo = "Shape (Collide)";
    let v = snap
        .nodes
        .iter()
        .find(|v| v.display_name == titulo)
        .unwrap_or_else(|| {
            let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
            panic!("o anuncio manda clicar no cartao `{titulo}` e a cena tem: {havia:?}")
        });
    let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
    for linha in ["Collide", "Collider Scale", "Collider Fit"] {
        assert!(
            rows.contains(&linha),
            "o anuncio manda mexer em `{linha}` no cartao `{titulo}`, e ele mostra {rows:?}"
        );
        assert!(
            ANUNCIO.contains(linha),
            "o gate defende `{linha}` e o anuncio nao a nomeia"
        );
    }
    let secs: Vec<&str> = v.sections.iter().map(|s| s.title).collect();
    assert!(
        secs.contains(&"Collision"),
        "o anuncio manda abrir a seccao `Collision`, e o cartao tem {secs:?}"
    );
    assert!(
        ANUNCIO.contains(titulo),
        "o anuncio tem de nomear o cartao `{titulo}`"
    );
    // E o que o `DEU ERRADO` promete: nenhum cartao `Collide` na cena.
    assert!(
        !snap.nodes.iter().any(|v| v.display_name == "Collide"),
        "apareceu um cartao `Collide` -- o colisor voltou para a linha da simulacao"
    );
}
