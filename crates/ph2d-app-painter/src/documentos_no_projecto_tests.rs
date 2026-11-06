use super::*;
use ph2d_asset::AssetDb;
use ph2d_render::WHITE_TILE_KEY;

struct Cena {
    sim: SimWorld,
    painter: PainterTool,
    e: Entity,
    antes: AssetId,
    depois: AssetId,
}

fn cena() -> Cena {
    let db = AssetDb::default();
    let (antes, depois) = (
        db.insert_image_rgba8(1, 1, vec![1, 1, 1, 255]),
        db.insert_image_rgba8(1, 1, vec![2, 2, 2, 255]),
    );
    let mut sim = SimWorld::new();
    let sprite = Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let e = sim
        .world_mut()
        .spawn((sprite, SpritePixels(antes), StableId(70), PaintedDoc(5)))
        .id();
    let mut painter = PainterTool::default();
    painter.bind_document(e.to_bits(), vec![255; 4], 1, 1);
    esquece_achatados();
    Cena {
        sim,
        painter,
        e,
        antes,
        depois,
    }
}

fn mostra(c: &mut Cena, id: AssetId) {
    c.sim.world_mut().entity_mut(c.e).insert(SpritePixels(id));
    reconcilia_achatados(&mut c.sim, &mut c.painter, &BTreeMap::new());
}

/// ⭐ *Píxeis por fora vencem as camadas: o documento sai, senão gravar e abrir re-assa-o por cima da edição.*
/// *Mutação: `achata` sem tirar o documento ⇒ o save colhe-o.*
#[test]
fn os_pixeis_de_fora_vencem_as_camadas_e_o_save_nao_as_grava() {
    let mut c = cena();
    let (antes, depois, e) = (c.antes, c.depois, c.e);
    assert!(achata(&mut c.sim, &mut c.painter, &[e], antes, depois));
    assert!(c.painter.document_entities().is_empty());
    assert!(
        c.sim.world().get::<PaintedDoc>(e).is_none(),
        "o carimbo faria os píxeis próprios saltarem o ficheiro"
    );
    let mut proximo = 1;
    assert!(carimba_e_colhe(&mut c.sim, &mut c.painter, &mut proximo).is_empty());
}

/// *Mutação: `reconcilia_achatados` sem o ramo de `antes` ⇒ o Undo não devolve as camadas.*
#[test]
fn o_undo_devolve_as_camadas_e_o_redo_tira_as_outra_vez() {
    let mut c = cena();
    let (antes, depois, e) = (c.antes, c.depois, c.e);
    achata(&mut c.sim, &mut c.painter, &[e], antes, depois);
    mostra(&mut c, antes);
    assert_eq!(c.painter.document_entities(), [e.to_bits()], "o Undo");
    mostra(&mut c, depois);
    assert!(c.painter.document_entities().is_empty(), "o Redo");
    mostra(&mut c, antes);
    assert_eq!(c.painter.document_entities(), [e.to_bits()], "e de novo");
    esquece_achatados();
}

/// *Quem não tinha camadas não leva aviso; e quem não recebeu píxeis fica com as dele.*
#[test]
fn sem_documento_nao_ha_aviso_e_os_outros_ficam() {
    let mut c = cena();
    let (antes, depois, e) = (c.antes, c.depois, c.e);
    let outro = c.sim.world_mut().spawn(StableId(71)).id();
    assert!(!achata(&mut c.sim, &mut c.painter, &[outro], antes, depois));
    assert_eq!(c.painter.document_entities(), [e.to_bits()]);
    esquece_achatados();
}
