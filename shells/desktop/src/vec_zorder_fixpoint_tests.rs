//! **O gate do PONTO FIXO** — o conserto do "o undo só faz uma etapa" (BUGS #15).
//!
//! A regra que estes gates estabelecem vale para **todo** sistema que roda no frame, não só
//! para o vetor:
//!
//! > **A captura do undo tem de ser PONTO FIXO dos sistemas.** Fotografar o estado, deixar o
//! > frame seguinte rodar sem input nenhum, e fotografar de novo tem de dar a MESMA foto.
//!
//! Quando não é, o diff por-frame do `post_frame_undo` lê a diferença que os **próprios
//! sistemas** produziram como se fosse ação do usuário: nasce um passo espúrio, ele **limpa a
//! pilha de redo**, e o Ctrl+Z seguinte desfaz o lixo que ele mesmo acabou de criar. Para o
//! usuário: *"o undo só faz uma etapa e não funciona mais"*.
//!
//! A causa era a ordem de z. Ela era projetada da lista do **painel**, publicada no prólogo do
//! frame — **antes** de o [`sync`] dar entidade à forma recém-criada. Quem o `reorder_to` não
//! conhece leva chave 0 e vai pro **FUNDO**; a cena só convergia um frame depois, e a captura
//! era tirada antes de convergir.
//!
//! ## Estes gates MORDEM (medido, não afirmado)
//!
//! - Ler a árvore **antes** do `sync` (o produto antigo, mutação em [`Frame::run`]) derruba
//!   **4 dos 5**.
//! - Tirar o `assign_missing_root_order` derruba o 5º —
//!   [`a_shape_parented_to_a_sprite_survives_the_respawn_in_the_same_z_order`], que **nasceu
//!   vermelho** e foi quem descobriu que o desempate por `Entity::to_bits()` era real, e não
//!   teoria.
//!
//! Cada gate morde uma causa distinta. Se você mexer aqui, refaça as duas mutações.

use super::*;
use crate::undo::ProjectState;
use ph2d_ecs::scene::{
    ComponentRegistry, HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot,
    register_ecs_components,
};
use ph2d_ecs::{TransformPropagationState, WorklistBuf};
use ph2d_flip::FlipDoc;
use ph2d_vec_scene::{VecScene, rectangle};

/// O pedaço do frame que **muta o estado que o undo fotografa** — a mesma sequência, na mesma
/// ordem, do `render_loop/mod.rs`: a ponte doc↔árvore, o assentamento do pivô, e a projeção
/// de z. (O `connector_live::upkeep` e o `flip_transform::settle_origins` correm no meio, mas
/// não tocam nada disto: sem conector e sem objeto Flip na cena, são no-op.)
struct Frame {
    walk: HierarchyWalkState,
    scratch: Vec<(Entity, u8, Option<Entity>)>,
    snap: HierarchySnapshot,
    reg: ComponentRegistry,
    prop: TransformPropagationState,
    worklist: WorklistBuf,
}

impl Frame {
    fn new(sim: &mut SimWorld) -> Self {
        // O MESMO registry do produto (`init.rs`). Um componente que não passa por ele é
        // silenciosamente DESCARTADO pelo snapshot — undo e save o perdem, sem erro nenhum.
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        ph2d_render::register_render_components(&mut reg);
        Self {
            walk: HierarchyWalkState::new(sim.world_mut()),
            scratch: Vec::new(),
            snap: HierarchySnapshot::new(),
            reg,
            prop: TransformPropagationState::new(sim.world_mut()),
            worklist: WorklistBuf::new(),
        }
    }

    /// Roda um frame de sistemas sobre o documento.
    fn run(&mut self, sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) {
        sync(sim, scene, map);
        crate::vec_transform::settle_origins(sim, scene, map, &[]);
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
        build_hierarchy_snapshot(
            sim.world(),
            &mut self.walk,
            &mut self.scratch,
            &mut self.snap,
        );
        let order = z_order(&self.snap);
        scene.reorder_to(&order);
    }

    /// O que o `post_frame_undo` fotografa no fim do frame.
    fn capture(&mut self, sim: &SimWorld, scene: &VecScene) -> ProjectState {
        ProjectState::capture(
            sim,
            scene,
            &FlipDoc::new(),
            &self.reg,
            &mut self.prop,
            &mut self.worklist,
        )
    }
}

/// A ordem de z da cena (fundo → topo), que é o que o `reorder_to` escreve.
fn z(scene: &VecScene) -> Vec<VecPathId> {
    scene.paths().iter().map(|p| p.id).collect()
}

/// Três formas soltas, criadas AGORA — como o Shape Builder as cria (ainda sem entidade).
fn three_fresh_shapes(scene: &mut VecScene) -> [VecPathId; 3] {
    [
        scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0])),
        scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0])),
        scene.push_path(rectangle([4.0, 0.0], [5.0, 1.0])),
    ]
}

/// **A forma nasce JÁ na projeção — no mesmo frame.**
///
/// Era isto que estava quebrado: a projeção vinha da lista do painel, que só conhece as
/// entidades do prólogo do frame. A forma criada neste frame não estava lá, o `reorder_to` lhe
/// dava chave 0, e ela ia pro fundo da pilha de z.
#[test]
fn a_shape_born_this_frame_is_already_in_the_z_projection() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let ids = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);

    let order = z_order(&frame.snap);
    assert_eq!(
        order.len(),
        scene.paths().len(),
        "a projeção tem de cobrir TODA forma do documento — quem ficar de fora leva chave 0 \
         no `reorder_to` e vai pro FUNDO"
    );
    for id in ids {
        assert!(order.contains(&id), "a forma {id} nasceu fora da projeção");
    }
}

