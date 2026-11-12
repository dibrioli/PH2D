//! ⭐⭐⭐ **A RECONCILIAÇÃO ANTES DA CAPTURA** — a rede que apanha todo escritor TARDIO da árvore.
//!
//! # A lei, e porque ela precisou de uma segunda passagem

use crate::app_state::App;

/// Reconcilia a árvore com o documento e reprojecta a ordem de z — **a rede do fim do quadro**.
///
/// A sequência é a mesma do passe do desenho, e cada peça responde por um estado que um escritor
/// tardio pode ter deixado por fechar:
///
/// 1. [`ph2d_vec_entities::entities::sync`] — entidade apagada leva o caminho, caminho novo ganha entidade;
/// 2. `assign_missing_root_order` / `_stable_ids` / `_sibling_order` — uma entidade cunhada agora
///    entraria no snapshot **sem identidade e sem ordem**, e o primeiro `Ctrl+Z` não teria o que
///    repor. ⚠️ As três são **idempotentes**: se reescrevessem por quadro, o diff veria o arquétipo
///    de toda entidade mudar e cada quadro com entrada viraria um passo espúrio;
/// 3. a projecção — `build_hierarchy_snapshot` → [`ph2d_vec_entities::entities::z_order`] → `reorder_to`.
///
/// ⚠️ **Sem `hero_live` ela não faz nada, e isso é correcto**: o `HierarchyWalkState` e o
/// `HierarchySnapshot` vivem lá, e sem a tela da Hierarquia montada não há árvore lida neste
/// quadro — logo não há projecção para fechar.
impl App {
    pub(crate) fn settle_tree_before_capture(&mut self) {
        let anterior = self.undo_baseline.as_ref().map(|b| b.vec.clone());
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        ph2d_vec_entities::entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec.entities);
        // Todo vértice criado neste quadro ganha o seu `VertId` antes do passo de undo; uma forma
        // re-gerada herda os da fotografia anterior pela posição (W4.2).
        gfx.vec_scene.normaliza_ids_desde(anterior.as_deref());
        // ⭐⭐ **A ponte do FLIP, ao lado da vectorial** (censo `every_document_to_tree_bridge_is_in_the_net`,
        // 2026-09-08). Ela é o irmão exacto — bidireccional, mesma latência — e estava fora da rede:
        // apagar um objecto Flip pela Hierarquia deixava o documento com ele até ao quadro seguinte.
        ph2d_flip_entities::entities::sync(
            &mut gfx.sim,
            &mut gfx.flip,
            &mut self.flip_state.entities,
        );
        // ⭐⭐ **O ASSENTAMENTO DO PIVÔ é o TERCEIRO escritor derivado, e só o gate o disse.**
        // Uma entidade cunhada agora nasce com `Transform::default()`; quem lhe põe a origem no
        // centro da arte é este passe. Sem ele aqui, os dois controlos do gate do *duplicar*
        // passavam (a entidade existe, com `StableId`) e o **ponto fixo continuava vermelho**: o
        // quadro seguinte assentava o pivô sozinho, e o `Transform` mudava sem entrada nenhuma.
        //
        // ⚠️ **A lista dos que estão EM GESTO sai da MESMA porta do passe do desenho**
        // ([`ph2d_vec_entities::transform::gesture_paths`]) — um `&[]` aqui assentaria a forma que a mão
        // está a desenhar, e somar geometria + `Transform` desloca a arte de baixo do cursor.
        let drawing = ph2d_vec_entities::transform::gesture_paths(
            &self.vec.pen,
            &self.vec.shape,
            &self.vec.pencil,
        );
        ph2d_vec_entities::transform::settle_origins(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            &self.vec.entities,
            &drawing,
        );
        // ⭐⭐ **E o pivô dos objectos FLIP, pelo mesmo motivo e noutra mídia** (achado pelo censo
        // `the_net_knows_every_derived_writer`, 2026-09-08). O `ph2d_flip_entities::transform::settle_origins`
        // corria no passe do desenho e **não** estava aqui — o mesmo buraco que o gate do *duplicar*
        // tinha apanhado para o vector, por uma porta que nenhum gate olhava.
        //
        // ⚠️ **O objecto EM GESTO fica de fora, e a régua é a do passe do desenho**: com o desenho
        // ou a borracha activos a mão escreve MUNDO a cada quadro, e somar geometria + `Transform`
        // deslocaria a arte de baixo do cursor.
        let flip_gesturing = (self.flip_state.draw.is_active() || self.flip_state.erasing)
            .then_some(self.flip_state.target.object)
            .flatten();
        ph2d_flip_entities::transform::settle_origins(
            &mut gfx.sim,
            &mut gfx.flip,
            &self.flip_state.entities,
            flip_gesturing,
        );
        ph2d_ecs::assign_missing_root_order(gfx.sim.world_mut());
        ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
        ph2d_ecs::assign_missing_sibling_order(gfx.sim.world_mut());
        let Some(live) = gfx.hero_live.as_mut() else {
            return;
        };
        crate::build_hierarchy_snapshot(
            gfx.sim.world(),
            &mut live.z_walk_state,
            &mut live.z_walk_scratch,
            &mut live.z_snapshot,
        );
        let order = ph2d_vec_entities::entities::z_order(gfx.sim.world(), &live.z_snapshot);
        gfx.vec_scene.reorder_to(&order);
    }
}
