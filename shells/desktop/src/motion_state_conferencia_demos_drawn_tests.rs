//! Gates da cena `=85` — **a forma que o artista desenha, e os dois eixos** (doc 89, folha 06).
//!
//! ⚠️ **O oráculo de cada linha é o MECANISMO, não «a figura mudou».** As duas primeiras
//! provam-se pela FORMA que a fileira desenha (é ela que o artista lê), e a terceira pelo
//! EIXO que se moveu — uma régua de excursão passaria numa implementação que só escalasse.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn cell(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Stream {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    match out.first() {
        Some(CookValue::Instances(s)) => s.clone(),
        _ => panic!("a celula emite instancias"),
    }
}

/// As alturas de uma fileira, na ordem do stream — o perfil que o olho lê.
fn heights(s: &Stream) -> Vec<f32> {
    match s.get("P") {
        Some(Column::Vec2(p)) => p.iter().map(|q| q[1]).collect(),
        _ => panic!("P"),
    }
}

fn row_of(case: Case) -> usize {
    ROWS_TABLE
        .iter()
        .position(|r| r.case == case)
        .expect("a linha existe")
}

/// **A CENA CONSTRÓI TODAS AS CÉLULAS** — duas por linha da [`ROWS_TABLE`].
#[test]
fn the_shape_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, count) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(count >= 8.0, "uma fileira precisa de pecas que cheguem");
    for sink in &sinks {
        assert!(cell(&doc, &reg, *sink).count() > 0, "toda celula desenha");
    }
}

/// **A LINHA DA ONDA JULGA-SE PARADA** — e é a `frequency = 0` que o garante.
///
/// ⚠️ Sem isto a cena mentiria no anúncio: ela diz *"não carregue Play"*, e um oscilador
/// com frequência viva desenharia coisa diferente a cada quadro. O gate coze em DOIS
/// instantes e exige o mesmo número.
#[test]
fn the_wave_row_is_frozen_in_time() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::Wave);
    for sink in [sinks[k * 2], sinks[k * 2 + 1]] {
        let mut c = Cook::new();
        let at = |c: &mut Cook, t: f64| match c.cook(&doc.graph, &reg, sink, t).expect("coze")[0] {
            CookValue::Instances(ref s) => heights(s),
            _ => panic!("instancias"),
        };
        let a = at(&mut c, 0.0);
        let b = at(&mut c, 1.7);
        assert_eq!(a, b, "a fileira da onda tem de ficar PARADA no tempo");
        // O controle: ela não está parada por ser plana.
        let swing = a.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
            - a.iter().fold(f32::INFINITY, |m, x| m.min(*x));
        assert!(swing > 0.5, "e ela desenha alguma coisa: {swing}");
    }
}

/// **A ONDA DESENHADA É UNIPOLAR E ASSIMÉTRICA; A SENOIDE É NENHUMA DAS DUAS.**
///
/// ⚠️ **A primeira régua que escrevi era a POSIÇÃO DO PICO, e ela é fraca aqui:** com 15
/// peças o índice anda de `1/14 = 0,071`, e os dois picos ficam a **um passo** um do outro
/// (`0,214` contra `0,286`) — a margem que eu tinha posto era maior que a resolução da
/// própria fileira. *Uma régua não pode exigir mais precisão do que a fixture tem.*
///
/// A régua certa é a que o OLHO usa: a curva autorada nunca desce abaixo do repouso (ela é
/// `[0,1]`, o quadrado do editor) e a senoide desce até `−1`. A fileira da direita fica
/// inteiramente **acima** da linha; a da esquerda atravessa-a. E a assimetria entra como
/// segunda metade — o pico ANTES, sem exigir mais que um passo.
#[test]
fn the_custom_wave_row_is_unipolar_and_peaks_before_the_sine() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::Wave);
    let plain = heights(&cell(&doc, &reg, sinks[k * 2]));
    let drawn = heights(&cell(&doc, &reg, sinks[k * 2 + 1]));
    // A altura de repouso da linha (o `offset_y` do `motion.transform`).
    let rest = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
    let below = |h: &[f32]| h.iter().filter(|y| **y < rest - 1e-4).count();
    assert!(
        below(&plain) > 2,
        "a senoide ATRAVESSA a linha de repouso: {plain:?}"
    );
    assert_eq!(
        below(&drawn),
        0,
        "a curva desenhada e' unipolar e fica toda ACIMA: {drawn:?}"
    );
    // E a assimetria: o pico da desenhada vem antes. A margem é **meio passo de índice** —
    // o mais fino que esta fileira sabe exprimir.
    let peak = |h: &[f32]| {
        h.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("finito"))
            .expect("nao vazio")
            .0 as f32
            / (h.len() - 1) as f32
    };
    let step = 1.0 / (COUNT - 1.0);
    assert!(
        peak(&drawn) < peak(&plain) - step * 0.5,
        "a curva desenhada pica ANTES da senoide ({} vs {}, passo {step})",
        peak(&drawn),
        peak(&plain)
    );
}

