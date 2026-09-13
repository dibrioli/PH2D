//! **Fase do quadro: O ASSENTAMENTO DAS ORIGENS E DA ÁRVORE** — as origens vectoriais e do Flip, a ordem de raiz,
//! os `StableId` e a ordem de irmãos em falta, a reordenação do Blend e o reparent da Hierarquia (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Todo escritor da árvore corre ANTES de ela ser lida, e a leitura antes da captura do undo.

use super::*;

/// O pedido de reparent da Hierarquia que o dreno do barramento recolheu neste quadro.
pub(super) struct TreeSettleIntents {
    pub(super) reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_tree_settle(
        &mut self,
        intents: TreeSettleIntents,
    ) -> Option<(
        Vec<u64>,
        Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            toasts,
            vec_scene,
            flip,
            hero_live,
            ..
        } = FrameGfx::of(gfx);
        let TreeSettleIntents {
            mut reparent_intent,
        } = intents;
        // **Envelope Objects (ADR-0129 Fatia 3):** SEM `upkeep` — o envelope não cria path
        // nenhum (o container não tem path), então não há entidade nova esperando o `sync`. Tudo
        // (assar + reparentar + pendurar) já aconteceu síncrono no `create`. O `settle` pula os
        // filhos (têm `ChildOf`) e o container (sem path, fora do mapa).
        // ADR-0112: a origem (o pivô) de um path nasce no centro do MUNDO. Assim
        // que a forma pára de crescer, ela vai para o centro dela. Quem está EM GESTO é
        // pulado, e a lista sai de UMA porta (`vec_gesture_paths`) — o porquê está lá.
        let drawing = ph2d_vec_entities::transform::gesture_paths(
            &self.vec.pen,
            &self.vec.shape,
            &self.vec.pencil,
        );
        ph2d_vec_entities::transform::settle_origins(sim, vec_scene, &self.vec.entities, &drawing);
        // ADR-0114/ADR-0111: idem para os objetos Flip — o pivô nasce no centro do
        // MUNDO; assim que a arte pára de crescer, ele vai para o centro dela (e a
        // geometria vira LOCAL). O objeto EM GESTO (desenho/borracha ativos) NÃO é
        // assentado — a mão escreve MUNDO a cada frame e somar geometria+Transform
        // deslocaria a arte do cursor.
        let flip_gesturing = (self.flip_state.draw.is_active() || self.flip_state.erasing)
            .then(|| flip.objects().first().map(|o| o.id))
            .flatten();
        ph2d_flip_entities::transform::settle_origins(
            sim,
            flip,
            &self.flip_state.entities,
            flip_gesturing,
        );
        // **A ordem de z é a projeção da árvore — e a árvore é lida AQUI, depois do
        // `sync`.** Não é arrumação (BUGS #15): a lista do painel foi publicada no
        // prólogo do frame, quando a forma recém-criada ainda não tinha entidade.
        // Projetar por ela punha a forma nova no FUNDO por um frame, e a captura do
        // undo — tirada no fim deste frame — deixava de ser ponto fixo dos sistemas.
        // Toda raiz ganha um `RootOrder` explícito ANTES de a árvore ser lida — e antes
        // da captura do fim do frame. Sem isto, a raiz sem ordem colate em `u32::MAX` e
        // a árvore a desempata por `Entity::to_bits()` (id de ALOCAÇÃO): o respawn do
        // undo troca os bits, a pilha de z se reordena sozinha a cada Ctrl+Z, e o passo
        // espúrio volta vestido de outra coisa. Não ter empate > escolher desempate.
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
        // **As duas gémeas do `RootOrder`** (ADR-0164 F1), aqui pela MESMA razão e no
        // MESMO sítio: elas têm de correr depois do `sync` (as entidades novas do quadro
        // já existem) e **antes da captura do fim do quadro**, senão o objeto criado neste
        // quadro entra no snapshot sem identidade e sem ordem — e o primeiro Ctrl+Z não
        // teria o que repor.
        //
        // ⚠️ `StableId` é a identidade DURÁVEL: ela sobrevive ao respawn do undo, que é a
        // propriedade inteira pela qual a wave existe. `SiblingOrder` faz a ordem entre
        // irmãos ser DADO — antes dela, reordenar não era desfazível nem sobrevivia a um
        // restore (classe BUGS #15), porque a ordem vivia na lista `Children` do bevy, que
        // é memória de runtime.
        //
        // ⚠️ **As duas são idempotentes**, e isso não é higiene: se reescrevessem por
        // quadro, o diff do undo veria o arquétipo de toda entidade mudar e **cada quadro
        // com input viraria um passo espúrio** — a doença que o `RootOrder` curou.
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
        // O Blend pediu uma sequência de z; agora as entidades existem (o `sync` rodou) e ela
        // pode ser escrita na ÁRVORE — que é quem manda no z (ADR-0110). Escrever na ordem do
        // vetor da cena seria a porta errada: a projeção abaixo a reescreve todo frame.
        for order in std::mem::take(&mut self.vec.restack) {
            ph2d_vec_entities::entities::restack(sim, &self.vec.entities, &order);
        }
        // ⭐⭐⭐ **O ARRASTO DA HIERARQUIA ESCREVE A ÁRVORE, LOGO ELE MORA AQUI** — ao lado do
        // `restack` e dos três `assign_missing_*`, e **antes** de a árvore ser lida.
        //
        // ⛔⛔ **Report do Enio, 2026-09-07: *«reordenei objectos na hierarquia e não funcionou
        // o undo»*.** Ele estava certo, e o defeito NÃO era o undo: era este dreno correr
        // ~2 340 linhas **depois** da projecção, dentro do `hierarchy::dispatch`. A sequência
        // medida com `PH2D_UNDO_LOG=1`:
        //
        //   * quadro N — a árvore muda (`RootOrder` de `[(1,0),(2,1),(3,2)]` para
        //     `[(1,0),(2,2),(3,1)]`), mas a projecção já correu sobre a árvore VELHA ⇒ a
        //     captura do fim do quadro guarda `world` novo com `vec` **velho**;
        //   * quadro N+1 — a projecção lê a árvore nova e reescreve a pilha; sem entrada, o
        //     passo é SUPRIMIDO e funde-se no seguinte;
        //   * mais tarde nasce um passo cujo conteúdo inteiro é `partes: ["vec"]`
        //     (`base=[0,1,2] atual=[0,2,1] · só a ORDEM=true`) — um **fantasma**;
        //   * `Ctrl+Z` repõe a pilha e **não** a árvore, a projecção do quadro seguinte
        //     re-deriva a pilha da árvore que ninguém desfez, e o fantasma **renasce**. O log
        //     do dono mostra o ciclo a repetir-se com a fila parada em `5`: cada `Ctrl+Z` gasta
        //     um passo que o próprio quadro volta a criar, e o passo REAL (o `["world"]`)
        //     nunca chega a ser alcançado.
        //
        // ⚠️ **É a doença que o [`ph2d_vec_entities::entities::z_order`] já documenta** — *«a captura
        // deixava de ser ponto fixo dos sistemas»* — a voltar por outra porta: ali era a forma
        // recém-nascida contra a lista do painel, aqui é a árvore reordenada contra a projecção
        // do mesmo quadro. ⇒ a lei não é sobre QUEM escreve, é sobre QUANDO: **todo escritor da
        // árvore corre antes de ela ser lida, e a leitura antes da captura.**
        //
        // ⚠️ **O `take` é load-bearing:** o `hierarchy::dispatch` lá em baixo continua a receber
        // o parâmetro (a assinatura é dele), e vê `None` — aplicar duas vezes reordenaria duas.
        if let Some(intent) = reparent_intent.take()
            && let Some(live) = hero_live.as_ref()
        {
            hero_intents::drain_reparent(intent, live, sim, toasts);
            self.title_dirty = true;
        }
        if let Some(live) = hero_live.as_mut() {
            crate::build_hierarchy_snapshot(
                sim.world(),
                &mut live.z_walk_state,
                &mut live.z_walk_scratch,
                &mut live.z_snapshot,
            );
            let order = ph2d_vec_entities::entities::z_order(sim.world(), &live.z_snapshot);
            vec_scene.reorder_to(&order);
        }
        Some((drawing, reparent_intent))
    }
}
