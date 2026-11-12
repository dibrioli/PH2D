//! A pegada SEM o grão escrita na MESMA visita do kernel (Bug #84, a rota da mistura `Pegada::Mascara::sem_grao`).

use super::{DirtyRect, PARALLEL_MIN_AREA, ShapeInput, stamp_dab_inner_com};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

/// [`super::stamp_dab_textured_masked_po`] (sem Shape, sem máscara do traço) que escreve, na MESMA visita, a pegada SEM o
/// grão em `liso` (RGBA do tamanho do canvas): ao bit o que [`super::stamp_dab_textured_masked`] com a textura desligada,
/// sem dente e sem pó escreveria no `liso` (a rota da mistura, `Pegada::Mascara::sem_grao`). Devolve as caixas do
/// carimbo com grão e da pegada lisa.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked_po_com_liso(
    buf: &mut [u8],
    liso: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
) -> (Option<DirtyRect>, Option<DirtyRect>) {
    if liso.len() < buf.len() {
        return (None, None);
    }
    stamp_dab_inner_com::<u8>(
        buf,
        Some(liso),
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        None,
        None,
        None,
        None,
        po,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
        None,
    )
}

/// [`stamp_dab_textured_masked_po_com_liso`] sobre o VAZIO, numa [`super::PegadaLocal`] do tamanho da caixa do dab: os
/// mesmos bytes, numa banda só (o paralelismo é entre os dabs do lote). `None` = o dab não toca a tela.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn pegada_textured_masked_po_com_liso(
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
    com_liso: bool,
) -> Option<(super::PegadaLocal, Option<DirtyRect>, Option<DirtyRect>)> {
    let mut saida = None;
    let mut liso_vazio: [u8; 0] = [];
    let (r, r2) = stamp_dab_inner_com::<u8>(
        &mut [],
        com_liso.then_some(&mut liso_vazio[..]),
        width,
        height,
        center,
        spec,
        coverage,
        false,
        tex,
        image,
        None,
        None,
        None,
        None,
        po,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
        Some(&mut saida),
    );
    saida.map(|p| (p, r, r2))
}

/// [`super::stamp_dab_textured_masked_po`] (com o Shape, sem máscara do traço nem força) sobre o VAZIO, numa
/// [`super::PegadaLocal`] do tamanho da caixa do dab, sem a pegada lisa: os mesmos bytes que ela deixa num buffer da
/// tela a zeros (a rota da mistura com o Shape, missão 4K onda 3). `None` = o dab não toca a tela.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn pegada_local(
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
) -> Option<(super::PegadaLocal, Option<DirtyRect>)> {
    let mut saida = None;
    let (r, _) = stamp_dab_inner_com::<u8>(
        &mut [],
        None,
        width,
        height,
        center,
        spec,
        coverage,
        false,
        tex,
        image,
        shape,
        None,
        None,
        None,
        po,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
        Some(&mut saida),
    );
    saida.map(|p| (p, r))
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub(super) fn stamp_dab_inner<M: crate::Tecto>(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: Option<crate::RampPaint<'_>>,
    mask: Option<&mut [M]>,
    forca: Option<&mut [u16]>,
    po: Option<crate::texture::PoDoPapel<'_>>,
    dab_rotation: [f32; 2],
    min_area: usize,
    capsula: Option<crate::capsula::Capsula>,
) -> Option<DirtyRect> {
    stamp_dab_inner_com(
        buf,
        None,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        ramp,
        mask,
        forca,
        po,
        dab_rotation,
        min_area,
        capsula,
        None,
    )
    .0
}

/// [`super::parallel_band_stamp_masked`] com um segundo RGBA do tamanho do canvas, `liso`, fatiado nas MESMAS linhas
/// (o `stride` é o mesmo): a pegada SEM o grão que a rota da mistura lê ([`super::stamp_band`]). Devolve
/// `(tocou o buf, tocou o liso)`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn parallel_band_stamp_com_liso<M: Send, F>(
    buf: &mut [u8],
    liso: Option<&mut [u8]>,
    mask: Option<&mut [M]>,
    forca: Option<&mut [u16]>,
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stride: usize,
    min_area: usize,
    stamp: F,
) -> (bool, bool)
where
    F: Fn(&mut [u8], Option<&mut [u8]>, Option<&mut [M]>, Option<&mut [u16]>, i64) -> (bool, bool)
        + Sync,
{
    super::bands::band_split(
        buf, liso, mask, forca, y0, y1, x0, x1, stride, min_area, stamp,
    )
}

/// O texel `i` da pegada lisa: a silhueta `w` à cobertura do dab, com a cor e o blend do pincel — o que o
/// [`super::stamp_band`] escreveria com a textura desligada e sem dente (`g = 1`). Devolve se escreveu.
#[inline]
pub(super) fn escreve(
    ctx: &super::DabCtx,
    l: &mut [u8],
    i: usize,
    w: f32,
    blend: crate::BrushBlend,
    branco_no_vazio: bool,
) -> bool {
    let prev = [
        f32::from(l[i]) / 255.0,
        f32::from(l[i + 1]) / 255.0,
        f32::from(l[i + 2]) / 255.0,
        f32::from(l[i + 3]) / 255.0,
    ];
    let a = if ctx.preserve_alpha {
        w * ctx.coverage * prev[3]
    } else {
        w * ctx.coverage
    };
    if a <= 0.0 {
        return false;
    }
    let out = if branco_no_vazio {
        super::bands::branco_no_vazio(a)
    } else {
        crate::blend_vazio::blend_over_pigment_modo(
            blend,
            prev,
            ctx.spec.color,
            a,
            ctx.spec.pigmento(),
            ctx.spec.blend_sobre_vazio,
        )
    };
    for (k, v) in out.iter().enumerate() {
        l[i + k] = super::encode(*v);
    }
    true
}
