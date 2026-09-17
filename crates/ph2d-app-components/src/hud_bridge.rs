//! **A ponte do HUD** (TOP-20 #20) — a raiz de um placar cola-se à vista da câmera do jogo.
//!
//! Irmã do [`crate::script_bridge`] e pela mesma lei: o que um motor escreve agora é
//! **pré-visualização**. A pose de um [`UiCanvas`] é reescrita a cada quadro (é isso que o cola à
//! vista) e passa pelo ledger do `ph2d-preview-drive` ⇒ **não entra no ficheiro nem no `Ctrl+Z`**.
//!
//! # ⛔ Sem câmera de jogo, NADA é conduzido — e isso é a lei, não uma guarda
//!
//! A vista é a da **câmera do jogo**, nunca a do editor: uma corrida que dependesse de onde o
//! artista rolou o ecrã seria outra corrida em cada máquina (é a razão escrita no
//! `fase_game_camera`, e o `DestroyOutside` do #12 já a herdou). Sem câmera na cena não há vista,
//! e então o canvas fica **onde o artista o pôs** — o `settle` do ledger devolve-lhe a pose
//! autorada no mesmo quadro, e o painel diz porquê.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform, UiCanvas};
use ph2d_hud::{Canvas, place};

/// ⭐ **A vista re-exportada**, para quem compõe não precisar de declarar a folha só para
/// nomear um rectângulo. *Uma segunda dependência por um nome de tipo é ruído no manifesto.*
pub use ph2d_hud::{Gesto, View, clique};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// **Quantos canvas há na cena** — o número que o painel mostra quando quer dizer *«isto não está
/// a ser conduzido por ninguém»*.
#[must_use]
pub fn canvas_count(sim: &mut SimWorld) -> usize {
    let world = sim.world_mut();
    world.query::<&UiCanvas>().iter(world).count()
}

/// ⭐ **Um quadro do HUD.** Devolve quantas raízes foram conduzidas.
///
/// ⚠️ **A rotação e o *skew* autorados SOBREVIVEM** — só a translação e a escala são derivadas.
/// Um HUD inclinado é uma decisão de desenho que o artista pode tomar, e apagá-la por arrasto
/// tornaria o controlo inalcançável; a posição e o tamanho, esses, são a razão de este passe
/// existir.
pub fn drive_canvases(sim: &mut SimWorld, vista: Option<View>, drive: &mut PreviewDrive) -> usize {
    let antes: Vec<(Entity, Transform, UiCanvas)> = {
        let world = sim.world_mut();
        world
            .query::<(Entity, &Transform, &UiCanvas)>()
            .iter(world)
            .map(|(e, t, c)| (e, *t, *c))
            .collect()
    };
    let Some(vista) = vista else {
        // ⚠️ Não declarar É a resposta: o `settle` do fim do quadro esquece quem não foi
        // declarado, e o valor vivo volta a ser o do documento.
        return 0;
    };
    let mut n = 0;
    for (entity, era, cfg) in antes {
        let Some(caixa) = Canvas::new(cfg.ref_w, cfg.ref_h, cfg.fit) else {
            // Uma caixa impossível (lado zero ou não-finito) não conduz nada — ver o doc do
            // `Canvas::new`. O painel é que a acusa.
            continue;
        };
        let p = place(&caixa, vista);
        let agora = Transform {
            translation: Vec2::new(p.translate[0], p.translate[1]),
            scale: Vec2::new(p.scale[0], p.scale[1]),
            ..era
        };
        if agora != era {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
                *t = agora;
            }
            drive.driven(entity, Driven::CanvasPose(era), Driven::CanvasPose(agora));
        } else if drive.still_driving(entity, Driver::CanvasPose) {
            // ⚠️ A linha do meio da tabela do ledger: o motor continua a conduzir, e o facto de a
            // pose não ter mudado neste quadro não pode fazê-la voltar a ser documento.
            drive.driven(entity, Driven::CanvasPose(agora), Driven::CanvasPose(agora));
        }
        n += 1;
    }
    n
}

#[cfg(test)]
#[path = "hud_bridge_tests.rs"]
mod tests;
