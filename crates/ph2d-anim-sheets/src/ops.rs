//! **As operações das várias folhas** (plano GEF 26 §3.2–§3.3, F10): a folha nova com o metro por pixel e o Pivot da
//! primeira, e apagar uma folha deixando BURACOS (dono 08/11: nada some sozinho; a limpeza é do artista).

use ph2d_asset::AssetId;
use ph2d_ecs::{ExtraSheet, FrameRef, SpriteAnimFrames, SpriteAnimations, sheet_cells};

/// O quad inteiro: a caixa opaca de uma imagem sem píxeis conhecidos ou toda transparente.
const QUAD_INTEIRO: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// A folha `0` como a lei do §3.2 a lê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstSheet {
    /// O tamanho de UMA célula na tela (`Sprite.size`), m.
    pub size: [f32; 2],
    /// O Pivot: do centro do quad ao ponto de pivô, m locais, `y` para cima (`-Sprite::resolve_anchor`).
    pub pivot: [f32; 2],
    /// `[flip_x, flip_y]`: a caixa vira com a imagem.
    pub flip: [bool; 2],
    /// Os píxeis de uma célula.
    pub cell_px: [u32; 2],
    /// A união opaca das células, fração da célula `[u0, v0, u1, v1]` (`v` para baixo; `pivot::fracao_opaca`).
    pub opaque: Option<[f32; 4]>,
}

/// Uma imagem a carregar como folha.
#[derive(Clone, Copy, Debug)]
pub struct NewSheetImage<'a> {
    pub asset: AssetId,
    /// RGBA8 da imagem inteira, `px[0]` píxeis por linha.
    pub rgba: &'a [u8],
    pub px: [u32; 2],
    /// `[colunas, linhas]`.
    pub grid: [u32; 2],
}

/// ⭐ **A folha nova** (§3.2): o pixel tem o mesmo tamanho na tela que o da primeira, e o Pivot é a MESMA fração da
/// caixa opaca dela — os pés ficam no sítio ao trocar de folha (o contrário do Godot, que centra cada quadro).
#[must_use]
pub fn new_sheet(first: &FirstSheet, img: NewSheetImage<'_>) -> ExtraSheet {
    let grid = [img.grid[0].max(1), img.grid[1].max(1)];
    let cell = [
        (img.px[0] / grid[0]).max(1) as f32,
        (img.px[1] / grid[1]).max(1) as f32,
    ];
    let m_por_px = [
        first.size[0] / first.cell_px[0].max(1) as f32,
        first.size[1] / first.cell_px[1].max(1) as f32,
    ];
    let size = [cell[0] * m_por_px[0], cell[1] * m_por_px[1]];
    let opaque = ph2d_sprite_edit::pivot::fracao_opaca(
        img.rgba,
        img.px[0],
        [0, 0, img.px[0], img.px[1]],
        grid,
    );
    let pivot = carry_pivot(
        first.pivot,
        first.size,
        first.opaque.unwrap_or(QUAD_INTEIRO),
        size,
        opaque.unwrap_or(QUAD_INTEIRO),
        first.flip,
    );
    ExtraSheet {
        asset: img.asset,
        size,
        anchor: [-pivot[0], -pivot[1]],
        hframes: grid[0],
        vframes: grid[1],
    }
}

/// O Pivot `p` (m locais desde o centro de um quad `size`, `y` para cima) como fração da caixa opaca `f`, levado à
/// caixa `g` de um quad `size_k`. A caixa é da IMAGEM (`v` para baixo); o espelho vira-a como no ecrã.
#[must_use]
pub fn carry_pivot(
    p: [f32; 2],
    size: [f32; 2],
    f: [f32; 4],
    size_k: [f32; 2],
    g: [f32; 4],
    flip: [bool; 2],
) -> [f32; 2] {
    let vira = |t: f32, sim: bool| if sim { 1.0 - t } else { t };
    let u = vira(p[0] / size[0].max(f32::EPSILON) + 0.5, flip[0]);
    let v = vira(0.5 - p[1] / size[1].max(f32::EPSILON), flip[1]);
    let fu = (u - f[0]) / (f[2] - f[0]).max(f32::EPSILON);
    let fv = (v - f[1]) / (f[3] - f[1]).max(f32::EPSILON);
    let u = vira(g[0] + fu * (g[2] - g[0]), flip[0]);
    let v = vira(g[1] + fv * (g[3] - g[1]), flip[1]);
    [(u - 0.5) * size_k[0], (0.5 - v) * size_k[1]]
}

