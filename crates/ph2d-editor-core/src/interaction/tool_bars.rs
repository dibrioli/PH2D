//! ⭐ **As BARRAS de um módulo** — o que um módulo com ferramentas próprias (hoje o Vetor) publica em
//! todo quadro para o chrome pintar à volta da área: ① a coluna de ferramentas e o flyout de um
//! botão dela, ② o que entra na fila de cima, ④ as ações junto da seleção e ⑤ a dica do rodapé.
//!
//! ⚠️ **O módulo escreve, o chrome lê** (ADR-0075). O chrome não sabe o que é um nó; o módulo não sabe
//! onde a coluna cai. Os ids são os do PAINEL dono, e o clique segue pela porta que ele já tem — o
//! `HeroScreen::apply_event` entrega todo evento a todo painel, por id. *Um comando com dois ids tem
//! dois sítios a apodrecer.* Desenho: `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md` §3.

use crate::icons::IconId;
use crate::widget::ToolRailEntry;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorToken;

/// A cara de um botão flutuante.
#[derive(Clone, Debug, PartialEq)]
pub enum BarFace {
    Icon(IconId),
    /// Uma amostra de cor; `None` = sem tinta. `ring` = lê-se como CONTORNO (o miolo é o painel).
    Swatch {
        rgba: Option<[u8; 4]>,
        ring: bool,
    },
    /// O CONTORNO de uma forma, num quadrado unitário (`0..1`, y para baixo) — a mesma geometria
    /// que o módulo desenha, nunca um segundo desenho (a lei da barra do Quadro).
    Outline(ph2d_vector::BezPath),
}

/// A família de um botão — a COR do ícone. Forma distinta por ferramenta e cor por família: a
/// armadilha do Affinity V3 (ícones iguais entre si, estudo §2) não se repete.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarFamily {
    /// Seleccionar e verbos sem família — a tinta neutra.
    #[default]
    Neutral,
    /// Desenhar (caneta, lápis, forma, texto).
    Draw,
    /// Construir (construtor, conector, balde, moldura).
    Build,
    /// Cortar e esculpir cantos.
    Carve,
    /// O traço vivo (largura, mistura).
    Stroke,
}

impl BarFamily {
    /// O matiz da família. ⚠️ São as cores CATEGÓRICAS da casa (as das categorias de nós) — a paleta
    /// que já foi calibrada para se distinguir em todo tema; uma quinta paleta seria um token novo
    /// sem calibração.
    #[must_use]
    pub const fn tint(self) -> ColorToken {
        match self {
            Self::Neutral => ColorToken::Text2,
            Self::Draw => ColorToken::NodeCatSource,
            Self::Build => ColorToken::NodeCatTransform,
            Self::Carve => ColorToken::NodeCatFx,
            Self::Stroke => ColorToken::NodeCatOutput,
        }
    }
}

/// Um botão de uma barra flutuante.
#[derive(Clone, Debug, PartialEq)]
pub struct BarButton {
    pub id: NodeId,
    pub face: BarFace,
    /// O nome (já traduzido): a dica e o a11y.
    pub label: String,
    pub on: bool,
    pub family: BarFamily,
}

/// ⭐ **O que o módulo publica num quadro.**
#[derive(Clone, Debug, Default)]
pub struct ToolBars {
    /// ① a coluna, por grupos (um separador entre grupos).
    pub column: Vec<Vec<BarButton>>,
    /// ① o flyout aberto: o id do botão da coluna que o abre, e a grelha dele.
    pub flyout: Option<(NodeId, Vec<BarButton>)>,
    /// ② o que entra na fila ANTES dos chips do gizmo — a prioridade mais alta (sai por último).
    pub row_lead: Vec<ToolRailEntry>,
    /// ② o que entra DEPOIS dos chips do gizmo e antes dos pulldowns da área.
    pub row_tail: Vec<ToolRailEntry>,
    /// ④ as ações da seleção, por grupos. Vazio ⇒ sem barra.
    pub actions: Vec<Vec<BarButton>>,
    /// ④ o rectângulo da seleção no ECRÃ — a âncora da barra quando não está fixada.
    pub selection: Option<Rect>,
    /// ⑤ a dica da ferramenta na mão (o que o clique, o arrasto e os modificadores fazem).
    pub hint: String,
    /// ⑤ as leituras do rodapé (coordenadas, encaixe), já formatadas.
    pub readout: Vec<String>,
}

impl ToolBars {
    /// Todo id que estas barras pintam neste quadro — o gate *uma porta* conta-os.
    #[must_use]
    pub fn painted_ids(&self) -> Vec<NodeId> {
        let buttons = self
            .column
            .iter()
            .flatten()
            .chain(self.flyout.iter().flat_map(|(_, f)| f.iter()))
            .chain(self.actions.iter().flatten())
            .map(|b| b.id);
        let row = self
            .row_lead
            .iter()
            .chain(&self.row_tail)
            .filter_map(ToolRailEntry::node_id);
        buttons.chain(row).collect()
    }
}

/// O estado do chrome à volta das barras — o que o ARTISTA escolheu, não o que o módulo publica.
#[derive(Clone, Debug, Default)]
pub struct ToolBarsSlot {
    pub(crate) bars: Option<ToolBars>,
    /// ④ desligada pelo artista (o `✕` dela, ou *Ver ▸ Ações da seleção*).
    pub(crate) actions_off: bool,
    /// ④ fixada: deixa de seguir a seleção e fica no topo da área (nunca tapa a arte que se edita).
    pub(crate) actions_pinned: bool,
    /// ⑤ a última posição do ponteiro no ecrã — as coordenadas do rodapé.
    pub(crate) pointer: Option<[f32; 2]>,
    /// O relógio do chrome (ns), posto pelo `dispatch_tick` em todo quadro.
    pub(crate) clock_ns: u128,
    /// F4: o comando que a paleta mandou PISCAR onde mora, o pulldown que o continha NO MOMENTO da
    /// escolha (um interruptor troca de id ao ser executado), e quando.
    pub(crate) flash: Option<(NodeId, Option<NodeId>, u128)>,
    /// F5: os dedos no vidro e o widget dos modificadores (flutua sobre a área como as barras).
    pub(crate) touch: crate::interaction::TouchGestures,
    /// F5 (D10 decisão 2): a fila ② EMBAIXO da área — a opção de layout para tablet.
    pub(crate) row_bottom: bool,
}
