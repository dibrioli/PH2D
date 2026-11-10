//! As entradas do carimbo com a máscara do traço (Accumulate OFF): com força, com pó e com o piso de paralelismo nomeado.

use super::{DirtyRect, PARALLEL_MIN_AREA, ShapeInput, stamp_dab_inner};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

/// As [`super::stamp_dab_textured`], plus the **Accumulate-OFF** per-stroke coverage `mask` (canvas-sized, 1
/// byte/px): with `Some`, each pixel's stroke coverage caps at the dab target (Strength) — overlapping /
/// back-and-forth dabs build toward but never past it ([`crate::stroke_cover`]). `None` ⇒ exactly
/// [`super::stamp_dab_textured`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked(
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
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        mask,
        forca,
        None,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
    )
}

/// [`stamp_dab_textured_masked`] com o PÓ pousado antes do traço ([`crate::texture::PoDoPapel`]): o Height lê o
/// papel com o dente cheio.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked_po(
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
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        mask,
        forca,
        po,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
    )
}

/// [`stamp_dab_textured_masked`] com o **piso de paralelismo explícito** — a porta que NOMEIA o piso.
///
/// ⚠️ Com `min_area = usize::MAX` o kernel roda a **banda ÚNICA** que o caminho do cap rodava antes de
/// 2026-08-04, chamando o MESMO `stamp_band` com os mesmos argumentos. É isso que torna o gate de
/// identidade uma comparação contra **o produto**, e não contra uma segunda implementação escrita para
/// o teste (o precedente do `stamp_plain_dabs_banded_with`).
///
/// ⚠️ **Ela nasceu `#[cfg(test)]` e deixou de ser, e a diferença é ter CHAMADOR:** um `pub` sem chamador
/// é uma segunda resposta esperando alguém chamá-la (a lei do `warp_axis`/`serial_side`), mas a rota em
/// banda do LOTE tem um motivo de produto para pedir `usize::MAX` — **o paralelismo é do lote OU do dab,
/// nunca dos dois**. Aninhar é alcançável e não é hipótese: a 4096² com o pincel máximo (r = 512) uma
/// banda recebe `128 linhas × 1024 px = 131 072`, que **é exatamente** o [`PARALLEL_MIN_AREA`] ⇒ o
/// kernel dividiria de novo, dentro de cada uma das trinta e duas bandas.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked_with(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    dab_rotation: [f32; 2],
    min_area: usize,
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        None,
        None,
        None,
        None,
        mask,
        forca,
        None,
        dab_rotation,
        min_area,
        None,
    )
}
