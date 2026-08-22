//! **§5 9-Slice** — o snapshot que a seção lê e o commit que ela escreve.
//!
//! Irmão do [`super::inspector_ordering`], e pela mesma razão dele: uma família de seção por
//! ficheiro, com o cap de LOC a decidir o corte.
//!
//! A seção que a spec
//! ([`03_inspector_secoes.md`](../../../../docs/Sprite_projeto/03_inspector_secoes.md) §3.5)
//! declarou em 2026-05 e que nasceu em **2026-08-21**.
//!
//! # `SliceNine` é um componente OPCIONAL, e as duas metades disso importam
//!
//! - **Ausente** = o sprite de sempre. O snapshot leva `present: false`, e a seção mostra um
//!   convite em vez de zeros — *ausência de autoria não é «bordas a zero»*.
//! - **Anexar é inerte** (`SliceNine::INERT`): um botão que dá acesso a uma seção não pode
//!   mudar a cena. O artista liga o modo quando quiser ver o efeito.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld, SliceNine, World};
use ph2d_editor::{InspectorSliceInfo, InspectorSliceMixed, SliceFieldEdit};

use super::inspector_ordering::{queue_remove, queue_set};

/// Nome canónico do componente no `ComponentRegistry`. ⚠️ Uma string errada aqui não falha a
/// compilação: o comando é descartado em silêncio e a edição não acontece. É o mesmo nome que
/// `register_ecs_components` regista.
const SLICE_NINE: &str = "ph2d::ecs::SliceNine";

/// A autoria de 9-slice de uma entidade, ou o inerte quando o componente está ausente.
/// Partilhado pelo produtor do snapshot e pela comparação de BulkSelect, para que os dois
/// nunca divirjam.
fn slice_of(world: &World, entity: Entity) -> (bool, SliceNine) {
    match world.get::<SliceNine>(entity) {
        Some(s) => (true, *s),
        None => (false, SliceNine::INERT),
    }
}

/// Constrói o snapshot da §5, ou `None` quando a entidade não tem `Transform` (não é digna de
/// Inspector) — a mesma porta das outras seções.
#[allow(clippy::float_cmp)] // comparação exata: o MESMO valor guardado = não divergente
pub(super) fn build_slice_info(
    world: &World,
    entity_bits: u64,
    selected: &[u64],
    selected_count: usize,
) -> Option<InspectorSliceInfo> {
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let (present, s) = slice_of(world, entity);
    let mut mixed = InspectorSliceMixed::default();
    if selected.len() > 1 {
        for &bits in selected {
            let (p, o) = slice_of(world, Entity::from_bits(bits));
            mixed.present |= p != present;
            mixed.draw_mode |= o.draw_mode != s.draw_mode;
            mixed.borders |= o.borders != s.borders;
            mixed.size |= o.size != s.size;
            mixed.tile_modes |=
                o.tile_modes != s.tile_modes || o.centre_tile_mode != s.centre_tile_mode;
            mixed.tile_mode |= o.tile_mode != s.tile_mode;
            mixed.fill_center |= o.fill_center != s.fill_center;
        }
    }
    let mut tile_modes = [0u8; 8];
    for (i, m) in s.tile_modes.iter().enumerate() {
        tile_modes[i] = m.tag();
    }
    let centre_tile_mode = s.centre_tile_mode.tag();
    Some(InspectorSliceInfo {
        entity_bits,
        present,
        draw_mode_tag: s.draw_mode.tag(),
        borders: s.borders,
        size: s.size,
        tile_modes,
        centre_tile_mode,
        tile_mode_tag: s.tile_mode.tag(),
        fill_center: s.fill_center,
        selected_count,
        mixed,
    })
}

