//! Os gates da cena `=115` — o MATERIAL, e a prova de que ela ensina o que anuncia (doc 109 §7).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu** — as bolas são `source.shape`, que lê um EXTERNAL
//! que a shell publica: num cook virgem ele emite **zero** e os gates mediriam nada. É o mesmo
//! precedente que a `=110` e a `=114` já nomeiam.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// O que uma corrida devolve por quadrante: onde a bola começou e acabou, e o ÂNGULO dela no fim.
struct Corrida {
    inicio: Vec<[f32; 2]>,
    fim: Vec<[f32; 2]>,
    /// O `y` mais alto que a bola alcançou DEPOIS do primeiro toque — a régua do salto.
    pico_depois_do_toque: Vec<f32>,
    rot: Vec<f32>,
}

/// Monta a cena, deixa `mexe` ajustar os cartões e corre `secs` segundos.
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
    assert_eq!(formas.len(), 4, "quatro bolas");
    mexe(&mut state, &formas);
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let last = (secs * 60.0) as u64;
    let n = sinks.len();
    let mut c = Corrida {
        inicio: vec![[0.0; 2]; n],
        fim: vec![[0.0; 2]; n],
        pico_depois_do_toque: vec![f32::MIN; n],
        rot: vec![0.0; n],
    };
    // O `y` mínimo já visto por quadrante: o pico só conta DEPOIS de a bola ter descido.
    let mut fundo = vec![f32::MAX; n];
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
            if let Some(Column::Vec2(v)) = s.get("P")
                && let Some(p) = v.first()
            {
                if k == 0 {
                    c.inicio[i] = *p;
                }
                if k == last {
                    c.fim[i] = *p;
                }
                fundo[i] = fundo[i].min(p[1]);
                if p[1] < fundo[i] + 1e-4 {
                    // Ainda a descer (ou no ponto mais baixo): o salto conta a partir daqui.
                    c.pico_depois_do_toque[i] = p[1];
                } else {
                    c.pico_depois_do_toque[i] = c.pico_depois_do_toque[i].max(p[1]);
                }
            }
            if k == last
                && let Some(Column::Scalar(r)) = s.get("rot")
                && let Some(a) = r.first()
            {
                c.rot[i] = *a;
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

/// Os índices dos quadrantes, na ordem de [`quadrantes`].
const RAMPA_GELO: usize = 0;
const RAMPA_ATRITO: usize = 1;
const QUEDA_MORTA: usize = 2;
const QUEDA_VIVA: usize = 3;

/// ⭐⭐⭐ **A RESPOSTA AO REPORT, NA CENA** (*«os círculos não rotacionam com a colisão»*): a mesma
/// bola na mesma rampa RODA com `Friction 1` e NÃO roda com `Friction 0`.
///
/// ⚠️ **E o `Friction 0` é o controlo que vale mais que a metade que roda**: sem ele o gate ficava
/// verde sobre uma cena que gira tudo sempre, que é o defeito oposto e igualmente mudo.
#[test]
fn the_ball_with_friction_rolls_and_the_icy_one_does_not() {
    let c = corre(2.0, |_, _| {});
    let (gelo, rola) = (c.rot[RAMPA_GELO], c.rot[RAMPA_ATRITO]);
    let desceu = |i: usize| c.inicio[i][0] - c.fim[i][0];
    eprintln!(
        "  rampa │ gelo {gelo:.1}° (andou {:.2}) · atrito {rola:.1}° (andou {:.2})",
        desceu(RAMPA_GELO),
        desceu(RAMPA_ATRITO)
    );
    assert!(
        desceu(RAMPA_GELO) > 0.3 && desceu(RAMPA_ATRITO) > 0.3,
        "as duas bolas tem de DESCER a rampa"
    );
    // ⚠️ **`1e-3°` e não zero ao bit, e o número tem MECANISMO:** a alavanca da normal num disco é
    // zero em aritmética exacta, mas o `ponto − centro` de uma bola a `x = 1,35` é uma subtracção
    // de dois números `30×` maiores do que a diferença, e o que sobra é ruído de cancelamento
    // (`~1e-7` por tique). Medido nesta cena: **`0,0000075°` em 2 s** — sete milionésimos de grau.
    // ⛔ *Uma barra de zero exacto aqui mediria a aritmética, não a lei.*
    assert!(
        gelo.abs() < 1e-3,
        "com `Friction 0` a bola nao pode rodar, e rodou {gelo}°"
    );
    assert!(
        rola.abs() > 90.0,
        "com `Friction 1` a bola tem de RODAR (mais de um quarto de volta), e rodou {rola}°"
    );
    // ⭐ E no SENTIDO certo: a descer para `−x` ela roda no sentido ANTI-horário (graus > 0).
    assert!(
        rola > 0.0,
        "a rodar ao contrario do que a descida pede: {rola}"
    );
}

/// ⭐⭐ **E o que ela rodou é o que ROLAR pede** — o arco `andou / raio`, e não um número qualquer.
///
/// ⚠️ A barra é larga (`±35 %`) de propósito: a bola começa a derrapar e só depois agarra, então o
/// arco total fica ABAIXO do rolamento puro. O que este gate exclui é a outra ordem de grandeza —
/// um giro decorativo que não tem nada a ver com a distância percorrida.
#[test]
fn what_it_turns_is_what_rolling_asks_for() {
    let c = corre(2.0, |_, _| {});
    let andou = (c.inicio[RAMPA_ATRITO][0] - c.fim[RAMPA_ATRITO][0])
        .hypot(c.inicio[RAMPA_ATRITO][1] - c.fim[RAMPA_ATRITO][1]);
    let pedido = andou / RAIO * ph2d_contact::GRAUS;
    let razao = c.rot[RAMPA_ATRITO] / pedido;
    eprintln!(
        "  rolamento │ andou {andou:.3} · pedia {pedido:.0}° · rodou {:.0}° ({razao:.2}x)",
        c.rot[RAMPA_ATRITO]
    );
    assert!(
        (0.65..=1.35).contains(&razao),
        "o giro tem de ser o ARCO que a bola andou: pedia {pedido:.0}°, rodou {:.0}°",
        c.rot[RAMPA_ATRITO]
    );
}

/// ⭐⭐ **A BOLA SALTITANTE SALTA, E A MORTA NÃO** — a outra metade do material.
#[test]
fn the_bouncy_ball_comes_back_up_and_the_dead_one_stays_down() {
    let c = corre(2.0, |_, _| {});
    let sobe = |i: usize| c.pico_depois_do_toque[i] - c.fim[i][1].min(c.pico_depois_do_toque[i]);
    let (morta, viva) = (
        c.pico_depois_do_toque[QUEDA_MORTA],
        c.pico_depois_do_toque[QUEDA_VIVA],
    );
    eprintln!("  salto │ Bounciness 0 pico {morta:.3} · 0,9 pico {viva:.3} (chao {CHAO_Y})");
    let _ = sobe;
    assert!(
        morta < CHAO_Y + RAIO + 0.05,
        "com `Bounciness 0` a bola tem de MORRER no chao, e subiu ate' {morta:.3}"
    );
    assert!(
        viva > CHAO_Y + RAIO + 0.4,
        "com `Bounciness 0,9` ela tem de SALTAR, e o pico foi {viva:.3}"
    );
}

/// ⭐⭐⭐ **OS DOIS CONTROLOS QUE O ANÚNCIO MANDA MEXER FAZEM O QUE ELE DIZ.**
///
/// ⚠️ Um passo de smoke que manda arrastar um slider **afirma que ele tem efeito** — e esta casa já
/// pagou por escrever um passo impossível.
#[test]
fn the_two_material_sliders_do_what_the_announcement_says() {
    // `Friction` a 0 na bola que rolava: ela passa a escorregar sem virar.
    let travado = corre(2.0, |state, formas| {
        state
            .doc
            .graph
            .set_param(formas[RAMPA_ATRITO], param::FRICTION, 0.0);
    });
    eprintln!("  Friction 1 -> 0 │ rot {}°", travado.rot[RAMPA_ATRITO]);
    // A mesma barra de ruído do gate acima — ver o mecanismo lá.
    assert!(
        travado.rot[RAMPA_ATRITO].abs() < 1e-3,
        "arrastar `Friction` a 0 tem de parar a rotacao, e sobrou {}°",
        travado.rot[RAMPA_ATRITO]
    );

    // `Bounciness` a 0 na bola que saltava: ela passa a morrer no chão.
    let morta = corre(2.0, |state, formas| {
        state
            .doc
            .graph
            .set_param(formas[QUEDA_VIVA], param::BOUNCE, 0.0);
    });
    eprintln!(
        "  Bounciness 0,9 -> 0 │ pico {:.3}",
        morta.pico_depois_do_toque[QUEDA_VIVA]
    );
    assert!(
        morta.pico_depois_do_toque[QUEDA_VIVA] < CHAO_Y + RAIO + 0.05,
        "arrastar `Bounciness` a 0 tem de matar o salto"
    );
}

/// ⭐⭐⭐ **O QUE DIFERE ENTRE AS DUAS METADES DE UM PAR É UM PARAM DA FORMA — e mais nada.**
///
/// ⚠️ Se a diferença estivesse no OBSTÁCULO, a cena ensinaria que o material é do mundo. Ele é da
/// PEÇA, e é isso que este gate prende: os quatro `sim.collide` têm o mesmo atrito e a mesma
/// restituição, e os quatro `sim.zone` o mesmo relógio.
#[test]
fn only_the_shape_card_differs_between_the_halves_of_a_pair() {
    let mut state = MotionState::new();
    let _ = build(&mut state.doc, &state.registry).expect("a cena monta");
    let g = &state.doc.graph;
    let numero = |tipo: &str, p: &str| -> Vec<f32> {
        g.nodes()
            .iter()
            .filter(|n| n.type_name == tipo)
            .map(|n| {
                g.node_param_overrides(n.id)
                    .and_then(|o| o.get(p).copied())
                    .unwrap_or(f32::NAN)
            })
            .collect()
    };
    for p in ["friction", "restitution"] {
        let v = numero("sim.collide", p);
        assert_eq!(v.len(), 4, "quatro obstaculos");
        assert!(
            v.windows(2).all(|w| w[0] == w[1]),
            "o `{p}` do obstaculo tem de ser o MESMO nos quatro: {v:?}"
        );
    }
    let dur = numero("sim.zone", "duration");
    assert!(
        dur.windows(2).all(|w| w[0] == w[1]),
        "o relogio tem de ser o mesmo: {dur:?}"
    );
    // E os cartões das formas diferem exactamente nos dois números da pergunta.
    let (at, sal) = (
        numero("source.shape", param::FRICTION),
        numero("source.shape", param::BOUNCE),
    );
    assert_eq!(at[RAMPA_GELO], 0.0);
    assert_eq!(at[RAMPA_ATRITO], 1.0);
    assert_eq!(sal[QUEDA_MORTA], 0.0);
    assert!(sal[QUEDA_VIVA] > 0.5);
}

/// O anúncio da cena, lido como TEXTO — os nomes que o gate defende e os que o dono lê não podem
/// divergir em silêncio.
const ANUNCIO: &str = include_str!("motion_state_demo_announce.rs");

/// ⭐⭐⭐ **CADA LINHA QUE O ANÚNCIO MANDA CLICAR ESTÁ NO CARTÃO.**
///
/// ⛔⛔ Um passo que manda clicar numa linha **afirma que ela está lá**. Montada pelo ROTEADOR
/// (`monta("115")`), que é o que o smoke abre.
#[test]
fn every_row_the_announcement_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("115", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    for titulo in ["Friction 1: ROLA", "Bounciness 0,9: SALTA"] {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!("o anuncio manda clicar no cartao `{titulo}` e a cena tem: {havia:?}")
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        let secs: Vec<&str> = v.sections.iter().map(|s| s.title).collect();
        for linha in ["Friction", "Bounciness"] {
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
            secs.contains(&"Collision"),
            "o anuncio manda abrir a seccao `Collision`, e o cartao tem {secs:?}"
        );
        assert!(
            ANUNCIO.contains(titulo),
            "o anuncio tem de nomear o cartao `{titulo}`"
        );
    }
}
