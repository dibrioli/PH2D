//! R9a — os defeitos confirmados pela auditoria (10/11) no estado partilhado da aba e na A · List, e a faixa do Play.
//! Cada teste afirma o comportamento CERTO e foi visto VERMELHO antes da cura.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_app_input_map::view_list::state::{Typing, Zone};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key as DocKey};
use ph2d_input::{InputMap, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

#[test]
fn defeito_1_trocar_de_vista_com_a_escuta_armada_desarma_a() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let jump = m.hero.input_map.id("jump").expect("jump");
    let antes = m.hero.input_map.get(jump).expect("jump").bindings.clone();
    m.click(m.at(ListHit::Listen(jump).id()));
    assert_eq!(m.hero.store.input_map_listening(), Some(jump));
    m.click(m.segment(View::TryIt));
    m.physical(Key(u32::from('W')));
    assert_eq!(
        m.hero.input_map.get(jump).expect("jump").bindings,
        antes,
        "a tecla do bench do Try it ligou-se em silêncio ao jump"
    );
}

#[test]
fn defeito_1b_fechar_a_aba_com_a_escuta_armada_desarma_a() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let jump = m.hero.input_map.id("jump").expect("jump");
    m.click(m.at(ListHit::Listen(jump).id()));
    m.tab_mut().close(ph2d_app_input_map::TAB_ID);
    m.frame();
    assert!(
        m.hero.store.input_map_listening().is_none(),
        "aba fechada e a próxima tecla do editor viraria ligação do jump"
    );
}

#[test]
fn a_escuta_do_trigger_sobrevive_a_troca_de_vista_e_ao_fecho_da_aba() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let jump = m.hero.input_map.id("jump").expect("jump");
    m.hero.store.listen_for_binding_replacing(jump);
    m.click(m.segment(View::TryIt));
    m.tab_mut().close(ph2d_app_input_map::TAB_ID);
    m.frame();
    assert_eq!(m.hero.store.input_map_listening(), Some(jump));
}

#[test]
fn defeito_3_o_shift_de_uma_maiuscula_nao_apaga_o_que_se_escreveu_na_busca() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(ids::LIST_SEARCH));
    m.type_text("move ");
    for k in ["left_shift", "left_ctrl", "left_alt", "left"] {
        assert!(!m.physical(named(k)), "{k} com texto segue para o campo");
    }
    assert_eq!(
        m.tab().doc.field_text(ids::LIST_SEARCH),
        "move ",
        "o Shift para escrever uma maiúscula apagou a busca"
    );
    assert_eq!(m.tab().doc.list.search_key, None);
}

#[test]
fn com_a_busca_vazia_o_espaco_e_o_shift_viram_o_filtro_por_tecla() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(ids::LIST_SEARCH));
    let space = named("space");
    let took = ph2d_app_input_map::surface_raw_key(&mut m.hero, space.0, true, Some(" "));
    m.frame();
    assert!(took, "o Espaço com a busca vazia é a tecla procurada");
    assert_eq!(m.tab().doc.list.search_key, Some(space));
    assert!(m.physical(named("left_shift")));
    assert_eq!(m.tab().doc.list.search_key, Some(named("left_shift")));
}

#[test]
fn a_number_being_typed_never_lands_on_the_next_projects_action() {
    let mut m = Mesa::aberta(View::List);
    let jump = m.hero.input_map.id("jump").expect("jump");
    m.tab_mut().doc.list.typing = Some(Typing {
        action: jump,
        zone: Zone::Dead,
        buffer: "0.9".into(),
    });
    // Outro projecto, onde o MESMO id é outra acção; o que o `ao_hero` faz no load.
    let mut outro = InputMap::new();
    while outro.create(format!("other_{}", outro.len())) != jump {}
    m.hero.input_map = outro.clone();
    m.hero.store.stop_listening();
    m.frame();
    m.key(DocKey::Enter, None);
    assert_eq!(
        m.hero.input_map, outro,
        "o Enter escreveu a zona do `jump` do projecto anterior noutra acção"
    );
}

