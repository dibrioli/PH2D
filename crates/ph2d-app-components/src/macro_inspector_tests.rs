//! A ida (edição → componente) e a volta (componente → instantâneo) da secção MACRO.

use ph2d_ecs::{Macro, ModoDaMacro, SimWorld};
use ph2d_editor_core::macro_edits::{
    MACRO_DE_VERSAO_NOVA, MACRO_ESTRAGADA, MACRO_MODOS, MODO_EXPLICATIVO, MODO_FIEL,
    MacroFieldEdit as E, MacroGravacao,
};
use ph2d_macro::{MacroRecorder, RefRect};

use super::{apply, apply_all, build_info, gravacao, modo_de_u8, u8_do_modo};

fn cena(m: Macro) -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(m).id();
    (sim, e.to_bits())
}

/// Uma macro de duas trilhas: o Motion com 2 passos (o último aos 40 ms) e o Painter com 1.
fn gravada() -> Vec<u8> {
    let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let motion = rec.open_track("motion", 7, vec![1]);
    let painter = rec.open_track("painter", 7, vec![2]);
    rec.push(motion, 0, vec![3]).expect("tempo");
    rec.push(motion, 40_000, vec![4]).expect("tempo");
    rec.push(painter, 10_000, vec![5]).expect("tempo");
    ph2d_macro::codec::write(&rec.finish())
}

#[test]
fn o_modo_vai_e_volta_pelo_indice() {
    assert_eq!(MACRO_MODOS, 2);
    for (i, m) in [
        (MODO_FIEL, ModoDaMacro::Fiel),
        (MODO_EXPLICATIVO, ModoDaMacro::Explicativo),
    ] {
        assert_eq!(modo_de_u8(i), Some(m));
        assert_eq!(u8_do_modo(m), i);
    }
    assert_eq!(modo_de_u8(2), None, "fora da faixa é inerte");
}

#[test]
fn o_instantaneo_le_a_gravacao() {
    let (sim, bits) = cena(Macro {
        doc: gravada(),
        modo: ModoDaMacro::Explicativo,
    });
    let info = build_info(&sim, bits, 1).expect("info");
    assert_eq!(info.entity_bits, bits);
    assert_eq!(info.modo, MODO_EXPLICATIVO);
    assert_eq!(
        info.gravacao,
        MacroGravacao::Lida {
            trilhas: vec![("motion".into(), 2), ("painter".into(), 1)],
            duracao_us: 40_000,
        }
    );
    assert_eq!(
        (info.etapas, info.editada_desde, info.erro),
        (0, None, None),
        "as etapas são da família MACROS: a shell completa-as"
    );
    assert_eq!(gravacao(&[]), MacroGravacao::Vazia);
}

#[test]
fn uma_gravacao_que_nao_se_le_diz_porque() {
    let mut nova = gravada();
    // A major vem logo a seguir à assinatura `PH2DMACRO` (9 bytes), em little-endian.
    nova[9] = nova[9].wrapping_add(1);
    assert_eq!(
        gravacao(&nova),
        MacroGravacao::Ilegivel(MACRO_DE_VERSAO_NOVA)
    );
    assert_eq!(gravacao(b"lixo"), MacroGravacao::Ilegivel(MACRO_ESTRAGADA));
    for chave in [MACRO_DE_VERSAO_NOVA, MACRO_ESTRAGADA] {
        assert_ne!(ph2d_i18n::tr(chave), chave, "a frase de {chave} existe");
    }
}

#[test]
fn a_edicao_muda_o_modo_so_quando_muda() {
    let (mut sim, bits) = cena(Macro::default());
    assert!(!apply(&mut sim, bits, &E::Modo(MODO_FIEL)), "o mesmo valor");
    assert!(!apply(&mut sim, bits, &E::Modo(9)), "fora da faixa");
    assert!(apply_all(&mut sim, &[(bits, E::Modo(MODO_EXPLICATIVO))]));
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(
        sim.world().get::<Macro>(e).map(|m| m.modo),
        Some(ModoDaMacro::Explicativo)
    );
    let (mut sem, outro) = cena(Macro::default());
    sem.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(outro))
        .remove::<Macro>();
    assert!(
        !apply(&mut sem, outro, &E::Modo(MODO_EXPLICATIVO)),
        "sem o componente"
    );
}
