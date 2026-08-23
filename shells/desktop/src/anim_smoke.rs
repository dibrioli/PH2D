//! `PH2D_ANIM_SMOKE` — **a §11 Animation a andar** (spec Sprite 08).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_ANIM_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! Uma sprite com uma **tira de 8 células** — cada uma com um quadrado a subir e a mudar de cor,
//! para que o frame que está no ecrã se leia de relance — e **três animações** sobre a mesma
//! tira, que é a tese do modelo (§8.2: *frames são um pool, animações são intervalos nomeados*):
//!
//! | animação | células | direção | o que prova |
//! |---|---|---|---|
//! | `idle` | 0-3 | Ping-Pong | vai e volta sem repetir as pontas, e respira (`hold`) |
//! | `walk` | 0-7 | Forward | a tira inteira, em ciclo |
//! | `attack` | 4-7 | Forward, 1 volta | **pára no ÚLTIMO frame** — a pose final é o resultado |
//!
//! ⚠️ **As três partilham as MESMAS células.** É isto que o modelo do Aseprite compra e que o
//! `SpriteFrames` do Godot (N arrays separados) não tem: `idle` e `walk` sobrepõem-se em 0-3 sem
//! um único pixel duplicado.
//!
//! # O gesto
//!
//! A sprite nasce selecionada e a tocar `walk`. Com a seção **Animation** aberta:
//!
//! 1. A barra **Frame N / 8** anda sozinha, e o desenho muda com ela.
//! 2. Clicar em **`attack`** na lista → ela toca uma vez e **fica na última célula**. Clicar em
//!    `walk` volta ao ciclo.
//! 3. **Speed** a `-1` toca ao contrário; a `0` pausa sem perder o sítio.
//! 4. Clicar em **`idle`** → vai e volta entre as quatro primeiras, com uma pausa no fim da volta.
//! 5. **Direction override** força uma direção sobre a que a animação declara; **Inherit** devolve.
//!
//! ⚠️ **Como saber que está errado:** a barra parada com `Playing` marcado, ou o `attack` a voltar
//! à primeira célula em vez de ficar na última.

use ph2d_core::Vec2;
use ph2d_ecs::{AnimDirection, AnimationTag, SpriteAnimations, SpriteAnimator};

/// Largura de uma célula, em pixels da fonte. Oito delas fazem a tira.
const CELL_PX: u32 = 64;
/// Quantas células a tira tem. ⚠️ É também o `hframes` da sprite — **o pool**.
const CELLS: u32 = 8;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_ANIM_SMOKE").is_some()
}

/// Desenha a tira: em cada célula, um quadrado que sobe e muda de matiz.
///
/// ⚠️ **As duas coisas ao mesmo tempo, de propósito.** Só a cor faria um piscar difícil de contar;
/// só a posição faria oito quadrados iguais. Com as duas, o artista lê *qual* frame está no ecrã
/// sem olhar para o número.
fn strip_pixels() -> Vec<u8> {
    let w = (CELL_PX * CELLS) as usize;
    let h = CELL_PX as usize;
    let mut px = vec![0u8; w * h * 4];
    let side = (CELL_PX / 2) as usize;
    for c in 0..CELLS as usize {
        // A matiz percorre o círculo ao longo da tira; a altura sobe.
        let t = c as f32 / (CELLS - 1) as f32;
        let (r, g, b) = hue(t);
        let top = ((h - side) as f32 * (1.0 - t)) as usize;
        for y in top..(top + side).min(h) {
            for x in 0..side {
                let px_x = c * CELL_PX as usize + (CELL_PX as usize - side) / 2 + x;
                let i = (y * w + px_x) * 4;
                px[i] = r;
                px[i + 1] = g;
                px[i + 2] = b;
                px[i + 3] = 255;
            }
        }
    }
    px
}

/// Matiz em `[0, 1]` → RGB saturado. ⚠️ Conteúdo de CENA, não chrome (o HR-15 fala da UI).
fn hue(t: f32) -> (u8, u8, u8) {
    let h = (t * 6.0).clamp(0.0, 5.999);
    let i = h as u32;
    let f = h - h.floor();
    let q = ((1.0 - f) * 255.0) as u8;
    let p = (f * 255.0) as u8;
    match i {
        0 => (255, p, 0),
        1 => (q, 255, 0),
        2 => (0, 255, p),
        3 => (0, q, 255),
        4 => (p, 0, 255),
        _ => (255, 0, q),
    } // LITERAL-COLOR-OK: conteúdo da cena
}

