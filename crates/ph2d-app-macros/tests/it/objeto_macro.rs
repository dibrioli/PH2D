//! W18 (`docs/Macros/16_W18_plano.md` §2.2): o Gravar sobre um objecto Macro grava o que as famílias
//! fazem DENTRO dele — a trilha da imagem leva a ficha de nascimento; fora dele, recusa em voz alta.

use crate::gate7::Banca;
use ph2d_ecs::{ChildOf, Entity, Macro, Name, StableId, Transform};
use ph2d_editor_core::macro_edits::{self as m, MacroRequest};

const IMAGEM: u64 = 10;
const OBJETO: u64 = 20;

/// O objecto Macro (vazio, como o Add o faz) e a imagem da banca; `dentro` = ela é filha dele.
fn monta(b: &mut Banca, dentro: bool) -> u64 {
    let w = b.sim.world_mut();
    let macro_ = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Macro"),
            Macro::default(),
            StableId(OBJETO),
        ))
        .id();
    let mut img = w.entity_mut(Entity::from_bits(b.bits));
    img.insert((Transform::IDENTITY, Name::new("Tela"), StableId(IMAGEM)));
    if dentro {
        img.insert(ChildOf(macro_));
    }
    b.escolhido = Some(macro_.to_bits());
    macro_.to_bits()
}

fn grava(b: &mut Banca) {
    b.m.pede(MacroRequest::Grava);
    b.quadro();
    assert!(b.m.armado(), "o objecto Macro arma vazio");
    b.pincelada([20.0, 30.0], [100.0, 30.0]);
    b.m.pede(MacroRequest::Para);
    b.quadro();
}

fn doc_de(b: &Banca, bits: u64) -> Option<ph2d_macro::MacroDoc> {
    let m = b.sim.world().get::<Macro>(Entity::from_bits(bits))?;
    (!m.doc.is_empty()).then(|| ph2d_macro::codec::read(&m.doc).expect("lê"))
}

#[test]
fn gravar_dentro_do_objeto_macro_da_a_macro_ao_objeto_e_a_ficha_a_imagem() {
    let mut b = Banca::nova();
    let dono = monta(&mut b, true);
    grava(&mut b);
    let doc = doc_de(&b, dono).expect("o objecto Macro ficou com a gravação");
    assert_eq!(doc.tracks.len(), 1);
    let t = &doc.tracks[0];
    assert_eq!(t.family, ph2d_tool_painter::macro_track::FAMILIA);
    assert_eq!(t.target, IMAGEM, "a trilha aponta a imagem pelo StableId");
    let n = t.nasce.as_ref().expect("a imagem nasce dentro do objecto Macro");
    assert_eq!(n.nome, "Tela");
    assert_eq!(n.inicio.0, 0, "ela já existia ao Gravar");
    assert_eq!(n.local, [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]);
    assert!(!t.events.is_empty());
    assert!(doc_de(&b, b.bits).is_none(), "a imagem não ficou com Macro própria");
}

#[test]
fn fora_do_objeto_macro_recusa_em_voz_alta_e_nada_se_grava() {
    let mut b = Banca::nova();
    let dono = monta(&mut b, false);
    b.m.pede(MacroRequest::Grava);
    b.quadro();
    b.pincelada([20.0, 30.0], [100.0, 30.0]);
    b.pinta();
    assert_eq!(
        ph2d_panel_macros::current_macros().recusa,
        Some(m::MACRO_RECUSA_FORA_DO_OBJETO),
        "o painel diz porquê"
    );
    b.m.pede(MacroRequest::Para);
    b.quadro();
    b.pinta();
    assert_eq!(
        ph2d_panel_macros::current_macros().recusa,
        Some(m::MACRO_RECUSA_NADA_GRAVADO)
    );
    assert!(doc_de(&b, dono).is_none(), "o objecto Macro continua vazio");
    assert!(doc_de(&b, b.bits).is_none());
}

#[test]
fn o_que_nasce_a_gravar_fica_dentro_do_objeto_macro_e_a_ficha_diz_quando() {
    let mut b = Banca::nova();
    let dono = monta(&mut b, false);
    b.m.pede(MacroRequest::Grava);
    b.quadro();
    for _ in 0..30 {
        b.quadro();
    }
    // A imagem nasce agora (o Add / a imagem nova da shell chamam `nasceu`).
    assert!(b.m.nasceu(&mut b.sim, b.bits));
    let pai = b
        .sim
        .world()
        .get::<ChildOf>(Entity::from_bits(b.bits))
        .map(|c| c.0.to_bits());
    assert_eq!(pai, Some(dono), "nasceu dentro do objecto Macro");
    b.pincelada([20.0, 30.0], [100.0, 30.0]);
    b.m.pede(MacroRequest::Para);
    b.quadro();
    let doc = doc_de(&b, dono).expect("gravou");
    let n = doc.tracks[0].nasce.clone().expect("com ficha");
    assert!(
        n.inicio.0 >= 30 * 16_667,
        "a ficha guarda quando nasceu: {} µs",
        n.inicio.0
    );
}
