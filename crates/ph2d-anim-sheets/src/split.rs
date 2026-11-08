//! **Split by rows**: uma animação por fileira da folha que tem desenho
//! ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)). «Tem desenho» é a caixa opaca do
//! Pivot, a mesma lei de transparência.

use ph2d_ecs::{AnimationTag, SpriteAnimFrames, SpriteAnimations, tag_cells};
use ph2d_sprite_edit::pivot::caixa_opaca;

/// As células com desenho de cada fileira ocupada, de cima para baixo. A fileira toda transparente não aparece.
#[must_use]
pub fn drawn_by_row(rgba: &[u8], w: u32, h: u32, hframes: u32, vframes: u32) -> Vec<Vec<u32>> {
    let (hf, vf) = (hframes.max(1), vframes.max(1));
    (0..vf)
        .map(|r| {
            let (y0, y1) = (r * h / vf, (r + 1) * h / vf);
            (0..hf)
                .filter(|c| {
                    let (x0, x1) = (c * w / hf, (c + 1) * w / hf);
                    caixa_opaca(rgba, w, [x0, y0, x1 - x0, y1 - y0]).is_some()
                })
                .map(|c| r * hf + c)
                .collect::<Vec<u32>>()
        })
        .filter(|cells: &Vec<u32>| !cells.is_empty())
        .collect()
}

/// A fileira já tem a sua animação: uma que fica só nela e toca todo o desenho dela. O «loop» da folha inteira não conta.
fn covered(
    lib: &SpriteAnimations,
    frames: Option<&SpriteAnimFrames>,
    drawn: &[u32],
    hf: u32,
    sheet: u8,
    sheets: &[u32],
) -> bool {
    let row = drawn.first().map_or(0, |c| c / hf);
    lib.iter().any(|t| {
        let list = frames.and_then(|f| f.get(&t.name));
        tag_cells(t, list, sheets).is_some_and(|seq| {
            let on: Vec<u32> = seq.iter().filter_map(|f| f.cell_on(sheet)).collect();
            on.len() == seq.len()
                && on.iter().all(|c| c / hf == row)
                && drawn.iter().all(|c| on.contains(c))
        })
    })
}

/// O primeiro `row N` livre a partir de `n`.
fn free_row_name(lib: &SpriteAnimations, n: &mut u32) -> String {
    loop {
        let name = ph2d_i18n::tr_with("anim_editor.row_name", &[("n", n)]);
        *n += 1;
        if lib.get(&name).is_none() {
            return name;
        }
    }
}

/// Junta a `lib` uma animação por fileira ainda sem a sua; o intervalo vai do primeiro ao último desenho da fileira,
/// na folha `sheet` (`hframes` é o dela). `sheets[k]` = células da folha `k`. Devolve as que entraram.
///
/// # Errors
/// A biblioteca recusou uma animação (cheia).
pub fn add_rows(
    lib: &mut SpriteAnimations,
    frames: Option<&SpriteAnimFrames>,
    rows: &[Vec<u32>],
    hframes: u32,
    sheet: u8,
    sheets: &[u32],
) -> Result<usize, ph2d_ecs::AnimTagError> {
    let (mut n, mut added) = (1, 0);
    for drawn in rows {
        if covered(lib, frames, drawn, hframes.max(1), sheet, sheets) {
            continue;
        }
        let (Some(from), Some(to)) = (drawn.first(), drawn.last()) else {
            continue;
        };
        let name = free_row_name(lib, &mut n);
        let mut tag = AnimationTag::new(name, *from, *to);
        tag.sheet = sheet;
        lib.insert(tag)?;
        added += 1;
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::FrameRef;

    /// Split by rows na folha 1: as animações nascem nela, e uma fileira da folha 0 com o mesmo desenho não a cobre.
    #[test]
    fn split_by_rows_na_outra_folha_nasce_nela() {
        let mut lib = SpriteAnimations::new();
        lib.insert(AnimationTag::new("idle", 0, 3)).unwrap();
        let rows = vec![vec![0, 1, 2], vec![6, 7]];
        let sheets = [4, 12];
        assert_eq!(add_rows(&mut lib, None, &rows, 6, 1, &sheets), Ok(2));
        let novas: Vec<_> = lib.iter().filter(|t| t.name != "idle").collect();
        assert_eq!(novas.len(), 2);
        assert!(novas.iter().all(|t| t.sheet == 1), "na folha escolhida");
        assert_eq!((novas[1].from, novas[1].to), (6, 7));
        // Já cobertas: correr outra vez não junta nada.
        assert_eq!(add_rows(&mut lib, None, &rows, 6, 1, &sheets), Ok(0));
    }

    #[test]
    fn uma_lista_de_outra_folha_nao_cobre_a_fileira() {
        let mut lib = SpriteAnimations::new();
        lib.insert(AnimationTag::new("mix", 0, 0)).unwrap();
        let mut frames = SpriteAnimFrames::default();
        frames.set("mix", FrameRef::run(0, [0, 1, 2]));
        let rows = vec![vec![0, 1, 2]];
        assert_eq!(
            add_rows(&mut lib, Some(&frames), &rows, 6, 1, &[4, 12]),
            Ok(1),
            "a lista é da folha 0; a fileira é da folha 1"
        );
        assert_eq!(
            add_rows(&mut lib, Some(&frames), &rows, 6, 0, &[4, 12]),
            Ok(0),
            "na folha 0 a lista já a cobre"
        );
    }
}
