//! ⭐⭐⭐ **AS PROPRIEDADES DE UMA CAMADA da pilha de aparência** (estudo 42 item 4, v20) — N
//! preenchimentos e N contornos numa forma.
//!
//! F44 (dono 01/11): as camadas são LINHAS dentro de Fill e Stroke ([`crate::paint_sections::fill_stroke`],
//! o Figma), e clicar numa abre ESTE corpo no painel à esquerda ([`crate::paint_side`]): a largura
//! (num contorno), o deslocamento, o offset de CAD e a quina dele, e a mistura. A cor, a opacidade,
//! o olho e o «−» ficam na linha.
//!
//! ⚠️ O índice é sempre o do DOCUMENTO (chão → topo); a lista pinta o topo em cima, e a inversão
//! mora num sítio só (`layer_lines`).

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_vec_render::blend;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;
use crate::state::PaintRow;

/// O glifo do olho: o travessão do misto da casa (o da amostra «—») quando a camada está visível
/// numas formas e escondida noutras (F22).
pub(crate) fn eye_icon(row: &PaintRow) -> IconId {
    if row.mixed.enabled {
        IconId::Minus
    } else if row.enabled {
        IconId::Eye
    } else {
        IconId::EyeClosed
    }
}

impl BodyCtx<'_> {
    /// ⭐ **A Position de um contorno EXTRA** (F44b) — os glifos da linha fixa da base, com os ids da
    /// camada aberta.
    pub(crate) fn layer_position_row(&mut self, row: &PaintRow, y: f32) -> f32 {
        use ph2d_vec_scene::StrokeAlign;
        let a = |k: StrokeAlign| !row.mixed.align && row.align == k;
        self.with_rows(|rc| {
            rc.icon_segmented(
                tr("panel.vector.stroke.align"),
                &[
                    (
                        ids::VECTOR_PAINT_ALIGN_INNER,
                        IconId::StrokeInside,
                        a(StrokeAlign::Inner),
                    ),
                    (
                        ids::VECTOR_PAINT_ALIGN_CENTRE,
                        IconId::StrokeCenter,
                        a(StrokeAlign::Centre),
                    ),
                    (
                        ids::VECTOR_PAINT_ALIGN_OUTER,
                        IconId::StrokeOutside,
                        a(StrokeAlign::Outer),
                    ),
                ],
                y,
            )
        })
    }

    /// As propriedades da camada ABERTA — o corpo do painel à esquerda (F44).
    pub(crate) fn layer_props(&mut self, row: &PaintRow, y: f32) -> f32 {
        // F44c: o Weight do contorno mora na LINHA dele, ao lado da cor.
        let mut y = y;
        // ⭐⭐⭐ **ONDE ela desenha** (v21) — o par `X`/`Y`, na MESMA `number_row` que o Transform e
        // o Vertex usam para as coordenadas deles.
        //
        // ⚠️ **Aparece nas DUAS espécies**, ao contrário da largura: um preenchimento deslocado é
        // precisamente o caso que motivou isto (a sombra dura), e escondê-lo num preenchimento
        // deixaria de fora o pedido que a wave responde.
        y = self.number_row(
            tr("panel.vector.paint.dx"),
            crate::ids::VECTOR_PAINT_DX,
            tr("panel.vector.paint.dy"),
            crate::ids::VECTOR_PAINT_DY,
            y,
        );
        // ⭐⭐⭐ **O OFFSET DE CAD** (v22) — a silhueta cresce (`>0`) ou encolhe (`<0`).
        y = self.lone_number_row(
            tr("panel.vector.paint.dilate"),
            crate::ids::VECTOR_PAINT_DILATE,
            y,
        );
        // ⚠️ **A QUINA só aparece com o offset ARMADO** — é a lei do «nenhum controlo mudo»: com
        // `dilate = 0` não há esquina nenhuma a formar, e três chips que não mudam nada são a
        // definição de um controlo morto sob o dedo.
        if row.dilate != 0.0 {
            let j = |k: u8| !row.mixed.dilate_join && row.dilate_join == k;
            y = self.with_rows(|rc| {
                rc.icon_segmented(
                    tr("panel.vector.paint.join"),
                    &[
                        (ids::VECTOR_PAINT_JOIN_MITER, IconId::JoinMiter, j(0)),
                        (ids::VECTOR_PAINT_JOIN_ROUND, IconId::JoinRound, j(1)),
                        (ids::VECTOR_PAINT_JOIN_BEVEL, IconId::JoinBevel, j(2)),
                    ],
                    y,
                )
            });
        }
        self.layer_blend_row(row.blend, row.mixed.blend, y)
    }

    /// O chip de mistura da camada aberta. O popover abre no passe DIFERIDO — a lei dos outros
    /// seis desta janela.
    fn layer_blend_row(&mut self, atual: ph2d_vec_scene::BlendMode, misto: bool, y: f32) -> f32 {
        let face = if misto {
            tr("panel.vector.mixed")
        } else {
            tr(atual.label_key())
        };
        if let Some(chip) = self.dropdown_row(
            crate::ids::VECTOR_PAINT_BLEND,
            tr("panel.vector.paint.blend"),
            face,
            Rect::new(self.inner_x, y, self.inner_w, self.row_h),
        ) {
            state::set_pending_paint_blend_dd(Some(chip));
        }
        y + self.row_h + self.row_gap
    }
}

/// **O popover dos modos de mistura de uma CAMADA** — pintado no passe diferido, por cima de todas
/// as seções. Espelho do irmão do objecto, com o espaço de ids próprio.
pub(crate) fn paint_layer_blend_popover(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    chip: Rect,
    theme: ph2d_tokens::Theme,
) {
    let modos: Vec<ph2d_vec_scene::BlendMode> = blend::offered().collect();
    if modos.is_empty() {
        return;
    }
    let atual = state::current_appearance()
        .and_then(|a| state::open_layer().and_then(|i| a.layers.get(i).map(|r| r.blend)))
        .unwrap_or_default();
    let sel = modos.iter().position(|m| *m == atual).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = modos
        .iter()
        .enumerate()
        .map(|(i, m)| {
            DropdownOption::new(ids::vector_paint_blend_option_id(i), i, tr(m.label_key()))
        })
        .collect();
    let dd = Dropdown::new(crate::ids::VECTOR_PAINT_BLEND, "", options)
        .selected(sel)
        .open(true);

    let region = ctx.layout.popover_region();
    ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
}

#[cfg(test)]
mod tests {
    use super::{IconId, PaintRow, eye_icon};

    /// ⭐ F22 — o olho de uma camada visível numas e escondida noutras pinta o travessão do misto.
    #[test]
    fn o_olho_misto_pinta_o_travessao() {
        let mut row = PaintRow {
            is_fill: true,
            color: [0, 0, 0, 255],
            width: 0.0,
            enabled: true,
            opacity: 1.0,
            blend: ph2d_vec_scene::BlendMode::Normal,
            offset: [0.0, 0.0],
            dilate: 0.0,
            dilate_join: 1,
            ramp: None,
            align: ph2d_vec_scene::StrokeAlign::Centre,
            mixed: Default::default(),
        };
        assert_eq!(eye_icon(&row), IconId::Eye);
        row.enabled = false;
        assert_eq!(eye_icon(&row), IconId::EyeClosed);
        row.mixed.enabled = true;
        assert_eq!(eye_icon(&row), IconId::Minus);
    }
}
