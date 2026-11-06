use super::*;
use ph2d_asset::AssetDb;
use ph2d_render::WHITE_TILE_KEY;

/// O dono dos documentos (o Painter, no app): `bits → documento`.
type Dono = BTreeMap<u64, u32>;

struct Cena {
    w: World,
    dono: Dono,
    t: Achatados<u32>,
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
    let mut w = World::new();
    let sprite = Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let e = w
        .spawn((sprite, SpritePixels(antes), StableId(70), PaintedDoc(5)))
        .id();
    Cena {
        w,
        dono: BTreeMap::from([(e.to_bits(), 9)]),
        t: Achatados::default(),
        e,
        antes,
        depois,
    }
}

impl Cena {
    fn achata(&mut self, quem: &[Entity]) -> bool {
        let Self { w, dono, t, .. } = self;
        t.achata(w, quem, (self.antes, self.depois), |b| dono.remove(&b))
    }

    fn mostra(&mut self, id: AssetId) {
        self.w.entity_mut(self.e).insert(SpritePixels(id));
        let dono = std::cell::RefCell::new(&mut self.dono);
        self.t.reconcilia(
            &mut self.w,
            &BTreeMap::new(),
            |b| dono.borrow_mut().remove(&b),
            |b, d| {
                dono.borrow_mut().insert(b, d);
            },
        );
    }
}

/// ⭐ *Píxeis por fora vencem as camadas: o documento sai, e o carimbo também.*
/// *Mutação: `achata` sem tirar o documento ⇒ o save colhe-o.*
#[test]
fn os_pixeis_de_fora_vencem_as_camadas() {
    let mut c = cena();
    let e = c.e;
    assert!(c.achata(&[e]));
    assert!(c.dono.is_empty());
    assert!(c.w.get::<PaintedDoc>(e).is_none());
}

/// *Mutação: `reconcilia` sem o ramo de `antes` ⇒ o Undo não devolve as camadas.*
#[test]
fn o_undo_devolve_as_camadas_e_o_redo_tira_as_outra_vez() {
    let mut c = cena();
    let (e, antes, depois) = (c.e, c.antes, c.depois);
    c.achata(&[e]);
    c.mostra(antes);
    assert_eq!(c.dono[&e.to_bits()], 9, "o Undo");
    c.mostra(depois);
    assert!(c.dono.is_empty(), "o Redo");
    c.mostra(antes);
    assert_eq!(c.dono[&e.to_bits()], 9, "e de novo");
}

/// *Quem não tinha camadas não leva aviso; e quem não recebeu píxeis fica com as dele.*
#[test]
fn sem_documento_nao_ha_aviso_e_os_outros_ficam() {
    let mut c = cena();
    let outro = c.w.spawn(StableId(71)).id();
    assert!(!c.achata(&[outro]));
    assert_eq!(c.dono.len(), 1);
}