/// **A ESCADA DESENHADA VOLTA; A `Linear` NÃO.**
///
/// ⚠️ O oráculo é a MONOTONIA. A `Linear` sobe do começo ao fim; o V desenhado sobe até ao
/// meio e desce — e nenhuma das oito famílias enumeradas faz isso, então este gate separa
/// *"leu a minha curva"* de *"caiu numa ease qualquer"*.
#[test]
fn the_custom_ease_row_comes_back_and_the_linear_one_does_not() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::Ease);
    let plain = heights(&cell(&doc, &reg, sinks[k * 2]));
    let drawn = heights(&cell(&doc, &reg, sinks[k * 2 + 1]));
    let rises = |h: &[f32]| h.windows(2).all(|w| w[1] >= w[0] - 1e-6);
    assert!(rises(&plain), "a Linear sobe sempre: {plain:?}");
    assert!(!rises(&drawn), "a desenhada tem de VOLTAR: {drawn:?}");
    // E ela volta ao ponto de partida (o V fecha), o que a distingue de um ruído.
    assert!(
        (drawn[0] - drawn[drawn.len() - 1]).abs() < 0.05,
        "o V fecha onde abriu: {drawn:?}"
    );
}

/// **SÓ A METADE CURADA DA LINHA 3 MEXE NO EIXO X.**
///
/// ⚠️ O oráculo é o ESPAÇAMENTO horizontal, e não o vertical: as duas metades mexem em Y
/// (o canal de sempre é o Y), então uma régua de excursão não separa nada. O que muda é
/// que à direita as colunas deixam de estar igualmente espaçadas.
#[test]
fn only_the_cured_half_of_the_two_axis_row_disturbs_the_spacing() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::TwoAxis);
    // O desvio-padrão dos passos horizontais: zero numa fileira perfeitamente espaçada.
    let jitter = |s: &Stream| {
        let Some(Column::Vec2(p)) = s.get("P") else {
            panic!("P")
        };
        let steps: Vec<f32> = p.windows(2).map(|w| w[1][0] - w[0][0]).collect();
        let m = steps.iter().sum::<f32>() / steps.len() as f32;
        (steps.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / steps.len() as f32).sqrt()
    };
    let plain = jitter(&cell(&doc, &reg, sinks[k * 2]));
    let cured = jitter(&cell(&doc, &reg, sinks[k * 2 + 1]));
    assert!(
        plain < 1e-5,
        "a metade de sempre mantem o espacamento EXACTO: {plain}"
    );
    assert!(
        cured > 0.02,
        "a curada tem de desarrumar o espacamento: {cured}"
    );
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drawn_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let s = cell(&doc, &reg, *sink);
        let Some(Column::Vec2(p)) = s.get("P") else {
            panic!("P")
        };
        let row = k / 2;
        let half = k % 2;
        let cx = if half == 0 { -COL_X } else { COL_X };
        let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in p {
            mx = mx.max((q[0] - cx).abs());
            my = my.max((q[1] - cy).abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}
