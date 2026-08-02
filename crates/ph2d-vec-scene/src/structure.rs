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
    /// As MOLDURAS que recortam neste frame (`ph2d_ecs::VecFrame`), já resolvidas para o
    /// intervalo que cada uma ocupa na pilha de z. Vazio = nenhuma moldura recorta, e o desenho é
    /// **byte-idêntico** ao mundo pré-moldura.
    ///
    /// ⚠️ **A ordem é de FORA para DENTRO.** Duas molduras aninhadas podem abrir no MESMO path (o
    /// descendente mais ao fundo é o mesmo para as duas), e a camada de clip é uma pilha: abrir a
    /// de dentro primeiro fecharia na ordem errada. Quem produz esta lista caminha a árvore de
    /// cima para baixo, e é isso que torna o emparelhamento LIFO correto por construção.
    pub clips: Vec<VecClipSpan>,
    /// **A tinta que os TOKENS produzem neste modo** (`ph2d_ecs::VecBindings` resolvido contra o
    /// tema vigente). Vazio = nada bindado, e o desenho é **byte-idêntico** ao mundo pré-token.
    ///
    /// Mora aqui, e não num 8º argumento do `dispatch`, porque é a MESMA categoria de fato que os
    /// vizinhos: algo que só a shell sabe, projetado do ECS uma vez por frame. O documento não
    /// conhece tema nenhum.
    pub bound: Vec<crate::BoundPaint>,
}

/// **Uma moldura que recorta**, dita em termos da pilha de z: *do `first` (inclusive) até o
/// `frame` (exclusive), o desenho é recortado à silhueta do `frame`*.
///
/// Duas coisas fazem este par de ids bastar, e as duas são fatos do repo e não escolhas:
///
/// 1. **A sub-árvore é CONTÍGUA em z.** A pilha é a projeção DFS da árvore
///    (`vec_zorder::z_order`), e o passe de recorte de SPRITE já se apoia nisso literalmente
///    (*"a clip group's runs are contiguous — subtree contiguity in z"*). Logo o intervalo entre
///    o descendente mais ao fundo e a moldura é exatamente a sub-árvore dela.
/// 2. **A moldura vem por ÚLTIMO na própria sub-árvore.** O DFS lista o pai ANTES dos filhos e a
///    pilha de z é o inverso disso ⇒ um pai desenha na FRENTE dos filhos. Para um grupo isso é
///    invisível (grupo não tem geometria); a moldura é o primeiro pai COM geometria, e é por isso
///    que ela precisa de tratamento: o preenchimento dela é o **fundo** do card, e tem de ser
///    desenhado ao ABRIR o intervalo, não ao chegar a vez dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VecClipSpan {
    /// A moldura: a silhueta que recorta, e o preenchimento que faz de fundo.
    pub frame: VecPathId,
    /// O descendente mais ao FUNDO — onde o intervalo abre.
    pub first: VecPathId,
    /// A moldura **RECORTA** o conteúdo, ou só lhe serve de fundo?
    ///
    /// ⚠️ Toda moldura com conteúdo tem intervalo, e é isso que faz o preenchimento dela ser
    /// antecipado para a abertura — *o fundo do card*. Antes, o intervalo só existia quando ela
    /// recortava, e uma moldura de LAYOUT com recorte desligado desenhava por cima do próprio
    /// conteúdo (Enio, 2026-08-02: *"os filhos estão ficando atrás do pai, logo não podem ser
    /// vistos a menos que reduza a opacidade"*).
    ///
    /// Recortar é a metade OPCIONAL; ser fundo não é.
    pub clip: bool,
}

impl VecViewState {
    /// A tinta resolvida desta forma, se algum token a dirige.
    ///
    /// ⚠️ **Perguntada UMA vez por forma-FONTE**, e a resposta serve a forma e a toda geometria
    /// derivada dela (offset, pattern, espelho): as derivadas são cópias com id próprio, então
    /// procurá-las na tabela não acharia nada e o token pararia na borda do primeiro efeito — a
    /// forma re-vestiria e as cópias dela ficariam com a cor velha.
    #[must_use]
    pub fn bound_paint(&self, id: VecPathId) -> Option<&crate::BoundPaint> {
        self.bound.iter().find(|b| b.path == id)
    }

    /// O path não desenha.
    #[must_use]
    pub fn is_hidden(&self, id: VecPathId) -> bool {
        self.hidden.contains(&id)
    }

    /// O path pode ser agarrado no canvas (visível E destravado).
    ///
    /// ⚠️ **O recorte não entra aqui, de propósito.** Um filho que a moldura esconde continua
    /// selecionável (pela Hierarquia e pelo canvas) — é o que Figma e Illustrator fazem, e o
    /// contrário tornaria impossível recuperar algo que se arrastou para fora por engano.
    #[must_use]
    pub fn is_pickable(&self, id: VecPathId) -> bool {
        !self.hidden.contains(&id) && !self.locked.contains(&id)
    }

    /// As molduras cujo intervalo ABRE neste path, na ordem de fora para dentro.
    pub fn clips_opening_at(&self, id: VecPathId) -> impl Iterator<Item = &VecClipSpan> {
        self.clips.iter().filter(move |c| c.first == id)
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
