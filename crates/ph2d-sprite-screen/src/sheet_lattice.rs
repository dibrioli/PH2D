//! ⭐⭐ **A GRELHA DE UMA FOLHA, em metros locais** — onde as células de uma sprite com grelha caem, e
//! que caixa o gizmo dela envolve.
//!
//! ⚠️ **Mudou-se da shell para esta folha em 2026-09-16** (a `shells/desktop` só encolhe): é
//! geometria PURA — entra a sprite e a grelha, saem números —, e o irmão que a desenha
//! (`render_loop::sheet_grid_overlay`) e o gizmo (`render_loop::snapshots_gizmo`) ficam lá a
//! chamá-la. O `cell_count` e o `unfolded_quad`, que ela usa, já moravam aqui.

use ph2d_render::Sprite;

/// **A GRELHA em metros locais** — o retângulo que a folha aberta ocupa e onde a célula viva cai
/// dentro dele.
///
/// ⚠️ **Extraída para ser testável.** O desenho em si é traço; o que pode estar errado é ISTO — o
/// sinal do `Y`, o pivô, e o espelho. Uma função que devolve números tem gate; uma que empurra
/// caminhos para uma cena tem screenshot.
#[derive(Debug, PartialEq)]
pub struct Lattice {
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
pub fn lattice(
    spr: &Sprite,
    grid: ph2d_ecs::SpriteGrid,
    pixels_per_meter: f32,
    unfolded: bool,
) -> Option<Lattice> {
    let cells = crate::cell_count(grid)?;
    let (hf, vf) = (grid.hframes.max(1), grid.vframes.max(1));
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
    let live = grid.frame.min(cells - 1);
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

/// **A CAIXA DO GIZMO em metros LOCAIS** — `(centro, meia-extensão)`.
///
/// Enio, 2026-08-23: *«o gizmo da sprite deve englobar todas as células»*. Com a folha aberta ela é
/// o [`Lattice`]; sem ela, o quad de uma célula de sempre.
///
/// ⚠️ **A ESCOLHA vive aqui, e não no fio.** Ela estava em `snapshots::build_view` — um closure que
/// precisa de `HeroScreen` + `PresentWorld` + câmara, e **não é alcançável de um teste**: a mutação
/// que a desligava compilava e passava a suíte inteira. Aqui ela tem gate, e o que fica lá são duas
/// linhas que se conferem a olho. *Encolher o resíduo é o que se pode fazer quando o arnês não
/// existe; fingir que ele existe, não.*
///
/// `sheet_open` = a caixa «Show sheet on canvas» está marcada **para este sprite**;
/// `unfolded` = uma ferramenta pré-visualiza-o (a folha centra-se no pivô). Ver [`lattice`].
#[must_use]
pub fn gizmo_box(
    spr: &Sprite,
    grid: Option<ph2d_ecs::SpriteGrid>,
    pixels_per_meter: f32,
    sheet_open: bool,
    unfolded: bool,
) -> ([f32; 2], [f32; 2]) {
    let folded = (
        spr.resolve_anchor(pixels_per_meter),
        [spr.size[0] * 0.5, spr.size[1] * 0.5],
    );
    if !sheet_open {
        return folded;
    }
    match grid.and_then(|g| lattice(spr, g, pixels_per_meter, unfolded)) {
        Some(l) => (
            [(l.x0 + l.w * 0.5) as f32, (l.y0 - l.h * 0.5) as f32],
            [(l.w * 0.5) as f32, (l.h * 0.5) as f32],
        ),
        // Sem grelha não há folha para envolver — e a caixa é a de sempre.
        None => folded,
    }
}

#[cfg(test)]
#[path = "sheet_lattice_tests.rs"]
mod tests;
