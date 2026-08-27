//! **O PREENCHIMENTO COM PADRÃO** (plano 33, W2) — a tradução do modo, e a chamada da porta.
//!
//! # Porque a tradução mora DESTE lado
//!
//! O [`PatternMode`] vive na folha `ph2d-vec-pattern`, que é ZERO-dep de propósito (ela é
//! partilhada com a `ph2d-vec-scene`, que é pura e não conhece `vello`/`kurbo`). O `Extend` é do
//! `peniko`. ⇒ a seta de um para o outro tem de nascer na crate que alcança os dois, e essa é esta.
//!
//! # E porque isto é tão pequeno
//!
//! Porque a repetição é **do amostrador**: o Vello empacota `x_extend`/`y_extend` no `sample_alpha`
//! e o `fine.wgsl` honra-os. Não há laço de cópias, não há camada de clip e não há rasterização —
//! uma forma com padrão encoda **um** caminho, exactamente como uma cor chapada.

use ph2d_vec_pattern::PatternMode;
use ph2d_vector::{Affine, BezPath, Extend, Fill, ImageQuality, StableImage, VectorScene};

/// O modo do documento -> o `Extend` do amostrador. **Injectiva**, e há gate a exigi-lo: dois modos
/// no mesmo `Extend` seriam duas opções no painel a fazer a mesma coisa.
#[must_use]
pub fn extend_of(mode: PatternMode) -> Extend {
    match mode {
        PatternMode::Tile => Extend::Repeat,
        PatternMode::Mirror => Extend::Reflect,
        PatternMode::Clamp => Extend::Pad,
    }
}

/// Preenche `bp` com o ladrilho `tile`, repetido segundo `mode`.
///
/// - `transform` — o `câmara * Transform_da_entidade` que o resto do desenho já usa;
/// - `placement` — o afim **pixels do ladrilho -> espaço das âncoras**, saído do
///   [`ph2d_vec_pattern::placement`].
///
/// ⭐ O Vello compõe `transform * placement`, e é essa composição que faz o padrão cavalgar a pose
/// da forma sem uma linha de acompanhamento — a mesma lei que o `paint.rs` já escreveu para os
/// gradientes (*world-space, transforma junto com o path*).
///
/// ⚠️⚠️ **Isso inclui ESMAGAR sob escala não-uniforme, e é deliberado.** ⛔ Não é o bug #27: aquele
/// é a CANETA do traço (uma ferramenta, que não muda de feitio porque a forma esticou); um
/// preenchimento está colado à forma, e um gradiente radial já vira elipse hoje. O gate
/// `the_pattern_shears_with_the_shape_unlike_the_pen` mede as duas leis lado a lado.
#[allow(clippy::too_many_arguments)]
pub fn fill_pattern(
    target: &mut VectorScene,
    bp: &BezPath,
    rule: Fill,
    transform: Affine,
    tile: &StableImage,
    placement: [f64; 6],
    mode: PatternMode,
    quality: ImageQuality,
    alpha: f32,
) {
    let e = extend_of(mode);
    target.fill_path_image(
        bp,
        rule,
        transform,
        tile,
        Affine::new(placement),
        e,
        e,
        quality,
        alpha,
    );
}