/// Monta a cena. Devolve os bits da sprite.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<u64> {
    let cell = *next_cell;
    let w = CELL_PX * CELLS;
    let (_, bits) = crate::image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell,
        w,
        CELL_PX,
        strip_pixels(),
        Vec2::ZERO,
        pixels_per_meter,
        atlas_asset_map,
        "Anim Strip",
    )
    .ok()?;
    *next_cell += 1;

    let e = ph2d_ecs::Entity::from_bits(bits);
    // **A GRELHA é o pool**: oito células numa linha. Sem isto a §11 teria uma célula só.
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.hframes = CELLS;
        s.vframes = 1;
        s.frame = 0;
        // A sprite mostra UMA célula, então ela é quadrada no mundo.
        s.size = [s.size[0] / CELLS as f32, s.size[1]];
    }

    // As três animações, pela porta que impõe os caps — nunca `lib.0.push`.
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        frame_ms: 140,
        direction: AnimDirection::PingPong,
        hold_ms: 300,
        ..AnimationTag::new("idle", 0, 3)
    })
    .ok()?;
    lib.insert(AnimationTag {
        frame_ms: 90,
        ..AnimationTag::new("walk", 0, CELLS - 1)
    })
    .ok()?;
    lib.insert(AnimationTag {
        frame_ms: 110,
        repeat: Some(1),
        ..AnimationTag::new("attack", 4, CELLS - 1)
    })
    .ok()?;

    let mut player = SpriteAnimator::new("walk");
    player.playing = true;
    player.autoplay = true;
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
        ent.insert((lib, player));
    }
    Some(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cena tem de conter **as três formas de ciclo** — é a comparação que ensina o modelo.
    #[test]
    fn the_scene_shows_the_three_kinds_of_cycle() {
        let mut lib = SpriteAnimations::new();
        lib.insert(AnimationTag {
            direction: AnimDirection::PingPong,
            hold_ms: 300,
            ..AnimationTag::new("idle", 0, 3)
        })
        .unwrap();
        lib.insert(AnimationTag::new("walk", 0, CELLS - 1)).unwrap();
        lib.insert(AnimationTag {
            repeat: Some(1),
            ..AnimationTag::new("attack", 4, CELLS - 1)
        })
        .unwrap();

        assert_eq!(lib.len(), 3);
        assert_eq!(lib.get("idle").unwrap().direction, AnimDirection::PingPong);
        assert_eq!(
            lib.get("walk").unwrap().repeat,
            None,
            "a `walk` repete sempre"
        );
        assert_eq!(
            lib.get("attack").unwrap().repeat,
            Some(1),
            "a `attack` e' a que para' no ultimo frame"
        );
        // ⚠️ **`idle` e `walk` SOBREPOEM-SE**, e é essa a tese do modelo: um pool, intervalos
        // nomeados. Se as três fossem disjuntas, a cena não provaria nada que N arrays separados
        // não provassem também.
        let idle = lib.get("idle").unwrap();
        let walk = lib.get("walk").unwrap();
        assert!(
            idle.from >= walk.from && idle.to <= walk.to,
            "as animacoes tem de partilhar celulas"
        );
    }

    /// A tira tem uma célula por frame, e cada uma desenha alguma coisa.
    ///
    /// ⚠️ Uma célula transparente leria-se como um engasgo da animação, e o artista procuraria o
    /// defeito no tocador.
    #[test]
    fn every_cell_of_the_strip_draws_something() {
        let px = strip_pixels();
        let w = (CELL_PX * CELLS) as usize;
        for c in 0..CELLS as usize {
            let opaque = (0..CELL_PX as usize).any(|y| {
                (0..CELL_PX as usize).any(|x| {
                    let i = (y * w + c * CELL_PX as usize + x) * 4;
                    px[i + 3] > 0
                })
            });
            assert!(opaque, "a celula {c} esta' vazia");
        }
    }
}
