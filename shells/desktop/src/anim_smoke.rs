//! `PH2D_ANIM_SMOKE`: a §11 Animation a andar. Gestos e o porquê: `docs/Sprite_projeto/smoke_anim_a_andar.md`.

use ph2d_core::Vec2;
use ph2d_ecs::{AnimDirection, AnimationTag, SpriteAnimations, SpriteAnimator};

/// Largura de uma célula, em pixels da fonte. Oito delas fazem a tira.
const CELL_PX: u32 = 64;
/// Quantas células a tira tem. ⚠️ É também o `hframes` da sprite — **o pool**.
const CELLS: u32 = 8;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_ANIM_SMOKE").is_some()
}

/// Desenha a tira: em cada célula, um quadrado que sobe, muda de matiz e ENCOSTA na borda direita: a célula
/// seguinte não pode mostrá-lo em risco à esquerda (`docs/Sprite_projeto/22`).
///
/// ⚠️ **As duas coisas ao mesmo tempo:** só a cor faria um piscar difícil de contar; só a posição, oito
/// quadrados iguais.
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
                let px_x = c * CELL_PX as usize + CELL_PX as usize - side + x;
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
        // A sprite mostra UMA célula, então ela é quadrada no mundo.
        s.size = [s.size[0] / CELLS as f32, s.size[1]];
    }
    sim.world_mut().entity_mut(e).insert(ph2d_ecs::SpriteGrid {
        hframes: CELLS,
        vframes: 1,
        frame: 0,
        sheet: 0,
    });

    let lib = demo_library()?;

    let mut player = SpriteAnimator::new("walk");
    player.playing = true;
    player.autoplay = true;
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
        ent.insert((lib, player));
        if let Some(m) = ph2d_sfx_scene::marcadores_do_anim_smoke() {
            ent.insert(m);
        }
    }
    Some(bits)
}

/// **AS TRÊS ANIMAÇÕES da cena de smoke** — e cada uma existe para mostrar uma coisa diferente.
///
/// ⚠️ **Uma função, e não um bloco dentro do `spawn_if_enabled`:** aquele pede um `SpriteRenderer`
/// vivo e não é alcançável de um teste, então o gate desta cena reconstruía a biblioteca à mão —
/// e um gate que encena a cena mede a encenação. Com isto, ele corre a **mesma** biblioteca que o
/// Enio vê.
///
/// `None` só se um cap da §11 recusar uma tag, o que seria um bug deste ficheiro.
fn demo_library() -> Option<SpriteAnimations> {
    // Pela porta que impõe os caps — nunca `lib.0.push`.
    let mut lib = SpriteAnimations::new();
    // ⚠️ **A `idle` GRITA a cada volta** (spec §8.10): é o que mostra a contagem — um tique que
    // apanha atraso fecha vários ciclos e sai **um** sinal, com quantos.
    //
    // ⚠️ **E ela tem RITMO PRÓPRIO** (spec §8.12): a 3.ª célula dura quase meio segundo, as outras
    // um sexto disso. É a *antecipação* que um artista desenha à mão — e é o caso que o `frame_ms`
    // único não sabia exprimir, então uma animação importada saía com o tempo **achatado**. Aqui
    // ele vê-se sem importar nada: a sprite hesita numa célula e corre nas outras.
    lib.insert(AnimationTag {
        frame_ms: 140,
        per_frame_ms: vec![80, 80, 460, 80],
        direction: AnimDirection::PingPong,
        hold_ms: 300,
        signal_on_loop: "idle_cycle".into(),
        ..AnimationTag::new("idle", 0, 3)
    })
    .ok()?;
    lib.insert(AnimationTag {
        frame_ms: 90,
        ..AnimationTag::new("walk", 0, CELLS - 1)
    })
    .ok()?;
    // ⚠️ **A `attack` grita ao ACABAR, e a `walk` é CALADA** — as três juntas são a demonstração:
    // a que toca por omissão não diz nada (o default é o silêncio), a de uma volta anuncia o fim
    // uma vez, e a em ciclo anuncia cada volta.
    lib.insert(AnimationTag {
        frame_ms: 110,
        repeat: Some(1),
        signal_on_finish: "attack_done".into(),
        ..AnimationTag::new("attack", 4, CELLS - 1)
    })
    .ok()?;

    Some(lib)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cena tem de conter **as três formas de ciclo** — é a comparação que ensina o modelo.
    ///
    /// ⚠️ **Ele lê a biblioteca REAL** (`demo_library`), e não uma cópia dela: até 2026-08-23 este
    /// gate reconstruía as três tags à mão, então o que ele afirmava era sobre a reconstrução —
    /// mudar a cena não o reprovava. *Um gate que encena a cena mede a encenação.*
    #[test]
    fn the_scene_shows_the_three_kinds_of_cycle() {
        let lib = demo_library().expect("a biblioteca da cena tem de caber nos caps da §11");

        assert_eq!(lib.len(), 3);
        // ⚠️ **E a cena demonstra as DUAS features novas de 2026-08-23**, senão o smoke passa a
        // dizer o que já dizia ontem: a `idle` hesita numa célula (§8.12) e grita a cada volta
        // (§8.10); a `walk`, que é a que toca por omissão, é **uniforme e calada** — é o contraste
        // que ensina que o silêncio e o ritmo liso são o DEFAULT.
        let idle = lib.get("idle").unwrap();
        assert!(
            idle.has_per_frame_timing(),
            "a `idle` tem de HESITAR — e' ela que mostra a duracao por-quadro sem importar nada"
        );
        assert_eq!(idle.signal_on_loop, "idle_cycle");
        let walk = lib.get("walk").unwrap();
        assert!(!walk.has_per_frame_timing() && walk.signal_on_loop.is_empty());
        assert_eq!(lib.get("attack").unwrap().signal_on_finish, "attack_done");
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
