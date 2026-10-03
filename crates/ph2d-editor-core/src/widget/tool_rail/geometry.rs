//! ⭐⭐ **A geometria do rail** — onde cada entrada cai, nos dois eixos, e quanto ela avança.
//!
//! Irmão por tecto de LOC do `tool_rail.rs` (2026-10-02, ao entrarem os pulldowns largos da fila):
//! o tipo e o que o rail É ficam lá; a ARITMÉTICA de onde ele cai fica aqui, numa porta só.

use super::*;

/// **Onde uma entrada do rail cai** — a resposta da porta única [`entry_rects`].
///
/// `id` é `None` só para o [`ToolRailEntry::Divider`], e nesse caso `rect` é a LINHA que ele
/// desenha, não um alvo: um divisor não se clica.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntrySlot {
    /// O índice na lista de entradas — o que o `build_entry_a11y` pede.
    pub index: usize,
    /// O id do chip, ou `None` num divisor.
    pub id: Option<NodeId>,
    /// O rectângulo do chip em REPOUSO (ou a linha, num divisor).
    pub rect: Rect,
}

/// **O vão entre duas entradas vizinhas** — a geometria ([`entry_rects`], [`horizontal_lines`]) e a
/// legenda da fila (que ocupa o passo do chip) lêem-no daqui.
#[must_use]
pub fn entry_gap_px() -> f32 {
    Spacing::Xs.px()
}

/// ⭐⭐ **A PORTA ÚNICA da geometria de um rail** — *onde cai cada entrada?*
///
/// ⛔⛔ **Ela existe porque a resposta estava escrita TRÊS vezes**, e nada no repo ligava as
/// cópias: o pintor (`tool_rail/paint.rs`), o registo de hit do trilho (`hero/left_rail.rs`) e o
/// registo de hit do flyout — cada um com o seu `let mut y`, o seu `gap` e o seu `chip_x`. O
/// comentário do segundo dizia *«Hit-rects MUST mirror exactly what `paint_tool_rail` paints»*,
/// que é a confissão do defeito: **um espelho não é uma lei**. Um pintor horizontal com um hit
/// vertical compilaria e passaria a suíte inteira.
///
/// ⚠️ **O rect devolvido é o de REPOUSO**, antes do `hover_lift`: o desenho cresce, o alvo não —
/// um alvo que se move debaixo do dedo é um alvo que foge.
/// Quanto uma entrada AVANÇA no eixo — a mesma lei nos dois.
/// ⭐ **Quanto uma entrada avança ao longo do eixo** — pública desde 2026-08-31 porque a decisão do
/// transbordo (`hero::tool_bar::bar_split`) tem de usar **a mesma** aritmética que o
/// [`horizontal_lines`] e o [`entry_rects`]. Uma terceira cópia dela poria o `⋯` a discordar de
/// onde os chips de facto caem.
///
/// ⭐ Numa FILA o pulldown ([`ToolRailEntry::Compound`]) é tão largo quanto a face mais larga que
/// pode mostrar (`row_w`, de [`size_row_pulldowns`]); na COLUNA a largura é a da coluna e ele
/// avança um chip.
#[must_use]
pub fn entry_advance(entry: &ToolRailEntry, chip_px: f32, axis: RailAxis) -> f32 {
    match entry {
        ToolRailEntry::Divider => 1.0 + DIVIDER_GAP_PX * 2.0,
        ToolRailEntry::Compound { row_w, .. } if axis == RailAxis::Horizontal => row_w.max(chip_px),
        _ => chip_px,
    }
}

/// O tamanho de letra da face de um pulldown — o pintor e a medida lêem-no daqui.
#[must_use]
pub fn compound_face_font_px() -> f32 {
    TypeToken::Xxs.px()
}

/// ⭐⭐ **Mede a largura de cada pulldown na FILA** — a caixa em que a face mais larga que ele
/// pode mostrar cabe inteira (2026-10-02, escolha do dono: *botão mais largo*, como os pulldowns
/// do cabeçalho do Blender).
///
/// ⛔ Antes o chip era quadrado (`36 px`) e a face saía `G…` (*Global*), `S…` (*Selected*) e, no
/// tamanho de texto Large, `…`. A largura é a inversa exacta do orçamento do pintor
/// ([`crate::paint::rect_for_label`]), medida no estilo de texto ACTIVO e no peso em que ele pinta.
/// Gate: `nenhuma_face_de_pulldown_da_fila_e_cortada`.
pub fn size_row_pulldowns(rail: &mut ToolRail, text_system: &mut TextSystem) {
    let font = compound_face_font_px();
    for entry in &mut rail.entries {
        if let ToolRailEntry::Compound { faces, row_w, .. } = entry {
            let widest = faces
                .iter()
                .map(|f| text_system.prefix_width_weighted(f, font, FontWeight::MEDIUM))
                .fold(0.0_f32, f32::max);
            *row_w = crate::paint::rect_for_label(widest);
        }
    }
}

/// ⭐⭐ **A largura da COLUNA vertical (`F9`)**: o chip, ou o pulldown mais largo que ela mostra
/// (`row_w`, de [`size_row_pulldowns`]) — escolha do dono (02/10): *coluna mais larga*. Sem
/// pulldown mais largo que o chip é o [`RailButtonSize::rail_width_px`] de sempre.
/// Gate: `nenhuma_face_de_pulldown_da_coluna_e_cortada`.
#[must_use]
pub fn column_width_px(rail: &ToolRail, size: RailButtonSize) -> f32 {
    CHIP_X_OFFSET_PX + widest_pulldown(rail, size.chip_px()) + Spacing::Xs.px()
}

