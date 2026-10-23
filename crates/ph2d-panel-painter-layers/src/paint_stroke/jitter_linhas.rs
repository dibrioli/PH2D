//! ⭐⭐ **As fileiras do cartão Jitter — UMA tabela, pela ordem da tela.**
//!
//! Ordem do dono (23/10): *«seção Jitter Unit deveria ir para o topo da seção e os parâmetros numéricos deveriam
//! ser todos sliders. Procure ordenar cada parâmetro da forma mais inteligente possível.»* O Unit abre o cartão; o
//! resto agrupa-se pelo que cada número SORTEIA: quantas · onde · quando · tamanho e forma · ângulo · espelho ·
//! aparência. A ordem é DADO ([`FILEIRAS_DO_JITTER`]): o pintor, a altura do cartão e o gate leem a mesma lista.
//!
//! ⚠️ **As linhas da MARCA são barras no espaço da pista, e o chip no espaço VERDADEIRO.** A barra é o próprio id da
//! linha (`PAINTER_MARCA_FIELDS[i]`, o que o tool e o esmaecer já conhecem); o chip liga-se por
//! `link_slider_number_mapped` com a faixa da tabela do tool (`LinhaDaMarca::faixa`) — UMA lei, [`escala_da_marca`],
//! que o registo, a pista pintada e o encaminhamento leem. O tool continua a receber o valor verdadeiro.

use crate::paint_brush_rows::paint_dropdown_row;
use crate::paint_brush_top::paint_slider_chip_row;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_i18n::tr;
use ph2d_tokens::ROW_H_PX;
use ph2d_tool_painter::{
    BRUSH_JITTER_ABS_MAX_PX, BrushSettings, JitterUnit, LINHAS_DA_MARCA, LinhaDaMarca, ids,
};

/// Uma fileira do cartão.
#[derive(Clone, Copy)]
pub(crate) enum Fileira {
    /// O Unit (Brush / View) do Position — o único menu do cartão.
    Unidade,
    /// Uma barra do pincel: a pista `0..1` lida do snapshot; o tool mapeia-a para a sua faixa.
    Pincel {
        chave: &'static str,
        slider: NodeId,
        chip: NodeId,
        pista: fn(&BrushSettings) -> f32,
    },
    /// Uma linha da marca (`marca.rs` do tool), por barra.
    Marca(LinhaDaMarca),
}

/// ⭐ **A ordem da tela**, de cima para baixo.
pub(crate) const FILEIRAS_DO_JITTER: [Fileira; 14] = [
    Fileira::Unidade,
    // Quantas marcas cada ponto deixa — o Count transforma o resto de *tremer* em *espalhar*.
    Fileira::Pincel {
        chave: "panel.painter_layers.stroke.jitter.count",
        slider: ids::PAINTER_BRUSH_SPRAY_COUNT,
        chip: ids::PAINTER_BRUSH_SPRAY_COUNT_CHIP,
        pista: |b| super::jitter_card::spray_count_track(b.spray_count),
    },
    Fileira::Marca(LinhaDaMarca::Count),
    Fileira::Marca(LinhaDaMarca::Particle),
    // Onde cai cada marca.
    Fileira::Pincel {
        chave: "panel.painter_layers.stroke.jitter.position",
        slider: ids::PAINTER_BRUSH_JITTER,
        chip: ids::PAINTER_BRUSH_JITTER_CHIP,
        pista: pista_da_posicao,
    },
    Fileira::Marca(LinhaDaMarca::Across),
    // Quando: o intervalo entre marcas.
    Fileira::Pincel {
        chave: "panel.painter_layers.stroke.jitter.spacing",
        slider: ids::PAINTER_BRUSH_JITTER_SPACING,
        chip: ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
        pista: |b| b.jitter_spacing,
    },
    // Tamanho e forma.
    Fileira::Pincel {
        chave: "panel.painter_layers.stroke.jitter.scale",
        slider: ids::PAINTER_BRUSH_JITTER_SCALE,
        chip: ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
        pista: |b| b.jitter_scale,
    },
    Fileira::Marca(LinhaDaMarca::Roundness),
    // Ângulo: por marca e por traço.
    Fileira::Pincel {
        chave: "panel.painter_layers.stroke.jitter.rotation",
        slider: ids::PAINTER_BRUSH_JITTER_ROTATE,
        chip: ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
        pista: |b| b.jitter_rotate,
    },
    Fileira::Marca(LinhaDaMarca::AnglePerStroke),
    // Espelho.
    Fileira::Marca(LinhaDaMarca::FlipX),
    Fileira::Marca(LinhaDaMarca::FlipY),
    // Aparência.
    Fileira::Marca(LinhaDaMarca::Opacity),
];

