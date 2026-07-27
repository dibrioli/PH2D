//! **A FERRAMENTA DE JOINTS** (W-JointTools) — o que o ponteiro faz a uma cadeia
//! ARTICULADA.
//!
//! Irmã da [`crate::interaction::InteractionTool`], e separada dela por uma
//! razão que não é arrumação: aquelas três ferramentas empurram o **SOLVER** de
//! uma cena que está rodando (uma mola, um estouro, um campo) e esta família
//! AUTORA a cena com o relógio parado. Uma mesma lista com as duas obrigaria
//! todo consumidor a perguntar *"esta é das que precisam de Play ou das que
//! precisam de Pause?"* — e a resposta viveria num `match` por chamador, que é
//! como a sexta nasce fora da regra.
//!
//! # Os cinco modos, e por que são cinco e não dois
//!
//! Três deles respondem **quanto da cadeia um arrasto carrega**, e os outros
//! dois são gestos de POSE inteiramente diferentes:
//!
//! | modo | o press abre | o arrasto move |
//! |---|---|---|
//! | [`JointTool::Body`] | o arrasto normal | só o corpo pego |
//! | [`JointTool::Rig`] | o arrasto normal | o rig INTEIRO, âncoras inclusas |
//! | [`JointTool::Links`] | o arrasto normal | só os elos móveis; âncoras ficam |
//! | [`JointTool::Ik`] | a cinemática INVERSA | a cadeia dobra atrás da ponta |
//! | [`JointTool::Fk`] | a cinemática DIRETA | o elo gira na junta; filhos seguem |
//!
//! Os três primeiros já existiam no produto — dois deles como comportamento
//! (o arrasto simples e o que o Alt fazia) e o terceiro como uma versão que foi
//! MEDIDA e substituída (W-JG: *"faça arrastar a cadeia inteira independente do
//! tipo"*). Torná-los um rádio não acrescenta caminho de código nenhum: ele
//! **nomeia** o que já acontecia e devolve a terceira política, que tinha
//! desaparecido junto com a decisão.
//!
//! # O Alt continua significando o rig inteiro, e a lei mora numa função só
//!
//! O atalho da W-JG (Alt+arrastar carrega o componente conexo) é músculo de
//! quem já usa o editor, e ele **sobrevive à chegada do rádio**: com o Alt
//! apertado, [`JointTool::drag_reach`] devolve [`DragReach::Whole`] e
//! [`JointTool::gesture`] devolve `None`, seja qual for o modo escolhido.
//!
//! ⚠️ As duas metades saem da MESMA condição de propósito. *"Alt = arraste o rig
//! inteiro"* só é verdade se o Alt também **suprimir** o gesto de IK/FK — senão
//! Alt em modo IK abriria uma pose e o rig não iria a lugar nenhum, e o artista
//! teria descoberto que o atalho "não funciona às vezes". Duas funções, uma
//! decisão, e há gate exigindo que elas concordem.

/// Até onde um arrasto de corpo carrega a cadeia.
///
/// Os dois valores são as duas políticas da travessia única do
/// [`crate::joint_group`] — ver lá a tabela de *quem conduz*, que é a diferença
/// inteira entre eles.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DragReach {
    /// Todo corpo preso ao pego, **seja qual for o tipo** — o
    /// [`crate::jointed_rig`]. Um gancho Static e uma plataforma Kinematic vão
    /// junto, porque as duas âncoras de um joint viajam com os corpos delas e
    /// quem fica para trás deixa a restrição esticada.
    Whole,
    /// Só os elos **móveis**: as âncoras Static/Kinematic ficam onde estão — o
    /// [`crate::jointed_group`]. É o modo de posar um braço sem arrancar o ombro
    /// da parede.
    Dynamic,
}

/// O gesto que um press abre quando não é um arrasto comum.
///
/// Enum de dois valores em vez de dois `bool`s no chamador: os dois gestos são
/// mutuamente exclusivos por construção (o ponteiro é um só), e um par de
/// booleanos permite escrever o estado impossível.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JointGesture {
    /// Cinemática **inversa**: arrastar a ponta e deixar o solver achar os
    /// ângulos de toda a cadeia (W-IK).
    Ik,
    /// Cinemática **direta**: arrastar um elo e girá-lo na própria junta,
    /// levando os descendentes rigidamente.
    Fk,
}

/// Qual das cinco o artista escolheu.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum JointTool {
    /// O default, e é o comportamento que o editor sempre teve: arrastar move
    /// **só** o corpo pego. Um joint fica esticado até o Play resolvê-lo — o
    /// que se VÊ, pelo segmento âmbar do overlay.
    #[default]
    Body,
    /// Arrastar carrega o rig inteiro ([`DragReach::Whole`]).
    Rig,
    /// Arrastar carrega os elos móveis ([`DragReach::Dynamic`]).
    Links,
    /// Posar por cinemática inversa (W-IK).
    Ik,
    /// Posar por cinemática direta (W-FK).
    Fk,
}

impl JointTool {
    /// A ordem em que o painel pinta os chips — a mesma lista, um índice.
    pub const ALL: [Self; 5] = [Self::Body, Self::Rig, Self::Links, Self::Ik, Self::Fk];

