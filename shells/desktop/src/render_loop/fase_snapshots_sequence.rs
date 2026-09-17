//! **Fase do quadro: O INSTANTÂNEO DA CUTSCENE** (TOP-20 #19) — fase-filha da
//! [`super::fase_snapshots_publish`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de ARGUMENTOS da irmã** (ela chegou a `9` contra `7` ao
//! ganhar o documento da timeline e os dois relógios) **e é o certo por DEPENDÊNCIA**: esta é a
//! única secção do Inspector que lê o **documento da animação**, e é o que a separa das três que
//! leem a VM dos scripts, o relógio das partículas e o mundo do HUD.
//!
//! ⛔ **E a `ph2d-app-components` não vê a timeline, de propósito** (a lei do
//! [`ph2d_app_components::sequence_inspector`]): quem colhe os nomes e as durações é esta fase,
//! que vê as duas coisas.

use ph2d_app_components::sequence_inspector::{Cutscene, Relogios};
use ph2d_ecs::SimWorld;

/// Publica o que a secção *Sequence* mostra do objecto ACTIVO. `None` = ninguém escolhido ⇒ a
/// secção não existe, que é a lei do ADR-0166.
pub(super) fn publica(
    sim: &SimWorld,
    timeline: &ph2d_timeline::TimelineState,
    playhead: &ph2d_core::Playhead,
    escolhido: Option<u64>,
    quantos: usize,
) {
    // ⚠️ **A vista pela PORTA** (`super::fase_sequences::a_vista_deixa_correr`), que é a MESMA que
    // o `timeline_bridge` lê para decidir se a fase das cutscenes corre — senão o painel diria uma
    // coisa e o motor faria outra, e o artista acreditaria no painel.
    let relogios = Relogios {
        clock_playing: playhead.is_playing(),
        vista_deixa_correr: super::fase_sequences::a_vista_deixa_correr(
            timeline.keys_mode,
            timeline.container_open,
        ),
    };
    ph2d_panel_inspector::set_current_inspector_sequence(escolhido.and_then(|b| {
        // ⚠️ **A lista só se colhe quando há objecto escolhido** (dentro do `and_then`): sem
        // selecção não há secção nenhuma a alimentar, e o `Vec` seria trabalho por quadro para
        // ninguém.
        let cutscenes: Vec<Cutscene<'_>> = timeline
            .doc
            .containers()
            .iter()
            .enumerate()
            .map(|(i, c)| Cutscene {
                nome: c.name.as_str(),
                // ⚠️ **A porta ÚNICA da duração** (`container_length_seconds`): ela folda o
                // `length_override` autorado e o extent do interior, e uma segunda conta aqui
                // faria o aviso *«o relógio acaba antes»* medir outro número que o que toca.
                duracao: timeline.doc.container_length_seconds(i),
            })
            .collect();
        ph2d_app_components::sequence_inspector::build_info(sim, b, &cutscenes, relogios, quantos)
    }));
}
