//! **A CENA CONTÉM O FENÓMENO** — o gate que faltou à instrução de smoke anterior.
//!
//! ⚠️ O report do Enio (*"não há nenhuma animação ou movimento na cena de smoke"*) não foi um
//! defeito do produto: foi eu mandá-lo olhar para a fileira sub-UV da cena `=9`, que **não põe
//! o `speed`** e por isso é parada por construção. Uma redistribuição do tempo numa cena sem
//! tempo nenhum não muda um pixel, e nada em lado nenhum o dizia.
//!
//! ⇒ Estes gates afirmam, sobre o documento que esta cena de facto monta, as três coisas que a
//! instrução dela promete: **ela anda**, **os dois lados diferem**, e **a volta fecha ao mesmo
//! tempo dos dois lados**. *Uma promessa impressa no terminal é uma frase; isto é o que a
//! torna uma afirmação.*

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

/// Quantas células a folha desta cena tem (`cols × rows` do sub-UV).
const CELLS: f32 = 4.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    reg
}

/// A célula que cada lado mostra no instante `t` — `(esquerda, direita)`.
///
/// A `uv_cell` é `[escala_u, escala_v, desloc_u, desloc_v]` numa grelha `2 × 2`, então a
/// célula é `col + row · cols` reconstruída dos deslocamentos: a conta inversa da do nó.
///
/// ⚠️ **O cozinhador vem de FORA e é reusado.** Um `Cook::new()` por instante nunca devolve
/// nada de velho — e foi por isso que a 1.ª redacção destes gates ficou cega ao flipbook estar
/// **congelado** (o `motion.sub_uv` declarava-se `Pure` a ler o relógio). *A régua tem de ser
/// a do app, e o app reusa o cozinhador.*
fn cells_at(
    g: &ph2d_nodegraph::graph::Graph,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    cook: &mut Cook,
    t: f64,
) -> (i32, i32) {
    let read = |cook: &mut Cook, s: NodeId| -> i32 {
        let CookValue::Instances(st) = &cook.cook(g, reg, s, t).expect("coze")[0] else {
            panic!("instancias")
        };
        let Some(Column::Vec4(v)) = st.get(ph2d_node_motion_sub_uv::CELL_COLUMN) else {
            panic!("a cena tem de trazer a uv_cell — sem ela nao ha' flipbook nenhum")
        };
        let c = &v[0];
        let col = (c[2] / c[0]).round();
        let row = (c[3] / c[1]).round();
        (col + row * (1.0 / c[0]).round()) as i32
    };
    (read(cook, sinks[0]), read(cook, sinks[1]))
}

/// A MESMA metade de ritmo da cena, alimentada por uma grelha.
///
/// ⚠️ **A fonte da cena é um `source.object`**, que só devolve alguma coisa quando o app
/// publicou o objecto — cozê-la aqui leria um stream vazio e o gate não mediria nada (foi
/// exactamente o que aconteceu na 1.ª redacção destes quatro). A LEI mede-se sobre a mesma
/// função `flipbook`; o que só a cena tem é afirmado no gate do grafo REAL, mais abaixo.
fn scene() -> (ph2d_nodegraph::graph::Graph, Vec<NodeId>) {
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let mut sinks = Vec::new();
    for (k, holds) in [None, Some(HOLDS)].into_iter().enumerate() {
        let src = g.add_node("motion.grid");
        g.set_param(src, "rows", 1.0);
        g.set_param(src, "cols", 1.0);
        sinks.push(flipbook(&mut g, src, holds, k as f32 * 140.0));
    }
    (g, sinks)
}

