//! Os gates do [`crate::skin_live`] — o esqueleto como HOST.
//!
//! A **lei** da mistura (peso, órfão, C¹, escala) é da crate `ph2d-vec-skin` e está gateada lá. Aqui
//! mede-se o que só existe com um mundo ECS: prender não move nada · dobrar um osso dobra o desenho
//! · a hierarquia É a cinemática (mover o pai leva o filho) · apagar um osso não apaga a forma · os
//! dois verbos de soltar · e um segundo esqueleto não é apanhado por engano.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_vec_scene::{ShapeKind, cook};

/// Uma cena com UMA forma (um rectângulo deitado de `(0,0)` a `(40,10)`) e um esqueleto de dois
/// ossos ao longo dela. Devolve `(sim, scene, map, id, [osso_raiz, osso_ponta])`.
fn palco() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    (sim, scene, map, id, [raiz, ponta])
}

/// Um osso em `pos` (local do pai), comprimento `len`, força 1.
fn osso(sim: &mut SimWorld, nome: &str, pos: [f32; 2], len: f64, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            VecBone {
                length: len,
                strength: 1.0,
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

fn quadro(sim: &SimWorld, scene: &mut VecScene, id: VecPathId) -> VecPath {
    recook(sim, scene);
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o path")
        .clone()
}

/// Gira um osso, em graus.
fn gira(sim: &mut SimWorld, e: Entity, graus: f32) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .rotation = graus.to_radians();
}

fn pior_desvio(a: &VecPath, b: &VecPath) -> f64 {
    a.verts_all()
        .zip(b.verts_all())
        .flat_map(|(x, y)| {
            [
                (x.anchor, y.anchor),
                (x.in_handle, y.in_handle),
                (x.out_handle, y.out_handle),
            ]
        })
        .fold(0.0_f64, |m, (p, q)| {
            m.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs())
        })
}

/// ⭐⭐⭐ **PRENDER NÃO MOVE UM PIXEL.** A pose de repouso é a identidade por construção (doc 47
/// §2.5), então o artista carrega em *Bind* e **nada acontece** — que é exactamente o que ele
/// espera. Uma pele que salta ao ligar é o defeito nº 1 de todo pacote de rig.
#[test]
fn binding_a_shape_moves_nothing() {
    let (mut sim, mut scene, map, id, _) = palco();
    let antes = scene.paths()[0].clone();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio(&antes, &depois);
    assert!(
        pior < 1e-9,
        "prender moveu a forma em {pior} - o `rest` nao esta' a ser o composto `S-1 . B`"
    );
}

/// ⭐⭐⭐ **DOBRAR UM OSSO DOBRA O DESENHO** — a razão de existir da wave.
///
/// A ponta do rectângulo está dentro do alcance do 2.º osso e longe do 1.º; girar o 2.º levanta-a e
/// **deixa a base onde estava**. É a diferença entre um personagem e um recorte de papel.
#[test]
fn bending_a_bone_bends_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let repouso = quadro(&sim, &mut scene, id);
    gira(&mut sim, ossos[1], 60.0);
    let dobrado = quadro(&sim, &mut scene, id);

    let dir = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .filter(|a| a[0] > 30.0)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    let esq = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .filter(|a| a[0] < 2.0)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    assert!(
        dir(&dobrado) - dir(&repouso) > 3.0,
        "a ponta nao subiu: {} -> {}",
        dir(&repouso),
        dir(&dobrado)
    );
    assert!(
        (esq(&dobrado) - esq(&repouso)).abs() < 0.5,
        "a BASE mexeu-se ({} -> {}) - o osso da ponta esta' a mandar em toda a forma",
        esq(&repouso),
        esq(&dobrado)
    );
}

/// ⭐⭐ **A HIERARQUIA É A CINEMÁTICA** — girar o osso-pai leva o filho, sem uma linha de FK escrita
/// nesta linha de código. É o `propagate_transforms` da casa a fazer o trabalho, e é a razão de um
/// osso ser uma ENTIDADE (doc 47 §2.1).
///
/// ⚠️ Se alguém trocar a árvore por uma lista dentro de um componente, é este gate que fica
/// vermelho.
#[test]
fn turning_the_parent_bone_carries_the_child_because_the_tree_is_the_kinematics() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let repouso = quadro(&sim, &mut scene, id);
    gira(&mut sim, ossos[0], 40.0);
    let posado = quadro(&sim, &mut scene, id);
    let ponta = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    assert!(
        ponta(&posado) - ponta(&repouso) > 8.0,
        "girar a RAIZ tinha de levar a forma inteira: {} -> {}",
        ponta(&repouso),
        ponta(&posado)
    );
}

