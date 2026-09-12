//! ⭐⭐⭐ **A POSE do grupo** — os gates do report do Enio (2026-08-30).
//!
//! > *"Ao criar o grupo o gizmo do objeto pai deveria nascer na posição entre os filhos, mas nasceu
//! > no zero do mundo."*
//!
//! Um grupo não desenha nada, então **a pose dele é o gizmo dele**. Nascer na origem punha a alça
//! longe do conteúdo — muitas vezes fora do ecrã — e fazia girar o grupo acontecer em torno de um
//! ponto sem relação com ele.
//!
//! ⚠️ E a cura tem **duas metades**: centrar o grupo obriga a compensar os filhos, senão agrupar
//! MOVE o desenho; e desagrupar obriga a devolver a compensação, senão dissolver move-o de volta ao
//! contrário. Enquanto o grupo nascia na origem as duas eram somar zero, e por isso nenhuma existia.

use super::{group_entities, ungroup_entities};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};

/// Uma raiz com uma pose conhecida.
fn raiz(sim: &mut SimWorld, x: f32, y: f32, nome: &str) -> u64 {
    sim.world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(x, y),
                ..Transform::default()
            },
            Name::new(nome.to_string()),
        ))
        .id()
        .to_bits()
}

/// A pose no MUNDO, somando a cadeia de pais.
///
/// ⚠️ A soma pura basta **porque um grupo nasce sem rotação nem escala** — é essa a razão de a
/// compensação ser uma subtracção e não uma inversa de matriz. Se um dia o grupo nascer girado,
/// este helper mente e o gate tem de mudar com ele.
fn mundo(sim: &SimWorld, e: u64) -> Vec2 {
    let mut acc = Vec2::ZERO;
    let mut cur = Entity::from_bits(e);
    loop {
        acc += sim
            .world()
            .get::<Transform>(cur)
            .map_or(Vec2::ZERO, |t| t.translation);
        match sim.world().get::<ChildOf>(cur) {
            Some(c) => cur = c.parent(),
            None => return acc,
        }
    }
}

/// ⭐⭐⭐ **O grupo nasce ENTRE os filhos** — o report, medido.
#[test]
fn the_group_is_born_between_its_members_not_at_the_world_origin() {
    let mut sim = SimWorld::default();
    let a = raiz(&mut sim, 10.0, 4.0, "A");
    let b = raiz(&mut sim, 20.0, 8.0, "B");
    let g = group_entities(&mut sim, &[a, b], "G".into()).expect("dois topos distintos");

    let pose = sim
        .world()
        .get::<Transform>(Entity::from_bits(g))
        .expect("o grupo tem pose")
        .translation;
    assert!(
        (pose - Vec2::new(15.0, 6.0)).length() < 1e-5,
        "o grupo nasceu em {pose:?} e nao no meio dos membros (15, 6) - com (0,0) a alca aparece \
         longe do conteudo, muitas vezes fora do ecra, e girar o grupo roda em torno do nada"
    );
}

/// ⛔⛔ **AGRUPAR NÃO MOVE NADA.** A metade que a centragem obriga.
#[test]
fn grouping_does_not_move_a_single_pixel() {
    let mut sim = SimWorld::default();
    let a = raiz(&mut sim, 10.0, 4.0, "A");
    let b = raiz(&mut sim, 20.0, 8.0, "B");
    let (antes_a, antes_b) = (mundo(&sim, a), mundo(&sim, b));
    group_entities(&mut sim, &[a, b], "G".into()).expect("agrupa");
    for (nome, e, antes) in [("A", a, antes_a), ("B", b, antes_b)] {
        let depois = mundo(&sim, e);
        assert!(
            (depois - antes).length() < 1e-5,
            "{nome} moveu-se de {antes:?} para {depois:?} ao ser agrupado - um verbo de organizacao \
             que desloca o desenho nao e' um verbo de organizacao"
        );
    }
}

