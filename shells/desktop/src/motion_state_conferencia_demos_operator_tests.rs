//! Gates da cena `=77` (doc 89, folha 07).
//!
//! ⚠️ **A leitura da cena é uma SOMA de cores na tela, e nenhum gate headless a vê.** O que
//! se pode provar aqui é o que a torna capaz de a mostrar: que as metades autoram operadores
//! DIFERENTES, que o caminho de facto se CRUZA (sem cruzamento não há o que somar, e as duas
//! metades sairiam iguais com o produto correcto), e que a coluna chega ao stream.

use super::{ADD, build_operator_demo_document};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// **AS DUAS METADES DE CADA LINHA AUTORAM OPERADORES DIFERENTES.**
#[test]
fn each_row_authors_two_different_operators() {
    let mut state = MotionState::new();
    let _ = build_operator_demo_document(&mut state.doc, &state.registry).expect("a cena monta");
    for (ty, param) in [
        ("motion.trail", ph2d_node_motion_trail::ECHO_BLEND),
        ("motion.strobe", ph2d_node_motion_strobe::FLASH_BLEND),
    ] {
        let vals: Vec<f32> = state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == ty)
            .map(|n| {
                state
                    .doc
                    .graph
                    .node_param_overrides(n.id)
                    .and_then(|m| m.get(param).copied())
                    .unwrap_or(0.0)
            })
            .collect();
        assert_eq!(vals.len(), 2, "{ty}: duas bandas");
        assert!(vals.contains(&0.0), "{ty}: uma delas tem de ser o `Sink`");
        assert!(vals.contains(&ADD), "{ty}: e a outra `Add`");
    }
}

/// **A COLUNA CHEGA AO STREAM, E SÓ NA METADE QUE A PEDIU.**
///
/// ⚠️ O oráculo é o STREAM COZIDO, não o param: um param que o `eval` lesse e não usasse
/// deixaria este gate verde se ele olhasse a tabela de params (foi o modo de falha de três
/// mutações desta linha).
#[test]
fn only_the_chosen_half_carries_the_blend_column() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    // sinks: [rastro esquerdo, rastro direito, flash esquerdo, flash direito]
    let carries = |state: &mut MotionState, sink: ph2d_nodegraph::graph::NodeId| -> Option<f32> {
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.5)
            .expect("coze");
        match out[0].as_stream().get(ph2d_node_motion_trail::BLEND_COLUMN) {
            Some(Column::Scalar(v)) if !v.is_empty() => Some(v[0]),
            _ => None,
        }
    };
    for pair in [[sinks[0], sinks[1]], [sinks[2], sinks[3]]] {
        assert_eq!(carries(&mut state, pair[0]), None, "a esquerda nao escolhe");
        assert_eq!(carries(&mut state, pair[1]), Some(ADD), "a direita escolhe");
    }
}

/// **O CAMINHO CRUZA-SE A SI PRÓPRIO** — sem isto a cena não tem o que somar.
///
/// ⚠️ **É a metade que separa esta cena de uma que passaria com o produto CERTO e não
/// mostraria nada.** Uma órbita circular nunca se atravessa: os ecos ficariam lado a lado,
/// `Add` e `Normal` desenhariam o mesmo, e o smoke reprovaria a feature por causa da cena.
/// O oito é medido pelo X voltar ao mesmo sítio com o Y noutro.
#[test]
fn the_path_crosses_itself_so_the_echoes_have_something_to_add() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    let at = |state: &mut MotionState, t: f64| -> [f32; 2] {
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[0], t)
            .expect("coze");
        let Some(Column::Vec2(p)) = out[0].as_stream().get("P") else {
            panic!("P");
        };
        p[0]
    };
    // Varre um período inteiro e procura DOIS instantes com o mesmo x e y diferente — a
    // assinatura de um oito, e a de nada mais que este caminho pudesse ser.
    const STEPS: usize = 240;
    let period = 1.0 / f64::from(super::LOOP_HZ_FOR_TEST);
    let pts: Vec<[f32; 2]> = (0..STEPS)
        .map(|k| at(&mut state, period * k as f64 / STEPS as f64))
        .collect();
    let crossed = pts.iter().enumerate().any(|(i, a)| {
        pts.iter()
            .skip(i + STEPS / 8)
            .any(|b| (a[0] - b[0]).abs() < 0.02 && (a[1] - b[1]).abs() > 0.15)
    });
    assert!(
        crossed,
        "o caminho tem de se cruzar — senao `Add` e `Normal` desenham o mesmo"
    );
}