/// **A pilha de z é a árvore invertida.** A Hierarquia lista a primeira linha à FRENTE
/// (convenção Illustrator/Figma), então o fundo da pilha é a última linha.
///
/// Sem esta asserção o gate acima ficaria verde com a ordem embaralhada — "estar na projeção"
/// não diz nada sobre ONDE.
#[test]
fn the_z_stack_is_the_tree_read_backwards() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let [a, b, c] = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);

    // `sync` dá `RootOrder` crescente na ordem de criação ⇒ a Hierarquia lista a, b, c ⇒ a
    // pilha de z (fundo → topo) é c, b, a: a última forma desenhada fica por CIMA.
    let tree: Vec<VecPathId> = frame
        .snap
        .entries
        .iter()
        .filter_map(|e| e.vec_path)
        .collect();
    assert_eq!(tree, vec![a, b, c], "a árvore lista na ordem de criação");
    assert_eq!(z(&scene), vec![c, b, a], "a cena empilha ao contrário");
}

/// **O GATE-MÃE: a captura é ponto fixo dos sistemas.**
///
/// Fotografa no fim do frame da ação, deixa o frame seguinte rodar (sem input do usuário) e
/// fotografa de novo. Qualquer diferença aqui é uma mutação que os sistemas fizeram **sozinhos**
/// — e o `post_frame_undo` vai lê-la como se fosse o usuário.
#[test]
fn the_capture_is_a_fixed_point_of_the_systems() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map); // o frame da AÇÃO
    let shot = frame.capture(&sim, &scene); // ← é aqui que o undo fotografa

    frame.run(&mut sim, &mut scene, &mut map); // o frame seguinte, sem input nenhum
    let again = frame.capture(&sim, &scene);

    assert_eq!(
        shot, again,
        "os sistemas reescreveram o estado depois da foto — o diff do `post_frame_undo` vai \
         registrar isso como um passo do usuário, limpar o redo, e o Ctrl+Z seguinte vai \
         desfazer o próprio lixo"
    );
}

/// **O bug do Enio, de ponta a ponta: o Ctrl+Z restaura um estado que o frame seguinte NÃO
/// reescreve.**
///
/// É o gate mais próximo do produto — passa pelo `restore`, que **despawna e re-spawna** toda
/// entidade (ids de alocação NOVOS). Se a projeção de z dependesse de `Entity::to_bits()`, ela
/// mudaria no respawn e o passo espúrio voltaria vestido de outra coisa.
#[test]
fn undo_restores_a_state_that_the_next_frame_does_not_rewrite() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    // Frame 1: a cena base.
    three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);
    let baseline = frame.capture(&sim, &scene);

    // Frame 2: a AÇÃO (o Shape Builder produz formas novas).
    scene.push_path(rectangle([6.0, 0.0], [7.0, 1.0]));
    scene.push_path(rectangle([8.0, 0.0], [9.0, 1.0]));
    frame.run(&mut sim, &mut scene, &mut map);
    let after = frame.capture(&sim, &scene);
    assert_ne!(
        baseline, after,
        "a ação tem de mudar o estado (senão o gate é vazio)"
    );

    // Ctrl+Z: restaura o baseline (respawn — bits novos) e deixa o frame seguinte rodar.
    let (restored_scene, restored_map, _flip, _flip_map) = baseline.restore(&mut sim, &frame.reg);
    scene = restored_scene;
    map = restored_map;
    frame.run(&mut sim, &mut scene, &mut map);
    let settled = frame.capture(&sim, &scene);

    assert_eq!(
        settled, baseline,
        "o frame que corre DEPOIS do undo reescreveu o estado restaurado — é exatamente este \
         delta que virava um passo espúrio, limpava o redo, e fazia o 2º Ctrl+Z desfazer o lixo \
         do 1º ('o undo só faz uma etapa')"
    );
}

/// A mesma pergunta, com a forma **pendurada num sprite** (parentesco cruzado, ADR-0110).
///
/// Um sprite importado nasce **sem `RootOrder`** (`image_import.rs`), e a árvore desempata as
/// raízes sem ordem por `Entity::to_bits()` — o id de ALOCAÇÃO, que o respawn do undo TROCA.
/// Se esse desempate vazasse para a pilha de z, a cena sairia reordenada de todo Ctrl+Z.
#[test]
fn a_shape_parented_to_a_sprite_survives_the_respawn_in_the_same_z_order() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    // Dois sprites-raiz SEM `RootOrder` (como o import os cria), cada um com uma forma filha.
    let s1 = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Spr 1")))
        .id();
    let s2 = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Spr 2")))
        .id();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    frame.run(&mut sim, &mut scene, &mut map);
    for (path, parent) in [(a, s1), (b, s2)] {
        let e = Entity::from_bits(map[&path]);
        sim.world_mut().entity_mut(e).insert(ChildOf(parent));
    }
    frame.run(&mut sim, &mut scene, &mut map);

    let before = z(&scene);
    let shot = frame.capture(&sim, &scene);

    // Ctrl+Z sobre si mesmo: restaura a MESMA foto (respawn ⇒ bits novos) e roda um frame.
    let (restored_scene, restored_map, _flip, _flip_map) = shot.restore(&mut sim, &frame.reg);
    scene = restored_scene;
    map = restored_map;
    frame.run(&mut sim, &mut scene, &mut map);

    assert_eq!(
        z(&scene),
        before,
        "a pilha de z mudou só porque as entidades foram re-spawnadas — a projeção está \
         ancorada no id de ALOCAÇÃO, não no conteúdo"
    );
    assert_eq!(
        frame.capture(&sim, &scene),
        shot,
        "e a captura deixou de ser ponto fixo"
    );
}
