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
use ph2d_vec_scene::{VecPathId, VecScene};

/// Profundidade máxima da pilha (interações comuns, não infinitas). Igual ao
/// `HISTORY_CAP` do vetor.
const UNDO_CAP: usize = 256;

/// O estado mutável do projeto num instante: o mundo + a geometria vetorial.
///
/// `WorldSnapshot` cobre toda entidade com componente registrado — pose, nome,
/// árvore, trava, e as referências que ligam um path (`VecPathRef`) ou um objeto
/// Flip (`FlipObjectRef`) à entidade (ADR-0110/0114). `VecScene` e `FlipDoc` são
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

/// Os ids de forma que **sobreviveram** a um restore — a seleção que o artista tinha, menos o que o
/// estado restaurado não contém.
///
/// Separada de [`App::apply_project`] porque essa função exige `gfx` (janela + GPU) e não é
/// alcançável por um teste headless; a POLÍTICA, que é o que pode estar errado, é pura e gateada.
#[must_use]
pub(crate) fn surviving_selection(was: &[VecPathId], scene: &VecScene) -> Vec<VecPathId> {
    was.iter()
        .copied()
        .filter(|id| scene.paths().iter().any(|p| p.id == *id))
        .collect()
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
    /// a ponte, e limpa os **bits** de seleção (o restore dá ids de entidade NOVOS, então
    /// bits antigos ficam mortos). Re-arma o baseline para não re-detectar como mudança.
    ///
    /// ⚠️ **Os bits morrem; a SELEÇÃO não precisa morrer com eles.** Até 2026-07-18 esta função
    /// zerava também o `vec_pen`, e a consequência era um bug que o Enio reportou como *"o undo faz
    /// os pins sumirem, embora ainda funcionando"*: o envelope segue a deformar (o recook varre por
    /// QUERY) e o **overlay** — gaiola e pinos — é desenhado a partir da seleção, que já não existe.
    /// A ferramenta ficava funcionando e invisível.
    ///
    /// A seleção do pen é `VecPathId` e **viaja no snapshot** (a `VecScene` é parte do estado), então
    /// ela sobrevive: [`surviving_selection`] mantém os ids que ainda existem na cena restaurada, e o
    /// `sync_selection` do frame seguinte re-deriva os bits do mapa reconstruído. Nada de bits mortos
    /// atravessa — que era o perigo real que o zeramento defendia.
    pub(crate) fn apply_project(&mut self, state: &ProjectState) {
        // ANTES do restore: o que o artista tinha selecionado, em ids ESTÁVEIS.
        let was_selected = self.vec_pen.selected_paths().to_vec();
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
        // E de volta, filtrada pelo que sobreviveu. O `vec_sel` ficou zerado de propósito: no frame
        // seguinte o `sync_selection` vê "o pen mudou" e republica os bits NOVOS no gizmo.
        let alive = surviving_selection(&was_selected, &gfx.vec_scene);
        if !alive.is_empty() {
            self.vec_pen.select_many(&alive);
        }
        // Live Shapes: uma SESSÃO de texto viva reescreveria o `VecShape` (e a pose) da
        // entidade com os params dela a cada frame — ou seja, desfaria o undo no frame
        // seguinte. O estado restaurado é a verdade: a sessão termina (o objeto de texto
        // continua lá, editável pela seleção).
        self.vec_text_edit = None;
        self.vec_text_last_target = None;
        // O cache de poses dos rótulos é a memória de "o que EU escrevi no frame passado". O
        // restore põe poses novas no mundo sem passar por ele — deixá-lo faria o passe ler a
        // pose restaurada como um arrasto do usuário. É inócuo (o estado restaurado é
        // auto-consistente, e re-absorver devolve o MESMO offset), mas zerar é o honesto: a
        // memória não é mais de nada. O arm de rótulo pendente morre com a sessão de texto.
        self.vec_label_poses.clear();
        self.vec_label_pending = None;
        // O memo do Offset vivo é chaveado por `VecPathId`, e o restore RECICLA os ids: um id
        // que volta descrevendo outra forma acertaria o memo e desenharia o offset da forma
        // ANTERIOR sobre a nova, sem erro nenhum. O espelho do painel morre pela mesma razão
        // (a forma que ele espelhava pode ter deixado de ter offset).
        self.offset_live.forget();
        self.vec_offset_mirrored = None;
        // A mesma forma (mesmo id) pode voltar com OUTROS parâmetros — zerar o alvo
        // força a re-semente dos sliders, senão o painel seguiria mostrando o valor
        // que o undo acabou de desfazer.
        self.vec_shape_last_target = None;
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
            if Self::undo_log_on() {
                eprintln!("[undo] {} aplicado", if redo { "redo" } else { "undo" });
            }
        } else if Self::undo_log_on() {
            eprintln!("[undo] {} sem passo", if redo { "redo" } else { "undo" });
        }
    }

    /// **O log do undo está ligado?** Cacheado — os três sítios que o consultam correm por
    /// passo de undo, e ler o ambiente é caro o suficiente para não o fazer em cada um.
    pub(crate) fn undo_log_on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("PH2D_UNDO_LOG").is_some())
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
        // O clique no botão Undo/Redo da barra entra pela MESMA porta do Ctrl+Z (que pode
        // rotear para o Áudio, o Painter, o global ou o image-edit). Ele arma o
        // `undo_request` logo abaixo, e o passo é aplicado ainda neste frame.
        if let Some(redo) = self.undo_button.take() {
            self.undo_or_redo(redo);
        }
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
        if Self::undo_log_on() {
            let base = self.undo_baseline.as_ref();
            eprintln!(
                "[undo] passo registrado (fila undo={}) — diff: world={} vec={} flip={}",
                self.undo.depth() + 1,
                base.is_some_and(|b| b.world != current.world),
                base.is_some_and(|b| b.vec != current.vec),
                base.is_some_and(|b| b.flip != current.flip),
            );
            if let Some(b) = base
                && b.vec != current.vec
            {
                let ids = |v: &VecScene| v.paths().iter().map(|p| p.id).collect::<Vec<_>>();
                let (a, c) = (ids(&b.vec), ids(&current.vec));
                let mut sa = a.clone();
                let mut sc = c.clone();
                sa.sort_unstable();
                sc.sort_unstable();
                eprintln!(
                    "[undo]   vec: base={a:?} atual={c:?} · mesmos ids={} · só a ORDEM={}",
                    sa == sc,
                    sa == sc && a != c
                );
            }
        }
        if let Some(base) = self.undo_baseline.replace(current) {
            self.undo.push_undo(base);
        }
    }
}

#[cfg(test)]
#[path = "undo_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "undo_selection_tests.rs"]
mod selection_tests;
