//! ⭐ **O losango de cada campo é o canal que o ESCRITOR do campo grava** (plano Timeline 17, item 3):
//! uma chave só no canal de outro campo não acende este; uma chave em cada canal que o escritor
//! grava acende-o. O escritor de cada campo da Sprite é o campo do catálogo que o evento dele edita
//! (`event_color_tint` → `SpriteFieldEdit::Tint` = `tint` inteiro, alfa incluído; `event_sprite_value` →
//! `Sprite::opacity`; `cards_sprite_pivot` → o pivô), lido pelo rótulo do catálogo (`DONOS`).

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

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
    let pos = POSICAO.to_vec();
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
    let cantos = [insp::INSP_SPRITE_CORNER_TL, insp::INSP_SPRITE_CORNER_TR];
    let cantos = cantos
        .into_iter()
        .chain([insp::INSP_SPRITE_CORNER_BL, insp::INSP_SPRITE_CORNER_BR]);
    let canto = |n: u16| rotulo(&format!("component.field.sprite_corner_tint.{n}"));
    v.extend((1..=4).zip(cantos).map(|(n, id)| (id, canto(n), 8)));
    v
}

/// O losango de `id` com duas chaves só no canal `p` da entidade 11.
fn marca_so_com(id: NodeId, p: PropKind) -> Option<KeyMark> {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    let (v, i) = (AnimValue::Float(1.0), Interp::Linear);
    for t in [0.0, 1.0] {
        d.insert_key(11, p, RationalTime::from_seconds(t), v, i);
    }
    let inst = Instante {
        t_cut: 0.0,
        solo: true,
    };
    marcas(&d, &[11], inst, |_, _| Some(1.0)).get(&id).copied()
}

/// ⭐ Cada campo acende com cada canal que o seu escritor grava, e com nenhum canal de outra linha.
/// *Mutação: o pivô volta a levar `POSICAO` no grupo do losango ⇒ uma chave na Position acende o Pivot.*
#[test]
fn o_losango_de_cada_campo_e_o_canal_que_o_seu_escritor_grava() {
    let tabela = escritores();
    for (id, grava, linha) in &tabela {
        assert!(!grava.is_empty(), "{id:?}: sem canal no catálogo");
        assert!(!canal_do_campo(*id).is_empty(), "{id:?}: sem canal");
        for &p in grava {
            assert_eq!(marca_so_com(*id, p), Some(KeyMark::Chave), "{id:?} × {p:?}");
        }
        let alheios = tabela.iter().filter(|(_, _, l)| l != linha);
        let alheios = alheios
            .flat_map(|(_, g, _)| g)
            .filter(|p| !grava.contains(p));
        for &p in alheios {
            assert_ne!(
                marca_so_com(*id, p),
                Some(KeyMark::Chave),
                "{id:?} × {p:?} alheio"
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

/// A chave do rótulo que o Inspector pinta ao lado do campo (`sections/look.rs`, `cards_*`).
fn rotulo_do_inspector(id: NodeId) -> &'static str {
    use insp::*;
    match id {
        INSP_TRANSFORM_POS_X | INSP_TRANSFORM_POS_Y => "card.insp.position_px",
        INSP_TRANSFORM_ROT => "card.insp.rotation_deg",
        INSP_SPRITE_TINT_SWATCH => "component.field.sprite.1",
        INSP_SPRITE_SELF_TINT_SWATCH => "component.field.sprite.6",
        INSP_SPRITE_OPACITY | INSP_SPRITE_OPACITY_CHIP => "card.insp.opacity",
        INSP_SPRITE_TINT_FILL => "card.insp.silhouette",
        INSP_SPRITE_FLIP_X | INSP_SPRITE_FLIP_Y => "card.insp.flip",
        id if INSP_SPRITE_PIVOT_XY.contains(&id) => "card.insp.pivot_px",
        _ => "component.field.sprite_corner_tint.5",
    }
}

/// ⭐ **O canal diz o nome do campo cujo escritor o grava** (plano Timeline 17, item 9), sem a unidade
/// e sem o eixo. *Mutação: `DONOS` sprite.7 volta a canal OBJETO ⇒ a faixa do slider não diz «Opacity».*
#[test]
fn o_canal_diz_o_nome_do_campo_que_o_grava() {
    let sem_unidade = |s: &str| s.split(" (").next().unwrap_or_default().to_owned();
    let mentiras: Vec<String> = escritores()
        .into_iter()
        .flat_map(|(id, grava, _)| {
            let campo = sem_unidade(ph2d_i18n::tr(rotulo_do_inspector(id)));
            let faixas = grava
                .into_iter()
                .map(|p| sem_unidade(&ph2d_panel_timeline::nome_do_canal(p)));
            let mente =
                move |f: String| (!f.starts_with(&campo)).then(|| format!("«{campo}» × «{f}»"));
            faixas.filter_map(mente)
        })
        .collect();
    // ⚠️ Do dono (10/11): «Flip image» + ícones H/V lê-se «Flip H/V» na faixa; resolvido, a lista esvazia.
    let pendentes = ["«Flip image» × «Flip H»", "«Flip image» × «Flip V»"];
    assert_eq!(
        mentiras, pendentes,
        "o canal mente sobre o campo que o grava"
    );
}