/// Aplica uma [`SliceFieldEdit`].
///
/// ⚠️ **Toda edição de campo lê-modifica-escreve o componente atual** (ou o inerte, quando ele
/// ainda não existe): editar uma borda não pode repor as outras três nem o modo. E toda edição de
/// campo **anexa** o componente se ele faltar — mexer num controlo da seção é autorar, e exigir
/// carregar «Add» primeiro seria um passo que não explica nada.
pub(super) fn apply_slice_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: SliceFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    if matches!(edit, SliceFieldEdit::Detach) {
        queue_remove(queue, registry, entity_bits, SLICE_NINE);
        return;
    }
    let entity = Entity::from_bits(entity_bits);
    let mut s = sim
        .world()
        .get::<SliceNine>(entity)
        .copied()
        .unwrap_or(SliceNine::INERT);
    match edit {
        // Já tratado acima; repetido para o `match` ficar exaustivo sem braço-curinga — um
        // curinga aqui engoliria em silêncio a próxima variante que alguém acrescentasse.
        SliceFieldEdit::Detach => return,
        SliceFieldEdit::Attach => {
            // Anexar não é editar: se já existe, deixa como está em vez de repor o inerte.
            if sim.world().get::<SliceNine>(entity).is_some() {
                return;
            }
            s = SliceNine::INERT;
        }
        SliceFieldEdit::DrawMode(t) => s.draw_mode = ph2d_ecs::SliceDrawMode::from_tag(t),
        SliceFieldEdit::Border(i, v) => {
            if let Some(slot) = s.borders.get_mut(usize::from(i)) {
                *slot = v;
            }
        }
        SliceFieldEdit::SizeX(v) => s.size[0] = v,
        SliceFieldEdit::SizeY(v) => s.size[1] = v,
        SliceFieldEdit::RegionMode(i, t) => {
            if let Some(slot) = s.tile_modes.get_mut(usize::from(i)) {
                *slot = ph2d_ecs::TileRegionMode::from_tag(t);
            }
        }
        SliceFieldEdit::TileMode(t) => s.tile_mode = ph2d_ecs::SliceTileMode::from_tag(t),
        SliceFieldEdit::CentreMode(t) => {
            s.centre_tile_mode = ph2d_ecs::TileRegionMode::from_tag(t);
        }
        SliceFieldEdit::FillCenter(b) => s.fill_center = b,
    }
    // ⚠️ Saneia na PORTA DE ESCRITA também, não só na leitura: um valor infinito guardado
    // sobrevive ao save/load e reaparece como um sprite invisível três sessões depois.
    queue_set(queue, registry, entity_bits, SLICE_NINE, &s.sanitized());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{SliceDrawMode, TileRegionMode};

    /// Editar UMA borda preserva as outras três e o modo — a lei do ler-modificar-escrever.
    #[test]
    fn editing_one_border_preserves_its_siblings() {
        let mut s = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [1.0, 2.0, 3.0, 4.0],
            ..SliceNine::INERT
        };
        // O que o braço faz, sem precisar de um `World`.
        if let Some(slot) = s.borders.get_mut(2) {
            *slot = 9.0;
        }
        assert_eq!(s.borders, [1.0, 2.0, 9.0, 4.0]);
        assert_eq!(s.draw_mode, SliceDrawMode::Sliced, "o modo foi reposto");
    }

    /// Um índice fora de alcance não escreve em lado nenhum — nem entra em pânico.
    #[test]
    fn an_out_of_range_index_writes_nothing() {
        let mut s = SliceNine::INERT;
        assert!(s.borders.get_mut(9).is_none());
        assert!(s.tile_modes.get_mut(99).is_none());
    }

    /// A ordem `tile_modes` do snapshot é a ordem das regiões — tag a tag.
    #[test]
    fn the_snapshot_tile_modes_keep_the_region_order() {
        let mut s = SliceNine::INERT;
        s.tile_modes[ph2d_ecs::SliceRegion::Right as usize] = TileRegionMode::Mirror;
        let mut tags = [0u8; 8];
        for (i, m) in s.tile_modes.iter().enumerate() {
            tags[i] = m.tag();
        }
        assert_eq!(tags[ph2d_ecs::SliceRegion::Right as usize], 2);
        assert_eq!(tags.iter().filter(|t| **t != 0).count(), 1);
    }
}
