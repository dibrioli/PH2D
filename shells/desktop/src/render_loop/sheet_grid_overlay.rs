//! **AS LINHAS DA GRELHA** sobre a folha aberta — irmão do [`super::sheet_overlay`], que decora a
//! folha-OBJETO. Este decora a grelha de UMA sprite.
//!
//! Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam ou terminam»*.
//!
//! # ⚠️ As células fantasma sozinhas não respondem à pergunta
//!
//! O [`super::sim_extract_sheet`] põe a arte de todas as células no ecrã, e isso mostra a tira
//! inteira — mas **não mostra onde ela é cortada**. Numa folha cuja arte encosta de célula a
//! célula (um ciclo de caminhada, uma explosão), a imagem aberta lê-se como um desenho contínuo, e
//! o artista continua sem saber se o `hframes` está certo. *A pergunta é sobre os CORTES, e um
//! corte só se vê se for desenhado.*
//!
//! # Em pixels de TELA, e a razão é a mesma da faixa da folha
//!
//! Uma linha em metros engrossa ao aproximar e desaparece ao afastar. Estas linhas são uma
//! **legenda** — dizem *«o corte é aqui»* —, e a frase tem de ser igualmente legível em qualquer
//! zoom.
//!
//! # ⛔ O que ele NÃO desenha, e porquê
//!
//! **Números de célula.** Numa folha 8×8 seriam 64 rótulos por cima da arte que o artista está a
//! tentar ver — e a pergunta *«qual está no ecrã?»* já tem duas respostas melhores: a célula viva
//! é a única a 100% de opacidade, e ela leva um contorno de acento.

use ph2d_render::Sprite;
use ph2d_sprite_screen::sheet_lattice::lattice;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke, VectorScene};

/// Espessura de uma linha de corte, em **pixels de tela**.
const LINE_PX: f64 = 1.0;

/// Espessura do contorno da célula VIVA, em pixels de tela.
///
/// ⚠️ Mais grossa que as outras de propósito: ela responde a uma pergunta diferente (*«qual está no
/// ecrã?»*), e uma diferença de cor sozinha desaparece sobre arte colorida.
const LIVE_PX: f64 = 2.0;

/// Desenha a grelha da sprite cuja folha está aberta.
///
/// `px_per_world` é a escala do afim da câmara — é ela que traz as constantes de TELA acima para o
/// espaço em que a cena é montada, como no [`super::sheet_overlay`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    sim: &ph2d_ecs::SimWorld,
    entity: ph2d_ecs::Entity,
    // A folha deste sprite está **desdobrada** (uma ferramenta pré-visualiza-o)? Ver [`lattice`].
    unfolded: bool,
    pixels_per_meter: f32,
    cam: Affine,
    px_per_world: f64,
    theme: Theme,
    scene: &mut VectorScene,
) {
    if px_per_world <= 0.0 {
        return;
    }
    let Some(spr) = sim.world().get::<Sprite>(entity) else {
        return;
    };
    // ⚠️ Sem grelha não há cortes que desenhar — a sprite mostra a textura inteira.
    let Some(grid) = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied() else {
        return;
    };
    let Some(l) = lattice(spr, grid, pixels_per_meter, unfolded) else {
        return;
    };
    let (hf, vf) = (grid.hframes.max(1), grid.vframes.max(1));
    let (sx, sy) = (l.cell_w, l.cell_h);
    // A pose de MUNDO, pela mesma porta que o gizmo usa — uma resposta só a *«onde isto está»*.
    let Some(wt) = ph2d_ecs::world_transform(sim.world(), entity) else {
        return;
    };
    let xf = cam
        * Affine::translate((f64::from(wt.translation.x), f64::from(wt.translation.y)))
        * Affine::rotate(f64::from(wt.rotation))
        * Affine::scale_non_uniform(f64::from(wt.scale.x), f64::from(wt.scale.y));

    let (x0, y0, w, h) = (l.x0, l.y0, l.w, l.h);
    let (cx, cy) = (l.live_cx, l.live_cy);

    // A espessura volta ao espaço local, e a escala do OBJETO entra na conta: uma sprite a 2×
    // engrossaria a linha ao dobro se só se dividisse pelo zoom da câmara.
    //
    // ⚠️ **O MAIOR dos dois eixos**, sob escala não-uniforme: um traço tem uma espessura só, e
    // dividir pelo menor engrossaria a linha em vez de a manter na medida pedida. *Entre errar
    // para mais fino e para mais grosso, uma legenda erra para mais fino.*
    let obj_scale = f64::from(wt.scale.x.abs())
        .max(f64::from(wt.scale.y.abs()))
        .max(f64::EPSILON);
    let line_w = LINE_PX / (px_per_world * obj_scale);
    let live_w = LIVE_PX / (px_per_world * obj_scale);
    let grid = resolve(ColorToken::Border, theme);

    // ⚠️ **UM caminho para todas as linhas**, e não um por linha: o custo do Vello é por objeto de
    // desenho, e uma folha 8×8 são 18 traços que cabem num só (a lei do `paint_batch`).
    let mut path = BezPath::new();
    for c in 0..=hf {
        let x = x0 + f64::from(c) * sx;
        path.move_to((x, y0));
        path.line_to((x, y0 - h));
    }
    for r in 0..=vf {
        let y = y0 - f64::from(r) * sy;
        path.move_to((x0, y));
        path.line_to((x0 + w, y));
    }
    stroke(scene, &path, xf, grid, line_w);

    // A célula VIVA — a única que está de facto no documento.
    let mut live_path = BezPath::new();
    let (lx0, ly0) = (cx - 0.5 * sx, cy + 0.5 * sy);
    live_path.move_to((lx0, ly0));
    live_path.line_to((lx0 + sx, ly0));
    live_path.line_to((lx0 + sx, ly0 - sy));
    live_path.line_to((lx0, ly0 - sy));
    live_path.close_path();
    stroke(
        scene,
        &live_path,
        xf,
        resolve(ColorToken::Accent, theme),
        live_w.max(line_w),
    );
}

/// Traço, pela mesma porta do irmão [`super::sheet_overlay`].
fn stroke(scene: &mut VectorScene, path: &BezPath, xf: Affine, color: Color, width: f64) {
    scene.inner_mut().stroke(
        &Stroke::new(width.max(f64::EPSILON)),
        xf,
        &Brush::Solid(color),
        None,
        path,
    );
}

fn resolve(token: ColorToken, theme: Theme) -> Color {
    ph2d_editor_core::paint::token_to_vello(token.resolve(theme))
}

#[cfg(test)]
#[path = "sheet_grid_overlay_tests.rs"]
mod tests;