    /// **Que conjunto um arrasto de corpo carrega**, dado o estado do Alt.
    /// `None` = só o corpo pego.
    ///
    /// O chamador só a consulta com o relógio PARADO: tocando, a pose é do
    /// solver e o `settle` a reimpõe no tick seguinte de qualquer forma (a
    /// condição 2 da W-JG, que continua valendo).
    #[must_use]
    pub fn drag_reach(self, alt: bool) -> Option<DragReach> {
        if alt {
            return Some(DragReach::Whole);
        }
        match self {
            Self::Rig => Some(DragReach::Whole),
            Self::Links => Some(DragReach::Dynamic),
            // Os dois modos de POSE não carregam rig: eles abrem um gesto
            // próprio, e um arrasto de gizmo por baixo dele seria um segundo
            // autor do mesmo `Transform` no mesmo frame.
            Self::Body | Self::Ik | Self::Fk => None,
        }
    }

    /// **Que gesto um press abre**, dado o estado do Alt. `None` = nenhum, e o
    /// arrasto normal acontece.
    ///
    /// Ver o cabeçalho: o Alt suprime os dois gestos porque ele significa *"leve
    /// o rig inteiro"*, e isso é um arrasto.
    #[must_use]
    pub fn gesture(self, alt: bool) -> Option<JointGesture> {
        if alt {
            return None;
        }
        match self {
            Self::Ik => Some(JointGesture::Ik),
            Self::Fk => Some(JointGesture::Fk),
            Self::Body | Self::Rig | Self::Links => None,
        }
    }

    /// **Este modo trabalha com o relógio PARADO?**
    ///
    /// A porta única do relógio: o painel pergunta para escrever a dica, e a
    /// shell — indiretamente, pelas duas funções acima, que o chamador só
    /// consulta em repouso. Enumerar *quais* modos são de repouso em dois
    /// lugares é como o sexto nasceria de fora da regra.
    ///
    /// O `Body` é o único que não é: ele não é um modo, é a **ausência** de um
    /// (o arrasto de gizmo comum, que funciona tocando ou parado como sempre
    /// funcionou).
    #[must_use]
    pub fn runs_at_rest(self) -> bool {
        !matches!(self, Self::Body)
    }

    /// Wire string ↔ variante, numa função por par — o mapeamento escrito duas
    /// vezes apodrece exatamente como o `BodyKind::tag` do W4 documenta.
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Rig => "rig",
            Self::Links => "links",
            Self::Ik => "ik",
            Self::Fk => "fk",
        }
    }

    /// Inverso de [`Self::tag`]. `None` para um tag que não existe — o chamador
    /// decide, em vez de receber `Body` em silêncio.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "body" => Some(Self::Body),
            "rig" => Some(Self::Rig),
            "links" => Some(Self::Links),
            "ik" => Some(Self::Ik),
            "fk" => Some(Self::Fk),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O Alt é UMA decisão, e as duas metades dela têm de concordar.**
    ///
    /// A mutação que este gate existe para pegar é tirar o `if alt` de UMA das
    /// funções: com o Alt apertado em modo IK, o press abriria uma pose e o rig
    /// não iria a lugar nenhum — o atalho "não funcionando às vezes", que é a
    /// forma mais cara de bug de UI porque o artista aprende a não confiar nele.
    #[test]
    fn alt_always_means_the_whole_rig_and_never_a_pose() {
        for tool in JointTool::ALL {
            assert_eq!(
                tool.drag_reach(true),
                Some(DragReach::Whole),
                "{tool:?} com Alt tinha de carregar o rig inteiro"
            );
            assert_eq!(
                tool.gesture(true),
                None,
                "{tool:?} com Alt abriu um gesto de pose — o arrasto do rig some"
            );
        }
    }

    /// Sem Alt, cada modo faz o que o nome dele diz, e **exatamente um** dos dois
    /// canais responde por vez.
    #[test]
    fn each_mode_answers_on_exactly_one_channel() {
        for tool in JointTool::ALL {
            let reach = tool.drag_reach(false);
            let gesture = tool.gesture(false);
            assert!(
                reach.is_none() || gesture.is_none(),
                "{tool:?} abriria um gesto E carregaria um rig — dois autores do \
                 mesmo Transform no mesmo frame"
            );
        }
        assert_eq!(JointTool::Body.drag_reach(false), None);
        assert_eq!(JointTool::Rig.drag_reach(false), Some(DragReach::Whole));
        assert_eq!(JointTool::Links.drag_reach(false), Some(DragReach::Dynamic));
        assert_eq!(JointTool::Ik.gesture(false), Some(JointGesture::Ik));
        assert_eq!(JointTool::Fk.gesture(false), Some(JointGesture::Fk));
    }

    /// O round-trip do wire string, sobre a lista inteira — um tag escrito à mão
    /// no gate driftaria da tabela que ele deveria vigiar.
    #[test]
    fn every_tag_round_trips() {
        for tool in JointTool::ALL {
            assert_eq!(JointTool::from_tag(tool.tag()), Some(tool));
        }
        assert_eq!(JointTool::from_tag("pose"), None);
    }
}