/// ⭐⭐ **O GATE QUE FALTAVA: a cena REAL põe o `speed`, e um dos lados põe o ritmo.**
///
/// ⚠️ É a afirmação que teria evitado o report de 2026-08-28. A fileira sub-UV da cena `=9`
/// deixa o `speed` no default — `0` — e é parada por construção; mandar alguém procurar uma
/// mudança de RITMO nela é pedir para olhar para uma coisa que não anda. *Uma cena de smoke
/// tem de CONTER o fenómeno, e isso é uma propriedade do grafo que ela monta.*
#[test]
fn the_real_scene_gives_the_flipbook_a_clock_and_only_one_side_a_rhythm() {
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let sinks = build_holds_graph(&mut g);
    assert_eq!(sinks.len(), 2, "um sink por lado");

    let uvs: Vec<NodeId> = (0..g.nodes().len() as u32)
        .map(NodeId)
        .filter(|n| g.node(*n).is_some_and(|i| i.type_name == "motion.sub_uv"))
        .collect();
    assert_eq!(uvs.len(), 2, "um flipbook por lado");
    for u in &uvs {
        let speed = g
            .node_param_overrides(*u)
            .and_then(|m| m.get("speed"))
            .copied()
            .unwrap_or(0.0);
        assert!(
            speed.abs() > 0.0,
            "um flipbook com `speed = 0` esta' PARADO — a cena nao contem o fenomeno"
        );
    }
    let with_rhythm = uvs
        .iter()
        .filter(|u| {
            g.node_text_param_overrides(**u)
                .and_then(|m| m.get(ph2d_node_motion_sub_uv::HOLDS_KEY))
                .is_some_and(|v| !v.trim().is_empty())
        })
        .count();
    assert_eq!(
        with_rhythm, 1,
        "exactamente UM lado leva o ritmo — o outro e' o controle"
    );
}

/// ⭐⭐ **A CENA ANDA** — a leitura que o report de 2026-08-28 provou que faltava.
///
/// Os dois lados têm de trocar de quadrante ao longo de uma volta. É o gate mais barato que
/// existe e o único que a instrução anterior teria precisado.
#[test]
fn both_sides_actually_change_cell_over_one_cycle() {
    let (g, sinks) = scene();
    let reg = registry();
    let mut cook = Cook::new();
    let cycle = f64::from(CELLS / SPEED);
    let mut left = Vec::new();
    let mut right = Vec::new();
    for k in 0..24 {
        let (l, r) = cells_at(&g, &reg, &sinks, &mut cook, k as f64 * cycle / 24.0);
        left.push(l);
        right.push(r);
    }
    let distinct = |v: &Vec<i32>| {
        let mut s = v.clone();
        s.sort_unstable();
        s.dedup();
        s.len()
    };
    assert_eq!(
        distinct(&left),
        4,
        "a ESQUERDA tem de percorrer as quatro: {left:?}"
    );
    assert_eq!(distinct(&right), 4, "a DIREITA tambem: {right:?}");
}

/// ⭐ **E os dois lados DIFEREM** — senão a cena mostra o mesmo duas vezes e o artista
/// conclui que a feature não existe.
#[test]
fn the_two_sides_do_not_show_the_same_thing() {
    let (g, sinks) = scene();
    let reg = registry();
    let mut cook = Cook::new();
    let cycle = f64::from(CELLS / SPEED);
    let differs = (0..48)
        .map(|k| cells_at(&g, &reg, &sinks, &mut cook, k as f64 * cycle / 48.0))
        .filter(|(l, r)| l != r)
        .count();
    assert!(
        differs > 8,
        "so' {differs} de 48 instantes diferem — o ritmo nao esta' a chegar ao ecra"
    );
}

/// ⭐⭐ **A VOLTA FECHA AO MESMO TEMPO DOS DOIS LADOS** — a leitura que separa *redistribuir*
/// de *abrandar*, e a promessa que o desenho faz.
///
/// Ao fim de uma volta exacta, os dois têm de estar no MESMO quadrante em que começaram. Se a
/// direita ficasse para trás, a lista de pesos teria virado uma segunda resposta a *«quão
/// rápido»* — o defeito que o desenho existe para não ter.
#[test]
fn one_full_cycle_brings_both_sides_back_to_where_they_started() {
    let (g, sinks) = scene();
    let reg = registry();
    let mut cook = Cook::new();
    let cycle = f64::from(CELLS / SPEED);
    let (l0, r0) = cells_at(&g, &reg, &sinks, &mut cook, 0.0);
    for turns in 1..4 {
        let (l, r) = cells_at(&g, &reg, &sinks, &mut cook, cycle * f64::from(turns));
        assert_eq!(l, l0, "a esquerda fugiu ao fim de {turns} voltas");
        assert_eq!(
            r, r0,
            "a DIREITA fugiu ao fim de {turns} voltas — os pesos viraram velocidade"
        );
    }
}

