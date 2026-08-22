//! `PH2D_SLICE_SMOKE` — **9-slice, lado a lado com o que ele conserta.**
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_SLICE_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! Duas caixas do MESMO desenho — uma moldura de cantos redondos de 64×64 — esticadas ao mesmo
//! tamanho grande. A da **esquerda** é um sprite normal: os cantos redondos viram elipses e a
//! borda engorda. A da **direita** tem 9-slice: os cantos ficam redondos, a borda mantém a
//! espessura, e só o miolo estica.
//!
//! ⚠️ **A comparação É o smoke.** Uma caixa sozinha com 9-slice ligado parece só «uma caixa»; o
//! que a feature faz só é visível contra o que acontece sem ela. Por isso as duas nascem juntas,
//! do mesmo pixel, no mesmo tamanho.
//!
//! A da direita nasce **selecionada**: é ela que abre a seção **9-Slice** no Inspector, onde as
//! bordas, o modo e a grelha 3×3 de modos por-região se mexem ao vivo.

use ph2d_core::Vec2;

/// Lado da textura de origem, em pixels.
const SRC_PX: u32 = 64;
/// Raio do canto redondo, em pixels — **é também a borda do 9-slice**: a borda tem de conter o
/// canto inteiro, senão o canto é cortado ao meio e estica na mesma.
const RADIUS_PX: f32 = 14.0;
/// Espessura do traço da moldura, em pixels.
const STROKE_PX: f32 = 3.0;
/// Tamanho a que as duas caixas são esticadas, em metros.
const TARGET: [f32; 2] = [4.0, 2.0];

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_SLICE_SMOKE").is_some()
}

/// Distância assinada de um ponto ao contorno de um retângulo de cantos redondos centrado na
/// origem, com meio-lado `h` e raio `r`. Negativa dentro.
fn rounded_rect_sdf(px: f32, py: f32, h: f32, r: f32) -> f32 {
    let qx = px.abs() - (h - r);
    let qy = py.abs() - (h - r);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - r
}

/// Os pixels da moldura: traço claro, miolo escuro, fora transparente.
fn frame_pixels() -> Vec<u8> {
    let n = SRC_PX as usize;
    let mut px = vec![0u8; n * n * 4];
    let half = SRC_PX as f32 * 0.5;
    for y in 0..n {
        for x in 0..n {
            // Centro do texel (a lei do pixel-centre: subtrai meio pixel do canto).
            let fx = x as f32 + 0.5 - half;
            let fy = y as f32 + 0.5 - half;
            let d = rounded_rect_sdf(fx, fy, half, RADIUS_PX);
            let i = (y * n + x) * 4;
            let rgba: [u8; 4] = if d > 0.0 {
                [0, 0, 0, 0] // LITERAL-COLOR-OK: fora da moldura
            } else if d > -STROKE_PX {
                [235, 170, 60, 255] // LITERAL-COLOR-OK: traço da moldura (âmbar)
            } else {
                [32, 38, 56, 255] // LITERAL-COLOR-OK: miolo (azul-escuro)
            };
            px[i..i + 4].copy_from_slice(&rgba);
        }
    }
    px
}

/// Cria as duas caixas. Devolve os bits da que fica selecionada (a fatiada).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<u64> {
    let pixels = frame_pixels();
    let cell = *next_cell;
    let asset_id = asset_db.insert_image_rgba8(SRC_PX, SRC_PX, pixels.clone());
    atlas_asset_map.insert(cell, asset_id);
    let fetch = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        asset_db
            .get(aid)?
            .image_rgba8()
            .map(|(_, _, p)| p.into_owned())
    };
    if renderer
        .insert_atlas_sprite_with_regrow(cell, SRC_PX, SRC_PX, &pixels, fetch)
        .is_err()
    {
        atlas_asset_map.remove(&cell);
        return None;
    }
    *next_cell += 1;

    // As duas, do mesmo pixel e do mesmo tamanho. A diferença é SÓ o componente.
    let (_, plain) = crate::image_import::spawn_sprite(
        sim,
        crate::image_import::PackedSource::Atlas { cell_idx: cell },
        Vec2::new(-2.4, 0.0),
        TARGET,
        "Panel (plain)",
    );
    let (_, sliced) = crate::image_import::spawn_sprite(
        sim,
        crate::image_import::PackedSource::Atlas { cell_idx: cell },
        Vec2::new(2.4, 0.0),
        TARGET,
        "Panel (9-slice)",
    );
    let _ = plain;

    // ⚠️ A borda é o RAIO do canto: mais estreita corta o canto ao meio e ele volta a esticar.
    let e = ph2d_ecs::Entity::from_bits(sliced);
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
        ent.insert(ph2d_ecs::SliceNine {
            draw_mode: ph2d_ecs::SliceDrawMode::Sliced,
            borders: [RADIUS_PX; 4],
            ..ph2d_ecs::SliceNine::INERT
        });
    }
    Some(sliced)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A textura contém as três zonas que o 9-slice precisa de distinguir: fora (transparente),
    /// traço e miolo. ⚠️ Um fixture só com duas delas não conteria o fenómeno — sem o canto
    /// redondo não há nada que a esticadela estrague, e o smoke não provaria nada.
    #[test]
    fn the_frame_has_a_transparent_outside_a_stroke_and_a_fill() {
        let px = frame_pixels();
        let n = SRC_PX as usize;
        let at = |x: usize, y: usize| -> [u8; 4] {
            let i = (y * n + x) * 4;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };
        // O canto do quadrado está FORA do retângulo redondo.
        assert_eq!(at(0, 0)[3], 0, "o canto tinha de ser transparente");
        // O meio da aresta de cima está no traço.
        assert_eq!(at(n / 2, 1)[3], 255, "o traço de cima nao esta opaco");
        assert_ne!(
            at(n / 2, 1),
            at(n / 2, n / 2),
            "traço e miolo tem de diferir"
        );
        // O centro é o miolo.
        assert_eq!(at(n / 2, n / 2)[3], 255);
    }

    /// ⚠️ **A borda tem de conter o canto inteiro.** Se `borders < RADIUS`, a fatia do canto
    /// corta o arco ao meio e a metade de dentro estica — exatamente o defeito que o smoke
    /// existe para mostrar curado.
    #[test]
    fn the_authored_border_covers_the_whole_corner() {
        assert!(
            RADIUS_PX <= SRC_PX as f32 * 0.5,
            "um raio maior que meio lado nao e' um canto, e' um circulo"
        );
        // O componente que o smoke anexa usa o raio como borda.
        assert_eq!(RADIUS_PX, 14.0);
    }
}
