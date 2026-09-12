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

use crate::undo::ProjectState;
use ph2d_ecs::scene::{
    ComponentRegistry, HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot,
    register_ecs_components,
};
use ph2d_ecs::{ChildOf, Entity, SimWorld};
use ph2d_ecs::{Name, Transform};
use ph2d_flip::FlipDoc;
use ph2d_vec_entities::entities::sync;
use ph2d_vec_entities::entities::zorder::*;
use ph2d_vec_entities::entity_map::VecEntityMap;
use ph2d_vec_scene::VecPathId;
use ph2d_vec_scene::{VecScene, rectangle};

/// O pedaço do frame que **muta o estado que o undo fotografa** — a mesma sequência, na mesma
/// ordem, do `render_loop/mod.rs`: a ponte doc↔árvore, o assentamento do pivô, e a projeção
/// de z. (O `connector_live::upkeep` e o `flip_transform::settle_origins` correm no meio, mas
/// não tocam nada disto: sem conector e sem objeto Flip na cena, são no-op.)
pub(super) struct Frame {
    walk: HierarchyWalkState,
    scratch: Vec<(Entity, u8, Option<Entity>)>,
    snap: HierarchySnapshot,
    reg: ComponentRegistry,
    /// A cache da captura incremental (F2) — ela tem de SOBREVIVER entre quadros, senão o
    /// ponto fixo que estes gates medem seria sempre uma primeira captura.
    undo_cache: ph2d_ecs::scene::incremental::CaptureCache,
}

