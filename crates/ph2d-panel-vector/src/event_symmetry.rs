//! **O roteamento do slider de SEGMENTS da simetria radial** — irmão de [`super`] pelo teto de
//! 600 LOC do painel, com o mesmo corte do `event_texpat` e do `event_contour`: um assunto, uma
//! porta.
//!
//! ⛔⛔ **Este ficheiro nasceu de um CONTROLO MORTO, e a espécie é a terceira** (caça de
//! 2026-08-30): o id `VECTOR_SYM_SEGMENTS` aparecia em quatro sítios — o `ids`, o
//! `populate_symmetry`, o `paint_symmetry` e o censo de arquitectura — e os **quatro eram
//! declaração, registo ou pintura**. Nenhum braço de evento. O artista arrastava a barra, o
//! número mudava no chip, e a contagem que a rosácea usa ficava pregada no `6` do default para
//! sempre.
//!
//! ⚠️ **O que um `grep` não vê é o TERCEIRO passo:** *o painel escreve onde · quem lê · o leitor
//! DECIDE?* Aqui não havia sequer o primeiro — nada era escrito —, e mesmo assim o gate de
//! focalizabilidade (`the_painted_control_reaches_a_consumer`) o listava, porque ele mede se o
//! ponteiro alcança o widget, nunca se o valor alcança um consumidor.
//!
//! # A conversão mora na FRONTEIRA
//!
//! O que sai daqui é a **contagem de cópias**, não o track: a shell recebe o número que o
//! documento guarda (`SymmetryStyle::segments`) e nunca precisa saber que existe uma barra
//! `0..=1`. É a mesma lei que o cabeçalho do [`super`] escreve para os outros sliders — e a porta
//! da conversão é uma só, [`crate::paint_symmetry::track_to_segments`], a inversa exacta do mapa
//! que o `populate_symmetry` dá ao par slider↔chip.

use super::forward_track;
use ph2d_editor_core::panel::PanelHostInternal;

/// `Some(consumido)` se `id` é o slider de *Segments* ou o chip ligado a ele; `None` se não é
/// nenhum dos dois.
///
/// ⚠️ **O CHIP é engolido de propósito.** Editá-lo já espelha para o slider, que emite o próprio
/// `ValueChanged` (a porta única do dispatch) — sem esta metade o valor viajaria **duas vezes** por
/// tecla, e a segunda viagem carregaria o mesmo número. É a mesma cicatriz que o `VECTOR_WIDTH_NUM`
/// e os outros doze chips deste painel já carregam.
pub(super) fn segments_slider_event(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    use crate::ids;
    use crate::paint_symmetry::{segments_to_track, track_to_segments};
    if id == ids::VECTOR_SYM_SEGMENTS {
        // O default é o do vocabulário, e não um literal: ele só é lido se o slider não estiver
        // registado, e um número escrito à mão aqui seria a segunda resposta a *"quantas cópias
        // uma rosácea faz por omissão?"*.
        let repouso = segments_to_track(ph2d_symmetry::SymmetryStyle::default().segments);
        return Some(forward_track(host, id, repouso, |t| {
            f64::from(track_to_segments(t as f32))
        }));
    }
    (id == ids::VECTOR_SYM_SEGMENTS_NUM).then_some(true)
}
