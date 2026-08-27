//! **AUTORAR a lei de um padrão de textura** (plano 33, W5) — a porta única entre a secção
//! *Pattern* do painel e o documento.
//!
//! ⚠️⚠️ **NÃO confundir com o [`crate::pattern_live`]**, que é o *Pattern Along Path* (plano 23).
//!
//! # Uma porta, um passo de undo
//!
//! Todo controlo da secção desagua aqui, e cada mudança é **um** passo de undo — a mesma disciplina
//! do `apply_vec_set_fill_kind`. E o `if` de igualdade no fim é o que impede um passo espúrio quando
//! o slider re-publica o valor que já lá estava (o defeito que fazia todo quadro virar undo).

use ph2d_vec_pattern::{PatternMode, TileKind};
use ph2d_vec_scene::{Paint, PatternFill, PatternSource, VecScene};

/// O que a secção *Pattern* pede ao documento.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TexPatCmd {
    /// Trocar o reticulado (`0` Grid · `1` Brick · `2` Column · `3` Hex).
    Tile(u8),
    /// Trocar a lei de repetição (`0` Tile · `1` Mirror · `2` Clamp).
    Mode(u8),
    /// O desfasamento é `1/n`.
    OffsetDenom(f64),
    /// O lado MAIOR de uma cópia, em unidades de mundo.
    Size(f64),
    /// O vão acrescentado, em unidades de mundo (negativo = sobreposição).
    Gap(f64),
    /// A rotação do padrão, em graus.
    Angle(f64),
    /// Trocar a ARTE, mantendo a lei.
    Source(PatternSource),
}

/// O reticulado que o índice do painel nomeia. ⚠️ Porta única: o painel oferece por índice, e a
/// tradução vive **aqui**, num sítio só.
fn tile_of(i: u8) -> TileKind {
    match i {
        1 => TileKind::BrickRow,
        2 => TileKind::BrickCol,
        3 => TileKind::Hex,
        _ => TileKind::Grid,
    }
}

/// O índice do painel para um reticulado — a gémea de [`tile_of`].
#[must_use]
pub(crate) fn tile_index(k: TileKind) -> u8 {
    match k {
        TileKind::Grid => 0,
        TileKind::BrickRow => 1,
        TileKind::BrickCol => 2,
        TileKind::Hex => 3,
    }
}

fn mode_of(i: u8) -> PatternMode {
    match i {
        1 => PatternMode::Mirror,
        2 => PatternMode::Clamp,
        _ => PatternMode::Tile,
    }
}

/// O índice do painel para uma lei de repetição — a gémea de [`mode_of`].
#[must_use]
pub(crate) fn mode_index(m: PatternMode) -> u8 {
    match m {
        PatternMode::Tile => 0,
        PatternMode::Mirror => 1,
        PatternMode::Clamp => 2,
    }
}

/// O lado MAIOR de uma cópia — o número que o slider *Size* mostra.
///
/// ⭐ **O painel autora UM número e o documento guarda DOIS**, e a diferença é o aspecto da arte:
/// oferecer os dois lados deixaria o artista esmagar a imagem sem querer, e nascer esticado é a
/// primeira coisa que ele leria como *"a ferramenta deformou a minha imagem"*.
#[must_use]
pub(crate) fn longer_side(size: [f64; 2]) -> f64 {
    size[0].max(size[1])
}

/// Reescala o par preservando o aspecto, para que o lado maior meça `longer`.
fn with_longer_side(size: [f64; 2], longer: f64) -> [f64; 2] {
    let cur = longer_side(size);
    // ⚠️ `is_finite()` ANTES da comparação: um `NaN` reprova toda desigualdade e escorregaria pela
    // porta de trás, deixando um `size` de `NaN` que apaga a forma sem erro nenhum.
    if !cur.is_finite() || cur <= 0.0 || !longer.is_finite() || longer <= 0.0 {
        return [longer.max(f64::EPSILON), longer.max(f64::EPSILON)];
    }
    let s = longer / cur;
    [size[0] * s, size[1] * s]
}

/// Aplica `cmd` ao padrão da forma selecionada. No-op silencioso quando não há forma, quando ela
/// não tem padrão, ou quando o valor já era esse.
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    cmd: TexPatCmd,
) {
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(Paint::Pattern(cur)) = scene.path(sel).and_then(|p| p.fill.as_ref()) else {
        return;
    };
    let mut next: PatternFill = (**cur).clone();
    match cmd {
        TexPatCmd::Tile(i) => next.kind = tile_of(i),
        TexPatCmd::Mode(i) => next.mode = mode_of(i),
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        TexPatCmd::OffsetDenom(n) => next.offset_denom = n.clamp(1.0, 255.0).round() as u8,
        TexPatCmd::Size(v) => next.size = with_longer_side(next.size, v),
        TexPatCmd::Gap(v) => next.gap = [v, v],
        TexPatCmd::Angle(deg) => next.angle = deg.to_radians(),
        TexPatCmd::Source(s) => next.source = s,
    }
    if next == **cur {
        return;
    }
    let pre = scene.clone();
    if let Some(path) = scene.path_mut(sel) {
        path.fill = Some(Paint::Pattern(Box::new(next)));
        history.push_undo(pre);
    }
}

#[cfg(test)]
#[path = "texture_pattern_edit_tests.rs"]
mod tests;