impl Frame {
    pub(super) fn new(sim: &mut SimWorld) -> Self {
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
            undo_cache: ph2d_ecs::scene::incremental::CaptureCache::new(),
        }
    }

    /// Roda um frame de sistemas sobre o documento.
    pub(super) fn run(&mut self, sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) {
        self.run_with_drag(sim, scene, map, None);
    }

    /// O mesmo frame, **com o arrasto da Hierarquia dentro dele** — no sítio exacto em que o
    /// `render_loop/mod.rs` o aplica: depois do `sync` e dos `assign_missing_*`, e **antes** de a
    /// árvore ser lida pela projecção.
    ///
    /// ⛔⛔ **Era esta a lacuna que deixou passar o report de 2026-09-07.** Este arnês diz-se *«o
    /// pedaço do frame que muta o estado que o undo fotografa»* e **não continha o dreno do
    /// reparent** — logo nenhum gate deste ficheiro podia ver que, no produto, ele corria ~2 340
    /// linhas DEPOIS da projecção. *Um arnês que modela o quadro mede exactamente os escritores
    /// que alguém se lembrou de lhe pôr dentro.*
    fn run_with_drag(
        &mut self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &mut VecEntityMap,
        drag: Option<(
            &crate::HeroLive,
            ph2d_editor::screens::hero::HierReparentIntent,
        )>,
    ) {
        sync(sim, scene, map);
        ph2d_vec_entities::transform::settle_origins(sim, scene, map, &[]);
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
        if let Some((live, intent)) = drag {
            let mut toasts = ph2d_editor::ToastQueue::new();
            crate::hero_intents::drain_reparent(intent, live, sim, &mut toasts);
        }
        build_hierarchy_snapshot(
            sim.world(),
            &mut self.walk,
            &mut self.scratch,
            &mut self.snap,
        );
        let order = z_order(sim.world(), &self.snap);
        scene.reorder_to(&order);
    }

    /// ⭐⭐⭐ **O quadro com uma mutação TARDIA** — aplicada **depois** da projecção, que é onde o
    /// `render_loop::hierarchy::dispatch` de facto corre (~2 300 linhas abaixo dela).
    ///
    /// ⚠️ **Ela não modela um verbo, modela a CLASSE.** Apagar, duplicar e *Remove from Sheet* são
    /// gestos diferentes com plumbing diferente (o `HeroScreen`, a voz, a câmara), e nenhum deles é
    /// montável num teste de unidade. O que os três têm em comum — e é a única coisa que decide o
    /// ponto fixo — é **mutar a árvore ou a cena DEPOIS de a árvore ter sido lida**. *Um gate por
    /// verbo mediria a plumbing; este mede a lei.*
    pub(super) fn run_with_late(
        &mut self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &mut VecEntityMap,
        late: impl FnOnce(&mut SimWorld, &mut VecScene),
    ) {
        self.run(sim, scene, map);
        late(sim, scene);
        self.settle(sim, scene, map);
    }

    /// ⭐⭐⭐ **A RECONCILIAÇÃO ANTES DA CAPTURA** — o espelho de
    /// [`crate::vec_tree_settle::App::settle_tree_before_capture`], que no produto corre entre o
    /// `serve_prefab_exit` e o `post_frame_undo`.
    ///
    /// ⚠️ **É a REDE, não um sítio melhor para os escritores.** A projecção de cima continua onde
    /// está porque ~40 consumidores da cena a leem antes do desenho; esta segunda passagem existe
    /// só para que a FOTOGRAFIA seja ponto fixo. *Uma serve o que se vê no quadro, a outra o que se
    /// guarda dele.*
    ///
    /// **Mutação que deve sangrar:** apagar a chamada no [`Frame::run_with_late`] — os dois gates
    /// dos verbos tardios voltam a vermelho, com a mensagem do passo fantasma.
    pub(super) fn settle(
        &mut self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &mut VecEntityMap,
    ) {
        sync(sim, scene, map);
        // ⭐⭐ **O assentamento do pivô entra aqui, e foi o gate do *duplicar* que o disse**: sem
        // ele os dois controlos passavam e o ponto fixo continuava vermelho, porque a entidade
        // cunhada agora nasce com `Transform::default()` e o quadro seguinte assentava-a sozinho.
        ph2d_vec_entities::transform::settle_origins(sim, scene, map, &[]);
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
        build_hierarchy_snapshot(
            sim.world(),
            &mut self.walk,
            &mut self.scratch,
            &mut self.snap,
        );
        let order = z_order(sim.world(), &self.snap);
        scene.reorder_to(&order);
    }

    /// A ordem de z que a árvore dita AGORA — a metade de leitura da passagem, isolada para a
    /// medição por peça (`the_second_pass_costs_this_much_of_a_frame`) poder dizer onde está o teto.
    pub(super) fn snapshot_now(&mut self, sim: &mut SimWorld) -> Vec<VecPathId> {
        build_hierarchy_snapshot(
            sim.world(),
            &mut self.walk,
            &mut self.scratch,
            &mut self.snap,
        );
        z_order(sim.world(), &self.snap)
    }

    /// O que o `post_frame_undo` fotografa no fim do frame.
    pub(super) fn capture(&mut self, sim: &mut SimWorld, scene: &VecScene) -> ProjectState {
        ProjectState::capture(
            // Nada sob condução: estes gates são do ponto FIXO dos sistemas de vetor.
            &ph2d_preview_drive::PreviewDrive::default(),
            sim,
            scene,
            &FlipDoc::new(),
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &self.reg,
            &mut self.undo_cache,
            None,
        )
    }
}

/// A ordem de z da cena (fundo → topo), que é o que o `reorder_to` escreve.
pub(super) fn z(scene: &VecScene) -> Vec<VecPathId> {
    scene.paths().iter().map(|p| p.id).collect()
}

/// Três formas soltas, criadas AGORA — como o Shape Builder as cria (ainda sem entidade).
pub(super) fn three_fresh_shapes(scene: &mut VecScene) -> [VecPathId; 3] {
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

    let order = z_order(sim.world(), &frame.snap);
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

/// **A forma NOVA nasce na FRENTE** — e desde a lei de Godot (2026-08-04) isso é o FIM da lista.
///
/// A Hierarquia lista a ÚLTIMA linha à frente (a convenção das game engines, que o Enio pediu:
/// *"em Godot os objetos mais abaixo na hierarquia aparecem na frente"*), e a pilha de z é a lista
/// **na ordem**. O `sync` dá ao path novo o **maior** `RootOrder`, que é a última linha:
/// **desenhar uma forma em cima de outra a põe por cima**, e ela entra no fim da lista — que é o
/// outro pedido do mesmo report (*"um objeto novo vai para o último abaixo na hierarquia"*).
///
/// ⚠️ **As duas metades deste gate já estiveram invertidas, e é por isso que ele afirma AS DUAS.**
/// O que importa não é o número: é que *a mais nova aparece por cima* — e uma delas sozinha fica
/// verde sob a convenção errada.
#[test]
fn a_new_shape_is_born_in_front() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let [a, b, c] = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);

    // A árvore lista a mais NOVA por último (última linha = frente).
    let tree: Vec<VecPathId> = frame
        .snap
        .entries
        .iter()
        .filter_map(|e| e.vec_path)
        .collect();
    assert_eq!(
        tree,
        vec![a, b, c],
        "a Hierarquia lista a mais nova na ULTIMA linha"
    );
    assert_eq!(
        z(&scene),
        vec![a, b, c],
        "a pilha de z (do fundo ao topo) poe a mais nova por CIMA"
    );
}

