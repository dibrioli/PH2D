//! Undo/redo GLOBAL do editor — uma fila só, muitos tipos de objeto.
//!
//! A unidade é o **estado do projeto** num instante: o mundo (ECS) mais a geometria
//! vetorial. Undo e save gravam a MESMA captura — o save só adiciona os bytes das
//! imagens por cima (`crate::project`, bloco seguinte).
//!
//! **Snapshot-based, não command-based.** Cada passo guarda o estado ANTES da ação
//! e o restaura; não há operação invertível por-tipo. É o que faz "mover, deletar,
//! reparentear, criar, editar nó" caberem numa fila só, sem um `match` gigante — e
//! subsume o `ph2d_vec_edit::History` (o `VecScene` já está na captura).
//!
//! **Registro por DIFF, não por gesto.** [`crate::App::post_frame_undo`] roda uma vez
//! por frame: se houve input e nenhum gesto está em curso, compara o estado atual com
//! o baseline; qualquer diferença vira um passo. Um só ponto torna toda ação
//! desfazível (gizmo, pen, tecla, botão) sem instrumentar cada site.
//!
//! **Escopo (Enio 2026-07-09):** objetos, hierarquia e canvas. NÃO toca painéis —
//! as configs de painel têm undo próprio, com botões no header (bloco à parte).

use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Entity, SimWorld, Transform, TransformPropagationState, With, WorklistBuf};
use ph2d_flip::FlipDoc;
use ph2d_vec_scene::VecScene;

/// Profundidade máxima da pilha (interações comuns, não infinitas). Igual ao
/// `HISTORY_CAP` do vetor.
const UNDO_CAP: usize = 256;

/// O estado mutável do projeto num instante: o mundo + a geometria vetorial.
///
/// `WorldSnapshot` cobre toda entidade com componente registrado — pose, nome,
/// árvore, trava, e as referências que ligam um path (`VecPathRef`) ou um objeto
/// Flip (`FlipObjectRef`) à entidade (ADR-0110/0113). `VecScene` e `FlipDoc` são
/// as geometrias, que vivem fora do ECS. Juntos são o projeto inteiro exceto os
/// pixels dos sprites (estáveis, não mudam a cada ação — o save os anexa à parte).
///
/// `FlipDoc` é determinístico (Vec/BTreeMap/ids monotônicos), então — ao contrário
/// do `WorldSnapshot` — não precisa de `canonicalize`: capturar o mesmo estado
/// duas vezes dá `FlipDoc`s iguais e o diff de undo não registra passo espúrio.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectState {
    pub(crate) world: WorldSnapshot,
    pub(crate) vec: VecScene,
    pub(crate) flip: FlipDoc,
}

impl ProjectState {
    /// Captura o estado atual. `prop`/`worklist` são scratch reusado (o
    /// `world_to_snapshot` é zero-alloc além do crescimento do próprio snapshot).
    #[must_use]
    pub(crate) fn capture(
        sim: &SimWorld,
        vec: &VecScene,
        flip: &FlipDoc,
        registry: &ComponentRegistry,
        prop: &mut TransformPropagationState,
        worklist: &mut WorklistBuf,
    ) -> Self {
        let mut world = WorldSnapshot::new();
        // O snapshot só falha se um componente registrado não (de)serializa — um bug
        // de registro, não estado do usuário. Um estado vazio é o degradado seguro.
        let _ = world_to_snapshot(sim.world(), prop, worklist, registry, &mut world);
        canonicalize(&mut world);
        Self {
            world,
            vec: vec.clone(),
            flip: flip.clone(),
        }
    }

    /// Restaura este estado. Limpa as entidades editáveis do mundo, re-spawna do
    /// snapshot, e devolve as geometrias + as **pontes reconstruídas** (vetor e
    /// Flip; os mapas são runtime-only — sem o rebuild, o `sync` duplicaria as
    /// formas/objetos).
    ///
    /// O chamador atribui os quatro: `gfx.vec_scene = vec; gfx.flip = flip;
    /// self.vec_entities = vec_map; self.flip_entities = flip_map`.
    #[must_use]
    pub(crate) fn restore(
        &self,
        sim: &mut SimWorld,
        registry: &ComponentRegistry,
    ) -> (
        VecScene,
        crate::vec_entities::VecEntityMap,
        FlipDoc,
        crate::flip_entities::FlipEntityMap,
    ) {
        // 1. Limpa: toda entidade editável tem `Transform` (sprites, formas,
        //    objetos Flip, grupos). O despawn cascateia por `ChildOf`, então um
        //    filho já removido é benigno.
        let editable: Vec<Entity> = {
            let mut q = sim.world_mut().query_filtered::<Entity, With<Transform>>();
            q.iter(sim.world()).collect()
        };
        for e in editable {
            let _ = sim.world_mut().despawn(e);
        }
        // 2. Re-spawna do snapshot (ids do mundo são novos — o snapshot é portável).
        let _ = snapshot_to_world(sim.world_mut(), &self.world, registry);
        // 3. Reconstrói as pontes a partir dos `VecPathRef`/`FlipObjectRef`
        //    restaurados.
        let vec_map = crate::vec_entities::rebuild_map(sim);
        let flip_map = crate::flip_entities::rebuild_map(sim);
        (self.vec.clone(), vec_map, self.flip.clone(), flip_map)
    }
}

