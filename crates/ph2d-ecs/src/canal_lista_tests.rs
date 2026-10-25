//! Os gates da família LISTA (F30 §A1).

use super::*;
use ph2d_component_desc::lista::codigo;

/// A chave leva o item inteiro (32 bits) e volta a ele; o grupo parte-se pelo item.
#[test]
fn a_chave_leva_o_item_e_o_grupo_parte_se_por_ele() {
    let a = chave(codigo::TIMERS, 2, 0, ItemId(0x0003_0004)).expect("cabe");
    assert_eq!(item(a), ItemId(0x0003_0004));
    let b = chave(codigo::TIMERS, 2, 0, ItemId(4)).expect("cabe");
    assert_ne!(a.grupo(), b.grupo(), "dois timers são dois grupos");
    assert!(!a.mesmo_grupo(b));
    let x = chave(codigo::ANCORAS, 2, 0, ItemId(9)).expect("cabe");
    let y = chave(codigo::ANCORAS, 2, 1, ItemId(9)).expect("cabe");
    assert!(x.mesmo_grupo(y), "x e y da mesma âncora gravam-se juntos");
    assert_eq!(
        a.info().expect("info").rotulo,
        "component.field.timer_fields.2"
    );
}

/// Toda chave de todo item resolve (o catálogo e a família dizem o mesmo).
#[test]
fn toda_chave_de_item_resolve() {
    for l in lista::todas() {
        let ks: Vec<_> = chaves_do_item(l, ItemId(77)).collect();
        assert!(!ks.is_empty(), "{}", l.componente);
        for k in ks {
            assert!(k.info().is_some() && k.membro().is_some(), "{k:?}");
        }
    }
}