/// O Position: a pista é `0..1` nas DUAS unidades (View projeta os px absolutos sobre `0..MAX`).
fn pista_da_posicao(b: &BrushSettings) -> f32 {
    if b.jitter_unit == JitterUnit::View.to_u8() {
        b.jitter_absolute_px / BRUSH_JITTER_ABS_MAX_PX
    } else {
        b.jitter
    }
}

/// O índice da linha `l` em [`LINHAS_DA_MARCA`] (e nos ids dela).
pub(crate) fn indice_da_marca(l: LinhaDaMarca) -> usize {
    LINHAS_DA_MARCA
        .iter()
        .position(|x| *x == l)
        .unwrap_or_default()
}

/// ⭐ **A lei pista ↔ número de uma linha da marca**: `número = pista × escala + deslocamento`, saída da faixa da
/// tabela do tool. O registo do chip, a pista pintada e o encaminhamento leem só esta.
pub(crate) fn escala_da_marca(l: LinhaDaMarca) -> (f32, f32) {
    let (lo, hi, _) = l.faixa();
    ((hi - lo) as f32, lo as f32)
}

/// O índice da linha da marca cuja BARRA é `id`, se ela está no cartão Jitter.
pub(crate) fn barra_da_marca(id: NodeId) -> Option<usize> {
    FILEIRAS_DO_JITTER.iter().find_map(|f| match f {
        Fileira::Marca(l) if ids::PAINTER_MARCA_FIELDS[indice_da_marca(*l)] == id => {
            Some(indice_da_marca(*l))
        }
        _ => None,
    })
}

/// A altura que a fileira `f` ocupa (o mesmo avanço que o pintor dá) — a do cartão é a soma.
pub(crate) fn altura(f: Fileira, inner_w: f32) -> f32 {
    match f {
        Fileira::Unidade => ph2d_tokens::row_pitch_px(),
        _ => {
            ph2d_editor_core::widget::slider_with_chip_height(ROW_H_PX, inner_w)
                + ph2d_tokens::control_gap_px()
        }
    }
}

/// Pinta as fileiras pela ordem da tabela; devolve o `y` seguinte.
pub(crate) fn paint_fileiras(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    brush: &BrushSettings,
) -> f32 {
    for f in FILEIRAS_DO_JITTER {
        y = match f {
            Fileira::Unidade => {
                let (ny, open) = paint_dropdown_row(
                    ctx,
                    theme,
                    x,
                    w,
                    y,
                    "panel.painter_layers.stroke.jitter.unit",
                    ids::PAINTER_BRUSH_JITTER_UNIT,
                    brush.jitter_unit,
                    super::jitter_card::jitter_unit_name(brush.jitter_unit),
                );
                if let Some(r) = open {
                    state::set_pending_brush_jitter_unit_dd(Some((r, brush.jitter_unit)));
                }
                ny
            }
            Fileira::Pincel {
                chave,
                slider,
                chip,
                pista,
            } => paint_slider_chip_row(ctx, theme, x, w, y, tr(chave), slider, chip, pista(brush)),
            Fileira::Marca(l) => {
                let i = indice_da_marca(l);
                let (escala, desloc) = escala_da_marca(l);
                let pista = (brush.marca[i] as f32 - desloc) / escala;
                paint_slider_chip_row(
                    ctx,
                    theme,
                    x,
                    w,
                    y,
                    tr(super::jitter_card::ROTULOS_DA_MARCA[i]),
                    ids::PAINTER_MARCA_FIELDS[i],
                    ids::PAINTER_MARCA_CHIPS[i],
                    pista,
                )
            }
        };
    }
    y
}

/// **Encaminha a barra de uma linha da marca** como o valor VERDADEIRO (`SetValue`), o que o tool sempre recebeu.
///
/// ⚠️ O número é o do CHIP quando ele está em dia com a pista (digitado: o valor exacto, sem a volta pela pista em
/// `f32`); senão a pista projectada pela MESMA ligação do store (um polegar mexido sem espelho no chip).
pub(crate) fn encaminha_marca(host: &mut dyn PanelHostInternal, id: NodeId) -> bool {
    let Some(i) = barra_da_marca(id) else {
        return false;
    };
    let chip = ids::PAINTER_MARCA_CHIPS[i];
    let store = host.store();
    let pista = store.slider(id).map(|(_, v)| v).unwrap_or(0.0);
    let pela_pista = f64::from(store.linked_chip_display(chip, pista));
    let (escala, _) = escala_da_marca(LINHAS_DA_MARCA[i]);
    let tolerancia = f64::from(f32::EPSILON * escala.abs().max(1.0)) * 4.0; // LITERAL-PX-OK: folga de arredondamento (comportamento, não layout)
    let v = match store.number_value(chip) {
        Some(c) if (c - pela_pista).abs() <= tolerancia => c,
        _ => pela_pista,
    };
    host.bus_mut()
        .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
            ph2d_editor_core::tool::PanelEvent::SetValue(id, v),
        ));
    true
}
