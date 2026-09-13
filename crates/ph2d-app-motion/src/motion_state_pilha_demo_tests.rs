//! Os gates da cena `=114` — e a prova de que ela ensina o que anuncia (doc 109 W3 e §5).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu.** As peças são `source.shape`, que lê um EXTERNAL
//! que a shell publica (`motion_shape_gen::publish`): num cook virgem ele emite **zero**, e as duas
//! taças sairiam vazias com os gates a medir nada — o precedente que a cena `=110` já nomeia.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column};

/// As posições de cada metade num instante — uma nuvem por metade.
type PorMetade = Vec<Vec<[f32; 2]>>;

/// O que uma corrida devolve: as nuvens no instante zero e no fim, e as meias extensões de MUNDO
/// da caixa que a metade da direita declarou (lidas do stream COZIDO, não de uma constante).
struct Corrida {
    inicio: PorMetade,
    fim: PorMetade,
    meia_direita: Option<[f32; 2]>,
    /// O ÂNGULO de cada peça da direita no fim — vazio quando ninguém rodou (doc 109 §6).
    rot_direita: Vec<f32>,
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
        meia_direita: None,
        rot_direita: Vec::new(),
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
                && let (Some(Column::Vec2(m)), Some(Column::Vec2(sz))) =
                    (s.get(COLLIDER_BOX_COLUMN), s.get("size"))
                && let (Some(m0), Some(s0)) = (m.first(), sz.first())
            {
                c.meia_direita = Some([m0[0] * s0[0].abs(), m0[1] * s0[1].abs()]);
            }
            if k == last
                && i == 1
                && let Some(Column::Scalar(r)) = s.get("rot")
            {
                c.rot_direita = r.clone();
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

/// Uma corrida com `params` escritos no cartão da forma da DIREITA.
fn direita_com(params: &[(&'static str, f32)]) -> Corrida {
    corre(2.6, |state, formas| {
        for (nome, valor) in params {
            state.doc.graph.set_param(formas[1], *nome, *valor);
        }
    })
}

/// **A pilha da direita com a rotação TRAVADA** — a régua de geometria da cena.
///
/// ⚠️ **Travada de propósito:** desde o doc 109 §6 as peças TOMBAM, e um quadrado a 45° toca o
/// vizinho pela quina — o vão típico de uma pilha que roda vai até à DIAGONAL, e uma barra de
/// «encostado» medida sobre ela não separaria o encosto do ar que o dono fotografou. A rotação tem
/// gate próprio (`the_pieces_tumble_unless_the_card_locks_the_rotation`).
fn pilha_travada(params: &[(&'static str, f32)]) -> Vec<[f32; 2]> {
    let mut todos = vec![(param::LOCK_ROTATION, 1.0)];
    todos.extend_from_slice(params);
    direita_com(&todos).fim[1].clone()
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

/// A LARGURA da pilha: da peça mais à esquerda à mais à direita.
fn largura(p: &[[f32; 2]]) -> f32 {
    let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
        (lo.min(q[0]), hi.max(q[0]))
    });
    hi - lo
}

const PECAS: usize = (ROWS * COLS) as usize;

/// ⭐⭐⭐ **Só a metade com `Collide` mantém as peças separadas — ENCOSTADAS, sem ar.**
///
/// ⚠️ O lado é o da caixa lida do STREAM cozido (`collider_box × |size|`), e não uma constante desta
/// cena: se a medição do contorno mudar, o gate segue o produto em vez de o contradizer.
///
/// ⛔⛔ **A barra de CIMA é o report do dono** (doc 109 §5, com foto: *«collider impreciso»*): o
/// círculo à volta do quadrado, a 1.ª redacção, assentava a `141 %` do lado — `41 %` de ar. Sem ela
/// o gate ficava verde sobre o defeito que o dono fotografou.
///
/// ⚠️ **E o CONTROLO é a outra metade**, senão o gate ficava verde sobre duas taças a fazer o mesmo.
#[test]
fn only_the_half_whose_shape_collides_keeps_the_pieces_apart() {
    // Com a rotação travada — ver [`pilha_travada`] para porquê.
    let c = direita_com(&[(param::LOCK_ROTATION, 1.0)]);
    assert_eq!(c.fim.len(), 2, "a cena tem duas metades");
    for (i, metade) in c.fim.iter().enumerate() {
        assert_eq!(
            metade.len(),
            PECAS,
            "a metade {i} tem de trazer as {PECAS} pecas -- vazia, a forma nao foi publicada"
        );
    }
    let m = c.meia_direita.expect("a metade da direita declara a caixa");
    let lado = 2.0 * m[0].max(m[1]);
    let (sem, com) = (vizinho_mediano(&c.fim[0]), vizinho_mediano(&c.fim[1]));
    eprintln!(
        "  vao tipico │ Collide off {sem:.4} · Collide on {com:.4} ({:.0}% do lado) · meia {m:?}",
        com / lado * 100.0
    );
    assert!(
        com >= lado * 0.85,
        "com Collide o vao tipico mede {com:.4} contra o lado declarado {lado:.4}"
    );
    assert!(
        com <= lado * 1.15,
        "a pilha tem AR: o vao tipico mede {com:.4} contra o lado {lado:.4} -- o defeito do report"
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
/// ⚠️ Um passo de smoke que manda mexer num controlo **afirma que ele tem efeito**. Todos no cartão
/// da forma da direita: desligar `Collide` (a pilha volta a borrão) · `Collider Width` e
/// `Collider Height` juntos (a pilha incha, monotonamente) · só `Collider Width` (a pilha ALARGA) ·
/// `Collider Shape = Circle` (os círculos também encostam, pelo raio que toca os lados) e
/// `Collider Radius` (incham).
#[test]
fn the_controls_the_announcement_names_do_what_it_says() {
    let ligado = pilha_travada(&[]);
    let desligado = pilha_travada(&[(param::COLLIDE, 0.0)]);
    let (on, off) = (vizinho_mediano(&ligado), vizinho_mediano(&desligado));
    eprintln!("  Collide     on {on:.4} · off {off:.4}");
    assert!(off < on * 0.6, "desligar a caixa tem de desfazer a pilha");

    let escalas = [0.6_f32, 1.0, 1.4];
    let v: Vec<f32> = escalas
        .iter()
        .map(|k| {
            vizinho_mediano(&pilha_travada(&[
                (param::COLLIDER_WIDTH, *k),
                (param::COLLIDER_HEIGHT, *k),
            ]))
        })
        .collect();
    eprintln!("  Width+Height {escalas:?} -> {v:?}");
    assert!(
        v[0] < v[1] && v[1] < v[2],
        "largura e altura juntas incham a pilha: {v:?}"
    );

    let larga = pilha_travada(&[(param::COLLIDER_WIDTH, 1.6)]);
    eprintln!(
        "  Width       1.0 -> largura {:.4} · 1.6 -> largura {:.4}",
        largura(&ligado),
        largura(&larga)
    );
    assert!(
        largura(&larga) > largura(&ligado) * 1.1,
        "so' a largura alarga a pilha: {:.4} contra {:.4}",
        largura(&larga),
        largura(&ligado)
    );

    let circulo = vizinho_mediano(&pilha_travada(&[(param::COLLIDER_SHAPE, 1.0)]));
    let circulo_maior = vizinho_mediano(&pilha_travada(&[
        (param::COLLIDER_SHAPE, 1.0),
        (param::COLLIDER_RADIUS, 1.4),
    ]));
    let lado = 2.0 * LADO;
    eprintln!(
        "  Circle      {circulo:.4} ({:.0}% do lado) · Radius 1.4 {circulo_maior:.4}",
        circulo / lado * 100.0
    );
    assert!(
        circulo >= lado * 0.85 && circulo <= lado * 1.15,
        "o circulo toca os lados do quadrado: {circulo:.4} contra {lado:.4}"
    );
    assert!(circulo_maior > circulo * 1.2, "o raio incha a pilha");
}

/// ⭐⭐⭐ **AS PEÇAS TOMBAM — e o botão `Lock Rotation` prende-as** (doc 109 §6, report do dono:
/// *«precisa destravar a rot. e colocar outro botão para travar rotação»*).
///
/// ⚠️ **As duas metades num gate**: *«tomba»* passa com uma cena que gira tudo sempre, e *«trava»*
/// passa com uma que nunca gira. E travada a coluna do ângulo **nem nasce** — a cena sai como saía
/// antes da rotação existir.
#[test]
fn the_pieces_tumble_unless_the_card_locks_the_rotation() {
    let solta = direita_com(&[]);
    let angulos = &solta.rot_direita;
    assert_eq!(
        angulos.len(),
        PECAS,
        "a metade que colide tem de trazer um angulo por peca"
    );
    let maior = angulos.iter().fold(0.0_f32, |m, a| m.max(a.abs()));
    let tortas = angulos.iter().filter(|a| a.abs() > 5.0).count();
    eprintln!("  rotacao │ maior {maior:.1}° · {tortas} de {PECAS} acima de 5°");
    assert!(
        maior > 15.0 && tortas >= 5,
        "as pecas tem de TOMBAR ao cair umas sobre as outras: maior {maior:.1}°, {tortas} tortas"
    );

    let travada = direita_com(&[(param::LOCK_ROTATION, 1.0)]);
    assert!(
        travada.rot_direita.is_empty(),
        "travada, a coluna do angulo nem nasce: {:?}",
        &travada.rot_direita[..travada.rot_direita.len().min(4)]
    );
}

/// O anúncio da cena, lido como TEXTO — para os nomes que o gate abaixo defende e os que o dono lê
/// não poderem divergir em silêncio.
const ANUNCIO: &str = include_str!("motion_state_demo_announce.rs");

/// As rows que o cartão `titulo` mostra, com os params como o dono os vê.
fn rows_do_cartao(m: &MotionState, titulo: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let v = snap
        .nodes
        .iter()
        .find(|v| v.display_name == titulo)
        .unwrap_or_else(|| {
            let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
            panic!("o anuncio manda clicar no cartao `{titulo}` e a cena tem: {havia:?}")
        });
    assert!(
        !snap.nodes.iter().any(|v| v.display_name == "Collide"),
        "apareceu um cartao `Collide` -- o colisor voltou para a linha da simulacao"
    );
    (
        v.params.iter().map(|c| c.hint.label).collect(),
        v.sections.iter().map(|s| s.title).collect(),
    )
}

/// ⭐⭐⭐ **CADA PASSO DO ANÚNCIO É POSSÍVEL NO APP.**
///
/// ⛔⛔ Um passo que manda clicar numa linha **afirma que ela está no cartão** — e esta casa já pagou
/// por escrever um passo impossível. Montada pelo ROTEADOR (`monta("114")`), que é o que o smoke
/// abre, e com os params carimbados no cartão como o dono os vê. ⚠️ O `Collider Radius` só existe
/// com `Circle`: o gate troca a forma como o anúncio manda, e confere que a largura sai.
#[test]
fn every_row_the_announcement_names_is_on_the_card() {
    let titulo = "Shape (Collide)";
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("114", &mut m.doc, &m.registry);
    let (rows, secs) = rows_do_cartao(&m, titulo);
    for linha in [
        "Collide",
        "Collider Shape",
        "Collider Width",
        "Collider Height",
    ] {
        assert!(
            rows.contains(&linha),
            "o anuncio manda mexer em `{linha}` no cartao `{titulo}`, e ele mostra {rows:?}"
        );
        assert!(
            ANUNCIO.contains(linha),
            "o gate defende `{linha}` e o anuncio nao a nomeia"
        );
    }
    assert!(
        !rows.contains(&"Collider Radius"),
        "com Box o raio nao aparece: {rows:?}"
    );
    assert!(
        secs.contains(&"Collision"),
        "o anuncio manda abrir a seccao `Collision`, e o cartao tem {secs:?}"
    );
    assert!(
        ANUNCIO.contains(titulo),
        "o anuncio tem de nomear o cartao `{titulo}`"
    );

    let direita = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .find(|id| {
            m.doc
                .graph
                .node_param_overrides(*id)
                .and_then(|o| o.get(param::COLLIDE).copied())
                .is_some_and(|v| v >= 0.5)
        })
        .expect("a forma da direita");
    m.doc.graph.set_param(direita, param::COLLIDER_SHAPE, 1.0);
    let (rows, _) = rows_do_cartao(&m, titulo);
    assert!(
        rows.contains(&"Collider Radius") && !rows.contains(&"Collider Width"),
        "com Circle o raio aparece e a largura sai: {rows:?}"
    );
    assert!(ANUNCIO.contains("Collider Radius"));
}
