//! **PEGAR O CORPO NO PLAY** (W-Grab) — o gesto que faltava para a cena ser
//! cutucável enquanto ela roda.
//!
//! Antes disto o Play era só de LEITURA. A pose de um corpo dinâmico é escrita
//! pelo `readback` a cada dispatch, então um arrasto de gizmo durante o play
//! escreve o `Transform` e é sobrescrito no MESMO frame: o artista assiste a
//! simulação e não pode empurrar, puxar nem atirar nada. Todo laboratório de
//! física 2D deixa (Algodoo, o testbed do Box2D, o RUBE, o play mode da Unity),
//! e é assim que se testa uma cena.
//!
//! # A LEI: o relógio é o interruptor
//!
//! **Em repouso**, arrastar um corpo AUTORA a pose dele (e com Alt carrega o rig
//! articulado — W-JG). **Tocando**, arrastar um corpo dinâmico é a **MÃO**: uma
//! mola macia do cursor até o ponto que você pegou.
//!
//! O mesmo gesto, dois significados, decididos pelo MESMO predicado que a
//! condição 2 do `joint_rig_drag` já usa (`!playhead.is_playing()`) — do outro
//! lado. As duas metades são irmãs de propósito: em repouso a pose é do
//! DOCUMENTO e a mão não faria sentido; tocando a pose é do SOLVER e a autoria
//! não faria (o readback a sobrescreve).
//!
//! # As três condições
//!
//! 1. **O relógio está ANDANDO.** Ver acima.
//! 2. **A física está ARMADA** (o toggle `Physics` do transporte, W4b). Sem ela
//!    a ponte faz `hold` e **não dá passo nenhum**, então a mola não puxaria
//!    nada: seria um gesto morto, com a assinatura clássica de *"a ferramenta
//!    não funciona"* em vez de *"a ferramenta não está aqui"*.
//! 3. **O corpo é DINÂMICO** — respondido pelo wrapper (`grab_body` recusa), não
//!    por uma cópia da regra aqui: um joint não move corpo estático nem
//!    kinematic (massa infinita, o fato que a W-BakeJoint mediu). A recusa
//!    devolve `false` e o chamador deixa o caminho de sempre acontecer, o que é
//!    o que mantém *selecionar* e *arrastar* um cenário estático funcionando
//!    durante o play.
//!
//! ⚠️ **A TRAVA é honrada** (`Locked`/`GroupedChildren`): cutucar não persiste
//! nada, mas a trava é o artista dizendo *"não mexa neste objeto"*, e uma exceção
//! aqui seria a única porta do app que a ignora.
//!
//! # O que a mão NÃO faz, e é decisão
//!
//! - **Não abre arrasto de gizmo** quando pega: o `Transform` escrito por um
//!   arrasto durante o play é sobrescrito pelo readback do mesmo frame, então
//!   deixar os dois abertos seria um gesto inerte cavalgando um vivo — *inócuo
//!   porque outra coisa o desfaz* é precisamente o raciocínio que apodrece.
//! - **Não muda a seleção** — nem precisa: o press que pega é o MESMO que o pick
//!   de canvas já usa para selecionar, e a seleção continua acontecendo depois.
//!   Um clique sem arrasto, aliás, **não move nada**: a âncora nasce EXATAMENTE
//!   no ponto pego, então o erro da mola é zero e ela não empurra.
//! - **Não zera a velocidade ao soltar** (o wrapper não toca nela) ⇒ **atirar**
//!   cai de graça, e é metade do valor do gesto.
//! - **Não sobrevive a um scrub**: um rewind solta a mão (`bridge::grab`, regra
//!   2). Um cutucão não está no documento, então nenhum replay o reproduz — a
//!   alternativa seria a resposta de um tick depender do cache.
//!
//! # Nomeado, não construído
//!
//! Soltar deixa **um passo de undo** cujo diff é a pose de play do corpo. É a
//! forma pré-existente de QUALQUER clique durante o play (o `readback` da física
//! e o apply da timeline escrevem `Transform` a cada frame, e o `held_button`
//! suprime só o gesto em voo), não um efeito desta wave; a cura de verdade é uma
//! lei no roteador de undo sobre *pose escrita pelo solver não é estado autorado*,
//! que precisa distinguir isso de uma edição REAL feita durante o play — outra
//! decisão, outro domínio.

use ph2d_ecs::Entity;
use ph2d_physics_ecs::{InteractionSettings, InteractionTool, PhysicsBridge};

use crate::App;

/// **Quantos ticks o flash de uma explosão vive.** Um estouro é instantâneo, então
/// sem marca decaída o único vestígio visível são corpos que se moveram — a mesma
/// razão (e a mesma ordem de grandeza) do flash de contato do W-TickContacts.
pub(crate) const BLAST_FLASH_TICKS: u32 = 12;

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
pub(crate) fn take_hold(
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
pub(crate) fn poke_at(
    physics: &mut PhysicsBridge,
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
        // Já filtrado acima; o braço existe para o `match` ser exaustivo sem um
        // `_ =>` que engoliria uma ferramenta nova em silêncio.
        InteractionTool::Hand => None,
        InteractionTool::Explode => {
            Some(physics.explode(world_point, s.blast_radius, s.blast_impulse))
        }
        InteractionTool::Attract => {
            physics.attract(&s, world_point);
            Some(0)
        }
    }
}

impl App {
    /// **A mão segue o cursor.** No-op sem mão em voo, então roda em todo Move
    /// junto dos outros `advance_*`.
    ///
    /// ⚠️ Não marca `any_input_this_frame`: o evento que trouxe o cursor até aqui
    /// já decidiu isso, e marcá-lo aqui armaria o diff de undo por um movimento
    /// de mouse.
    pub(crate) fn advance_body_grab(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // `is_poking` and not `is_grabbing`: the attract field follows the cursor
        // by the same rule, and asking the narrower question is how the second
        // sustained tool would have been born outside it.
        if !gfx.physics.is_poking() {
            return;
        }
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world(self.last_pointer, window);
        gfx.physics.move_grab(world);
        gfx.physics.move_attract(world);
    }

    /// **Soltar.** Chamado no TOPO do `on_mouse_input` para todo release de
    /// botão, antes de qualquer `return` — e as duas metades dessa frase são
    /// load-bearing:
    ///
    /// - **no topo**, porque aquele handler tem muitos early-returns e uma mão
    ///   que sobrevive ao release fica colada no cursor para sempre;
    /// - **todo botão**, porque uma mão não é um modificador: soltar qualquer
    ///   botão encerra o gesto, e enumerar quais soltam é a forma que apodrece.
    ///
    /// No-op sem mão, que é o caso de quase todo release do app.
    pub(crate) fn release_body_grab(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.physics.release_grab();
            // The attract field ends on the same release, and for the same
            // reason: a sustained tool that survives the button is a tool stuck
            // to the cursor forever.
            gfx.physics.stop_attract();
        }
    }
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
pub(crate) fn age_blast_flash(flash: &mut Option<([f32; 2], f32, u32)>) {
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
