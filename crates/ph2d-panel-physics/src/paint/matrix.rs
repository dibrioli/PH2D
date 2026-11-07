//! The collision-layer matrix — a triangular grid of toggles.
//!
//! One cell is one **fact about a pair**, not two facts about two directions:
//! rapier ANDs both ways, so a half-set pair means "no collision", and
//! `LayerMatrix::set` writes both halves. That is why only the lower triangle
//! is drawn — the mirror cell would be a second control for the same checkbox.
//!
//! ⚠️ The cells are registered in a LOOP, which
//! `architecture_panel_wiring_parity` cannot see. The seam test that clicks
//! every cell is therefore not redundant with the arch gates — it is the only
//! thing standing between this widget and being painted but dead.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_physics_ecs::LayerMatrix;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};

/// Cell edge. Small enough that 8 columns plus the row label fit the dock, big
/// enough to hit — the grid is 8 × this wide, and the dock is ~300 px.
const CELL_PX: f32 = 22.0; // LITERAL-PX-OK: panel grid metric (matrix cell edge)

/// O passo da face clássica: a célula cheia enquanto as colunas cabem; depois encolhem todas, para
/// nenhuma ficar fora de alcance (esta face não tem barra).
fn passo_das_colunas(largura: f32, colunas: usize, cheio: f32) -> f32 {
    (largura / colunas.max(1) as f32).min(cheio)
}

/// Paint the triangular matrix. Returns the y it ended at.
pub(super) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    matrix: LayerMatrix,
    x: f32,
    y_in: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let label_w = CELL_PX; // the row's own layer number, same width as a cell
    let font = TypeToken::Sm.px();

    let mut y = y_in;
    // ⭐ As células são pintáveis por arrasto (`toggle_paint`): cada uma declara o que MOSTRA.
    ph2d_editor_core::interaction::toggle_paint::limpa_grupo(crate::ids::PHYSICS_LAYER_PAINT_GROUP);

    // ⭐ O losango da matriz (canal MUNDO `CAMADAS`, F41): UM, no canto das legendas — a matriz é
    // um controlo e as 36 células gravam-se juntas.
    let canto = Rect::new(x, y, label_w, font + gap);
    let cells = &crate::ids::PHYSICS_LAYER_CELL;
    if let Some(alvo) =
        ph2d_editor_core::widget::key_mark::paint_decorator_for(ctx.scene, theme, canto, cells)
    {
        ctx.host.hit_index_mut().register(alvo, canto);
    }

    // Column headers, offset by the label gutter.
    let visiveis = crate::state::visible_layers();
    let step = passo_das_colunas(
        ctx.slot.w - x + ctx.slot.x - label_w - gap,
        visiveis.len(),
        CELL_PX + gap,
    );
    // O vão encolhe com o passo: nas 32, o que sobra é da célula, que é o que a mão aponta.
    let cell_px = step - gap.min(step / 4.0);
    // Com as células pequenas as legendas não cabem todas: uma em cada tantas, sem se atropelarem.
    let legenda_a_cada = ((font + gap) / step).ceil().max(1.0) as usize;
    for (pos, &j) in visiveis.iter().enumerate().step_by(legenda_a_cada) {
        let cx = x + label_w + gap + pos as f32 * step;
        paint_text(
            ctx.text_system,
            ctx.scene,
            &j.to_string(),
            cx + (cell_px - font) * 0.5,
            y,
            font,
            CELL_PX,
            resolve(ColorToken::Text2, theme),
        );
    }
    y += font + gap;

    for (linha, &i) in visiveis.iter().enumerate() {
        // Row label.
        let destacada = crate::state::destaque() == Some(i);
        if linha % legenda_a_cada == 0 || destacada {
            paint_text(
                ctx.text_system,
                ctx.scene,
                &i.to_string(),
                x + (label_w - font) * 0.5,
                y + (cell_px - font) * 0.5,
                font,
                label_w,
                resolve(
                    if destacada {
                        ColorToken::Accent
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                ),
            );
        }
        for (pos, &j) in visiveis.iter().take(linha + 1).enumerate() {
            let cell = Rect::new(x + label_w + gap + pos as f32 * step, y, cell_px, cell_px);
            let on = matrix.collides(i, j);
            // On = accent fill, off = the panel's recessed surface. A checkmark
            // glyph at this size would be mud; the fill IS the state.
            let token = if on {
                ColorToken::Accent
            } else {
                ColorToken::Bg3
            };
            fill_rounded_rect(
                ctx.scene,
                cell,
                ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
                resolve(token, theme),
            );
            // The diagonal is a layer against ITSELF, and turning it off is a
            // real and useful thing (a layer whose members ignore each other),
            // so it is a cell like any other — just outlined, so the eye can
            // find the axis of symmetry.
            if i == j {
                // FRAME-RAW-OK: a diagonal da matriz de camadas, contornada para o olho achar o eixo de simetria
                ph2d_editor_core::paint::stroke_rounded_rect(
                    ctx.scene,
                    cell,
                    ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
                    StrokeToken::Default.px(),
                    resolve(ColorToken::Text2, theme),
                );
            }
            let id = crate::ids::PHYSICS_LAYER_CELL[crate::ids::physics_layer_cell_index(i, j)];
            ctx.host.hit_index_mut().register(id, cell);
            ph2d_editor_core::interaction::toggle_paint::celula(
                id,
                crate::ids::PHYSICS_LAYER_PAINT_GROUP,
                on,
                cell,
            );
        }
        y += step;
    }
    // ⭐ **A matriz é UM controlo** — o mapa de colisão entre camadas, um VALOR (cada célula liga ou
    //    desliga um par), como a grelha de `32` bits do Inspector (`bitmask_grid32`), que já se
    //    declara assim. Sem isto o censo das entradas lia as `36` células como `36` comandos
    //    (medido 2026-09-24: `36` dos `49` do painel).
    ph2d_editor_core::widget::composto::grupo(crate::ids::PHYSICS_LAYER_CELL.iter().copied());
    y
}
