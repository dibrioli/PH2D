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
//! # ⭐⭐ O CANVAS TEM UM DONO, e todo layout o NOMEIA
//!
//! O Workspace do Blender tem um `Mode:` — *«switch to this Mode when activating the workspace»*.
//! Ortogonais, **com um atalho declarado**; ⛔ não acoplados. Aqui é o [`LayoutSpec::canvas`].
//!
//! ⛔⛔ **Ele era `Option<&str>`, e o `None` significava *«não mexe na ferramenta em mãos»* — o
//! defeito que o Enio reportou em 2026-08-31:** *«se abro Nodes e depois Model, o grafo de Nodes
//! persiste»*. A causa não é o grafo: é que **quase todo painel deste app pertence à FERRAMENTA,
//! não ao layout** (a `motion_bridge` escreve `motion_graph`/`motion_params` a cada quadro a
//! partir de `tools.active() == motion`), então um layout que não larga a ferramenta traz os
//! painéis dela atrás — e a lista de abertos, que se diz **absoluta**, é reescrita pela ponte no
//! quadro seguinte. ⚠️ Valia para **dois** dos seis (*Model* e *Animate*), e a segunda mordida
//! via-se na foto dele: as abas *Inspector | Vector* no dock direito do *Animate*.
//!
//! ⇒ **não há herança.** O *Model* entrega o canvas ao modelador; o *Animate* entrega-o à
//! ferramenta neutra (o `move`, que é o que este app tem em vez de *«nenhuma»* — ver
//! [`CanvasOwner`]).
//!
//! # ⛔ E por isso a lista de abertos ENCOLHEU
//!
//! Um layout só pode comandar o que mais ninguém escreve. O `motion_graph`, o `vector`, o `flip`,
//! o `painter_layers` e companhia vêm **com a ferramenta**; nomeá-los aqui era decoração que o
//! quadro seguinte reescrevia. O que sobra — a hierarquia, a linha do tempo, o painel do
//! modelador — é o que o layout de facto arruma. O gate que defende a fronteira vive em
//! `shells/desktop/tests/a_layout_never_commands_a_panel_a_bridge_owns.rs`, porque o censo de quem
//! é da ferramenta só existe nas pontes.
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

/// ⭐⭐ **Quem fica com o CANVAS quando este layout abre.** Sem `None`: ver o cabeçalho.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CanvasOwner {
    /// A ferramenta com este id (o do registry de ferramentas).
    ///
    /// ⚠️ **`Tool("move")` é o que este app tem em vez de «nenhuma»:** o registry nunca fica sem
    /// ferramenta activa (`activate_default` no arranque, e o `set_active` exige um id), e todo
    /// gesto de *largar* — o `CancelActiveTool`, o pill do vetor, o do motion — é escrito como
    /// *«volta à de omissão»*. ⇒ um layout que só quer a cena pede o `move`, e não fica calado.
    Tool(&'static str),
    /// ⭐ O **modelador implícito** (ADR-0161). Ele não é uma `Tool` — o traçado e a navegação
    /// moram no shell de propósito, e é isso que mantém a superfície congelada `Tool=12` fora do
    /// caminho —, então não há `tool_id` que o exprima: quem o arma é a **visibilidade do painel**
    /// (`field3d_smoke::set_armed_by_panel`).
    ///
    /// ⚠️ **E é por isso que ele não pede ferramenta nenhuma.** Quem larga a que estava em mãos é
    /// a metade simétrica da lei do `field3d_mode` — *tomar o canvas liberta quem o tinha* —, que
    /// corre no shell quando este painel abre, por qualquer porta (esta aba **ou** o menu
    /// *Window*). Pedir o `move` aqui faria a ponte ler a nossa própria mão como *«outro tomou o
    /// canvas»* e fechar o painel que a abriu.
    Model3d,
}