/// O pulldown mais largo do rail, nunca menos que um chip.
fn widest_pulldown(rail: &ToolRail, chip_px: f32) -> f32 {
    rail.entries
        .iter()
        .filter_map(|e| match e {
            ToolRailEntry::Compound { row_w, .. } => Some(*row_w),
            _ => None,
        })
        .fold(chip_px, f32::max)
}

/// A distância entre duas LINHAS de uma fila horizontal — rótulo, folga, chip, e o respiro.
#[must_use]
pub fn line_pitch(chip_px: f32) -> f32 {
    LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX + chip_px + Spacing::Xs.px()
}

/// **Quantas LINHAS uma fila horizontal precisa** para caber em `width`.
///
/// ⚠️ Ela existe porque a ALTURA da faixa depende da largura da área, e a largura da área **não**
/// depende da altura da faixa — não há circularidade, há duas passagens.
#[must_use]
pub fn horizontal_lines(rail: &ToolRail, width: f32, size: RailButtonSize) -> usize {
    if width <= 0.0 {
        return 1;
    }
    let gap = entry_gap_px();
    let chip_px = size.chip_px();
    let mut lines = 1usize;
    let mut along = 0.0_f32;
    for (index, entry) in rail.entries.iter().enumerate() {
        if index > 0 {
            along += gap;
        }
        let advance = entry_advance(entry, chip_px, RailAxis::Horizontal);
        if along + advance > width && along > 0.0 {
            lines += 1;
            along = 0.0;
        }
        along += advance;
    }
    lines
}

#[must_use]
pub fn entry_rects(
    rail: &ToolRail,
    rect: Rect,
    size: RailButtonSize,
    axis: RailAxis,
) -> Vec<EntrySlot> {
    let gap = entry_gap_px();
    let chip_px = size.chip_px();
    // O deslocamento no eixo TRANSVERSAL: na coluna o chip afasta-se da borda esquerda para dar
    // sítio ao rótulo rodado; na fila ele desce para o rótulo caber por cima.
    let cross = match axis {
        RailAxis::Vertical => rect.x + CHIP_X_OFFSET_PX,
        RailAxis::Horizontal => rect.y + LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX,
    };
    let mut cross = cross;
    let mut along = match axis {
        RailAxis::Vertical => rect.y,
        RailAxis::Horizontal => rect.x,
    };
    let mut out = Vec::with_capacity(rail.entries.len());
    for (index, entry) in rail.entries.iter().enumerate() {
        if index > 0 {
            along += gap;
        }
        // ⛔⛔ **A FILA QUEBRA DE LINHA quando não cabe** — e o motivo é que o transbordo era
        // **mudo**: a faixa blinda a tinta E o hit (`push_clip`), e o `HitIndex::register`
        // DESCARTA um rect totalmente cortado ⇒ um chip a mais não ficava truncado, ficava
        // **inexistente**, sem nada no ecrã a dizê-lo. Medido em 2026-08-30: a 1280 px com o
        // preset *Large*, o *Undo* e o *Redo* desapareciam; com as colunas arrastadas ao máximo
        // (`DOCK_W_MAX`) desapareciam **os dezasseis**.
        //
        // ⚠️ Só o eixo HORIZONTAL quebra: a coluna corre no lado longo da janela e nunca teve
        // este problema, e fazê-la quebrar mudaria uma geometria que ninguém pediu.
        if axis == RailAxis::Horizontal {
            let advance = entry_advance(entry, chip_px, axis);
            if along + advance > rect.x + rect.w && along > rect.x {
                along = rect.x;
                cross += line_pitch(chip_px);
            }
        }
        let advance = entry_advance(entry, chip_px, axis);
        let r = match entry {
            ToolRailEntry::Divider => {
                // O divisor é uma linha FINA no eixo, centrada no transversal — a mesma lei nos
                // dois eixos, com `w` e `h` trocados.
                let len = Spacing::Xl2.px();
                match axis {
                    RailAxis::Vertical => Rect::new(
                        rect.x + (rect.w - len) * 0.5,
                        along + DIVIDER_GAP_PX,
                        len,
                        1.0,
                    ),
                    RailAxis::Horizontal => Rect::new(
                        along + DIVIDER_GAP_PX,
                        rect.y + (rect.h - len) * 0.5,
                        1.0,
                        len,
                    ),
                }
            }
            // Na COLUNA todo pulldown tem a largura do mais largo ([`column_width_px`]). ⚠️ Ela
            // VIAJA medida: re-derivá-la como `rect.w − recuos` perdia um ULP e `Selected` saía
            // `Select…` (o texto media `42,700977` num orçamento de `42,700974`).
            ToolRailEntry::Compound { .. } if axis == RailAxis::Vertical => {
                Rect::new(cross, along, widest_pulldown(rail, chip_px), chip_px)
            }
            _ => match axis {
                RailAxis::Vertical => Rect::new(cross, along, chip_px, chip_px),
                RailAxis::Horizontal => Rect::new(along, cross, advance, chip_px),
            },
        };
        out.push(EntrySlot {
            index,
            id: entry.node_id(),
            rect: r,
        });
        along += advance;
    }
    out
}
