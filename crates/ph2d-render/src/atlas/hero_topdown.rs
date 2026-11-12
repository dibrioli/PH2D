//! ⭐ **O boneco de fábrica VISTO DE CIMA** (ficha GEF 05, cena 24): o mesmo boneco verde de capacete, desenhado
//! de cima para o Top-Down Player — 2 linhas (parado · a andar) × 6 células. O SVG é
//! [`super::fabrica::TOPDOWN_HERO_ART`] (`docs/GameEngineFuncional/ferramentas/arte_svg.py`, `boneco_de_cima`).
//! Olha para a DIREITA (+X), o ângulo 0 do motor: com Facing = Move a imagem gira para onde ele anda.

use super::HeroTag;

/// Chave reservada da folha.
pub const TOPDOWN_HERO_KEY: u32 = u32::MAX - 11;
/// Lado de uma célula, em texels (o do boneco de lado: 1 m no mundo).
pub const TOPDOWN_HERO_CELL_PX: u32 = super::HERO_SHEET_CELL_PX;
/// Células por linha (a animação mais longa, andar).
pub const TOPDOWN_HERO_COLS: u32 = 6;
/// Linhas.
pub const TOPDOWN_HERO_ROWS: u32 = 2;

const _: () = assert!(TOPDOWN_HERO_KEY > super::FIRST_IMPORT_KEY);

/// A animação parada (a de entrada) e a de andar.
pub const TOPDOWN_HERO_IDLE: &str = "idle";
pub const TOPDOWN_HERO_WALK: &str = "walk";

/// As animações: parado respira e pisca no último quadro; andar alterna os pés.
pub const TOPDOWN_HERO_TAGS: [HeroTag; 2] = [
    HeroTag {
        name: TOPDOWN_HERO_IDLE,
        from: 0,
        to: 3,
        per_frame_ms: &[250, 250, 250, 150],
        once: false,
    },
    HeroTag {
        name: TOPDOWN_HERO_WALK,
        from: 6,
        to: 11,
        per_frame_ms: &[90; 6],
        once: false,
    },
];

/// ⭐ `e` passa a mostrar o boneco visto de cima: a folha, as animações e o tocador em `idle`.
///
/// # Errors
/// Uma animação da tabela recusada pela biblioteca.
pub fn veste_o_heroi_de_cima(
    world: &mut ph2d_ecs::World,
    e: ph2d_ecs::Entity,
) -> Result<(), String> {
    let mut tags = ph2d_ecs::SpriteAnimations::new();
    for t in TOPDOWN_HERO_TAGS {
        let mut tag = ph2d_ecs::AnimationTag::new(t.name, t.from, t.to);
        tag.frame_ms = t.per_frame_ms[0];
        for (i, ms) in t.per_frame_ms.iter().enumerate() {
            tag.set_own_ms(i, *ms);
        }
        tag.repeat = t.once.then_some(1);
        tags.insert(tag).map_err(|e| format!("{e:?}"))?;
    }
    let mut animator = ph2d_ecs::SpriteAnimator::new(TOPDOWN_HERO_IDLE);
    animator.autoplay = true;
    world.entity_mut(e).insert((
        crate::Sprite::atlas(TOPDOWN_HERO_KEY, [1.0, 1.0], [1.0; 4]),
        ph2d_ecs::SpriteGrid {
            hframes: TOPDOWN_HERO_COLS,
            vframes: TOPDOWN_HERO_ROWS,
            frame: 0,
            sheet: 0,
        },
        tags,
        animator,
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::fabrica::{TOPDOWN_HERO_ART, factory_pixels};

    /// Os texels RGBA da célula `c` da folha, linha a linha.
    fn celula(c: u32) -> Vec<u8> {
        let art = TOPDOWN_HERO_ART;
        let [cw, ch] = art.cell_px();
        let (col, row) = (c % art.grid[0], c / art.grid[0]);
        let px = factory_pixels(&art);
        (0..ch)
            .flat_map(|y| (0..cw).map(move |x| (x, y)))
            .flat_map(|(x, y)| {
                let o = (((row * ch + y) * art.px[0] + col * cw + x) * 4) as usize;
                px[o..o + 4].to_vec()
            })
            .collect()
    }

    /// ⭐ A folha casa com a grelha, as tags cabem nela e cada célula que uma tag toca tem desenho; as do andar
    /// mexem de quadro a quadro, e as PUPILAS (a cor `#434A5F`) estão todas na metade da frente (+X) — é o que
    /// diz para onde ele olha quando a imagem gira.
    #[test]
    fn o_heroi_de_cima_olha_para_a_direita_e_anda() {
        let art = TOPDOWN_HERO_ART;
        assert_eq!(art.key, TOPDOWN_HERO_KEY);
        assert_eq!(art.grid, [TOPDOWN_HERO_COLS, TOPDOWN_HERO_ROWS]);
        assert_eq!(art.cell_px(), [TOPDOWN_HERO_CELL_PX; 2]);
        let cw = TOPDOWN_HERO_CELL_PX as usize;
        for t in TOPDOWN_HERO_TAGS {
            assert_eq!(t.per_frame_ms.len() as u32, t.to - t.from + 1, "{}", t.name);
            for c in t.from..=t.to {
                let px = celula(c);
                assert!(
                    px.chunks(4).any(|p| p[3] == 255),
                    "{} célula {c} vazia",
                    t.name
                );
                let (mut frente, mut tras) = (0, 0);
                for (i, p) in px.chunks(4).enumerate() {
                    if p == [0x43, 0x4A, 0x5F, 255] {
                        if i % cw > cw / 2 {
                            frente += 1
                        } else {
                            tras += 1
                        }
                    }
                }
                let olhos_abertos = !(t.name == TOPDOWN_HERO_IDLE && c == t.to);
                assert_eq!(tras, 0, "{} célula {c}: pupila atrás", t.name);
                assert_eq!(
                    frente > 0,
                    olhos_abertos,
                    "{} célula {c}: pupilas {frente}",
                    t.name
                );
            }
        }
        let walk = &TOPDOWN_HERO_TAGS[1];
        for c in walk.from..=walk.to {
            let seguinte = walk.from + (c - walk.from + 1) % (walk.to - walk.from + 1);
            assert!(
                celula(c) != celula(seguinte),
                "o andar parado na célula {c}"
            );
        }
    }
}
