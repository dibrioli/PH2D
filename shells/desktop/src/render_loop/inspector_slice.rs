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

use super::inspector_ordering::queue_set;

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
    // ⭐ **A §5 aparece se, e só se, o componente estiver lá** (ADR-0166 / F3). A *face vazia* —
    // publicar com `present: false` para mostrar o «+ Add 9-Slice» — era a ÚNICA rota para a
    // feature, e por isso não podia ser apagada antes de existir outra. Hoje a rota é o `+` do
    // cabeçalho, e o censo (`component_reach_tests`) prova que ela alcança este componente.
    world.get::<SliceNine>(entity)?;
    let (present, s) = slice_of(world, entity);
    let mut mixed = InspectorSliceMixed::default();
    if selected.len() > 1 {
        for &bits in selected {
            let (p, o) = slice_of(world, Entity::from_bits(bits));
            // ⚠️ O bit EFETIVO, não as duas metades: sem componente e com ele desligado são o
            // mesmo estado, e compará-los em separado acende «divergente» sobre um acordo.
            mixed.enabled |= (p && o.draw_mode.is_nine()) != (present && s.draw_mode.is_nine());
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
/// campo **anexa** o componente se ele faltar — mexer num controlo da seção é autorar. É isso que
/// faz a caixa «Enable 9-slice» bastar sozinha: ela manda um `DrawMode(1)`, e o anexo vem de
/// borda. ⚠️ Houve aqui um `Attach` explícito, e ele morreu com o botão «+ Add»: dois passos de
/// undo para um clique, e uma variante que nenhum controlo produzia.
pub(super) fn apply_slice_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: SliceFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let mut s = sim
        .world()
        .get::<SliceNine>(entity)
        .copied()
        .unwrap_or(SliceNine::INERT);
    if !write_field(&mut s, edit) {
        return;
    }
    // ⚠️ Saneia na PORTA DE ESCRITA também, não só na leitura: um valor infinito guardado
    // sobrevive ao save/load e reaparece como um sprite invisível três sessões depois.
    queue_set(queue, registry, entity_bits, SLICE_NINE, &s.sanitized());
}

/// Escreve UM campo no componente. Devolve sempre `true` hoje — o `bool` fica porque a próxima
/// edição que **não** seja de campo (uma que apague, uma que reponha) precisa dele, e porque é
/// ele que impede um braço novo de escrever em silêncio sobre o inerte.
///
/// ⚠️ **É uma função à parte para poder ser CHAMADA por teste, e a razão é uma mutação que
/// sobreviveu** (2026-08-22). Os testes deste ficheiro re-escreviam o corpo de cada braço à mão —
/// «o que o braço faz, sem precisar de um `World`» — e por isso mediam a sua própria cópia:
/// apagar `s.centre_tile_mode = m` da produção deixava o gate do «Tile all» **verde**. *Um gate
/// que guarda a sua própria cópia mede-se a si mesmo*, e a cura é sempre a mesma: dar-lhe a coisa
/// real para chamar.
pub(super) fn write_field(s: &mut SliceNine, edit: SliceFieldEdit) -> bool {
    match edit {
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
        SliceFieldEdit::AllRegions(t) => {
            // ⚠️ **As NOVE, o miolo incluído** — «Tile all» que deixasse o miolo a esticar seria
            // um atalho que falha exatamente na maior área. Os cantos entram na volta e o
            // `sanitized()` normaliza-os de volta a fixo, que é o que um canto pode ser.
            let m = ph2d_ecs::TileRegionMode::from_tag(t);
            s.tile_modes = [m; 8];
            s.centre_tile_mode = m;
        }
        SliceFieldEdit::FillCenter(b) => s.fill_center = b,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{SliceDrawMode, TileRegionMode};

    /// Editar UMA borda preserva as outras três e o modo — a lei do ler-modificar-escrever.
    ///
    /// ⚠️ **Chama `write_field`, o braço REAL.** A versão anterior deste teste re-escrevia o
    /// corpo do braço à mão e por isso media a sua própria cópia.
    #[test]
    fn editing_one_border_preserves_its_siblings() {
        let mut s = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [1.0, 2.0, 3.0, 4.0],
            ..SliceNine::INERT
        };
        assert!(write_field(&mut s, SliceFieldEdit::Border(2, 9.0)));
        assert_eq!(s.borders, [1.0, 2.0, 9.0, 4.0]);
        assert_eq!(s.draw_mode, SliceDrawMode::Sliced, "o modo foi reposto");
    }

    /// ⚠️ **«Tile all» escreve nas NOVE, o miolo incluído.**
    ///
    /// O miolo é a maior área ladrilhada: um atalho que o deixasse de fora falharia exatamente
    /// onde mais se vê. E é a conveniência que o modo `Tiled` dava antes de ser retirado — só
    /// que agora ela ESCREVE na grelha em vez de a reinterpretar por trás, o que a torna
    /// visível, editável célula a célula depois, e desfazível num `Ctrl+Z`.
    #[test]
    fn tile_all_writes_the_ninth_cell_too() {
        let mut s = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [8.0; 4],
            ..SliceNine::INERT
        };
        assert!(write_field(&mut s, SliceFieldEdit::AllRegions(1)));
        assert_eq!(
            s.centre_tile_mode,
            TileRegionMode::Repeat,
            "o miolo ficou de fora do «all» — e' a maior area ladrilhada"
        );
        assert!(s.tile_modes.iter().all(|m| *m == TileRegionMode::Repeat));
        // E o saneamento devolve os CANTOS a fixo: um canto nunca ladrilha.
        let c = s.sanitized();
        for r in ph2d_ecs::SliceRegion::ALL {
            let (col, row) = r.cell();
            let want = if col != 1 && row != 1 {
                TileRegionMode::Stretch
            } else {
                TileRegionMode::Repeat
            };
            assert_eq!(c.region_mode(r), want, "{r:?} saiu errado do «Tile all»");
        }
        assert_eq!(c.centre_tile_mode, TileRegionMode::Repeat);
        // «Stretch all» e' a volta atras -- a capacidade que o antigo `Tiled` NAO tinha.
        assert!(write_field(&mut s, SliceFieldEdit::AllRegions(0)));
        assert!(s.tile_modes.iter().all(|m| *m == TileRegionMode::Stretch));
        assert_eq!(s.centre_tile_mode, TileRegionMode::Stretch);
    }

    /// Um índice fora de alcance não escreve em lado nenhum — nem entra em pânico.
    ///
    /// ⚠️ Também pelo braço real: um `get_mut` que passasse a indexar cru entraria em pânico, e
    /// um teste que só verificasse `get_mut(9).is_none()` nunca o veria.
    #[test]
    fn an_out_of_range_index_writes_nothing() {
        let mut s = SliceNine::INERT;
        let before = s;
        assert!(write_field(&mut s, SliceFieldEdit::Border(9, 5.0)));
        assert!(write_field(&mut s, SliceFieldEdit::RegionMode(99, 1)));
        assert_eq!(s, before, "um indice fora de alcance escreveu algures");
    }

    /// ⚠️ **«Divergente» mede-se no que o utilizador VÊ, não na representação.**
    ///
    /// Um sprite **sem** o componente e um **com ele desligado** estão no mesmo estado: 9-slice
    /// desligado. O snapshot comparava as duas metades em separado (`present` e `draw_mode`), e
    /// por isso uma seleção dos dois acendia o «divergente» da caixa sobre um **acordo** —
    /// exatamente a mentira que a auditoria de 2026-08-22 procurava noutros sítios.
    #[test]
    fn absent_and_switched_off_do_not_count_as_a_disagreement() {
        let mut sim = ph2d_ecs::SimWorld::default();
        let off = sim
            .world_mut()
            .spawn((ph2d_ecs::Transform::default(), SliceNine::INERT))
            .id()
            .to_bits();
        let absent = sim
            .world_mut()
            .spawn(ph2d_ecs::Transform::default())
            .id()
            .to_bits();
        let both = [off, absent];
        let info = build_slice_info(sim.world(), off, &both, 2).expect("snapshot");
        assert!(
            !info.mixed.enabled,
            "sem componente e desligado sao o MESMO estado — a caixa nao pode dizer «divergente»"
        );

        // E o controlo positivo: um LIGADO ao lado de um desligado diverge de verdade.
        let on = sim
            .world_mut()
            .spawn((
                ph2d_ecs::Transform::default(),
                SliceNine {
                    draw_mode: SliceDrawMode::Sliced,
                    ..SliceNine::INERT
                },
            ))
            .id()
            .to_bits();
        let mixed = [off, on];
        let info = build_slice_info(sim.world(), off, &mixed, 2).expect("snapshot");
        assert!(
            info.mixed.enabled,
            "ligado e desligado TEM de divergir — senao o gate acima passa por estar sempre falso"
        );
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
