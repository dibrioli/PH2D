//! **PEGAR O CORPO NO PLAY** (W-Grab) — o gesto que faltava para a cena ser
//! cutucável enquanto ela roda.
//!
//! História e porquê: `git log` deste ficheiro.

use ph2d_ecs::Entity;
use ph2d_physics_ecs::{InteractionSettings, InteractionTool, PhysicsBridge};

/// **Quantos ticks o flash de uma explosão vive.** Um estouro é instantâneo, então
/// sem marca decaída o único vestígio visível são corpos que se moveram — a mesma
/// razão (e a mesma ordem de grandeza) do flash de contato do W-TickContacts.
pub const BLAST_FLASH_TICKS: u32 = 12;

/// **A porta única do press, para a MÃO.** `true` = a mão pegou, e o chamador NÃO
/// deve abrir arrasto de gizmo.
///
/// As condições 1 e 2 são decididas pelo chamador (é ele que vê o relógio e o
/// transporte) e chegam como argumentos, para que a decisão inteira seja
/// testável sem janela; a 3 é do wrapper.
///
/// ⚠️ **Recusa quando a ferramenta em mãos não é a mão** — as outras duas não
/// precisam de corpo e são interceptadas ANTES do pick, por
/// [`poke_at`]. A pergunta é feita por `InteractionTool::needs_a_body`, a porta
/// única: enumerar aqui *quais* ferramentas seguram é como a quarta nasceria de
/// fora da regra.
pub fn take_hold(
    physics: &mut PhysicsBridge,
    settings: &InteractionSettings,
    entity: Entity,
    world_point: [f32; 2],
    playing: bool,
    simulating: bool,
) -> bool {
    if !playing || !simulating || !settings.tool.needs_a_body() {
        return false;
    }
    physics.grab_with(entity, world_point, settings.hold_spec())
}

/// **A porta única do press, para as ferramentas de PONTO** (explosão e atração).
///
/// Devolve `Some(corpos_atingidos)` quando o gesto foi consumido — o chamador
/// então NÃO deve continuar para o pick de canvas nem para o gizmo. `None`
/// significa *"esta não é a minha vez"*, e aí tudo o de sempre acontece.
///
/// As mesmas três condições da mão, menos a que ela não compartilha: o relógio tem
/// de estar ANDANDO e a física ARMADA (sem passo a força não move nada e o gesto
/// seria morto), mas **não é preciso haver nada sob o cursor** — é essa a diferença
/// que faz destas duas uma família própria.
pub fn poke_at(
    physics: &mut PhysicsBridge,
    // ⚠️ **A cena entra aqui porque a EXPLOSÃO precisa de saber quem é player**
    // (`W-Launch`): o `PhysicsWorld::explode` pula todo corpo que não é
    // `Dynamic`, e quem responde *"a lei escreve a pose deste?"* é o
    // `pose_owner`, que lê o ECS.
    sim: &ph2d_ecs::SimWorld,
    settings: &InteractionSettings,
    world_point: [f32; 2],
    playing: bool,
    simulating: bool,
) -> Option<usize> {
    if !playing || !simulating || settings.tool.needs_a_body() {
        return None;
    }
    let s = settings.clamped();
    match s.tool {
        // Já filtrado acima (`needs_a_body`); o braço existe para o `match` ser
        // exaustivo sem um `_ =>` que engoliria uma ferramenta nova em silêncio.
        InteractionTool::Hand => None,
        InteractionTool::Explode => {
            Some(physics.explode(sim, world_point, s.blast_radius, s.blast_impulse))
        }
        InteractionTool::Attract => {
            physics.attract(&s, world_point);
            Some(0)
        }
    }
}

/// **A mão segue o cursor.** No-op sem mão em voo, então roda em todo Move junto dos
/// outros `advance_*`.
///
/// ⚠️ **Não devolve «autorou»**, ao contrário dos irmãos da pose: o evento que trouxe o
/// cursor até aqui já decidiu isso, e marcá-lo aqui armaria o diff de undo por um
/// movimento de mouse.
///
/// ⚠️ Era um `impl App` (W2/L2 Fase B). Recebe a ponte e o ponto **já em mundo** — a
/// conversão `ecrã → mundo` fica com a câmera, que é da shell.
pub fn advance_body_grab(physics: &mut PhysicsBridge, world: [f32; 2]) {
    // `is_poking` and not `is_grabbing`: the attract field follows the cursor
    // by the same rule, and asking the narrower question is how the second
    // sustained tool would have been born outside it.
    if !physics.is_poking() {
        return;
    }
    physics.move_grab(world);
    physics.move_attract(world);
}

/// **Soltar.** Chamado no TOPO do `on_mouse_input` para todo release de botão, antes
/// de qualquer `return` — e as duas metades dessa frase são load-bearing:
///
/// - **no topo**, porque aquele handler tem muitos early-returns e uma mão que
///   sobrevive ao release fica colada no cursor para sempre;
/// - **todo botão**, porque uma mão não é um modificador: soltar qualquer botão
///   encerra o gesto, e enumerar quais soltam é a forma que apodrece.
///
/// No-op sem mão, que é o caso de quase todo release do app.
pub fn release_body_grab(physics: &mut PhysicsBridge) {
    physics.release_grab();
    // The attract field ends on the same release, and for the same reason: a
    // sustained tool that survives the button is a tool stuck to the cursor forever.
    physics.stop_attract();
}

/// **The blast flash ages.** Called once per frame from the physics dispatch
/// phase; drops the mark when it runs out.
///
/// Its own channel rather than something derived from the world, exactly like
/// `ContactFlash`: a blast leaves NO state behind (it is an impulse), so a mark
/// that read the world would have nothing to read
/// ([[feedback_a_transient_event_marker_is_its_own_channel]]).
///
/// ⚠️ A free function over the FIELD, not a method on `App`: the caller already
/// holds `self.gfx` borrowed at that point in the frame, and the borrow checker
/// only lets disjoint fields through.
pub fn age_blast_flash(flash: &mut Option<([f32; 2], f32, u32)>) {
    if let Some((_, _, ticks)) = flash.as_mut() {
        *ticks = ticks.saturating_sub(1);
        if *ticks == 0 {
            *flash = None;
        }
    }
}

#[cfg(test)]
#[path = "body_grab_tests.rs"]
mod tests;