/// **A pose SEGURA é a terceira, e ela fica o triplo** — a leitura que o texto da cena
/// promete, medida sobre o documento que a cena de facto monta.
#[test]
fn the_third_quadrant_is_the_one_that_lingers() {
    let (g, sinks) = scene();
    let reg = registry();
    let mut cook = Cook::new();
    let cycle = f64::from(CELLS / SPEED);
    let mut held = [0usize; 4];
    let mut plain = [0usize; 4];
    const N: usize = 600;
    for k in 0..N {
        let (l, r) = cells_at(&g, &reg, &sinks, &mut cook, k as f64 * cycle / N as f64);
        plain[(l.rem_euclid(4)) as usize] += 1;
        held[(r.rem_euclid(4)) as usize] += 1;
    }
    // ⚠️ O CONTROLE primeiro: à esquerda as quatro têm de ocupar a MESMA fatia. Sem ele,
    // uma cena em que tudo demora o mesmo passaria a metade de baixo por acidente.
    let span = plain.iter().max().unwrap() - plain.iter().min().unwrap();
    assert!(span * 20 < N, "a esquerda nao e' uniforme: {plain:?}");
    // E à direita a terceira (índice 2) leva o triplo das outras.
    let unit = (held[0] + held[1] + held[3]) as f32 / 3.0;
    let ratio = held[2] as f32 / unit;
    assert!(
        (ratio - 3.0).abs() < 0.3,
        "a terceira devia durar 3x e durou {ratio:.2}x ({held:?})"
    );
}

/// ⭐⭐⭐ **O FLIPBOOK CONTINUA A ANDAR COM UM COZINHADOR REUSADO** — o gate que apanha o
/// defeito que o report de 2026-08-28 de facto tinha.
///
/// ⚠️⚠️ **É a segunda causa do mesmo report, e a primeira cura não lhe tocou.** Depois de a
/// cena passar a ter relógio, ela continuava parada: o `motion.sub_uv` declarava-se
/// `Effect::Pure` e lia `ctx.playhead()`, e a impressão digital do memo só inclui o relógio
/// para um nó `Temporal`. Ele cozinhava UMA vez e devolvia o mesmo stream para sempre.
///
/// ⚠️ **E nenhum dos gates irmãos o via**, porque todos construíam um `Cook::new()` por
/// instante — *um memo que nasce vazio nunca devolve nada de velho*. Quem reusa o cozinhador é
/// o app. **A régua tem de ser a do app.**
#[test]
fn the_flipbook_keeps_moving_under_one_persistent_cook() {
    let (g, sinks) = scene();
    let reg = registry();
    let mut cook = Cook::new();
    let cycle = f64::from(CELLS / SPEED);
    let mut seen: Vec<(i32, i32)> = Vec::new();
    for k in 0..24 {
        seen.push(cells_at(
            &g,
            &reg,
            &sinks,
            &mut cook,
            k as f64 * cycle / 24.0,
        ));
    }
    let left: std::collections::BTreeSet<i32> = seen.iter().map(|(l, _)| *l).collect();
    let right: std::collections::BTreeSet<i32> = seen.iter().map(|(_, r)| *r).collect();
    assert_eq!(
        left.len(),
        4,
        "com o cozinhador REUSADO a esquerda tem de percorrer as quatro celulas: {seen:?}"
    );
    assert_eq!(right.len(), 4, "e a direita tambem: {seen:?}");
}
