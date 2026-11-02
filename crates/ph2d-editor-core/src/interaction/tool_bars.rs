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
        /// A seleção múltipla tem cores diferentes: pinta-se o «misto» (plano 31 da UI nova).
        mixed: bool,
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
    /// ① os botões da coluna que guardam MAIS ferramentas (abrem um flyout): o pintor marca-os com
    /// o triângulo no canto de baixo (Photoshop, Aseprite). Opt-in por botão.
    pub grouped: Vec<NodeId>,
    /// ② o que entra na fila ANTES dos chips do gizmo — a prioridade mais alta (sai por último).
    pub row_lead: Vec<ToolRailEntry>,
    /// ② o que entra DEPOIS dos chips do gizmo e antes dos pulldowns da área.
    pub row_tail: Vec<ToolRailEntry>,
    /// ④ as ações da seleção, por grupos. Vazio ⇒ sem barra.
    pub actions: Vec<Vec<BarButton>>,
    /// ④ a 2.ª linha, por baixo da 1.ª (o Alinhar do vetor, dono 15/10). Vazia ⇒ uma linha só.
    pub actions_below: Vec<Vec<BarButton>>,
    /// ④ o rectângulo da seleção no ECRÃ — a âncora da barra quando não está fixada.
    pub selection: Option<Rect>,
    /// ⑤ a dica da ferramenta na mão (o que o clique, o arrasto e os modificadores fazem).
    pub hint: String,
    /// ⑤ as leituras do rodapé (coordenadas, encaixe), já formatadas.
    pub readout: Vec<String>,
    /// ⭐ **O tinto do palco** — a lavagem translúcida que o módulo pede sobre a área de desenho,
    /// por cima da arte e por baixo das réguas e gizmos (F17: a vista de interface do vetor). Vem
    /// com as barras porque só vale com a ferramenta do módulo na mão. `None` = palco limpo.
    pub stage_tint: Option<ColorToken>,
    /// ⭐ Estas barras são as do PINTOR, que o chrome deriva da lista do trilho em todo quadro
    /// ([`crate::screens::hero::painter_column`]), e não as de um módulo. Um módulo que larga a mão só
    /// limpa as SUAS: sem esta marca, o vetor inactivo apagaria a coluna do pintor em todo quadro.
    pub painter_rail: bool,
    /// ⭐ Estas barras são as do PIXEL SPRITE ▸ Draw (ADR-0196), publicadas pela família: como as do
    /// pintor, não são de um módulo com barra de controle — a fila guarda o seletor de modo, e um
    /// módulo que larga a mão não as limpa.
    pub pixel_rail: bool,
}

impl ToolBars {
    /// Uma coluna de FAMÍLIA (o pintor, o Pixel) e não as barras de um módulo: a fila de cima mostra
    /// os pulldowns da área (o seletor de modo), e o módulo inactivo não as apaga.
    #[must_use]
    pub fn is_family_column(&self) -> bool {
        self.painter_rail || self.pixel_rail
    }

    /// Todo id que estas barras pintam neste quadro — o gate *uma porta* conta-os.
    #[must_use]
    pub fn painted_ids(&self) -> Vec<NodeId> {
        let buttons = self
            .column
            .iter()
            .flatten()
            .chain(self.flyout.iter().flat_map(|(_, f)| f.iter()))
            .chain(self.actions.iter().flatten())
            .chain(self.actions_below.iter().flatten())
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
    /// F16: onde cada chip da fila ② caiu no último quadro — a âncora de um popover que o MÓDULO
    /// pinta por baixo do chip (o painel pinta antes da fila, logo lê o quadro anterior).
    pub(crate) row_slots: Vec<(NodeId, crate::zones::Rect)>,
    /// O objecto activo é de uma FAMÍLIA que não usa a ferramenta da mão (o Pixel Sprite,
    /// ADR-0196): o vetor larga as barras e o painel mesmo com a ferramenta dele na mão.
    pub(crate) family_owns_canvas: bool,
}
