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
//! | `idle` | 0-3 | Ping-Pong | vai e volta sem repetir as pontas, respira (`hold`), **hesita numa célula** (§8.12) e **grita a cada volta** (§8.10) |
//! | `walk` | 0-7 | Forward | a tira inteira, em ciclo — **uniforme e CALADA**, que é o default |
//! | `attack` | 4-7 | Forward, 1 volta | **pára no ÚLTIMO frame** — e **anuncia o fim**, uma vez |
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
//!    ⚠️ **Repare que ela HESITA na 3.ª célula** (460 ms contra 80 das outras): é a *antecipação*
//!    que um artista desenha à mão, e o caso que um `frame_ms` único não sabia exprimir — uma
//!    animação importada saía com o tempo **achatado** (spec §8.12). Abaixo do campo `Frame ms`
//!    aparece a linha *«this animation has per-frame timing»*; na `walk` ela não aparece.
//!    ⚠️ E a cada volta sai um **sinal** (`idle_cycle`) como um aviso na tela; o `attack` manda um
//!    (`attack_done`) ao acabar, e a `walk` — a que toca por omissão — **não diz nada**. O silêncio
//!    é o default (spec §8.10).
//! 5. **Direction override** força uma direção sobre a que a animação declara; **Inherit** devolve.
//! 6. **Rewind** devolve a imagem à primeira célula da animação escolhida.
//! 7. Depois de o `attack` acabar, **Playing** volta a tocá-lo do princípio — num clique só.
//! 8. **Arrastar a barra `Frame N / 8`** move a célula debaixo do dedo — e **pausa** a reprodução.
//! 9. Na seção **Sprite Sheet**, marcar **«Show sheet on canvas»**: a tira inteira abre-se ao lado
//!    da célula viva, esmaecida, com as **linhas dos cortes** por cima e a célula que está no ecrã
//!    contornada. É como se vê se o `H Frames` corta onde a arte espera.
//!
//! 10. Com a sprite selecionada, entre no **Painter**: a folha **abre-se inteira**, cada quadro no
//!     lugar dele e no tamanho certo — e uma **célula extra, acima dela, toca a animação** enquanto
//!     se pinta, **mesmo com o transporte pausado**. ⚠️ Antes disto a tira saía esmagada 8:1 dentro
//!     de uma célula (report do Enio, com foto), e o ponteiro tinha o mesmo esmagamento: as duas
//!     contas eram a mesma, e ambas erradas.
//! 11. Com **Show sheet on canvas** marcado, a **caixa de seleção envolve a folha inteira** — não
//!     uma célula no meio de oito. Arrastar uma alça escala a folha toda, porque as células saem do
//!     tamanho do sprite.
//! 12. Com **Show sheet on canvas** marcado **e** o Painter aberto, as **linhas caem em cima dos
//!     cortes** da arte. ⚠️ Elas caíam desalinhadas (2.º report, com foto): a folha pintada
//!     centra-se no pivô e a grelha dispunha-se à volta da célula viva — duas âncoras, e as linhas
//!     seguiam a errada.
//!
//! ⚠️ **Os passos 2, 6 e 7 são a auditoria de 2026-08-23** ([doc 21](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)):
//! até ela, o passo 2 deixava a sprite muda (escolher outra não retomava a que se esgotara), o
//! *Rewind* repunha contadores sem mexer na imagem, e a caixa precisava de dois cliques.
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

    let lib = demo_library()?;

    let mut player = SpriteAnimator::new("walk");
    player.playing = true;
    player.autoplay = true;
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
        ent.insert((lib, player));
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
