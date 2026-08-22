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
//!
//! # As três cenas, e porque são três
//!
//! | `=n` | O que ela mostra |
//! |---|---|
//! | `1` | **Os CANTOS** — sprite normal contra 9-slice, ao mesmo tamanho. |
//! | `2` | **A EMENDA contra a borda** — ladrilho cortado contra `Whole` + espelho (paridade). |
//! | `3` | **`Sliced` contra `Tiled`** — esticar contra repetir. |
//!
//! ⚠️ **A cena 3 nasceu de uma auditoria, não de uma feature nova** (2026-08-22). O Enio reportou
//! *«tanto faz estar em Sliced ou Tiled»*, e a medição inocentou o motor: os dois modos produzem
//! geometria diferente (gate `sliced_and_tiled_are_not_the_same_drawing`). Quem não continha o
//! fenómeno eram as **cenas 1 e 2** — as faixas das duas texturas são de cor UNIFORME ao longo do
//! eixo que ladrilha, e esticar uma faixa lisa é indistinguível de a repetir. *Um smoke só mostra
//! o que a sua textura contém* — a lei dos fixtures, aplicada ao sítio onde o dono do produto
//! aprende a ferramenta.

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

/// Borda da cena 2, em pixels — e o lado de cada faixa da fonte.
const PARITY_BORDER_PX: u32 = 16;
/// A cena 2 é uma barra larga e baixa: **seis** ladrilhos em X (par — é a paridade que ela
/// estuda) e **um** em Y. Com `pixels_per_meter = 100`, um ladrilho do miolo mede 0,32 m.
const PARITY_TARGET: [f32; 2] = [0.32 + 0.32 * 6.0, 0.32 + 0.32];

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_SLICE_SMOKE").is_some()
}

/// Barras da cena 3, em pixels — a estrutura que torna «esticar» distinguível de «repetir».
const STRIPE_PX: u32 = 8;
/// A cena 3 estica ~4,7× em X sobre a faixa central, o bastante para a diferença gritar.
const MODES_TARGET: [f32; 2] = [2.24, 0.64];

/// Qual cena. `1` (o default) é a moldura; `2` é a emenda contra a borda; `3` é Sliced vs Tiled.
fn scene() -> u32 {
    std::env::var("PH2D_SLICE_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(1)
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

/// Os pixels do estudo de PARIDADE.
///
/// ⚠️ **As duas bordas laterais têm cores DIFERENTES de propósito** — verde à esquerda, roxo à
/// direita — e o miolo é um degradê horizontal azul→vermelho. Sem isso não se consegue ver *de
/// que ponta da fonte* veio cada pedaço, e o fenómeno que esta cena mede é exatamente esse: com
/// um número PAR de ladrilhos espelhados, a faixa fecha invertida e a borda que combina com ela é
/// a do lado OPOSTO — a direita passa a ser a esquerda ao espelho, e fica **verde**.
///
/// *Um fixture que só prova o que contém:* com as duas bordas da mesma cor esta cena passaria
/// verde sobre a geometria errada.
fn parity_pixels() -> Vec<u8> {
    let n = SRC_PX as usize;
    let b = PARITY_BORDER_PX as usize;
    let mut px = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let rgba: [u8; 4] = if x < b {
                [80, 200, 110, 255] // LITERAL-COLOR-OK: borda ESQUERDA (verde)
            } else if x >= n - b {
                [160, 90, 210, 255] // LITERAL-COLOR-OK: borda DIREITA (roxo)
            } else if y < b || y >= n - b {
                [110, 116, 130, 255] // LITERAL-COLOR-OK: bordas de cima/baixo (cinza neutro)
            } else {
                // Miolo: degradê horizontal, para a emenda entre ladrilhos ter direção.
                let t = (x - b) as f32 / (n - 2 * b - 1) as f32;
                let mix = |a: f32, c: f32| (a + (c - a) * t) as u8;
                [mix(50.0, 230.0), mix(90.0, 80.0), mix(220.0, 60.0), 255] // LITERAL-COLOR-OK
            };
            let i = (y * n + x) * 4;
            px[i..i + 4].copy_from_slice(&rgba);
        }
    }
    px
}