/// **O FLASH DE FACTO DISPARA** — a metade que os outros gates desta cena não viam.
///
/// ⚠️ **Nasceu de um smoke reprovado** (Enio, 2026-08-22: *"o flash não é evidente, talvez
/// por questão de intensidade"*). Não era a intensidade: o `pulse.beat` não lia a geometria
/// e não tinha laço `pre`, então emitia ZERO linhas e o strobe **nunca acendia**. Os gates
/// que existiam ficaram verdes — eles mediam que o param estava autorado e que a coluna
/// chegava ao stream, e as duas coisas eram verdade com a cena parada. *Um param autorado
/// não é um efeito acontecido.*
///
/// O oráculo é o TINT no pico contra o tint em repouso, medido no stream cozido.
#[test]
fn the_flash_actually_fires() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    let peak = peak_tint(&mut state, sinks[3]);
    let ink = super::INK_FOR_TEST;
    let moved = (0..3)
        .map(|k| (peak[k] - ink[k]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        moved > 0.25,
        "o flash tem de MOVER o tint: repouso {ink:?} → pico {peak:?}"
    );
}

/// **E O PICO TEM FOLGA PARA SOMAR** — a lei que este arquivo já escrevia para o rastro, e
/// que a 1ª versão da linha do flash violou.
///
/// Um `flash` branco a `amount = 1` põe a peça em branco SATURADO, e branco somado a branco
/// continua branco: `Add` e `Normal` desenhariam o mesmo com o produto perfeitamente correcto.
#[test]
fn the_peak_leaves_headroom_for_the_sum() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    let peak = peak_tint(&mut state, sinks[3]);
    for (k, v) in peak.iter().take(3).enumerate() {
        assert!(
            *v < 0.95,
            "o canal {k} chegou a {v}: sem folga, somar nao muda nada"
        );
    }
}

/// **AS PEÇAS DO FLASH SOBREPÕEM-SE** — sem isto a soma não tem onde aparecer.
///
/// ⚠️ **É o irmão exacto do gate do oito do rastro**, e a mesma lei: `Add` só difere de
/// `Normal` onde há sobreposição. A 1ª versão desta linha tinha UMA peça — as duas metades
/// eram forçosamente iguais, e o produto estava certo.
///
/// O oráculo é a geometria COZIDA (a distância entre vizinhas contra o tamanho delas), nunca
/// a fórmula do anel: um `motion.scale` esquecido mudaria o tamanho sem mudar a fórmula.
#[test]
fn the_flash_pieces_overlap_so_the_sum_has_somewhere_to_show() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sinks[3], 0.0)
        .expect("coze");
    let s = out[0].as_stream();
    let Some(Column::Vec2(p)) = s.get("P") else {
        panic!("P")
    };
    let Some(Column::Vec2(sz)) = s.get("size") else {
        panic!("size")
    };
    assert!(p.len() >= 3, "uma roseta, nao uma peca: {}", p.len());
    // A menor distancia entre duas pecas tem de ser MENOR que o tamanho delas.
    let mut closest = f32::INFINITY;
    for (i, a) in p.iter().enumerate() {
        for b in p.iter().skip(i + 1) {
            closest = closest.min((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    let width = sz[0][0];
    assert!(
        closest < width,
        "as pecas nao se tocam ({closest} de distancia contra {width} de largura) — \
         `Add` e `Normal` desenhariam o mesmo"
    );
}

/// O tint no PICO do envelope, varrendo dois períodos do metrónomo.
fn peak_tint(state: &mut MotionState, sink: ph2d_nodegraph::graph::NodeId) -> [f32; 4] {
    let mut best = ([0.0f32; 4], 0.0f32);
    for k in 0..120 {
        let t = f64::from(k) / 60.0;
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("coze");
        let tint = match out[0].as_stream().get("tint") {
            Some(Column::Vec4(v)) if !v.is_empty() => v[0],
            _ => [1.0; 4],
        };
        let lum = tint[0] + tint[1] + tint[2];
        if lum > best.1 {
            best = (tint, lum);
        }
    }
    best.0
}