/// Reordena as entidades do snapshot por **conteúdo**, não por `Entity::to_bits()`.
///
/// O `world_to_snapshot` ordena por `to_bits()` — o id de ALOCAÇÃO do bevy, que muda
/// a cada spawn. Então dois estados logicamente iguais, mas cujas entidades foram
/// spawnadas em ordem diferente (o que um restore SEMPRE causa), produzem snapshots
/// byte-diferentes. Para o diff de undo isso é fatal: registraria um passo espúrio a
/// cada frame com input, e o Ctrl+Z pareceria "não fazer nada" (Enio 2026-07-09).
///
/// A chave de ordenação é a serialização dos componentes da entidade (já id-sorted
/// pelo registry), então a ordem passa a ser função do CONTEÚDO — estável entre
/// re-spawns. O `parent` (índice no vec) é remapeado para a nova ordem. Duas
/// entidades byte-idênticas empatam; a ordem entre elas é indiferente (são
/// intercambiáveis), então o snapshot resultante é o mesmo de qualquer jeito.
fn canonicalize(snap: &mut WorldSnapshot) {
    let key = |row: &ph2d_ecs::scene::EntitySnapshotRow| -> Vec<u8> {
        let mut k = Vec::new();
        for b in &row.components {
            k.extend_from_slice(&b.type_id.to_le_bytes());
            k.extend_from_slice(&b.data);
        }
        k
    };
    let n = snap.entities.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| key(&snap.entities[a]).cmp(&key(&snap.entities[b])));
    // new_index[old] = posição de `old` na ordem canônica.
    let mut new_index = vec![0u32; n];
    for (new, &old) in order.iter().enumerate() {
        new_index[old] = new as u32;
    }
    let old_rows = std::mem::take(&mut snap.entities);
    // `order` indexa `old_rows`; move cada linha para a posição nova.
    let mut slots: Vec<Option<ph2d_ecs::scene::EntitySnapshotRow>> =
        old_rows.into_iter().map(Some).collect();
    let mut reordered: Vec<ph2d_ecs::scene::EntitySnapshotRow> = Vec::with_capacity(n);
    for &old in &order {
        let mut row = slots[old].take().expect("cada índice usado uma vez");
        row.parent = row.parent.map(|p| new_index[p as usize]);
        reordered.push(row);
    }
    snap.entities = reordered;
}

/// A pilha de undo/redo global. O registro de passos é dirigido por **diff de
/// estado** no [`crate::App::post_frame_undo`] (não por begin/commit de gesto), então
/// a API é só `push_undo` + `undo`/`redo`.
#[derive(Default)]
pub(crate) struct ProjectUndo {
    undo: Vec<ProjectState>,
    redo: Vec<ProjectState>,
}

impl ProjectUndo {
    /// Empurra um estado-pré (o baseline antes da ação detectada). Limpa o redo.
    pub(crate) fn push_undo(&mut self, pre: ProjectState) {
        if self.undo.len() >= UNDO_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Quantos passos há na pilha de undo (para o log de diagnóstico).
    pub(crate) fn depth(&self) -> usize {
        self.undo.len()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Desfaz: devolve o estado anterior; empurra o `current` pro redo.
    #[must_use]
    pub(crate) fn undo(&mut self, current: ProjectState) -> Option<ProjectState> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    /// Refaz: devolve o próximo estado; empurra o `current` de volta pro undo.
    #[must_use]
    pub(crate) fn redo(&mut self, current: ProjectState) -> Option<ProjectState> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }
}

impl crate::App {
    /// Captura o estado do projeto AGORA. `None` se o gfx ainda não inicializou.
    #[must_use]
    pub(crate) fn capture_project(&mut self) -> Option<ProjectState> {
        let gfx = self.gfx.as_mut()?;
        Some(ProjectState::capture(
            &gfx.sim,
            &gfx.vec_scene,
            &gfx.flip,
            &gfx.component_registry,
            &mut gfx.prop_state,
            &mut gfx.worklist,
        ))
    }

