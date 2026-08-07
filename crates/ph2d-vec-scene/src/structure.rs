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

/// O que a árvore do editor diz sobre os paths neste parent: quais estão
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
    /// ⚠️ **A ordem é de FORA para DENTRO**, e a lista é **laminar** (bem-aninhada) por
    /// construção de quem a produz: a camada de clip é uma pilha, e dois intervalos que se
    /// cruzassem fariam um `pop_layer` fechar a camada de outra pessoa.
    pub clips: Vec<VecClipSpan>,
    /// **A tinta que os TOKENS produzem neste modo** (`ph2d_ecs::VecBindings` resolvido contra o
    /// tema vigente). Vazio = nada bindado, e o desenho é **byte-idêntico** ao mundo pré-token.
    ///
    /// Mora aqui, e não num 8º argumento do `dispatch`, porque é a MESMA categoria de fato que os
    /// vizinhos: algo que só a shell sabe, projetado do ECS uma vez por frame. O documento não
    /// conhece tema nenhum.
    pub bound: Vec<crate::BoundStyle>,
    /// **ONDE O AUTO LAYOUT PÔS ESTA FORMA** — o afim de MUNDO que a moldura lhe deu neste frame
    /// (ADR-0153). Vazio = ninguém flui, e tudo se lê exactamente como antes.
    ///
    /// # Por que uma POSE, e não só a geometria assada
    ///
    /// O passe do layout já assa o resultado dentro da `LiveGeometry`, e é por isso que a forma
    /// **aparece** no lugar certo. Mas quem não desenha a geometria — as ÂNCORAS do modo Node, a
    /// caixa do gizmo, o hit-test — lê a pose AUTORADA, e ela não se mexeu: as âncoras ficavam no
    /// lugar de origem e o clique procurava a forma onde ela já não está (Enio, 2026-08-02:
    /// *"os Path das formas aparecem no lugar de origem e talvez por isso não consigo
    /// selecioná-las"*).
    ///
    /// ⚠️ **É uma pose porque o layout é uma pose** — `translate ∘ scale` sobre a geometria de
    /// mundo, sem reshape nenhum. Um Offset ou um Pattern, que MUDAM a curva, não entram aqui: as
    /// âncoras deles ficam mesmo na fonte, que é o que o modo Node edita (a convenção do
    /// `inkscape:original-d`).
    ///
    /// ⚠️ **Composição:** o mundo desta forma é `pose ∘ xform_of(id)` — a pose vem DEPOIS, porque
    /// ela age sobre a geometria já posta no mundo.
    pub poses: Vec<(VecPathId, crate::Xform)>,
}

/// **Uma moldura que recorta**, dita em termos da pilha de z: *do `frame` (exclusive) até o `last`
/// (inclusive), o desenho é recortado à silhueta do `frame`*.
///
/// ⚠️ **A moldura é o PRIMEIRO membro da própria sub-árvore**, desde que a pilha de z passou a ser
/// o DFS **na ordem** (a lei de Godot — o filho desenha sobre o pai, `vec_zorder::z_order`). O
/// preenchimento dela é o fundo do card porque ela desenha primeiro, e não porque alguém antecipa
/// o desenho dela: a versão anterior deste tipo carregava um `clip: bool` justamente porque TODO
/// pai precisava de um intervalo só para ser antecipado. Hoje o intervalo é sobre **recortar**, e
/// mais nada — quem não recorta não tem intervalo.
///
/// ⚠️ **O `last` sai da pilha FINAL, não do DFS.** O Z é global e sobrepõe a árvore, então um
/// descendente com Z alto pode sair de dentro do intervalo — e sai *mesmo*: ele deixa de ser
/// recortado. É o significado literal de *"o Z sobrepõe a ordem na hierarquia"*, e resolver o
/// intervalo contra o DFS o descreveria onde a forma já não está.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VecClipSpan {
    /// A moldura: a silhueta que recorta, e o preenchimento que faz de fundo.
    pub frame: VecPathId,
    /// O último path que o recorte alcança — onde o intervalo FECHA.
    pub last: VecPathId,
}

impl VecViewState {
    /// A pose que o AUTO LAYOUT deu a `id` neste frame — identidade quando ele não a colocou.
    ///
    /// ⚠️ **Porta única.** Quem precisa de saber *onde esta forma está* — as âncoras, a caixa do
    /// gizmo, o hit-test — pergunta aqui; duas leituras diferentes da mesma tabela é como o
    /// clique volta a procurar a forma onde ela não está.
    #[must_use]
    pub fn layout_pose(&self, id: VecPathId) -> crate::Xform {
        self.poses
            .iter()
            .find(|(p, _)| *p == id)
            .map_or(crate::Xform::IDENTITY, |(_, x)| *x)
    }

    /// A tinta resolvida desta forma, se algum token a dirige.
    ///
    /// ⚠️ **Perguntada UMA vez por forma-FONTE**, e a resposta serve a forma e a toda geometria
    /// derivada dela (offset, pattern, espelho): as derivadas são cópias com id próprio, então
    /// procurá-las na tabela não acharia nada e o token pararia na borda do primeiro efeito — a
    /// forma re-vestiria e as cópias dela ficariam com a cor velha.
    #[must_use]
    pub fn bound_style(&self, id: VecPathId) -> Option<&crate::BoundStyle> {
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

    /// As molduras cujo intervalo ABRE neste path — ou seja, as que **são** este path.
    pub fn clips_opening_at(&self, id: VecPathId) -> impl Iterator<Item = &VecClipSpan> {
        self.clips.iter().filter(move |c| c.frame == id)
    }

    /// As molduras cujo intervalo FECHA neste path, **de dentro para fora**.
    ///
    /// ⚠️ A inversão é o que emparelha o LIFO: a lista vem de fora para dentro, e a camada mais
    /// interna tem de fechar primeiro.
    pub fn clips_closing_at(&self, id: VecPathId) -> impl Iterator<Item = &VecClipSpan> {
        self.clips.iter().rev().filter(move |c| c.last == id)
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
