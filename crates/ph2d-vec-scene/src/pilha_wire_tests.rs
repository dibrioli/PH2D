use ph2d_item_id::ItemId;

use super::with_pilha_layout_v23;
use crate::effect::{FxEntry, PathEffect};
use crate::fx_trim::TrimSpec;
use crate::{Paint, PaintEntry, Rgba8};

/// O espelho V24 lê o que o `Serialize` derivado grava (uma divergência de ordem reprova aqui).
#[test]
fn a_forma_viva_da_pilha_le_o_que_grava() {
    let mut t = PaintEntry::fill(Paint::solid(Rgba8::new(1, 2, 3, 4)));
    t.offset = [1.5, -2.0];
    t.dilate = 0.5;
    t.dilate_join = 2;
    t.id = ItemId(7);
    let mut e = FxEntry::new(PathEffect::Trim(TrimSpec {
        start: 0.2,
        end: 0.7,
        offset: 0.1,
    }));
    e.id = ItemId(9);
    let par = (vec![t], vec![e]);
    let b = postcard::to_allocvec(&par).unwrap();
    let lido: (Vec<PaintEntry>, Vec<FxEntry>) = postcard::from_bytes(&b).unwrap();
    assert_eq!(lido, par);
    // Dentro do âmbito v23 os mesmos bytes NÃO se lêem iguais (o id seria lido como o próximo campo).
    let torto =
        with_pilha_layout_v23(|| postcard::from_bytes::<(Vec<PaintEntry>, Vec<FxEntry>)>(&b));
    assert!(torto.as_ref() != Ok(&par));
}