    /// Aplica um estado restaurado ao App: restaura o mundo + a geometria, reconstrói
    /// a ponte, e limpa toda seleção (o restore dá ids de entidade NOVOS, então bits
    /// antigos ficam mortos). Re-arma o baseline para não re-detectar como mudança.
    pub(crate) fn apply_project(&mut self, state: &ProjectState) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let (vec, map, flip, flip_map) = state.restore(&mut gfx.sim, &gfx.component_registry);
        gfx.vec_scene = vec;
        gfx.flip = flip;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.clear_all_selection();
        }
        self.vec_entities = map;
        self.flip_entities = flip_map;
        self.vec_sel = crate::vec_selection::VecSelSync::default();
        self.vec_pen.clear();
        self.undo_baseline = Some(state.clone());
        self.title_dirty = true;
    }

    /// Desfaz (ou refaz) um passo da fila global: empurra o estado atual pro outro
    /// lado e aplica o restaurado.
    pub(crate) fn apply_undo(&mut self, redo: bool) {
        let Some(current) = self.capture_project() else {
            return;
        };
        let restored = if redo {
            self.undo.redo(current)
        } else {
            self.undo.undo(current)
        };
        if let Some(state) = restored {
            self.apply_project(&state);
            if std::env::var_os("PH2D_UNDO_LOG").is_some() {
                eprintln!("[undo] {} aplicado", if redo { "redo" } else { "undo" });
            }
        } else if std::env::var_os("PH2D_UNDO_LOG").is_some() {
            eprintln!("[undo] {} sem passo", if redo { "redo" } else { "undo" });
        }
    }

    /// Chamado UMA vez por frame, depois de `render_frame` (com `self` livre e o
    /// estado já reconciliado pelo `sync`). Duas coisas, nesta ordem:
    ///
    /// 1. **Ctrl+Z/Y** pendente: aplica e re-arma o baseline (então o passo 2 vê o
    ///    estado restaurado == baseline e não registra um passo espúrio).
    /// 2. **Auto-commit**: se houve input neste frame e nenhum gesto está em curso,
    ///    compara o estado atual com o baseline; se mudou, o baseline antigo (o
    ///    estado-pré) vira um passo de undo e o novo vira o baseline. É o ponto único
    ///    que torna "tudo que o usuário fizer" desfazível, por diff — sem instrumentar
    ///    cada ação.
    pub(crate) fn post_frame_undo(&mut self) {
        let had_input = std::mem::take(&mut self.any_input_this_frame);
        if let Some(redo) = self.undo_request.take() {
            self.apply_undo(redo);
            return;
        }
        // Estabelece o baseline no PRIMEIRO frame (gfx pronto), antes de qualquer
        // ação. Sem isto a primeira ação não teria pré-estado e não seria desfazível.
        if self.undo_baseline.is_none() {
            self.undo_baseline = self.capture_project();
            return;
        }
        // Um gesto em andamento (botão pressionado) muta a cada Move; espera o fim
        // para não registrar um passo por frame.
        if !had_input || self.held_button.is_some() {
            return;
        }
        let Some(current) = self.capture_project() else {
            return;
        };
        if self.undo_baseline.as_ref() == Some(&current) {
            return; // nada mudou desde o último passo
        }
        if std::env::var_os("PH2D_UNDO_LOG").is_some() {
            eprintln!(
                "[undo] passo registrado (fila undo={})",
                self.undo.depth() + 1
            );
        }
        if let Some(base) = self.undo_baseline.replace(current) {
            self.undo.push_undo(base);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
    use ph2d_ecs::{Name, VecPathRef};
    use ph2d_render::register_render_components;
    use ph2d_vec_scene::rectangle;

    fn registry() -> ComponentRegistry {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        register_render_components(&mut reg);
        reg
    }

    /// Uma cena de teste: 1 sprite nomeado + 1 forma vetorial com a entidade dela.
    fn scene() -> (SimWorld, VecScene) {
        let mut sim = SimWorld::new();
        let mut vec = VecScene::new();
        sim.world_mut().spawn((
            Transform::from_translation(ph2d_core::Vec2::new(3.0, 4.0)),
            Name::new("Spr"),
        ));
        let id = vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
        sim.world_mut()
            .spawn((Transform::default(), Name::new("Path 0"), VecPathRef(id)));
        (sim, vec)
    }

    fn capture(sim: &mut SimWorld, vec: &VecScene, reg: &ComponentRegistry) -> ProjectState {
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut wl = WorklistBuf::new();
        ProjectState::capture(sim, vec, &FlipDoc::new(), reg, &mut prop, &mut wl)
    }

    /// Capturar → mexer no mundo E na geometria → restaurar devolve o estado exato,
    /// e a ponte path↔entidade fica reconstruída (uma entrada por forma).
    #[test]
    fn capture_then_restore_round_trips_world_and_geometry() {
        let reg = registry();
        let (mut sim, mut vec) = scene();
        let before = capture(&mut sim, &vec, &reg);

        // Muda tudo: move o sprite, apaga a forma, desenha outra.
        {
            let mut q = sim.world_mut().query::<&mut Transform>();
            for mut t in q.iter_mut(sim.world_mut()) {
                t.translation.x += 100.0;
            }
        }
        vec = VecScene::new();
        vec.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));

        let (rvec, map, _rflip, _fmap) = before.restore(&mut sim, &reg);
        vec = rvec;

        // A geometria voltou ao original.
        assert_eq!(vec.paths().len(), 1);
        assert_eq!(vec.paths()[0].verts[0].anchor, [0.0, 0.0]);
        // A ponte tem exatamente a forma restaurada, e aponta uma entidade viva.
        assert_eq!(map.len(), 1);
        let (&pid, &bits) = map.iter().next().unwrap();
        assert_eq!(pid, vec.paths()[0].id);
        assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());
        // O sprite voltou à posição original (a captura preservou o Transform).
        let names: Vec<f32> = {
            let mut q = sim.world_mut().query::<(&Name, &Transform)>();
            q.iter(sim.world())
                .filter(|(n, _)| n.as_str() == "Spr")
                .map(|(_, t)| t.translation.x)
                .collect()
        };
        assert_eq!(names, vec![3.0]);
    }

    /// A ponte reconstruída faz o `sync` seguinte virar NO-OP — não duplica.
    #[test]
    fn the_rebuilt_bridge_makes_the_next_sync_a_noop() {
        let reg = registry();
        let (mut sim, vec) = scene();
        let snap = capture(&mut sim, &vec, &reg);
        let (mut rvec, mut map, _rflip, _fmap) = snap.restore(&mut sim, &reg);

        let entities_before = map.len();
        crate::vec_entities::sync(&mut sim, &mut rvec, &mut map);
        assert_eq!(map.len(), entities_before, "sync não spawnou nada");
        // Uma entidade por path, e nenhuma órfã (a contagem de VecPathRef == paths).
        let vecref_count = {
            let mut q = sim.world_mut().query::<&VecPathRef>();
            q.iter(sim.world()).count()
        };
        assert_eq!(vecref_count, rvec.paths().len());
    }

    /// push_undo/undo/redo alternam corretamente (o registro por-diff é testado no
    /// nível do App; aqui é a pilha pura).
    #[test]
    fn push_undo_then_undo_redo_alternate() {
        let reg = registry();
        let (mut sim, vec) = scene();
        let s0 = capture(&mut sim, &vec, &reg);
        let mut stack = ProjectUndo::default();
        assert!(!stack.can_undo() && !stack.can_redo());

        // Um passo: s0 é o estado-pré.
        let mut vec1 = vec.clone();
        vec1.push_path(rectangle([5.0, 5.0], [6.0, 6.0]));
        let s1 = ProjectState {
            world: s0.world.clone(),
            vec: vec1,
            flip: FlipDoc::new(),
        };
        stack.push_undo(s0.clone());
        assert!(stack.can_undo() && !stack.can_redo());

        // Undo devolve o pré (s0) e arma o redo com o atual (s1).
        let back = stack.undo(s1.clone()).unwrap();
        assert_eq!(back, s0);
        assert!(stack.can_redo());
        // Redo devolve s1 e re-arma o undo.
        let fwd = stack.redo(s0.clone()).unwrap();
        assert_eq!(fwd, s1);
        assert!(stack.can_undo());
        // push_undo limpa o redo.
        let _ = stack.undo(s1.clone());
        stack.push_undo(s0.clone());
        assert!(!stack.can_redo());
    }
    /// O diff de undo depende disto: capturar o MESMO estado duas vezes tem de dar
    /// snapshots IGUAIS. Se não, o `post_frame_undo` registraria um passo espúrio a
    /// cada frame com input (Enio 2026-07-09: "precisa de mais de 1 Ctrl+Z").
    #[test]
    fn capturing_the_same_state_twice_is_identical() {
        let reg = registry();
        let (mut sim, vec) = scene();
        let a = capture(&mut sim, &vec, &reg);
        let b = capture(&mut sim, &vec, &reg);
        assert_eq!(a, b, "captura não-determinística -> diff espúrio");
    }

    /// E depois de um restore (ids de entidade NOVOS), capturar de novo tem de dar o
    /// mesmo snapshot — senão o primeiro frame pós-undo registra um passo fantasma.
    #[test]
    fn capture_after_restore_matches_the_restored_state() {
        let reg = registry();
        let (mut sim, vec) = scene();
        let snap = capture(&mut sim, &vec, &reg);
        let (rvec, _, _, _) = snap.restore(&mut sim, &reg);
        let again = capture(&mut sim, &rvec, &reg);
        assert_eq!(snap, again, "restore->capture != capture original");
    }
    /// ADR-0113: o `FlipDoc` entra na captura como 3º campo. Round-trip por
    /// capture→restore, a ponte objeto↔entidade é reconstruída, e capturar o
    /// mesmo estado 2× é idêntico (o flip é determinístico — sem diff espúrio).
    #[test]
    fn flip_survives_capture_restore_and_rebuilds_bridge() {
        use ph2d_flip::{Hold, KeyKind};
        let reg = registry();
        let mut sim = SimWorld::new();
        let vec = VecScene::new();
        // Documento Flip: um objeto (camada + frame + traço) + a entidade-ponte.
        let mut flip = FlipDoc::new();
        let oid = flip.push_object("Char");
        {
            let obj = flip.object_mut(oid).unwrap();
            let l = obj.add_layer("L");
            let d = obj
                .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
                .unwrap();
            obj.drawing_mut(d).unwrap().strokes.push(Default::default());
        }
        sim.world_mut().spawn((
            Transform::default(),
            Name::new("Char"),
            ph2d_ecs::FlipObjectRef(oid.0),
        ));

        let snap = {
            let mut prop = TransformPropagationState::new(sim.world_mut());
            let mut wl = WorklistBuf::new();
            ProjectState::capture(&sim, &vec, &flip, &reg, &mut prop, &mut wl)
        };

        // Muda o flip (adiciona objeto) e restaura ao capturado.
        flip.push_object("Extra");
        let (_rvec, _vmap, rflip, fmap) = snap.restore(&mut sim, &reg);
        assert_eq!(rflip.objects().len(), 1, "o flip voltou ao capturado");
        assert_eq!(fmap.len(), 1, "a ponte flip reconstruída tem o objeto");
        let (&fid, &bits) = fmap.iter().next().unwrap();
        assert_eq!(fid, oid);
        assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());

        // Capturar o mesmo estado 2× = idêntico (sem passo espúrio de undo).
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut wl = WorklistBuf::new();
        let a = ProjectState::capture(&sim, &vec, &rflip, &reg, &mut prop, &mut wl);
        let b = ProjectState::capture(&sim, &vec, &rflip, &reg, &mut prop, &mut wl);
        assert_eq!(a, b, "flip determinístico -> sem diff espúrio");
    }

    /// REGRESSÃO (Enio 2026-07-09: "ao deletar não volta no mesmo lugar"). O
    /// `RootOrder` (a posição de z / linha na hierarquia) tem de sobreviver ao
    /// ciclo capturar → deletar → restaurar.
    #[test]
    fn delete_then_restore_preserves_z_order() {
        use ph2d_ecs::RootOrder;
        let reg = registry();
        let mut sim = SimWorld::new();
        let vec = VecScene::new();
        // Três sprites com ordens de z distintas.
        let a = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("A"), RootOrder(0)))
            .id();
        let b = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("B"), RootOrder(1)))
            .id();
        let _c = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("C"), RootOrder(2)))
            .id();
        let _ = (a, b);

        let snap = capture(&mut sim, &vec, &reg);
        // Deleta o do meio (B).
        sim.world_mut().despawn(b);
        // Restaura o estado pré-delete.
        let _ = snap.restore(&mut sim, &reg);

        // Cada nome volta com o RootOrder original.
        let mut got: Vec<(String, u32)> = {
            let mut q = sim.world_mut().query::<(&Name, &RootOrder)>();
            q.iter(sim.world())
                .map(|(n, r)| (n.as_str().to_owned(), r.0))
                .collect()
        };
        got.sort();
        assert_eq!(
            got,
            vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)],
            "o objeto deletado volta na MESMA posição de z"
        );
    }
}