/// ⛔ **APAGAR UM OSSO NÃO APAGA A FORMA.** O que resta renormaliza-se sozinho; a forma continua a
/// desenhar-se. Sem isto, um `Delete` na Hierarquia colapsaria o desenho na origem — sem uma linha
/// de erro, porque a soma dos pesos seria zero.
#[test]
fn deleting_a_bone_does_not_delete_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let antes = quadro(&sim, &mut scene, id);
    sim.world_mut().entity_mut(ossos[1]).despawn();
    let depois = quadro(&sim, &mut scene, id);
    assert!(
        depois.verts_all().all(|v| v.anchor[0].is_finite()),
        "a forma virou NaN ao perder um osso"
    );
    assert!(
        pior_desvio(&antes, &depois) < 1e-9,
        "apagar um osso EM REPOUSO mudou a forma - a renormalizacao esta' errada"
    );
}

/// **Os dois verbos de soltar.** *Release* devolve o que o artista desenhou; *Expand* fica com o que
/// ele está a ver. Adivinhar qual deles ele quer é que não.
#[test]
fn releasing_gives_back_the_drawing_and_expanding_keeps_the_pose() {
    for (keep, volta) in [(Keep::Source, true), (Keep::Deformed, false)] {
        let (mut sim, mut scene, map, id, ossos) = palco();
        let autorada = scene.paths()[0].clone();
        bind(&mut sim, &scene, &map, &[id], None);
        gira(&mut sim, ossos[1], 60.0);
        let posada = quadro(&sim, &mut scene, id);
        assert_eq!(release(&mut sim, &mut scene, &map, &[id], keep), 1);
        let ficou = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .expect("o path")
            .clone();
        let alvo = if volta { &autorada } else { &posada };
        assert!(
            pior_desvio(alvo, &ficou) < 1e-9,
            "{keep:?} ficou com a geometria errada"
        );
        // E a pele foi-se: o recook seguinte não pode voltar a deformar.
        let depois = quadro(&sim, &mut scene, id);
        assert!(
            pior_desvio(&ficou, &depois) < 1e-9,
            "{keep:?} deixou pele viva"
        );
    }
}

/// ⚠️ **UM SEGUNDO ESQUELETO NÃO É APANHADO POR ENGANO.** Com um osso apontado, o Bind leva a árvore
/// DELE; sem nenhum, leva tudo (a leitura certa de *"há um esqueleto só"*, que é o caso comum).
#[test]
fn a_second_skeleton_is_only_bound_when_it_is_the_one_pointed_at() {
    let (mut sim, _scene, _map, _id, ossos) = palco();
    let outro = osso(&mut sim, "Other", [200.0, 0.0], 10.0, None);
    // ⚠️ **Comparado como CONJUNTO**: a ORDEM dentro de uma pele não tem sentido (os pesos
    // normalizam-se), só precisa de ser determinística — e ordenar por `to_bits` é ordenar por id
    // de ALOCAÇÃO, que numa fixtura não é o que se quer afirmar. O que importa é *quem* entra.
    let conjunto = |v: Vec<Entity>| {
        let mut s: Vec<u64> = v.into_iter().map(|e| e.to_bits()).collect();
        s.sort_unstable();
        s
    };
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(ossos[1]))),
        conjunto(vec![ossos[0], ossos[1]])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(outro))),
        conjunto(vec![outro])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, None)),
        conjunto(vec![ossos[0], ossos[1], outro]),
        "sem semente, o esqueleto e' a cena"
    );
}

/// ⚠️ **A ORDEM dos ossos numa pele não é carregada** — permutá-la devolve o mesmo desenho.
///
/// É este gate que autoriza `skeleton_of` a ordenar por `to_bits`: se a ordem decidisse alguma
/// coisa, ordenar por id de ALOCAÇÃO seria o defeito que o `CLAUDE.md` §5 nomeia. O que ela decide
/// é só a ordem da soma em `f64`, e a medida está aqui.
#[test]
fn the_order_of_the_bones_in_a_skin_does_not_change_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    gira(&mut sim, ossos[1], 55.0);
    let direita = quadro(&sim, &mut scene, id);

    let e = Entity::from_bits(map[&id]);
    sim.world_mut()
        .get_mut::<VecSkin>(e)
        .expect("a pele")
        .bones
        .reverse();
    let avessa = quadro(&sim, &mut scene, id);
    let pior = pior_desvio(&direita, &avessa);
    assert!(
        pior < 1e-12,
        "permutar os ossos moveu a forma em {pior} - a ordem esta' a decidir algo"
    );
}