/// O que um layout arruma.
pub struct LayoutSpec {
    /// O nome na aba.
    pub title: &'static str,
    /// O que a chave do ficheiro guarda. ⚠️ `snake_case` e não o `Debug`, pela razão do
    /// [`Slot::wire`]: renomear a variante não pode apagar a arrumação gravada de ninguém.
    pub wire: &'static str,
    /// Os painéis que ele abre — **a lista completa**, e tudo o que não está aqui fecha.
    ///
    /// ⛔ **Só painéis que o layout POSSUI.** Um painel cuja visibilidade uma ponte escreve a cada
    /// quadro a partir da ferramenta activa não é comandável daqui: nomeá-lo é uma declaração que
    /// o quadro seguinte reescreve. Ver o cabeçalho do módulo.
    pub open: &'static [&'static str],
    /// Painéis que ele põe num encaixe diferente do que eles declaram. Vazio na maioria.
    pub slots: &'static [(&'static str, Slot)],
    /// ⭐⭐ **A quem ele entrega o canvas** — sem `None`, e é isso que impede uma tarefa de herdar
    /// o modo da anterior. Ver [`CanvasOwner`] e o cabeçalho.
    pub canvas: CanvasOwner,
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
            // ⚠️ A hierarquia está em **todos**: ela é o *que existe*, e nenhuma tarefa deste app
            // se faz sem ela. Uma tarefa que a dispensasse seria o Runtime, que é um dos dois
            // bloqueados.
            //
            // ⚠️⚠️ **O `inspector` NÃO está em nenhum, e a ausência é a decisão.** Ele tem dois
            // donos: seis pontes escrevem-no na BORDA de uma tomada (`insert("inspector",
            // !active)` — o painel da ferramenta substitui-o visualmente), e o layout escreve-o na
            // pintura. Como as pontes correm depois, o que o layout dissesse era desmentido no
            // quadro seguinte em exactamente as transições que interessam. *Um campo com dois
            // escritores e uma ordem fixa tem um dono só — e não é quem escreve primeiro.*
            Self::Drawing2d => LayoutSpec {
                title: "Draw",
                wire: "drawing_2d",
                // ⚠️ O `painter_layers` vem COM a ferramenta (`painter_bridge`), e por isso não se
                // nomeia aqui — ver o cabeçalho.
                open: &["hierarchy"],
                slots: &[],
                canvas: CanvasOwner::Tool("painter"),
            },
            Self::Vector => LayoutSpec {
                title: "Vector",
                wire: "vector",
                open: &["hierarchy"],
                slots: &[],
                canvas: CanvasOwner::Tool("vector"),
            },
            Self::Flip => LayoutSpec {
                title: "Flip",
                wire: "flip",
                open: &["hierarchy"],
                slots: &[],
                canvas: CanvasOwner::Tool("flip"),
            },
            Self::Modeling3d => LayoutSpec {
                title: "Model",
                wire: "modeling_3d",
                // ⚠️ Abrir o painel **é** entrar no modo (`set_armed_by_panel`) — por isso ele é do
                // layout, e a ferramenta em mãos é largada pela lei do `field3d_mode`, no shell.
                open: &["hierarchy", "model3d"],
                slots: &[],
                canvas: CanvasOwner::Model3d,
            },
            Self::Animation => LayoutSpec {
                title: "Animate",
                wire: "animation",
                open: &["hierarchy", "timeline"],
                slots: &[],
                canvas: CanvasOwner::Tool("move"),
            },
            Self::Nodes => LayoutSpec {
                title: "Nodes",
                wire: "nodes",
                // ⚠️ O `motion_graph` é o **centro** (ele parte a área de desenho) e o
                // `motion_params` é da ponte — os dois vêm com a ferramenta. A linha do tempo é
                // nomeada porque é do layout, e a ponte do motion só a **abre** por cortesia
                // (nunca a fecha), o que é a mesma resposta por dois caminhos.
                open: &["hierarchy", "timeline"],
                slots: &[],
                canvas: CanvasOwner::Tool("motion"),
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