/// **Cena 2 — a emenda contra a borda, e a sua cura.** Duas barras empilhadas, do mesmo pixel e
/// do mesmo tamanho, com o bordo direito alinhado para a comparação ser directa:
///
/// - **em cima:** `Continuous` + `Repeat` — o último ladrilho fica cortado a meio e encosta na
///   borda roxa numa coluna qualquer do degradê. É a emenda que o smoke do Enio marcou.
/// - **em baixo:** `Whole` + miolo `Mirror` — seis ladrilhos inteiros, e a borda direita passa a
///   ser a **esquerda espelhada** (fica verde) porque seis é par. Zero emenda, à custa de o
///   bordo direito trocar de identidade — e é essa troca que o Enio tem de julgar.
fn spawn_parity(sim: &mut ph2d_ecs::SimWorld, cell: u32) -> Option<u64> {
    let mk = |sim: &mut ph2d_ecs::SimWorld, y: f32, name: &str| -> u64 {
        crate::image_import::spawn_sprite(
            sim,
            crate::image_import::PackedSource::Atlas { cell_idx: cell },
            Vec2::new(0.0, y),
            PARITY_TARGET,
            name,
        )
        .1
    };
    let cut = mk(sim, 0.6, "Bar (cut tile)");
    let whole = mk(sim, -0.6, "Bar (whole + mirror)");
    let borders = [PARITY_BORDER_PX as f32; 4];
    let base = ph2d_ecs::SliceNine {
        draw_mode: ph2d_ecs::SliceDrawMode::Tiled,
        borders,
        ..ph2d_ecs::SliceNine::INERT
    };
    let author = |sim: &mut ph2d_ecs::SimWorld, bits: u64, s: ph2d_ecs::SliceNine| {
        if let Ok(mut e) = sim
            .world_mut()
            .get_entity_mut(ph2d_ecs::Entity::from_bits(bits))
        {
            e.insert(s);
        }
    };
    author(
        sim,
        cut,
        ph2d_ecs::SliceNine {
            centre_tile_mode: ph2d_ecs::TileRegionMode::Repeat,
            tile_mode: ph2d_ecs::SliceTileMode::Continuous,
            ..base
        },
    );
    author(
        sim,
        whole,
        ph2d_ecs::SliceNine {
            centre_tile_mode: ph2d_ecs::TileRegionMode::Mirror,
            tile_mode: ph2d_ecs::SliceTileMode::Whole,
            ..base
        },
    );
    Some(whole)
}

/// Os pixels da cena 3 — **a textura que distingue esticar de repetir**.
///
/// ⚠️ **Toda faixa que ladrilha tem ESTRUTURA ao longo do eixo em que ladrilha**, e é essa a
/// única coisa que esta função faz de diferente das outras duas: barras verticais no miolo e nas
/// faixas de cima e de baixo. Esticar uma faixa lisa e repeti-la dão a mesma imagem — foi por
/// isso que as cenas 1 e 2 não mostravam o `Draw Mode` (auditoria 2026-08-22). Aqui, esticar
/// alarga as barras e repetir multiplica-as: a diferença conta-se com o dedo.
fn modes_pixels() -> Vec<u8> {
    let n = SRC_PX as usize;
    let b = PARITY_BORDER_PX as usize;
    let stripe = STRIPE_PX as usize;
    let mut px = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let corner = (x < b || x >= n - b) && (y < b || y >= n - b);
            let bar = ((x - x.min(b)) / stripe).is_multiple_of(2);
            let rgba: [u8; 4] = if corner {
                [250, 200, 70, 255] // LITERAL-COLOR-OK: canto (âmbar) — o que fica FIXO
            } else if x < b || x >= n - b {
                [120, 128, 145, 255] // LITERAL-COLOR-OK: faixas laterais (cinza)
            } else if y < b || y >= n - b {
                // Faixas de cima/baixo: barras, para se ver o que elas fazem em X.
                if bar {
                    [90, 100, 120, 255] // LITERAL-COLOR-OK
                } else {
                    [150, 160, 180, 255] // LITERAL-COLOR-OK
                }
            } else if bar {
                [40, 70, 190, 255] // LITERAL-COLOR-OK: miolo, barra escura
            } else {
                [225, 90, 70, 255] // LITERAL-COLOR-OK: miolo, barra clara
            };
            let i = (y * n + x) * 4;
            px[i..i + 4].copy_from_slice(&rgba);
        }
    }
    px
}

