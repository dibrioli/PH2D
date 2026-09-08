//! ⭐⭐⭐ **A RECONCILIAÇÃO ANTES DA CAPTURA** — a rede que apanha todo escritor TARDIO da árvore.
//!
//! # A lei, e porque ela precisou de uma segunda passagem
//!
//! A captura do undo tem de ser **PONTO FIXO dos sistemas**: fotografar, deixar o quadro seguinte
//! correr sem entrada nenhuma, e fotografar outra vez tem de dar a MESMA foto. Quando não é, o
//! diff por-quadro do [`crate::undo`] lê a convergência dos próprios sistemas como acção do
//! artista — nasce um passo **fantasma**, ele limpa a pilha de redo, e o `Ctrl+Z` seguinte gasta-se
//! a desfazer o lixo que o quadro acabou de criar. Para o dono: *«o undo só faz uma etapa»*.
//!
//! A F4.6c curou UM escritor — o arrasto da Hierarquia — **movendo-o** para antes da projecção de
//! z, e o handoff de 2026-09-07 deixou a pergunta escrita: *«os outros verbos tardios — apagar,
//! duplicar, Remove from Sheet — escrevem a árvore no MESMO sítio tardio e têm a mesma latência
//! estrutural. NÃO foi medido se produzem o fantasma.»*
//!
//! **Foi medido, e produzem.** Os dois gates de [`crate::vec_zorder_late_writers_tests`] nasceram
//! vermelhos, e o mecanismo é o do [`crate::vec_entities::sync`], que é bidireccional:
//!
//! | verbo tardio | o que ele escreve | o que o `sync` do quadro SEGUINTE faz sozinho |
//! |---|---|---|
//! | apagar | despawna a **entidade** | tira o caminho do **documento** ⇒ passo `["vec"]` |
//! | duplicar | põe o caminho no **documento** | cunha a **entidade** ⇒ passo `["world"]` |
//!
//! ⚠️ **As duas metades da mesma latência** — um gate só deixaria metade da classe por medir.
//!
//! # ⛔ Porque a cura NÃO foi mover estes, nem mover a leitura
//!
//! - **Mover os escritores** custou-nos uma linha no reparent porque o `drain_reparent` é
//!   auto-contido; apagar e duplicar vivem dentro do [`crate::render_loop::hierarchy::dispatch`],
//!   ~2 300 linhas de verbos com dependências no que o quadro computou entretanto. E **não fecha a
//!   classe**: o próximo verbo tardio que alguém escrever renasce com o defeito.
//! - **Mover a LEITURA para o fim** foi medido e recusado: entre a projecção (`reorder_to`) e o
//!   `dispatch` correm ~40 consumidores da cena — `envelope_live`, `skeleton_live`, `pattern_live`,
//!   `align_live`, o hit-test e o próprio desenho. A ordem das `paths` **é** a ordem de pintura;
//!   projectar só no fim daria a todos eles um quadro de atraso.
//!
//! ⇒ a leitura fica onde está, **para o desenho**, e esta porta corre outra vez **para a
//! fotografia**. São duas perguntas, não duas respostas à mesma: uma serve o que se VÊ neste
//! quadro, a outra o que se GUARDA dele.
//!
//! ⚠️ **É o sítio e a razão do [`crate::instance_sync::App::sync_instances`]**, que corre a dois
//! passos daqui com o mesmo doc: *depois do quadro, e antes da captura — senão a escrita do sync
//! vira um passo de undo que ninguém deu.* A lei já estava escrita ali, para outro sistema.

use crate::app_state::App;

/// Reconcilia a árvore com o documento e reprojecta a ordem de z — **a rede do fim do quadro**.
///
/// A sequência é a mesma do passe do desenho, e cada peça responde por um estado que um escritor
/// tardio pode ter deixado por fechar:
///
/// 1. [`crate::vec_entities::sync`] — entidade apagada leva o caminho, caminho novo ganha entidade;
/// 2. `assign_missing_root_order` / `_stable_ids` / `_sibling_order` — uma entidade cunhada agora
///    entraria no snapshot **sem identidade e sem ordem**, e o primeiro `Ctrl+Z` não teria o que
///    repor. ⚠️ As três são **idempotentes**: se reescrevessem por quadro, o diff veria o arquétipo
///    de toda entidade mudar e cada quadro com entrada viraria um passo espúrio;
/// 3. a projecção — `build_hierarchy_snapshot` → [`crate::vec_entities::z_order`] → `reorder_to`.
///
/// ⚠️ **Sem `hero_live` ela não faz nada, e isso é correcto**: o `HierarchyWalkState` e o
/// `HierarchySnapshot` vivem lá, e sem a tela da Hierarquia montada não há árvore lida neste
/// quadro — logo não há projecção para fechar.
impl App {
    pub(crate) fn settle_tree_before_capture(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
        // ⭐⭐ **O ASSENTAMENTO DO PIVÔ é o TERCEIRO escritor derivado, e só o gate o disse.**
        // Uma entidade cunhada agora nasce com `Transform::default()`; quem lhe põe a origem no
        // centro da arte é este passe. Sem ele aqui, os dois controlos do gate do *duplicar*
        // passavam (a entidade existe, com `StableId`) e o **ponto fixo continuava vermelho**: o
        // quadro seguinte assentava o pivô sozinho, e o `Transform` mudava sem entrada nenhuma.
        //
        // ⚠️ **A lista dos que estão EM GESTO sai da MESMA porta do passe do desenho**
        // ([`crate::vec_transform::gesture_paths`]) — um `&[]` aqui assentaria a forma que a mão
        // está a desenhar, e somar geometria + `Transform` desloca a arte de baixo do cursor.
        let drawing =
            crate::vec_transform::gesture_paths(&self.vec_pen, &self.vec_shape, &self.vec_pencil);
        crate::vec_transform::settle_origins(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            &self.vec_entities,
            &drawing,
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
        let order = crate::vec_entities::z_order(gfx.sim.world(), &live.z_snapshot);
        gfx.vec_scene.reorder_to(&order);
    }
}
