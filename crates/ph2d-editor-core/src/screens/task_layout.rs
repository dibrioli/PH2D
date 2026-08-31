//! ⭐⭐⭐ **OS LAYOUTS POR TAREFA** — a decisão **D7**, e o eixo que a **D3** separa dos outros dois.
//!
//! > | eixo | quem decide | onde vive | o que muda |
//! > |---|---|---|---|
//! > | **Layout** | o **utilizador** | barra de cima (abas) | que **áreas** existem e que editor está em cada |
//! > | **Modo** | o **tipo do objecto** | cabeçalho da área | ferramentas, atalhos, aspecto da vista |
//! > | **Ferramenta** | o utilizador, dentro do modo | toolbar da área | o **gesto** do ponteiro |
//!
//! ⇒ um layout **arruma painéis**. Ele não é um modo e não é uma ferramenta.
//!
//! # ⭐ A costura com o Modo é UM CAMPO OPCIONAL, e é a do Blender
//!
//! O Workspace do Blender tem um `Mode:` — *«switch to this Mode when activating the workspace»*.
//! Ortogonais, **com um atalho declarado**; ⛔ não acoplados. Aqui é o [`TaskLayout::tool`]: escolher
//! *Vetor* arruma os painéis **e** pega na ferramenta de vetor, porque um layout de vetor com o
//! canvas noutro modo é uma arrumação que não serve para nada. Um layout sem ferramenta declarada
//! **não mexe** na que está em mãos.
//!
//! # ⛔ DOIS dos oito não existem, e o bloqueador é de outra pessoa
//!
//! A D7 lista oito. **Código** precisa de um editor de texto, que este app não tem; **Runtime**
//! precisa do `shells/game`/R1, **adiado pelo Enio**. ⇒ eles não são abas mudas: *uma aba que não
//! faz nada é o controlo morto que este repo mais paga.* Ficam nomeados no handoff, com o
//! bloqueador, e entram no dia em que o bloqueador cair.

use crate::screens::slot::Slot;

/// Um layout por tarefa (**D7**).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskLayout {
    /// Pintar — canvas grande, camadas.
    Drawing2d,
    /// ⭐ Desenho vetorial, **separado do raster por decisão do Enio**.
    Vector,
    /// ⭐ Animação quadro-a-quadro, **layout próprio** além do modo `Draw`.
    Flip,
    /// Modelar e esculpir.
    Modeling3d,
    /// ⚠️ **Não é o layout onde se pode animar** (a D8 diz que as timelines funcionam em todos) —
    /// é aquele onde a **ênfase** é o tempo: linha do tempo grande, canvas pequeno. *Distinção de
    /// proporção, não de capacidade.*
    Animation,
    /// O grafo no centro, com pré-visualização.
    Nodes,
}

/// O que um layout arruma.
pub struct LayoutSpec {
    /// O nome na aba.
    pub title: &'static str,
    /// O que a chave do ficheiro guarda. ⚠️ `snake_case` e não o `Debug`, pela razão do
    /// [`Slot::wire`]: renomear a variante não pode apagar a arrumação gravada de ninguém.
    pub wire: &'static str,
    /// Os painéis que ele abre — **a lista completa**, e tudo o que não está aqui fecha.
    pub open: &'static [&'static str],
    /// Painéis que ele põe num encaixe diferente do que eles declaram. Vazio na maioria.
    pub slots: &'static [(&'static str, Slot)],
    /// ⭐ **A ferramenta que ele pega**, ou `None` — a costura opcional com o Modo (ver o
    /// cabeçalho). O id é o do registry de ferramentas.
    pub tool: Option<&'static str>,
}

impl TaskLayout {
    /// Os seis, na ordem em que aparecem na barra — a fonte de toda varredura.
    ///
    /// ⚠️ **A ordem é a da D7** (menos os dois bloqueados), e não a alfabética: ela é lida da
    /// esquerda para a direita por quem escolhe, e a tabela dele começa no desenho.
    pub const ALL: [Self; 6] = [
        Self::Drawing2d,
        Self::Vector,
        Self::Flip,
        Self::Modeling3d,
        Self::Animation,
        Self::Nodes,
    ];

    /// ⭐ **A TABELA** — o que cada layout arruma. A fonte única.
    #[must_use]
    pub const fn spec(self) -> LayoutSpec {
        match self {
            // ⚠️ A hierarquia e o inspector estão em **todos**: eles são o *que existe* e o *o que
            // isto é*, e nenhuma tarefa deste app se faz sem os dois. Uma tarefa que os dispensasse
            // seria o Runtime, que é um dos dois bloqueados.
            Self::Drawing2d => LayoutSpec {
                title: "Draw",
                wire: "drawing_2d",
                open: &["hierarchy", "inspector", "painter_layers"],
                slots: &[],
                tool: Some("painter"),
            },
            Self::Vector => LayoutSpec {
                title: "Vector",
                wire: "vector",
                open: &["hierarchy", "inspector", "vector"],
                slots: &[],
                tool: Some("vector"),
            },
            Self::Flip => LayoutSpec {
                title: "Flip",
                wire: "flip",
                open: &["hierarchy", "inspector", "flip", "flip_frames"],
                slots: &[],
                tool: Some("flip"),
            },
            Self::Modeling3d => LayoutSpec {
                title: "Model",
                wire: "modeling_3d",
                open: &["hierarchy", "inspector", "model3d"],
                slots: &[],
                // ⛔ Sem ferramenta: o módulo 3D **toma o canvas** por outro caminho (o pill MODEL,
                // hoje o menu *Window*), e activá-lo daqui seria a segunda porta para o mesmo facto.
                tool: None,
            },
            Self::Animation => LayoutSpec {
                title: "Animate",
                wire: "animation",
                open: &["hierarchy", "inspector", "timeline"],
                slots: &[],
                tool: None,
            },
            Self::Nodes => LayoutSpec {
                title: "Nodes",
                wire: "nodes",
                // ⚠️ O `motion_graph` é o **centro** (ele parte a área de desenho), e por isso não
                // disputa coluna com ninguém — ver `slot_tabs` e a decisão D5.
                open: &["hierarchy", "motion_graph", "motion_params"],
                slots: &[],
                tool: Some("motion"),
            },
        }
    }

    /// A volta do [`LayoutSpec::wire`]. `None` para o que não se reconhece — *um layout de um build
    /// mais novo cai no de omissão*, que é o comportamento certo para uma preferência.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.spec().wire == s)
    }
}

impl Default for TaskLayout {
    /// ⚠️ **O de omissão é o DESENHO**, e não é uma escolha arbitrária: é o primeiro da tabela da
    /// D7, e é a tarefa que o app abre a fazer desde que existe.
    fn default() -> Self {
        Self::Drawing2d
    }
}

#[cfg(test)]
#[path = "task_layout_tests.rs"]
mod tests;