/// ⭐⭐⭐ **PRENDER À CENA INTEIRA DÁ O MESMO DESENHO QUE PRENDER AO ESQUELETO CERTO** — e é isto
/// que autoriza o `Bind` sem cerimónia (o botão prende a tudo quando o artista não apontou um osso).
///
/// ⚠️ **A propriedade é do SUPORTE FINITO, não da bondade da implementação:** um osso longe está
/// fora do raio de todo ponto da forma ⇒ peso `0` ⇒ a normalização devolve exactamente os mesmos
/// números; e um ponto órfão prende-se ao mais PRÓXIMO, que é do esqueleto certo. ⛔ Com a lei
/// global (`1/d²`) isto seria FALSO, e o segundo esqueleto arrastaria a forma um pouco.
#[test]
fn binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton() {
    let desenho = |semente: bool| {
        let (mut sim, mut scene, map, id, ossos) = palco();
        // Um segundo esqueleto, LONGE — o que o `Bind` sem semente também apanharia.
        let outro = osso(&mut sim, "Far", [400.0, 0.0], 30.0, None);
        osso(&mut sim, "Far2", [30.0, 0.0], 30.0, Some(outro));
        let raiz = semente.then_some(ossos[0]);
        assert_eq!(bind(&mut sim, &scene, &map, &[id], raiz), 1);
        gira(&mut sim, ossos[1], 50.0);
        quadro(&sim, &mut scene, id)
    };
    let pior = pior_desvio(&desenho(true), &desenho(false));
    assert!(
        pior < 1e-12,
        "o esqueleto LONGE mudou o desenho em {pior} - o suporte deixou de ser finito"
    );
}

/// **Os ossos que o overlay desenha saem da POSE, não do que foi escrito.** Um osso filho herda a
/// pose do pai — se este gate ficar vermelho, o artista vê o osso num sítio e a forma dobra noutro.
#[test]
fn the_drawn_bone_is_where_the_hierarchy_puts_it() {
    let (mut sim, _scene, _map, _id, ossos) = palco();
    gira(&mut sim, ossos[0], 90.0);
    let segs = bone_segments(&sim);
    let (_, a, b) = segs
        .iter()
        .find(|(bits, _, _)| *bits == ossos[1].to_bits())
        .copied()
        .expect("o osso da ponta");
    // Raiz em (0,5) girada 90°: a ponta dela vai para (0,25); o filho sai dali para cima.
    //
    // ⚠️ **A barra é `1e-5` porque a ROTAÇÃO é `f32`** (o `Transform` da casa), e `cos(90°)` em
    // `f32` vale `−4,37e-8`; a 45 unidades de distância isso lê-se como `−1,75e-6`. *Uma barra de
    // `1e-6` aqui não mediria o produto, mediria o tipo do campo de rotação.*
    assert!(
        a[0].abs() < 1e-5 && (a[1] - 25.0).abs() < 1e-5,
        "a origem do osso filho ficou em {a:?}"
    );
    assert!(
        b[0].abs() < 1e-5 && (b[1] - 45.0).abs() < 1e-5,
        "a ponta do osso filho ficou em {b:?}"
    );
}

/// ⚠️ **SONDA da cena de smoke** (report do Enio, 2026-09-06: *"o bind não funciona e nenhuma forma
/// pode ser deformada"*): a MESMA sequência do `vec_bone_smoke`, com as MESMAS portas.
#[test]
fn probe_the_smoke_sequence() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(crate::build_smoke::shape(
        ph2d_vec_scene::ShapeKind::RoundRect,
        [-8.5, 2.0],
        [-1.5, 3.0],
        &[0.5],
        [230, 170, 90],
    ));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    // A cadeia, como o smoke a faz: pela porta do GESTO, em coordenadas de MUNDO.
    let mut pai: Option<Entity> = None;
    let mut raiz = None;
    for i in 0..3 {
        let x = -8.2 + f64::from(i) * 2.1333;
        let bits =
            crate::bone_gesture::create(&mut sim, pai, [x, 2.5], [x + 2.1333, 2.5]).expect("osso");
        pai = Some(Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    eprintln!(
        "[probe] ossos = {:?}",
        crate::skin_live::bone_segments(&sim)
    );
    let n = bind(&mut sim, &scene, &map, &[id], raiz);
    eprintln!("[probe] bind devolveu {n}");
    let antes = quadro(&sim, &mut scene, id);
    // Posa o ÚLTIMO osso pela porta do gesto.
    let ultimo = pai.expect("ultimo");
    let ok = crate::bone_gesture::pose(&mut sim, ultimo, [-2.0, 6.0], false);
    eprintln!("[probe] pose devolveu {ok}");
    let depois = quadro(&sim, &mut scene, id);
    eprintln!("[probe] desvio = {}", pior_desvio(&antes, &depois));
    assert!(
        pior_desvio(&antes, &depois) > 0.5,
        "a forma NAO deformou - reproduzido o report"
    );
}