#[test]
fn um_undo_a_meio_de_escrever_um_numero_cancela_o_e_a_escuta() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let antes = m.hero.input_map.clone();
    let jump = m.hero.input_map.id("jump").expect("jump");
    m.tab_mut()
        .doc
        .pending
        .push(ph2d_app_input_map::InputMapEdit::Add("taunt".into()));
    m.frame();
    assert!(m.hero.input_map.id("taunt").is_some(), "o Add aplicou");
    m.tab_mut().doc.list.typing = Some(Typing {
        action: jump,
        zone: Zone::Press,
        buffer: "0.7".into(),
    });
    m.hero.store.listen_for_binding(jump);
    m.hero.input_map = antes.clone(); // o Ctrl+Z
    m.frame();
    assert!(m.tab().doc.list.typing.is_none(), "o número a meio caiu");
    assert!(m.hero.store.input_map_listening().is_none(), "e a escuta");
    m.key(DocKey::Enter, None);
    assert_eq!(m.hero.input_map, antes);
}

#[test]
fn a_vista_b_larga_a_accao_aberta_que_o_mapa_novo_nao_tem() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    let reload = m.hero.input_map.id("reload").expect("reload");
    m.tab_mut().doc.kb.open = Some(reload);
    m.hero.input_map = InputMap::new();
    m.frame();
    assert_eq!(m.tab().doc.kb.open, None);
}

#[test]
fn os_leitores_nao_seguem_a_ordem_dos_arquetipos() {
    use ph2d_ecs::{ActionTriggerRow, Name, SignalOnAction};
    let mut sim = ph2d_ecs::SimWorld::new();
    let row = |a: &str| ActionTriggerRow {
        action: a.into(),
        ..Default::default()
    };
    let _z = sim
        .world_mut()
        .spawn((Name("Zeta".into()), SignalOnAction(vec![row("jump")])))
        .id();
    let a = sim
        .world_mut()
        .spawn((Name("Alpha".into()), SignalOnAction(vec![row("jump")])))
        .id();
    let before = ph2d_app_input_map::readers::of_world(sim.world());
    let objs = |f: &ph2d_input::readers::ReaderFacts| {
        f.triggers.iter().map(|t| t.0.clone()).collect::<Vec<_>>()
    };
    assert_eq!(
        objs(&ph2d_app_input_map::readers::facts(sim.world())),
        ["Alpha", "Zeta"]
    );
    sim.world_mut().entity_mut(a).insert(ph2d_ecs::BlinkOff);
    assert_eq!(ph2d_app_input_map::readers::of_world(sim.world()), before);
}

const BANNER: &str = "The game is running. Changes here are tried live and undone when you stop.";

#[test]
fn com_o_play_a_correr_cada_vista_diz_no_topo_que_as_mudancas_se_desfazem_no_stop() {
    let mut m = Mesa::aberta(View::List);
    m.sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    for v in View::ALL {
        m.click(m.segment(v));
        m.frame();
        let c = &m.tab().doc.census;
        let banner = c.texts.iter().find(|t| t.1 == BANNER);
        let Some((r, _, cut)) = banner else {
            panic!("{v:?} sem a faixa do Play");
        };
        assert!(!cut, "{v:?}: a faixa cortada");
        assert!(
            r.y + r.h <= m.tab().doc.body().y,
            "{v:?}: a faixa sobre a vista"
        );
        assert!(
            c.filled.iter().any(
                |f| f.1 == ph2d_tokens::ColorToken::InputMapWarningBg && f.0.contains(r.x, r.y)
            ),
            "{v:?}: a faixa na cor do aviso"
        );
    }
    m.sim.world_mut().remove_resource::<ph2d_ecs::RunOnCopy>();
    m.frame();
    assert!(!m.tab().doc.census.texts.iter().any(|t| t.1 == BANNER));
}
