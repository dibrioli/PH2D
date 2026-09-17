//! ⭐⭐⭐ **O VOCABULÁRIO DO MOVER DE VISTA DE CIMA** (TOP-20 #13) — o instantâneo e a edição,
//! num módulo abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Porque ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tags_edits`] e do [`crate::factory_edits`], e com o mesmo número
//! atrás: a catraca do DAG tolera a aresta `action_bus → screens` num **tecto** e escreve a cura ao
//! lado dela — *«os PAYLOADS do Inspector moram em `screens::hero::inspector_model*`; cura: descem
//! para um módulo de vocabulário abaixo do `action_bus`»*.
//!
//! *É o terceiro degrau da mesma migração, e cada um torna o resto mais barato.*
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! Quatro avisos, e cada um responde a uma forma diferente de *«pus o componente e ele não anda»*:
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `the body must be Kinematic` | um corpo dinâmico é do **solver**, e o mover não tem pose para escrever | trocar o *Body Type* |
//! | `a Platform Player on this object wins` | dois movers, um `Transform` — o de plataforma manda | tirar um dos dois |
//! | `only moves while the clock plays` | a ponte corre no passo fixo | carregar no play |
//! | `no body` | sem `RigidBody` não há o que mover | anexar o corpo (a paleta já o pede) |
//!
//! ⚠️ **Os dois primeiros são os que esta wave pagou a descobrir:** sem eles, um componente que
//! está a funcionar perfeitamente lê-se como partido, e os números no painel estão todos certos.

use ph2d_i18n::tr;
/// Em que direcções o corpo aceita andar — o espelho do `ph2d_topdown::DirectionMode`.
///
/// ⚠️ **Uma tag SEPARADA do enum da lei**, como todos os selectores deste painel: o painel fala em
/// posições de segmentado e a lei fala em variantes. Ligar os dois pelo `as u8` faria reordenar um
/// enum trocar o que um clique escreve — **e compila**.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorMoveDirections {
    /// Qualquer ângulo.
    Free,
    /// Os oito rumos.
    #[default]
    Eight,
    /// Os quatro cardeais.
    Four,
    /// Só na horizontal da intenção.
    AxisX,
    /// Só na vertical da intenção.
    AxisY,
}

impl InspectorMoveDirections {
    /// Todas, na ordem do segmentado. ⛔ A posição é a tag do clique — não reordene.
    pub const ALL: [Self; 5] = [
        Self::Free,
        Self::Eight,
        Self::Four,
        Self::AxisX,
        Self::AxisY,
    ];

    /// O rótulo que o artista lê (inglês, HR-15).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Free => tr("panel.topdown.free"),
            Self::Eight => tr("panel.topdown.8_dir"),
            Self::Four => tr("panel.topdown.4_dir"),
            Self::AxisX => tr("panel.topdown.x_only"),
            Self::AxisY => tr("panel.topdown.y_only"),
        }
    }

    /// A posição no segmentado — a tag do clique.
    #[must_use]
    pub fn tag(self) -> u8 {
        Self::ALL
            .iter()
            .position(|m| *m == self)
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0)
    }
}

/// De que ângulo o tabuleiro é visto — o espelho do `ph2d_topdown::Viewpoint`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorViewpoint {
    /// Vista de cima a direito — a identidade.
    #[default]
    TopDown,
    /// O losango clássico, duas células por uma.
    Iso2to1,
    /// O isométrico verdadeiro, 30°.
    Iso30,
    /// A elevação que o artista escrever.
    Custom,
}

impl InspectorViewpoint {
    /// Todos, na ordem do segmentado. ⛔ A posição é a tag do clique.
    pub const ALL: [Self; 4] = [Self::TopDown, Self::Iso2to1, Self::Iso30, Self::Custom];

    /// O rótulo que o artista lê.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::TopDown => tr("panel.topdown.top_down"),
            Self::Iso2to1 => tr("panel.topdown.iso_2_1"),
            Self::Iso30 => tr("panel.topdown.iso_30"),
            Self::Custom => tr("panel.topdown.custom"),
        }
    }

    /// ⭐ **Este modo LÊ o ângulo?** — é esta pergunta que esconde a linha nos outros três.
    ///
    /// ⚠️ *Um painel que mostra um knob que o modo não sabe ler* é o defeito que o painel do
    /// L-System desta casa pagou, e que o `SignalVerb::uses_arg` já escreve como lei.
    #[must_use]
    pub const fn uses_angle(self) -> bool {
        matches!(self, Self::Custom)
    }

    /// A posição no segmentado — a tag do clique.
    #[must_use]
    pub fn tag(self) -> u8 {
        Self::ALL
            .iter()
            .position(|m| *m == self)
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0)
    }
}

/// Para onde o objecto olha — o espelho do `ph2d_topdown::RotationMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorFacing {
    /// Não roda.
    #[default]
    None,
    /// Vira para onde anda.
    Movement,
    /// Vira para o rumo cardeal mais perto.
    Snap4,
    /// Vira para um dos oito rumos.
    Snap8,
}