/// O quadro `f` depois de apagar a folha `k`: os dela viram buraco, os das folhas seguintes descem uma.
#[must_use]
pub const fn after_deleting(f: FrameRef, k: u8) -> FrameRef {
    match f {
        FrameRef::Cell { sheet, .. } if sheet == k => FrameRef::Empty,
        FrameRef::Cell { sheet, cell } if sheet > k => FrameRef::Cell {
            sheet: sheet - 1,
            cell,
        },
        other => other,
    }
}

/// ⭐ **Apagar a folha `k` nas animações** (F10). Cada quadro dela vira [`FrameRef::Empty`] NO LUGAR; um intervalo na
/// folha `k` vira uma lista de vazios do mesmo comprimento (`sheets[j]` = células da folha `j`, ANTES de apagar);
/// as folhas depois de `k` descem uma. Nenhuma animação sai.
pub fn delete_sheet(
    lib: &mut SpriteAnimations,
    frames: &mut SpriteAnimFrames,
    sheets: &[u32],
    k: u8,
) {
    for t in &mut lib.0 {
        match frames.get(&t.name).map(<[FrameRef]>::to_vec) {
            Some(list) if !list.is_empty() => {
                frames.set(
                    &t.name,
                    list.into_iter().map(|f| after_deleting(f, k)).collect(),
                );
            }
            _ if t.sheet == k => {
                let n = t
                    .resolve(sheet_cells(sheets, k))
                    .map_or(1, |(lo, hi)| hi - lo + 1);
                frames.set(&t.name, vec![FrameRef::Empty; n as usize]);
                t.sheet = 0;
            }
            _ => {}
        }
        if t.sheet > k {
            t.sheet -= 1;
        }
    }
}

/// Quantos quadros vazios a animação `name` tem (`0` = nenhum alerta).
#[must_use]
pub fn empty_count(frames: Option<&SpriteAnimFrames>, name: &str) -> usize {
    frames
        .and_then(|f| f.get(name))
        .map_or(0, |l| l.iter().filter(|f| **f == FrameRef::Empty).count())
}

/// A animação `name` é TODA vazia (só buracos)?
#[must_use]
pub fn all_empty(frames: Option<&SpriteAnimFrames>, name: &str) -> bool {
    frames
        .and_then(|f| f.get(name))
        .is_some_and(|l| !l.is_empty() && l.iter().all(|f| *f == FrameRef::Empty))
}

/// ⭐ **Remove empty frames**: tira os buracos das animações que têm algum quadro de verdade. A toda vazia fica (é o
/// **Delete empty animations** que a apaga): tirar-lhe os buracos devolvia-a ao intervalo, que tocaria outra coisa.
/// Devolve se mudou alguma.
pub fn remove_empty_frames(lib: &SpriteAnimations, frames: &mut SpriteAnimFrames) -> bool {
    let mut mudou = false;
    for t in lib.iter() {
        let Some(list) = frames.get(&t.name) else {
            continue;
        };
        if !list.contains(&FrameRef::Empty) || all_empty(Some(frames), &t.name) {
            continue;
        }
        let cheia: Vec<FrameRef> = list
            .iter()
            .copied()
            .filter(|f| *f != FrameRef::Empty)
            .collect();
        frames.set(&t.name.clone(), cheia);
        mudou = true;
    }
    mudou
}

/// As animações que o **Delete empty animations** apaga: as TODAS vazias, na ordem da lista.
#[must_use]
pub fn empty_animations(lib: &SpriteAnimations, frames: Option<&SpriteAnimFrames>) -> Vec<String> {
    lib.iter()
        .filter(|t| all_empty(frames, &t.name))
        .map(|t| t.name.clone())
        .collect()
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
