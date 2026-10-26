//! ⭐ **A linha de UMA camada do Pixel Art** — o triângulo do grupo, o olho, o cadeado, o nome
//! (clique = activa, duplo clique = renomear) e o Solo. Mora aqui porque DOIS painéis a pintam (o
//! PIXEL e a linha do tempo do Pixel) e um painel não pode depender de outro
//! (`architecture_cycle_prevention`): cada um dá os seus ids, a pintura é uma só.

use crate::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use crate::pixel_panel::PixelLayerRow;
use crate::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// As partes de uma linha da lista de camadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelLayerWidget {
    /// O nome: clique = activa, duplo clique = renomear.
    Row,
    Eye,
    Lock,
    Solo,
    /// O triângulo do grupo.
    Collapse,
}

impl PixelLayerWidget {
    pub const ALL: [Self; 5] = [Self::Row, Self::Eye, Self::Lock, Self::Solo, Self::Collapse];

    /// O sufixo do id (cada painel prefixa o seu).
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Eye => "eye",
            Self::Lock => "lock",
            Self::Solo => "solo",
            Self::Collapse => "collapse",
        }
    }

    /// A chave da dica do ícone (o nome não tem).
    #[must_use]
    pub const fn tip_key(self) -> Option<&'static str> {
        match self {
            Self::Row => None,
            Self::Eye => Some("panel.pixel.layer.eye"),
            Self::Lock => Some("panel.pixel.layer.lock"),
            Self::Solo => Some("panel.pixel.layer.solo"),
            Self::Collapse => Some("panel.pixel.layer.collapse"),
        }
    }
}

/// Onde pintar: a cena, o texto, o store (leitura) e o índice de cliques.
pub struct LayerRowPainter<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
}

impl LayerRowPainter<'_> {
    fn icone(&mut self, id: NodeId, icon: IconId, r: Rect, on: bool) {
        let cor = if on {
            ColorToken::Text1
        } else {
            ColorToken::Text3
        };
        paint_icon(
            self.scene,
            icon,
            r,
            resolve(cor, self.theme),
            StrokeToken::Default.px(),
        );
        self.hit_index.register(id, r);
    }

    /// Pinta a linha de `row` em `(x, y, w)`; `rename` = o id do campo do renomear quando esta
    /// linha está a ser renomeada. Devolve o `y` da linha seguinte.
    pub fn paint(
        &mut self,
        row: &PixelLayerRow,
        (x0, y, w): (f32, f32, f32),
        active: bool,
        rename: Option<NodeId>,
        id_of: &dyn Fn(PixelLayerWidget) -> NodeId,
    ) -> f32 {
        let h = ROW_H_PX;
        let gap = Spacing::Xs.px();
        if active {
            let r = Rect::new(x0, y, w, h);
            let radius = crate::paint::frame_radius(self.theme, Radius::Sm.px());
            fill_rounded_rect(
                self.scene,
                r,
                radius,
                resolve(ColorToken::AccentSoft, self.theme),
            );
        }
        let recuo = f32::from(row.depth) * Spacing::Md.px();
        let mut x = x0 + recuo;
        let quadrado = |x: f32| Rect::new(x, y, h, h);
        if row.group {
            let icon = if row.collapsed {
                IconId::ChevronRight
            } else {
                IconId::ChevronDown
            };
            self.icone(id_of(PixelLayerWidget::Collapse), icon, quadrado(x), true);
            x += h + gap;
        }
        let olho = if row.visible {
            IconId::Eye
        } else {
            IconId::EyeClosed
        };
        self.icone(id_of(PixelLayerWidget::Eye), olho, quadrado(x), row.visible);
        x += h + gap;
        let cadeado = if row.locked {
            IconId::Lock
        } else {
            IconId::Unlock
        };
        self.icone(
            id_of(PixelLayerWidget::Lock),
            cadeado,
            quadrado(x),
            row.locked,
        );
        x += h + gap;
        let solo_x = x0 + w - h;
        self.icone(
            id_of(PixelLayerWidget::Solo),
            IconId::Circle,
            quadrado(solo_x),
            false,
        );
        let nome_r = Rect::new(x, y, (solo_x - gap - x).max(1.0), h);
        if let Some(campo) = rename {
            let (st, text, caret, anchor) = match self.store.get(campo) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, text.clone(), *caret, *selection_anchor),
                _ => (TextInputState::Focused, String::new(), 0, None),
            };
            let input = TextInput::new(campo, "").visual((st, self.store.hover_live(campo)));
            paint_text_input_with_buffer(
                &input,
                Some(text.as_str()),
                Some(caret),
                anchor,
                nome_r,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(campo, nome_r);
        } else {
            let nome = if row.reference {
                ph2d_i18n::tr_with("panel.pixel.layer.reference", &[("name", &row.name)])
            } else {
                row.name.clone()
            };
            let cor = if row.visible {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            };
            paint_text(
                self.text_system,
                self.scene,
                &nome,
                nome_r.x,
                y + (h - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                nome_r.w,
                resolve(cor, self.theme),
            );
            self.hit_index
                .register(id_of(PixelLayerWidget::Row), nome_r);
        }
        y + ph2d_tokens::row_pitch_px()
    }
}