impl InspectorFacing {
    /// Todos, na ordem do segmentado. ⛔ A posição é a tag do clique.
    pub const ALL: [Self; 4] = [Self::None, Self::Movement, Self::Snap4, Self::Snap8];

    /// O rótulo que o artista lê.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::None => tr("panel.topdown.don_t_turn"),
            Self::Movement => tr("panel.topdown.face_move"),
            Self::Snap4 => tr("panel.topdown.face_4"),
            Self::Snap8 => tr("panel.topdown.face_8"),
        }
    }

    /// ⭐ **Este modo LÊ a velocidade de viragem?** — esconde a linha em `None`.
    #[must_use]
    pub const fn uses_turn_speed(self) -> bool {
        !matches!(self, Self::None)
    }

    /// A posição no segmentado — a tag do clique.
    #[must_use]
    pub fn tag(self) -> u8 {
        Self::ALL
            .iter()
            .position(|m| *m == self)
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0)
    }
}

/// Snapshot da secção TOP-DOWN PLAYER da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorTopDownInfo {
    pub entity_bits: u64,
    pub speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub directions: InspectorMoveDirections,
    pub viewpoint: InspectorViewpoint,
    pub viewpoint_angle_deg: f32,
    pub facing: InspectorFacing,
    pub turn_speed_deg: f32,
    pub min_slide_angle_deg: f32,
    pub max_slides: u32,
    pub default_controls: bool,
    /// ⭐ O corpo é **cinemático**? `false` ⇒ o aviso `the body must be Kinematic`.
    pub body_is_kinematic: bool,
    /// ⭐ O objecto tem `RigidBody`? `false` ⇒ o aviso `no body`.
    pub has_body: bool,
    /// ⭐ Há um `PlatformPlayer` no MESMO objecto? `true` ⇒ o aviso do conflito.
    pub conflicts_with_platformer: bool,
    /// ⭐ O relógio está a andar? A corrida é isso.
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção TOP-DOWN PLAYER.
#[derive(Clone, Debug, PartialEq)]
pub enum TopDownFieldEdit {
    Speed(f32),
    Acceleration(f32),
    Deceleration(f32),
    Directions(InspectorMoveDirections),
    Viewpoint(InspectorViewpoint),
    ViewpointAngle(f32),
    Facing(InspectorFacing),
    TurnSpeed(f32),
    MinSlideAngle(f32),
    MaxSlides(u32),
    DefaultControls(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **A POSIÇÃO no array é a tag do clique** — reordenar faria um clique escrever outro modo,
    /// e compila.
    #[test]
    fn the_position_in_all_is_the_click_tag() {
        for (i, m) in InspectorMoveDirections::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(m.tag()), i, "a direccao {m:?} mudou de posicao");
        }
        for (i, m) in InspectorViewpoint::ALL.into_iter().enumerate() {
            assert_eq!(
                usize::from(m.tag()),
                i,
                "o viewpoint {m:?} mudou de posicao"
            );
        }
        for (i, m) in InspectorFacing::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(m.tag()), i, "o facing {m:?} mudou de posicao");
        }
    }

    /// ⭐⭐ **Só um modo de cada família lê o knob condicional dela.**
    ///
    /// ⚠️ É a lei do `SignalVerb::uses_arg`: sem ela o painel tem duas escolhas igualmente más —
    /// mostrar sempre tudo (um controlo morto em cada modo) ou esconder um campo que o modo lê (uma
    /// feature inalcançável). *As duas leem-se como «mexo e nada acontece».*
    #[test]
    fn only_one_mode_of_each_family_reads_its_conditional_knob() {
        let com_angulo: Vec<_> = InspectorViewpoint::ALL
            .into_iter()
            .filter(|v| v.uses_angle())
            .collect();
        assert_eq!(com_angulo, vec![InspectorViewpoint::Custom]);
        let sem_viragem: Vec<_> = InspectorFacing::ALL
            .into_iter()
            .filter(|f| !f.uses_turn_speed())
            .collect();
        assert_eq!(sem_viragem, vec![InspectorFacing::None]);
        // ⚠️ **O controlo do censo**: se um `ALL` encolher, este gate deixa de falar de todos.
        assert_eq!(InspectorViewpoint::ALL.len(), 4);
        assert_eq!(InspectorFacing::ALL.len(), 4);
        assert_eq!(InspectorMoveDirections::ALL.len(), 5);
    }

    /// ⚠️ **Os defaults do painel são os da LEI** — uma segunda cópia deles aqui divergiria no dia
    /// em que um mudasse, e o painel mostraria números que a cena não tem.
    #[test]
    fn the_panel_defaults_are_the_laws() {
        assert_eq!(InspectorViewpoint::default(), InspectorViewpoint::TopDown);
        assert_eq!(InspectorFacing::default(), InspectorFacing::None);
        assert_eq!(
            InspectorMoveDirections::default(),
            InspectorMoveDirections::Eight
        );
    }
}