/// **Cena 3 — `Sliced` contra `Tiled`.** Duas barras empilhadas, do mesmo pixel e do mesmo
/// tamanho, e a ÚNICA diferença entre elas é o `Draw Mode`:
///
/// - **em cima, `Sliced`:** as barras do miolo ESTICAM — poucas e largas.
/// - **em baixo, `Tiled`:** as barras REPETEM no tamanho original — muitas e finas.
///
/// Os cantos âmbar ficam idênticos nas duas, que é a metade que o 9-slice sempre garantiu.
fn spawn_modes(sim: &mut ph2d_ecs::SimWorld, cell: u32) -> Option<u64> {
    let mk = |sim: &mut ph2d_ecs::SimWorld, y: f32, name: &str| -> u64 {
        crate::image_import::spawn_sprite(
            sim,
            crate::image_import::PackedSource::Atlas { cell_idx: cell },
            Vec2::new(0.0, y),
            MODES_TARGET,
            name,
        )
        .1
    };
    let stretched = mk(sim, 0.6, "Bar (Sliced - stretches)");
    let tiled = mk(sim, -0.6, "Bar (Tiled - repeats)");
    for (bits, draw_mode) in [
        (stretched, ph2d_ecs::SliceDrawMode::Sliced),
        (tiled, ph2d_ecs::SliceDrawMode::Tiled),
    ] {
        if let Ok(mut e) = sim
            .world_mut()
            .get_entity_mut(ph2d_ecs::Entity::from_bits(bits))
        {
            e.insert(ph2d_ecs::SliceNine {
                draw_mode,
                borders: [PARITY_BORDER_PX as f32; 4],
                ..ph2d_ecs::SliceNine::INERT
            });
        }
    }
    Some(tiled)
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
    let which = scene();
    let pixels = match which {
        2 => parity_pixels(),
        3 => modes_pixels(),
        _ => frame_pixels(),
    };
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
    match which {
        2 => return spawn_parity(sim, cell),
        3 => return spawn_modes(sim, cell),
        _ => {}
    }

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

    /// ⚠️ **O fixture da paridade só prova o que contém.** As duas bordas laterais têm de ser
    /// **diferentes uma da outra** e diferentes do miolo: é a única coisa que torna visível de
    /// que ponta da fonte veio o pedaço que encosta na borda direita. Com as duas da mesma cor,
    /// a cena passaria verde por cima da geometria errada e não se veria nada.
    #[test]
    fn the_parity_fixture_can_tell_the_two_side_borders_apart() {
        let px = parity_pixels();
        let n = SRC_PX as usize;
        let b = PARITY_BORDER_PX as usize;
        let at = |x: usize, y: usize| -> [u8; 4] {
            let i = (y * n + x) * 4;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };
        let mid = n / 2;
        let (left, right) = (at(b / 2, mid), at(n - b / 2, mid));
        assert_ne!(left, right, "as duas bordas laterais tem de diferir");
        assert_ne!(
            left,
            at(mid, mid),
            "a borda esquerda confunde-se com o miolo"
        );
        // O miolo tem direção: a sua ponta esquerda e a direita não são a mesma cor, senão
        // espelhar e repetir dariam a mesma imagem e a cena nao mediria nada.
        assert_ne!(
            at(b, mid),
            at(n - b - 1, mid),
            "o degrade do miolo e' chato — nao se ve qual ladrilho esta invertido"
        );
    }

    /// ⚠️ **A cena 2 precisa de um número PAR de ladrilhos** — é a paridade par que dispara a
    /// troca de borda. Com um número ímpar ela desenharia o caso que já funcionava, e o smoke
    /// mostraria a ausência do defeito em vez da presença da cura.
    #[test]
    fn the_parity_scene_asks_for_an_even_tile_count() {
        // Um ladrilho do miolo, em metros, a 100 px/m (o default do projeto).
        let tile_m = (SRC_PX - 2 * PARITY_BORDER_PX) as f32 / 100.0;
        let edge_m = PARITY_BORDER_PX as f32 / 100.0;
        let tiles = (PARITY_TARGET[0] - 2.0 * edge_m) / tile_m;
        assert!(
            (tiles - tiles.round()).abs() < 1e-4,
            "o alvo nao cai num numero inteiro de ladrilhos: {tiles}"
        );
        assert_eq!(
            (tiles.round() as i32) % 2,
            0,
            "{tiles} ladrilhos e' IMPAR — a cena deixaria de mostrar a troca de borda"
        );
        assert!(tiles >= 2.0, "com menos de dois ladrilhos nao ha' espelho");
    }

    /// ⚠️ **A cena 3 só mede o que a sua textura contém — e o que ela tem de conter é ESTRUTURA
    /// ao longo do eixo que ladrilha.**
    ///
    /// Este é o teste que faltava às cenas 1 e 2, e é a razão de a auditoria de 2026-08-22 ter
    /// existido: as faixas das duas eram de cor uniforme em X, e esticar uma faixa lisa desenha
    /// exatamente o mesmo que a repetir. O `Draw Mode` funcionava e o smoke não o mostrava.
    #[test]
    fn the_modes_fixture_has_structure_along_the_axis_that_tiles() {
        let px = modes_pixels();
        let n = SRC_PX as usize;
        let b = PARITY_BORDER_PX as usize;
        let at = |x: usize, y: usize| -> [u8; 4] {
            let i = (y * n + x) * 4;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };
        let mid_y = n / 2;
        // A faixa CENTRAL alterna em X — senao esticar e repetir dao a mesma imagem.
        let colours: std::collections::BTreeSet<[u8; 4]> =
            (b..n - b).map(|x| at(x, mid_y)).collect();
        assert!(
            colours.len() >= 2,
            "o miolo e' liso em X: a cena 3 nao consegue mostrar Sliced contra Tiled"
        );
        // E a faixa de CIMA tambem, porque ela ladrilha no mesmo eixo.
        let top: std::collections::BTreeSet<[u8; 4]> = (b..n - b).map(|x| at(x, b / 2)).collect();
        assert!(top.len() >= 2, "a faixa de cima e' lisa em X");
        // O CANTO e' de uma cor propria: e' a metade que fica igual nos dois modos, e sem ela
        // nao se ve que o 9-slice preservou alguma coisa.
        let corner = at(b / 2, b / 2);
        assert_ne!(corner, at(mid_y, mid_y), "o canto confunde-se com o miolo");
        assert_ne!(
            corner,
            at(b / 2, mid_y),
            "o canto confunde-se com a lateral"
        );
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