/// **A forma nova entra no FIM de uma cena que já tem gente** — a metade que o gate acima não
/// alcança, e é ela que mede o pedido do Enio (*"um objeto novo vai para o último abaixo na
/// hierarquia"*).
///
/// ⚠️ **A mutação *"nascer no começo"* SOBREVIVEU ao gate irmão**, e a razão é fixture: lá as três
/// formas nascem num mundo VAZIO, onde *o próximo lugar livre* e *o lugar zero* são o mesmo número.
/// O fenômeno só existe quando já há raízes — então a fixture tem de ter DUAS levas.
#[test]
fn a_shape_born_into_a_populated_scene_lands_at_the_end() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let [a, b, c] = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);

    // A 2ª leva: uma forma desenhada DEPOIS, num documento que já tem três.
    let d = scene.push_path(rectangle([6.0, 0.0], [7.0, 1.0]));
    frame.run(&mut sim, &mut scene, &mut map);

    let tree: Vec<VecPathId> = frame
        .snap
        .entries
        .iter()
        .filter_map(|e| e.vec_path)
        .collect();
    assert_eq!(
        tree,
        vec![a, b, c, d],
        "a forma nova nao entrou no FIM da Hierarquia"
    );
    assert_eq!(
        *z(&scene).last().unwrap(),
        d,
        "e o fim da lista tem de ser a FRENTE do desenho"
    );
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
    let shot = frame.capture(&mut sim, &scene); // ← é aqui que o undo fotografa

    frame.run(&mut sim, &mut scene, &mut map); // o frame seguinte, sem input nenhum
    let again = frame.capture(&mut sim, &scene);

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
    let baseline = frame.capture(&mut sim, &scene);

    // Frame 2: a AÇÃO (o Shape Builder produz formas novas).
    scene.push_path(rectangle([6.0, 0.0], [7.0, 1.0]));
    scene.push_path(rectangle([8.0, 0.0], [9.0, 1.0]));
    frame.run(&mut sim, &mut scene, &mut map);
    let after = frame.capture(&mut sim, &scene);
    assert_ne!(
        baseline, after,
        "a ação tem de mudar o estado (senão o gate é vazio)"
    );

    // Ctrl+Z: restaura o baseline (respawn — bits novos) e deixa o frame seguinte rodar.
    let (restored_scene, restored_map, _flip, _flip_map) = baseline.restore(&mut sim, &frame.reg);
    scene = restored_scene;
    map = restored_map;
    frame.run(&mut sim, &mut scene, &mut map);
    let settled = frame.capture(&mut sim, &scene);

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
    let shot = frame.capture(&mut sim, &scene);

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
        frame.capture(&mut sim, &scene),
        shot,
        "e a captura deixou de ser ponto fixo"
    );
}

