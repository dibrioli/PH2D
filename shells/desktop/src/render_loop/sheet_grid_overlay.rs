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
    let Some(l) = lattice(spr, pixels_per_meter, unfolded) else {
        return;
    };
    let (hf, vf) = (spr.hframes.max(1), spr.vframes.max(1));
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

/// **A GRELHA em metros locais** — o retângulo que a folha aberta ocupa e onde a célula viva cai
/// dentro dele.
///
/// ⚠️ **Extraída para ser testável.** O desenho em si é traço; o que pode estar errado é ISTO — o
/// sinal do `Y`, o pivô, e o espelho. Uma função que devolve números tem gate; uma que empurra
/// caminhos para uma cena tem screenshot.
#[derive(Debug, PartialEq)]
pub(crate) struct Lattice {
    /// Canto superior-esquerdo da folha aberta, em metros locais.
    pub x0: f64,
    pub y0: f64,
    /// Extensão total.
    pub w: f64,
    pub h: f64,
    pub cell_w: f64,
    pub cell_h: f64,
    /// Centro da célula VIVA — onde o quad do sprite de facto está.
    pub live_cx: f64,
    pub live_cy: f64,
}

/// `None` quando não há grelha para desenhar.
///
/// # ⚠️ DUAS disposições, e a diferença é quem desenha a célula viva
///
/// - **Dobrada** (`unfolded == false`, a pré-visualização `Show sheet on canvas`): o quad real do
///   sprite **É** a célula viva, então a folha dispõe-se à volta dela.
/// - **Desdobrada** (`unfolded == true`, uma ferramenta pinta o sprite): há UM quad a cobrir a
///   folha inteira, centrado no pivô — e a célula viva está no *slot* dela, como as outras.
///
/// ⚠️ **Desenhar sempre a primeira desloca as linhas sobre a arte pintada** (report do Enio,
/// 2026-08-23, com foto): `pivô − (lcol + ½)·cw` só coincide com `pivô − hf·cw/2` quando
/// `lcol = hf/2 − ½`, que não é inteiro — logo elas **nunca** coincidem. O desvio é
/// `(lcol + ½ − hf/2)·cw`, e vale meia célula no caso fotografado (8 células, viva na 4);
/// noutro frame é maior. ⚠️ *A 1.ª versão do gate escreveu «meia célula sempre» e sangrou na hora.* *Duas disposições existem porque dois modos existem; o que
/// não pode existir é uma delas a descrever o outro.*
pub(crate) fn lattice(spr: &Sprite, pixels_per_meter: f32, unfolded: bool) -> Option<Lattice> {
    let cells = super::sim_extract_sheet::cell_count(spr)?;
    let (hf, vf) = (spr.hframes.max(1), spr.vframes.max(1));
    let (cell_w, cell_h) = (f64::from(spr.size[0]), f64::from(spr.size[1]));
    if cell_w <= 0.0 || cell_h <= 0.0 {
        return None;
    }
    // O centro da célula VIVA. ⚠️ É o `resolve_anchor`, e não a origem: o shader desenha o quad em
    // `anchor + quad_pos * size`, então é ali que a célula de facto está.
    let live_c = spr.resolve_anchor(pixels_per_meter);
    let (live_cx, live_cy) = (f64::from(live_c[0]), f64::from(live_c[1]));

    // ⚠️ **Sob FLIP, a grelha abre para o outro lado** — o `sim_extract_sheet::ghost` nega o
    // deslocamento, e as linhas têm de acompanhar. Espelhar o ÍNDICE da célula viva dá o mesmo
    // resultado com uma conta só, e deixa o retângulo dela onde ele está (o centro não se move —
    // só o que fica à volta dele).
    let live = spr.frame.min(cells - 1);
    let (mut lcol, mut lrow) = (live % hf, live / hf);
    if spr.flip_x {
        lcol = hf - 1 - lcol;
    }
    if spr.flip_y {
        lrow = vf - 1 - lrow;
    }
    let (w, h) = (f64::from(hf) * cell_w, f64::from(vf) * cell_h);
    let (x0, y0, cx, cy) = if unfolded {
        // A folha centra-se no PIVÔ (é onde o quad desdobrado está), e a célula viva ocupa o slot
        // dela — como as outras.
        let x0 = live_cx - w * 0.5;
        let y0 = live_cy + h * 0.5;
        (
            x0,
            y0,
            x0 + (f64::from(lcol) + 0.5) * cell_w,
            y0 - (f64::from(lrow) + 0.5) * cell_h,
        )
    } else {
        // ⚠️ `+`: a linha 0 está ACIMA da viva quando `lrow > 0`, porque o `V` cresce para baixo e
        // o `Y` do mundo para cima.
        (
            live_cx - (f64::from(lcol) + 0.5) * cell_w,
            live_cy + (f64::from(lrow) + 0.5) * cell_h,
            live_cx,
            live_cy,
        )
    };
    Some(Lattice {
        x0,
        y0,
        w,
        h,
        cell_w,
        cell_h,
        live_cx: cx,
        live_cy: cy,
    })
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
    ph2d_editor::paint::token_to_vello(token.resolve(theme))
}

#[cfg(test)]
#[path = "sheet_grid_overlay_tests.rs"]
mod tests;
