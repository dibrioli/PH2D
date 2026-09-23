//! **A ponte da PARALAXE** (plano [24](../../../docs/Components/24_plano_paralaxe.md), W1) — um
//! objecto guarda uma FRACÇÃO do movimento do mundo.
//!
//! Irmã do [`crate::hud_bridge`] e pela mesma lei: o que um motor escreve agora é
//! **pré-visualização**. A pose de quem tem [`ScrollFactor`] é recolocada a cada quadro e passa
//! pelo ledger ⇒ **não entra no ficheiro nem no `Ctrl+Z`**.
//!
//! # A lei, e a referência que ela usa
//!
//! ```text
//! saida = autorada + centro_da_vista · (1 − k)
//! ```
//!
//! Com a câmera na **origem** nada se mexe: o artista põe o fundo onde ele deve estar *quando a
//! câmera está em zero*. ⚠️ **A alternativa era guardar a posição da câmera no instante da
//! autoria** — e ela custa ESTADO, que teria de sobreviver ao ficheiro, ao `Ctrl+Z` e ao
//! rebobinar. A origem do mundo não custa nada, e **é o que o alvo faz** (medido: o declive do
//! `Parallax2D` da Godot é exactamente `1 − scroll_scale`).
//!
//! # ⛔ Sem câmera de jogo, NADA é conduzido — e isso é a lei, não uma guarda
//!
//! A vista é a da **câmera do jogo**, nunca a do editor, pela razão que o `fase_game_camera` já
//! escreve: uma corrida que dependesse de onde o artista rolou o ecrã seria outra corrida em cada
//! máquina. Sem câmera na cena o fundo fica **onde o artista o pôs** — o `settle` devolve-lhe a
//! pose autorada no mesmo quadro.
//!
//! # ⚠️ O caso que decide se esta ponte está CERTA: o artista ARRASTA o fundo
//!
//! O ledger tem uma lei genérica — *«se o `before` deste quadro não é o que o motor deixou no
//! anterior, outra mão escreveu»* — e ela sozinha **não serve aqui**, porque o que a outra mão
//! deixou no mundo é a pose **DESLOCADA**, não a autorada. Tomá-la como autorada faria o fundo
//! saltar `centro · (1 − k)` no quadro seguinte.
//!
//! ⇒ a ponte **reconhece a própria escrita**: ela sabe o que devia lá estar (`autorada` deslocada)
//! e, quando o vivo é outro, **recupera** o autorado subtraindo o mesmo deslocamento. A comparação
//! é exacta de propósito — o valor foi escrito por esta função, bit a bit.
//!
//! # ⛔ Quem tem `UiCanvas` fica de fora
//!
//! Um HUD já é conduzido pelo [`crate::hud_bridge`], e dois motores sobre o mesmo `Transform`
//! escreveriam um por cima do outro. *A raiz de um HUD é a paralaxe com fracção `0`, e quem a
//! implementa é aquela ponte.*

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, ScrollFactor, SimWorld, Transform, UiCanvas};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// **Quantos objectos da cena têm paralaxe** — o número que o painel mostra.
#[must_use]
pub fn parallax_count(sim: &mut SimWorld) -> usize {
    let world = sim.world_mut();
    world.query::<&ScrollFactor>().iter(world).count()
}