/// ⭐⭐⭐ **UM ARRASTO NA HIERARQUIA DEIXA A CAPTURA EM PONTO FIXO** — o report do Enio de
/// 2026-09-07 (*«reordenei objectos na hierarquia e não funcionou o undo»*), medido.
///
/// # O que estava partido
///
/// O dreno do reparent corria dentro do `hierarchy::dispatch`, **depois** da projecção de z do
/// mesmo quadro. A corrida com `PH2D_UNDO_LOG=1` mostra a doença inteira:
///
/// ```text
/// [hier] ordem das raizes logo depois de aplicar: [(1,0), (2,2), (3,1)]
/// [undo] passo registrado (fila undo=4) — diff: world=true vec=false     ← só metade
/// [undo] ⛔ o documento MUDOU em ["vec"] ... SUPRIMIDO — sem entrada     ← a outra, tarde
/// [undo] passo registrado (fila undo=5) — partes: ["vec"]               ← o FANTASMA
/// [undo]   vec: base=[0,1,2] atual=[0,2,1] · so a ORDEM=true
/// [undo] Ctrl+Z respondido por Global
/// [undo]   ordem das raizes depois do restauro: [(1,0), (2,2), (3,1)]   ← não voltou
/// ```
///
/// O `Ctrl+Z` repõe a pilha e não a árvore; a projecção do quadro seguinte re-deriva a pilha da
/// árvore que ninguém desfez, e o fantasma **renasce** — a fila fica parada em `5` e cada `Ctrl+Z`
/// gasta um passo que o próprio quadro volta a criar. O passo REAL nunca é alcançado.
///
/// # Porque é ESTE o gate
///
/// A lei do módulo já estava escrita: *a captura tem de ser ponto fixo dos sistemas*. O que
/// faltava era **um escritor da árvore dentro do arnês** — sem ele, o ponto fixo era medido sobre
/// um quadro em que ninguém reordenava nada.
///
/// ⚠️ **A primeira metade é o CONTROLO.** Sem ela, um arrasto que não reordena passaria por ponto
/// fixo — e um no-op é exactamente o que este report já produziu uma vez, quando o `before`/`after`
/// vazio mandava a peça para o fim de uma lista em que ela já era a última.
///
/// ⚠️ **Prova de mutação:** mover o `drain_reparent` do `run_with_drag` para DEPOIS do
/// `scene.reorder_to(&order)` — que é literalmente o produto de antes desta cura — deixa a segunda
/// captura diferente da primeira e o gate fica VERMELHO.
#[test]
fn a_hierarchy_drag_leaves_the_capture_a_fixed_point() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let [a, b, c] = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);
    assert_eq!(
        z(&scene),
        vec![a, b, c],
        "a cena nao partiu da ordem da arvore"
    );

    // A ponte nó ↔ entidade, montada como o prólogo do quadro a monta.
    let mut live = crate::HeroLive {
        bridge: crate::hero_bridge::EntityNodeMap::new(),
        walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        walk_scratch: Vec::new(),
        snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
        z_walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        z_walk_scratch: Vec::new(),
        z_snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
    };
    build_hierarchy_snapshot(
        sim.world(),
        &mut live.walk_state,
        &mut live.walk_scratch,
        &mut live.snapshot,
    );
    let _ = live.bridge.sync_from_snapshot(&live.snapshot);
    let node_of = |id: VecPathId| {
        let bits = *map.get(&id).expect("a forma tem entidade depois do sync");
        live.bridge
            .node_for(bits)
            .expect("a entidade esta na ponte")
    };

    // O GESTO: arrastar a última forma para ANTES da primeira.
    let intent = ph2d_editor::screens::hero::HierReparentIntent {
        dragged: node_of(c),
        new_parent: None,
        before: Some(node_of(a)),
        after: None,
    };
    frame.run_with_drag(&mut sim, &mut scene, &mut map, Some((&live, intent)));
    let depois_do_arrasto = frame.capture(&mut sim, &scene);

    // ⚠️ **CONTROLO**: o arrasto tem de ter movido alguma coisa, senão o ponto fixo abaixo é
    // vácuo. A `c` passa a ser a de TRÁS (o fundo da pilha é o começo da lista).
    assert_eq!(
        z(&scene),
        vec![c, a, b],
        "o arrasto real nao reordenou a pilha de z no MESMO quadro"
    );

    // E agora o quadro seguinte, **sem entrada nenhuma**.
    frame.run(&mut sim, &mut scene, &mut map);
    let quadro_seguinte = frame.capture(&mut sim, &scene);

    assert!(
        depois_do_arrasto == quadro_seguinte,
        "a captura do quadro do arrasto NAO e ponto fixo: o quadro seguinte mudou o documento \
         sozinho, e o `post_frame_undo` le isso como uma accao do artista — e' assim que nasce o \
         passo fantasma de `partes: [\"vec\"]` que o Ctrl+Z nunca consegue esgotar"
    );
}