/// ⛔⛔ **DESAGRUPAR TAMBÉM NÃO.** A metade inversa — sem ela, dissolver devolvia o desenho
/// deslocado de `-centro`, e é precisamente o gesto com que o artista confere o primeiro.
#[test]
fn ungrouping_puts_everything_back_exactly_where_it_was() {
    let mut sim = SimWorld::default();
    let a = raiz(&mut sim, 10.0, 4.0, "A");
    let b = raiz(&mut sim, 20.0, 8.0, "B");
    let (antes_a, antes_b) = (mundo(&sim, a), mundo(&sim, b));
    let g = group_entities(&mut sim, &[a, b], "G".into()).expect("agrupa");
    assert_eq!(ungroup_entities(&mut sim, &[g]), 1, "dissolveu um grupo");
    for (nome, e, antes) in [("A", a, antes_a), ("B", b, antes_b)] {
        let depois = mundo(&sim, e);
        assert!(
            (depois - antes).length() < 1e-5,
            "{nome} voltou a {depois:?} em vez de {antes:?} - a compensacao foi aplicada num \
             sentido e nao no inverso, e meia cura desloca o desenho na volta"
        );
    }
}

/// ⚠️ **O ANINHAMENTO tem de sobreviver às duas compensações.** Um grupo dentro de outro tem a
/// pose dele já em coordenadas do pai — somá-la duas vezes, ou nenhuma, só se vê aqui.
#[test]
fn nesting_a_group_inside_another_still_moves_nothing() {
    let mut sim = SimWorld::default();
    let a = raiz(&mut sim, 10.0, 4.0, "A");
    let b = raiz(&mut sim, 20.0, 8.0, "B");
    let c = raiz(&mut sim, 100.0, 0.0, "C");
    let antes = [mundo(&sim, a), mundo(&sim, b), mundo(&sim, c)];
    let dentro = group_entities(&mut sim, &[a, b], "in".into()).expect("interno");
    group_entities(&mut sim, &[dentro, c], "out".into()).expect("externo");
    for (i, (nome, e)) in [("A", a), ("B", b), ("C", c)].into_iter().enumerate() {
        let depois = mundo(&sim, e);
        assert!(
            (depois - antes[i]).length() < 1e-5,
            "{nome} moveu-se de {:?} para {depois:?} ao aninhar",
            antes[i]
        );
    }
}

/// ⛔⛔ **DISSOLVER UM GRUPO ANINHADO dissolve o CLICADO, não o de fora.**
///
/// A cura de aceitar o próprio grupo não podia ser apagar a condição `t != e`: um grupo aninhado
/// tem por ancestral de topo o grupo de FORA, e subir cegamente dissolveria o pai — o artista
/// carregava em *Ungroup* num grupo interno e via a estrutura de cima desfazer-se.
#[test]
fn ungrouping_a_nested_group_dissolves_the_one_you_clicked() {
    let mut sim = SimWorld::default();
    let a = raiz(&mut sim, 10.0, 4.0, "A");
    let b = raiz(&mut sim, 20.0, 8.0, "B");
    let c = raiz(&mut sim, 100.0, 0.0, "C");
    let dentro = group_entities(&mut sim, &[a, b], "in".into()).expect("interno");
    let fora = group_entities(&mut sim, &[dentro, c], "out".into()).expect("externo");

    assert_eq!(ungroup_entities(&mut sim, &[dentro]), 1);
    assert!(
        sim.world().get_entity(Entity::from_bits(fora)).is_ok(),
        "o grupo de FORA foi dissolvido - carregar em Ungroup num grupo interno desfez a estrutura \
         de cima, que e' o oposto do que o artista pediu"
    );
    assert!(
        sim.world().get_entity(Entity::from_bits(dentro)).is_err(),
        "o grupo clicado sobreviveu - o verbo nao fez nada"
    );
    for (nome, e) in [("A", a), ("B", b)] {
        assert_eq!(
            sim.world()
                .get::<ChildOf>(Entity::from_bits(e))
                .map(|c| c.parent().to_bits()),
            Some(fora),
            "{nome} devia ter subido para o grupo de fora"
        );
    }
}
