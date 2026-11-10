//! ⭐ **O losango de cada campo é o canal que o ESCRITOR do campo grava** (plano Timeline 17, item 3):
//! uma chave só no canal de outro campo não acende este; uma chave em cada canal que o escritor
//! grava acende-o. O escritor de cada campo da Sprite é o campo do catálogo que o evento dele edita
//! (`event_color_tint` → `SpriteFieldEdit::Tint` = `tint` inteiro, alfa incluído; `event_sprite_value` →
//! `Sprite::opacity`; `cards_sprite_pivot` → o pivô), lido pelo rótulo do catálogo (`DONOS`).

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 11;

fn rotulo(r: &str) -> Vec<PropKind> {
    let ks = ph2d_ecs::canal_componente::chaves_do_rotulo(r).iter();
    let props = ph2d_ecs::canal_componente::props_do_rotulo(r).iter();
    ks.map(|&k| PropKind::Field(k))
        .chain(props.filter_map(|&p| PropKind::from_target(AnimTarget::new(p))))
        .collect()
}

/// `(campo, o que o escritor dele grava, a LINHA a que pertence)`: os campos da mesma linha partilham o
/// losango (X/Y, os dois Flip, os quatro cantos — um losango por linha).
fn escritores() -> Vec<(NodeId, Vec<PropKind>, u8)> {
    let s = |n: u16| rotulo(&format!("component.field.sprite.{n}"));
    let canto = |n: u16| rotulo(&format!("component.field.sprite_corner_tint.{n}"));
    let pos = vec![
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Position,
    ];
    let mut v = vec![
        (insp::INSP_TRANSFORM_POS_X, pos.clone(), 0),
        (insp::INSP_TRANSFORM_POS_Y, pos, 0),
        (insp::INSP_TRANSFORM_ROT, vec![PropKind::Rotation], 1),
        (insp::INSP_SPRITE_TINT_SWATCH, s(1), 2),
        (insp::INSP_SPRITE_SELF_TINT_SWATCH, s(6), 3),
        (insp::INSP_SPRITE_OPACITY, s(7), 4),
        (insp::INSP_SPRITE_OPACITY_CHIP, s(7), 4),
        (insp::INSP_SPRITE_TINT_FILL, s(8), 5),
        (insp::INSP_SPRITE_FLIP_X, s(2), 6),
        (insp::INSP_SPRITE_FLIP_Y, s(3), 6),
        (insp::INSP_SPRITE_PIVOT_XY[0], s(4), 7),
        (insp::INSP_SPRITE_PIVOT_XY[1], s(4), 7),
    ];
    let cantos = [
        insp::INSP_SPRITE_CORNER_TL,
        insp::INSP_SPRITE_CORNER_TR,
        insp::INSP_SPRITE_CORNER_BL,
        insp::INSP_SPRITE_CORNER_BR,
    ];
    v.extend((1..=4).zip(cantos).map(|(n, id)| (id, canto(n), 8)));
    v
}

fn so_com(p: PropKind) -> TimelineDoc {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for t in [0.0, 1.0] {
        let v = AnimValue::Float(1.0);
        d.insert_key(E, p, RationalTime::from_seconds(t), v, Interp::Linear);
    }
    d
}

fn no_quadro(_: u64, _: PropKind) -> Option<f32> {
    Some(1.0)
}

/// ⭐ Cada campo acende com cada canal que o seu escritor grava, e com nenhum canal de outra linha.
/// *Mutação: o pivô volta a levar `POSICAO` no grupo do losango ⇒ uma chave na Position acende o Pivot.*
#[test]
fn o_losango_de_cada_campo_e_o_canal_que_o_seu_escritor_grava() {
    let inst = Instante {
        t_cut: 0.0,
        solo: true,
    };
    let tabela = escritores();
    for (id, grava, linha) in &tabela {
        assert!(
            !grava.is_empty(),
            "{id:?}: o escritor não tem canal no catálogo"
        );
        assert!(
            !canal_do_campo(*id).is_empty(),
            "{id:?}: sem canal no Inspector"
        );
        for &p in grava {
            let m = marcas(&so_com(p), &[E], inst, no_quadro);
            assert_eq!(
                m.get(id),
                Some(&KeyMark::Chave),
                "{id:?} não acende com {p:?}: {:?}",
                m.get(id)
            );
        }
        let alheios = tabela.iter().filter(|(_, _, l)| l != linha);
        for &p in alheios
            .flat_map(|(_, g, _)| g)
            .filter(|p| !grava.contains(p))
        {
            let m = marcas(&so_com(p), &[E], inst, no_quadro);
            assert_ne!(
                m.get(id),
                Some(&KeyMark::Chave),
                "{id:?} acende com {p:?}, que outro campo grava"
            );
        }
    }
}

/// ⭐ O losango mostra o que o clique grava: o mapa e o [`canal_do_campo`] dão o MESMO grupo a cada campo
/// (antes o mapa ficava com o último registo e o clique com o primeiro).
#[test]
fn o_mapa_e_o_clique_leem_o_mesmo_grupo() {
    let mut vistos = std::collections::BTreeSet::new();
    for (id, g) in todos() {
        if vistos.insert(id) {
            assert_eq!(canal_do_campo(id), g, "{id:?}");
        }
    }
    assert!(vistos.len() > CAMPOS.len() / 2, "a régua varreu os campos");
}