/// ⭐ **Um quadro da paralaxe.** Devolve quantos objectos foram conduzidos.
///
/// `centro` é o centro da vista da câmera do jogo, em metros — o que a `fase_game_camera`
/// devolveu, ou `None` quando não há câmera de jogo na cena.
pub fn drive_parallax(
    sim: &mut SimWorld,
    centro: Option<[f32; 2]>,
    drive: &mut PreviewDrive,
) -> usize {
    let antes: Vec<(Entity, Transform, ScrollFactor)> = {
        let world = sim.world_mut();
        world
            .query_filtered::<(Entity, &Transform, &ScrollFactor), bevy_ecs::prelude::Without<UiCanvas>>()
            .iter(world)
            .map(|(e, t, k)| (e, *t, *k))
            .collect()
    };
    let Some(centro) = centro else {
        // ⚠️ Não declarar É a resposta: o `settle` esquece quem não foi declarado, e o valor vivo
        // volta a ser o do documento.
        return 0;
    };
    let mut n = 0;
    for (entity, era, cfg) in antes {
        if cfg.e_neutro() {
            // ⛔ `k = 1` é o objecto do mundo: não se escreve um bit e **não se declara**, senão
            // toda cena com o componente anexado passaria a ter uma entrada viva no ledger.
            continue;
        }
        let autorada = base_autorada(drive, entity, era);
        let p = cfg.desloca([autorada.translation.x, autorada.translation.y], centro);
        let agora = Transform {
            translation: Vec2::new(p[0], p[1]),
            // ⚠️ O resto vem do VIVO e não do autorado: esta ponte só escreve a translação, e
            // roubar a rotação ao autorado apagaria o que outro motor tivesse escrito.
            ..era
        };
        let escreveu = agora != era;
        if escreveu {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
                *t = agora;
            }
        }
        // ⚠️⚠️ **O `before` é o AUTORADO e nunca o vivo.** Esta é a diferença de fundo para a
        // ponte do HUD, que passa `era`: ali a pose é função PURA da vista e o autorado nunca é
        // lido, aqui ele é a ENTRADA da lei — declarar o vivo escreveria a pose deslocada no
        // documento, e no quadro seguinte o fundo saltaria o deslocamento outra vez.
        //
        // ⚠️ E a condição é `escreveu || still_driving`, não `if/else`: o `still_driving` NUNCA
        // cria uma entrada (ver o doc dele), logo com a câmera na origem e nada escrito não nasce
        // condução nenhuma — o fundo está exactamente onde o artista o pôs. Mas se já havia
        // condução, ela tem de ser MANTIDA mesmo num quadro em que a vista não andou: esta ponte é
        // um condutor PERSISTENTE, e para ele *«não mudou»* e *«acabou»* são factos diferentes com
        // a mesma forma.
        if escreveu || drive.still_driving(entity, Driver::ParallaxPose) {
            drive.driven(
                entity,
                Driven::ParallaxPose(autorada),
                Driven::ParallaxPose(agora),
            );
        }
        n += 1;
    }
    n
}

/// **De que pose se parte** — ver o ⚠️ do cabeçalho sobre o arrasto.
///
/// # ⭐⭐⭐ O deslocamento é MEDIDO (`escrito − autorado`), nunca re-derivado da vista
///
/// A 1.ª redacção comparava o vivo com `desloca(memo, centro)` — o que esta função ESCREVERIA
/// agora — e recuperava o autorado subtraindo esse mesmo número. ⛔ **Está errado e falha mudo:**
/// o que está no mundo foi escrito com a vista do quadro ANTERIOR, logo com a câmera a andar as
/// duas nunca coincidem, toda leitura cai no ramo do arrasto, e o «autorado recuperado» anda com
/// a câmera na direcção oposta. Medido: o declive lia **`0,000`** para `k = 0` (o fundo parado no
/// ecrã, que é o caso que a paralaxe existe para produzir) com seis dos oito gates VERDES.
///
/// ⇒ o deslocamento que ESTA ponte aplicou é `last_written − authored`, e o ledger tem os dois.
/// Ele não depende de vista nenhuma, logo a recuperação é exacta mesmo entre duas vistas
/// diferentes.
///
/// ⚠️ **Sem ramo, e isso é a lei e não economia:** *«nada mexeu»* é o caso geral com o vivo igual
/// ao escrito, e a subtracção devolve o memo ao bit. Um `if era == escrito` seria a segunda
/// resposta à mesma pergunta — e a que envelhece é sempre a escrita à mão.
fn base_autorada(drive: &PreviewDrive, entity: Entity, era: Transform) -> Transform {
    let bits = entity.to_bits();
    let (Some(Driven::ParallaxPose(memo)), Some(Driven::ParallaxPose(escrito))) = (
        drive.authored(bits, Driver::ParallaxPose),
        drive.last_written(bits, Driver::ParallaxPose),
    ) else {
        // 1.º quadro deste objecto: o que está no mundo É o autorado.
        return era;
    };
    Transform {
        translation: Vec2::new(
            era.translation.x - (escrito.translation.x - memo.translation.x),
            era.translation.y - (escrito.translation.y - memo.translation.y),
        ),
        ..era
    }
}

#[cfg(test)]
#[path = "parallax_bridge_tests.rs"]
mod tests;
