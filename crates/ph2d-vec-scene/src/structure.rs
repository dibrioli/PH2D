//! Estrutura do documento (ADR-0110): a **pilha de z** e o **recorte** de
//! copy/paste. Nada mais.
//!
//! A árvore de objetos NÃO mora aqui. Nome, visibilidade, trava, parentesco e
//! grupo são da entidade ECS que representa o path (`ph2d_ecs::VecPathRef`), e
//! quem os desenha é o painel Hierarchy do editor — o mesmo que desenha sprites, e
//! por isso um path pode ser filho de um sprite e um grupo pode misturar tipos.
//!
//! O que sobra para o documento é o que é geometria: `paths` continua sendo a
//! pilha de z (fundo → topo) que o render e a booleana leem. Só que agora ela é
//! uma **projeção da árvore** — a shell a re-sincroniza a cada frame via
//! [`VecScene::reorder_to`], depois que a Hierarquia (a fonte da verdade) mudou.

use crate::{VecPath, VecPathId, VecScene};

/// O que a árvore do editor diz sobre os paths neste frame: quais estão
/// escondidos e quais estão travados (já com a herança dos ancestrais resolvida).
///
/// O documento não sabe disso — é a shell que projeta o ECS aqui, uma vez por
/// frame. Vazio = tudo visível e agarrável, que é o caso comum e o de todos os
/// testes puros.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VecViewState {
    pub hidden: Vec<VecPathId>,
    pub locked: Vec<VecPathId>,
}

impl VecViewState {
    /// O path não desenha.
    #[must_use]
    pub fn is_hidden(&self, id: VecPathId) -> bool {
        self.hidden.contains(&id)
    }

    /// O path pode ser agarrado no canvas (visível E destravado).
    #[must_use]
    pub fn is_pickable(&self, id: VecPathId) -> bool {
        !self.hidden.contains(&id) && !self.locked.contains(&id)
    }
}

/// Um recorte do documento: paths na ordem de z. É o que o clipboard guarda e o
/// que [`VecScene::paste_clip`] re-instancia com ids novos — o mesmo recorte pode
/// ser colado quantas vezes quiser.
///
/// O recorte carrega geometria e estilo, **não** estrutura: quem recria nome e
/// parentesco das cópias é a shell, ao spawnar as entidades delas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VecClip {
    pub paths: Vec<VecPath>,
}

impl VecClip {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

impl VecScene {
    /// Recorta `ids` do documento (sem removê-los), na ordem de z.
    #[must_use]
    pub fn copy_paths(&self, ids: &[VecPathId]) -> VecClip {
        VecClip {
            paths: self
                .paths
                .iter()
                .filter(|p| ids.contains(&p.id))
                .cloned()
                .collect(),
        }
    }

    /// Cola um recorte no topo da pilha, deslocado por `(dx, dy)`. Ids **reemitidos**
    /// (o recorte é reutilizável), e o deslocamento usa `translate_path` — que move
    /// os subpaths e a geometria do gradiente junto, ao contrário de mexer nos
    /// vértices à mão. Devolve os ids novos, na ordem de z.
    pub fn paste_clip(&mut self, clip: &VecClip, dx: f64, dy: f64) -> Vec<VecPathId> {
        let mut new_ids = Vec::with_capacity(clip.paths.len());
        for p in &clip.paths {
            let new_id = self.push_path(p.clone());
            if dx != 0.0 || dy != 0.0 {
                self.translate_path(new_id, dx, dy);
            }
            new_ids.push(new_id);
        }
        new_ids
    }

    /// Re-ordena a pilha de z para casar com `order` (**fundo → topo**).
    ///
    /// Um id ausente de `order` vai para o FUNDO, preservando a ordem relativa —
    /// é o path recém-criado, cuja entidade a árvore ainda não conhece neste frame.
    /// A projeção nunca perde um path. Devolve `true` se a ordem mudou.
    pub fn reorder_to(&mut self, order: &[VecPathId]) -> bool {
        let before: Vec<VecPathId> = self.paths.iter().map(|p| p.id).collect();
        // `sort_by_key` é estável → os ausentes (chave 0) mantêm a ordem entre si.
        self.paths
            .sort_by_key(|p| order.iter().position(|&o| o == p.id).map_or(0, |r| r + 1));
        before != self.paths.iter().map(|p| p.id).collect::<Vec<_>>()
    }
}
