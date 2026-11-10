//! Os pesos do texto da tabela ([`Look`]): os da casa por omissão; a superfície que dá o peso final
//! (a aba de scripts) passa os seus. Medir e pintar vão pela MESMA porta de cada papel.

use ph2d_editor_core::paint::{paint_text, paint_text_in_weight, paint_text_title};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

/// O papel de um texto da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// As células e a contagem do grupo.
    Body,
    /// O título de uma coluna.
    Head,
    /// O título de uma linha de grupo.
    Group,
}

/// Os pesos da tabela.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Look {
    /// Os da casa: corpo e cabeçalho no `Medium`, o grupo no `SemiBold`, com o reforço do preset.
    #[default]
    House,
    /// Pesos FINAIS, sem o reforço do preset ([`ph2d_text::com_peso_exato`]).
    Exact {
        body: FontWeight,
        head: FontWeight,
        group: FontWeight,
    },
}

impl Look {
    #[must_use]
    pub fn weight(self, role: Role) -> FontWeight {
        match (self, role) {
            (Self::House, Role::Group) => FontWeight::SEMI_BOLD,
            (Self::House, _) => FontWeight::MEDIUM,
            (Self::Exact { body, .. }, Role::Body) => body,
            (Self::Exact { head, .. }, Role::Head) => head,
            (Self::Exact { group, .. }, Role::Group) => group,
        }
    }

    /// A largura de `text` como [`Self::paint`] o pinta.
    pub fn width(self, ts: &mut TextSystem, text: &str, px: f32, role: Role) -> f32 {
        let w = self.weight(role);
        match self {
            Self::House => ts.prefix_width_weighted(text, px, w),
            Self::Exact { .. } => {
                ph2d_text::com_peso_exato(|| ts.prefix_width_weighted(text, px, w))
            }
        }
    }

    /// Pinta `text` numa linha, cortado em `max_w`.
    #[allow(clippy::too_many_arguments)]
    #[track_caller]
    pub fn paint(
        self,
        ts: &mut TextSystem,
        scene: &mut VectorScene,
        text: &str,
        [x, y]: [f32; 2],
        px: f32,
        max_w: f32,
        color: Color,
        role: Role,
    ) {
        match (self, role) {
            (Self::House, Role::Group) => paint_text_title(ts, scene, text, x, y, px, max_w, color),
            (Self::House, _) => paint_text(ts, scene, text, x, y, px, max_w, color),
            (Self::Exact { .. }, _) => {
                let w = self.weight(role);
                paint_text_in_weight(ts, scene, text, [x, y], px, max_w, color, w);
            }
        }
    }
}
