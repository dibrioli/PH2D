//! **O VIEWPOINT** — a mesma seta «cima», noutro tabuleiro.
//!
//! É *«o golpe do GDevelop»* que a síntese nomeia: um **menu** reprojecta a
//! entrada para uma vista isométrica **sem trocar de componente**. Pressionar
//! `→` num tabuleiro 2:1 anda para nordeste no ecrã, que é o que o artista lê
//! como *«uma casa para a direita»*.
//!
//! # ⛔ Esta metade NÃO tem oráculo instalado, e diz-se
//!
//! O GDevelop não está nesta máquina (a triagem do plano §1.1 tem a tabela), logo
//! aqui **não há paridade com ninguém** — há geometria, e gates que a afirmam. A
//! metade do deslize, essa, é portada do Godot com corpus.
//!
//! # A lei, e por que ela degenera EXACTAMENTE na identidade
//!
//! ```text
//! TopDown        identidade, ao bit  (nem roda, nem achata)
//! isométrico     roda 45°, achata o eixo vertical por k = tan(elevação), normaliza
//! ```
//!
//! Com `k` a elevação do eixo do tabuleiro sobre a horizontal do ecrã:
//! `2:1` ⇒ `26,565°` (`tan = 0,5`), o isométrico *«verdadeiro»* ⇒ `30°`.
//! A intenção `→ (1,0)` sai em `(0,894, 0,447)` — nordeste; a intenção `↑ (0,1)`
//! sai em `(−0,894, 0,447)` — noroeste. *São as duas arestas do losango.*
//!
//! ⚠️ **A identidade é um RAMO, não um caso-limite**: pôr `elevação = 45°` daria
//! `k = 1` e uma rotação de 45° que **não** é a identidade. Quem quer *«sem
//! isometria»* escolhe [`Viewpoint::TopDown`], e o gate exige bit-a-bit.

use crate::{Vec2, normalize};

/// **De que ângulo o tabuleiro é visto.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Viewpoint {
    /// Vista de cima a direito — **identidade, ao bit**. O default.
    #[default]
    TopDown,
    /// O losango clássico dos jogos 2D: duas células de largura por uma de
    /// altura ⇒ elevação `arctan(0,5) = 26,565°`.
    Isometric2to1,
    /// O isométrico *verdadeiro* da geometria: elevação `30°`.
    Isometric30,
    /// A elevação que o artista escrever (`TopDownLaw::viewpoint_angle_deg`).
    Custom,
}

impl Viewpoint {
    /// Todos, na ordem do painel. ⛔ **A fonte da contagem.**
    pub const ALL: [Self; 4] = [
        Self::TopDown,
        Self::Isometric2to1,
        Self::Isometric30,
        Self::Custom,
    ];

    /// O rótulo que o painel pinta.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TopDown => "Top-Down",
            Self::Isometric2to1 => "Isometric 2:1",
            Self::Isometric30 => "Isometric 30°",
            Self::Custom => "Custom Angle",
        }
    }

    /// A elevação em graus deste modo — `None` para a identidade.
    ///
    /// ⚠️ **O `Custom` devolve `None` aqui de propósito:** o número dele vive na
    /// config, e uma segunda cópia neste `match` seria a segunda resposta à mesma
    /// pergunta. Quem resolve as duas é [`reproject`].
    #[must_use]
    pub const fn elevation_deg(self) -> Option<f32> {
        match self {
            Self::TopDown | Self::Custom => None,
            // arctan(0,5), escrito com os dígitos que o `f32` guarda.
            Self::Isometric2to1 => Some(26.565_052),
            Self::Isometric30 => Some(30.0),
        }
    }

    /// Se este modo lê o campo `viewpoint_angle_deg` da config.
    ///
    /// ⭐ É esta pergunta que o painel usa para **esconder** a linha do ângulo nos
    /// outros três — *um painel que mostra um knob que o modo não sabe ler é o
    /// defeito que o L-System desta casa pagou*.
    #[must_use]
    pub const fn reads_angle(self) -> bool {
        matches!(self, Self::Custom)
    }
}

/// **A porta**: intenção quantizada ⇒ direcção de MUNDO.
///
/// O comprimento da intenção sobrevive; só a direcção é reprojectada.
#[must_use]
pub fn reproject(intent: Vec2, view: Viewpoint, custom_deg: f32) -> Vec2 {
    if matches!(view, Viewpoint::TopDown) {
        // ⚠️ Saída **ao bit** — nem uma multiplicação por 1,0.
        return intent;
    }
    let elev = view.elevation_deg().unwrap_or(custom_deg);
    // Uma elevação fora do intervalo aberto `(0°, 90°)` não descreve tabuleiro
    // nenhum: `0` achata tudo numa linha, `90` deixa de achatar. Cair na
    // identidade é a única saída que não inventa um losango.
    if !(elev.is_finite() && elev > 0.0 && elev < 90.0) {
        return intent;
    }
    let Some(dir) = normalize(intent) else {
        return [0.0, 0.0];
    };
    let comprimento = crate::len(intent);

    // Roda 45° — as duas teclas passam a apontar às arestas do losango.
    const COS45: f32 = core::f32::consts::FRAC_1_SQRT_2;
    let rx = dir[0] * COS45 - dir[1] * COS45;
    let ry = dir[0] * COS45 + dir[1] * COS45;
    // …e achata o vertical pela elevação. `libm` pelo motivo de sempre.
    let k = libm::tanf(elev.to_radians());
    let Some(unit) = normalize([rx, ry * k]) else {
        return [0.0, 0.0];
    };
    [unit[0] * comprimento, unit[1] * comprimento]
}

/// **O valor de FIO** — ver o irmão em [`crate::direction::to_wire`] para o
/// motivo de os números serem explícitos.
#[must_use]
pub const fn to_wire(v: Viewpoint) -> u8 {
    match v {
        Viewpoint::TopDown => 0,
        Viewpoint::Isometric2to1 => 1,
        Viewpoint::Isometric30 => 2,
        Viewpoint::Custom => 3,
    }
}

/// O inverso; desconhecido cai no default.
#[must_use]
pub const fn from_wire(v: u8) -> Viewpoint {
    match v {
        1 => Viewpoint::Isometric2to1,
        2 => Viewpoint::Isometric30,
        3 => Viewpoint::Custom,
        _ => Viewpoint::TopDown,
    }
}
