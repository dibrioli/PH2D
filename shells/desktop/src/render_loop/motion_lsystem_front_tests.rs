//! **A ORDEM E A MÉDIA** — em que camada cada folha se desenha, e em que ordem.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`); o corte segue o do
//! produto (o irmão `motion_lsystem_rows` é quem constrói estas linhas).
//!
//! ⛔⛔ **A afirmação que atravessa tudo aqui é «cada linha desenha-se EXACTAMENTE uma vez».**
//! Uma média nova é uma oportunidade para uma linha ser desenhada duas vezes (pelos dois
//! lowerings) ou nenhuma — e nenhuma das duas se vê num teste que só conte instâncias de um lado.

use crate::render_loop::motion_lsystem_gen::publish;
use crate::render_loop::motion_lsystem_testkit::*;
use ph2d_node_source_lsystem as ls;

/// ⛔⛔ **E DUAS FOLHAS NÃO SE EMPILHAM** — *"elas aparecem em cada segmento"*, a metade que era
/// do MOLDE e não da lei: `62` marcas em `30` sítios, folhas idênticas uma sobre a outra.
///
/// ⚠️ **Aqui a régua tem de ser a POSIÇÃO DA INSTÂNCIA**, não a do esqueleto: é o que o artista
/// vê, e é o que sobrevive a qualquer mudança de como a membrana escolhe as âncoras.
#[test]
fn no_two_leaves_are_drawn_on_top_of_each_other() {
    let (mut state, n) = factory_plant_with_leaf(5.0, false);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let inst = instances_of(&state, &key);
    let mut sitios: Vec<(i64, i64)> = inst
        .iter()
        .map(|i| ((i.world_pos[0] * 1e4) as i64, (i.world_pos[1] * 1e4) as i64))
        .collect();
    let total = sitios.len();
    assert!(total > 8, "so' {total} folhas");
    sitios.sort_unstable();
    sitios.dedup();
    assert_eq!(
        total,
        sitios.len(),
        "{total} folhas em {} sitios — elas empilham",
        sitios.len()
    );
}
/// ⭐⭐⭐ **A FRACÇÃO «À FRENTE» É A ORDEM DAS LINHAS** — report do Enio (2026-08-30): *"não temos
/// a opção de escolher quantas folhas são desenhadas na frente ou atrás dos galhos"*.
///
/// ⚠️ **A afirmação tem de ser sobre a POSIÇÃO da planta na lista**, e não sobre uma contagem:
/// a passagem vectorial desenha as linhas por ordem, então o que decide o z é quantas folhas
/// ficam DEPOIS da linha da planta. Um gate que contasse folhas passaria com todas atrás.
#[test]
fn the_front_fraction_puts_leaves_after_the_plant_in_the_row_order() {
    let leaves_after_plant = |front: f32| -> (usize, usize) {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::LEAF_FRONT, front);
        // ⚠️ **Uma FORMA desenhada**, não uma imagem: uma sprite desenha-se sempre antes do
        // vector (declarado: «Fase 1: vector over sprite»), e nenhuma ordem a move.
        publish_vector_object(&mut state, "folha", 77);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let geom = column_v1(&state, &key, "geometry_id");
        let plant = geom
            .iter()
            .position(|g| *g > 0.0 && *g != 77.0)
            .expect("a planta");
        let depois = geom.iter().skip(plant + 1).filter(|g| **g == 77.0).count();
        let antes = geom.iter().take(plant).filter(|g| **g == 77.0).count();
        (antes, depois)
    };
    let (antes0, depois0) = leaves_after_plant(0.0);
    assert!(antes0 > 0, "com 0 as folhas ficam todas atras da planta");
    assert_eq!(depois0, 0, "com 0 nenhuma folha pode ficar a' frente");
    let (antes1, depois1) = leaves_after_plant(1.0);
    assert_eq!(antes1, 0, "com 1 nenhuma folha fica atras");
    assert_eq!(depois1, antes0, "com 1 todas passam para a frente");
    // ⚠️ **E o meio é MISTO** — uma lei que só respondesse nos extremos seria um interruptor.
    let (antes, depois) = leaves_after_plant(0.5);
    assert!(
        antes > 0 && depois > 0,
        "a meio tem de haver folhas dos DOIS lados: {antes} atras, {depois} a' frente"
    );
    assert_eq!(antes + depois, antes0, "nenhuma folha se perdeu no caminho");
}
/// **As instâncias vectoriais desta corrente**, na ordem em que o passe as desenha.
fn vector_instances_of(
    state: &crate::motion_state::MotionState,
    key: &str,
) -> Vec<ph2d_eval_motion::VectorInstance> {
    let mut out = Vec::new();
    if let Some(e) = state.pump.cook.externals().get(key) {
        ph2d_eval_motion::lower_to_vector_instances_onto(
            &e.value,
            ph2d_render::SinkStyle::PLAIN,
            &mut out,
        );
    }
    out
}
/// ⭐⭐⭐ **UMA FOLHA QUE É IMAGEM VAI À FRENTE DOS GALHOS** — report do Enio (2026-08-30), três
/// vezes, e a última: *"busque o estado da arte, nada de armengos"*.
///
/// ⛔⛔ **O que o bloqueava:** a casa desenha os sprites no passe 1 (alvo HDR) e o vector no
/// passe 3 (a cena Vello), então **todo vector fica por cima de todo sprite** — nenhuma ordem de
/// linhas movia uma imagem para a frente de um galho.
///
/// ⭐ **A cura é uma TERCEIRA média**: com a fracção ligada, a folha-imagem desenha-se **na cena
/// vectorial**, como quad texturado, e ali a ordem manda. ⚠️ E **não custa cor**: o tonemap
/// desta casa é passagem pura para 8 bits, com gate a medi-la byte-exacta.
///
/// ⚠️ **A afirmação mais forte é a última:** cada linha desenha-se **exactamente uma vez**. Uma
/// média nova é uma oportunidade para uma linha ser desenhada duas vezes (pelos dois lowerings)
/// ou nenhuma — e nenhuma das duas se vê num teste que só conte instâncias de um lado.
#[test]
fn an_image_leaf_can_be_drawn_in_front_of_the_branches() {
    let retrato = |front: f32| {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::LEAF_FRONT, front);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let linhas = column_v1(&state, &key, "geometry_id").len();
        (
            linhas,
            instances_of(&state, &key),
            vector_instances_of(&state, &key),
        )
    };
    // 1. ⛔ **Com `0` nada muda**: as folhas são sprites, como sempre foram.
    let (linhas, sprites, vetores) = retrato(0.0);
    assert!(sprites.len() > 8, "so' {} sprites", sprites.len());
    assert_eq!(vetores.len(), 1, "so' a planta e' vectorial");
    assert_eq!(sprites.len() + vetores.len(), linhas, "cada linha uma vez");
    let folhas = sprites.len();
    // 2. ⭐ **Com `1` a copa inteira passa ao passe VECTORIAL, e fica DEPOIS da planta.**
    let (linhas, sprites, vetores) = retrato(1.0);
    assert!(
        sprites.is_empty(),
        "nenhuma folha pode ficar no passe das sprites"
    );
    assert_eq!(vetores.len(), folhas + 1, "a planta mais as folhas todas");
    assert_eq!(sprites.len() + vetores.len(), linhas, "cada linha uma vez");
    let planta = vetores
        .iter()
        .position(|v| v.geometry_id > 0)
        .expect("a planta");
    assert_eq!(planta, 0, "com 1 todas as folhas ficam DEPOIS da planta");
    for v in &vetores[1..] {
        assert_eq!(v.geometry_id, 0, "uma folha nao tem geometria");
        assert!(
            v.atlas_uv[2] > v.atlas_uv[0],
            "a folha tem de levar a REGIAO da textura: {:?}",
            v.atlas_uv
        );
    }
    // 3. ⭐⭐ **E a meio há folhas dos DOIS lados da planta.**
    let (linhas, sprites, vetores) = retrato(0.5);
    assert!(
        sprites.is_empty(),
        "a copa nao se divide entre os dois passes"
    );
    assert_eq!(sprites.len() + vetores.len(), linhas, "cada linha uma vez");
    let planta = vetores
        .iter()
        .position(|v| v.geometry_id > 0)
        .expect("a planta");
    assert!(
        planta > 0 && planta < vetores.len() - 1,
        "a meio a planta tem de ter folhas antes E depois: {planta} de {}",
        vetores.len()
    );
}
