//! O Lock por campo atravessa o painel de cartões: a mão em cima do campo, a tecla na aba, e o sorteio respeita.

use crate::Mesa;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_host::{Modifiers, PointerKind};
use ph2d_panel_sfxlab::campo_sob_o_mouse;
use ph2d_panel_sfxlab::tabela::{self, CAMPOS};
use ph2d_sfx_recipe::{Recipe, factory};
use ph2d_tool_registry::hash_node_id;

fn metal() -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Metal Hit")
        .map(|(_, r)| r)
        .expect("som de fábrica")
}

fn tecla(m: &mut Mesa, c: char) -> bool {
    let k = BoardKey::Char(c);
    let mods = Modifiers::default();
    let out = board_keys::key(&mut m.hero, &mut m.ts, k, mods, true, None, None);
    let _ = board_keys::key(&mut m.hero, &mut m.ts, k, mods, false, None, None);
    m.quadro();
    out.consumed
}

fn poe_a_mao(m: &mut Mesa, id: ph2d_editor_core::NodeId) {
    let r = m
        .rect(id)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    m.mao(PointerKind::Move, r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.quadro();
    m.quadro();
}

#[test]
fn l_trava_o_campo_sob_o_mouse_e_o_roll_o_respeita() {
    let mut m = Mesa::nova();
    let id = m.abre("Metal Hit", metal());
    let base = CAMPOS
        .iter()
        .find(|c| c.id == hash_node_id("sfxlab_body_base"))
        .expect("campo");
    poe_a_mao(&mut m, base.id);
    assert_eq!(
        campo_sob_o_mouse(),
        Some(base.id),
        "o painel diz o que está sob a mão"
    );

    assert!(tecla(&mut m, 'l'));
    let travada = m.camada(id).expect("camada");
    assert_eq!(travada.travas, vec![base.id.0]);
    let valor = base.le(&m.receita(id), Some(&travada));
    let outro = CAMPOS
        .iter()
        .find(|c| c.id == hash_node_id("sfxlab_body_decay"))
        .expect("campo");
    let outro_antes = outro.le(&m.receita(id), Some(&travada));

    for _ in 0..30 {
        assert!(tecla(&mut m, 'r'));
    }
    let depois = m.camada(id).expect("camada");
    assert_eq!(
        base.le(&m.receita(id), Some(&depois)),
        valor,
        "o travado não anda"
    );
    assert_ne!(
        outro.le(&m.receita(id), Some(&depois)),
        outro_antes,
        "controle: o livre anda"
    );

    // A mesma tecla, no mesmo campo, destrava.
    poe_a_mao(&mut m, base.id);
    assert!(tecla(&mut m, 'l'));
    assert!(m.camada(id).expect("camada").travas.is_empty());
}

#[test]
fn l_trava_a_escolha_sob_o_mouse_e_o_roll_nao_a_troca() {
    let mut m = Mesa::nova();
    let id = m.abre("Metal Hit", metal());
    let dela = tabela::OPCOES
        .iter()
        .find(|o| o.grupo == tabela::Grupo::Law && (o.aceso)(&metal().layers[0]) == Some(true))
        .expect("opção acesa");
    poe_a_mao(&mut m, dela.id);
    assert!(tecla(&mut m, 'l'));
    assert_eq!(m.camada(id).expect("camada").travas, vec![dela.id.0]);
    for _ in 0..60 {
        assert!(tecla(&mut m, 'r'));
    }
    assert_eq!((dela.aceso)(&m.camada(id).expect("camada")), Some(true));
}

#[test]
fn com_o_campo_focado_o_digito_e_dele_e_o_explore_nao_sorteia() {
    let mut m = Mesa::nova();
    let id = m.abre("Metal Hit", metal());
    let base = hash_node_id("sfxlab_body_base");
    m.digita(base, "250");
    let c = tabela::campo(base).expect("campo");
    let receita = m.receita(id);
    assert_eq!(c.le(&receita, receita.layers.first()), Some(250.0));
    assert_eq!(
        m.docs().exploracao(id).expect("aba").sessao.ouvidos.len(),
        0
    );
}

#[test]
fn o_explore_nao_tira_os_cartoes_nem_os_medidores() {
    let mut m = Mesa::nova();
    let id = m.abre("Metal Hit", metal());
    let k = BoardKey::Tab;
    let mods = Modifiers::default();
    assert!(board_keys::key(&mut m.hero, &mut m.ts, k, mods, true, None, None).consumed);
    let _ = board_keys::key(&mut m.hero, &mut m.ts, k, mods, false, None, None);
    m.assenta();
    assert!(m.docs().exploracao(id).expect("aba").aberta());
    assert!(
        m.rect(hash_node_id("sfxlab_body_base")).is_some(),
        "os cartões seguem"
    );
    // Mexer num campo dentro do Explore refaz as oito em volta do som novo.
    m.digita(hash_node_id("sfxlab_body_base"), "250");
    m.assenta();
    let receita = m.receita(id);
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("grade");
    assert_eq!(g.base, receita);
}
