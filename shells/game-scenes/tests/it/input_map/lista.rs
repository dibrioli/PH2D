//! **A · List** pelas portas reais (plano §3): Add (clique e Enter, recusa com o porquê), a escuta no lugar
//! (tecla, Esc), × e lixo no hover, a busca por tecla com o pé, e o arrasto da zona morta.

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_editor_core::doc_surface::Key as DocKey;
use ph2d_input::{ActionId, Binding, GamepadAxis, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

fn id(m: &Mesa, name: &str) -> ActionId {
    m.hero.input_map.id(name).expect("a acção existe")
}

/// A mesa na vista A, com o mapa de fábrica.
fn lista() -> Mesa {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = ph2d_input::InputMap::with_player_defaults();
    m.frame();
    m
}

/// Fábrica + fire `J`, reload sem tecla, laser `H`, e o move_left com a manche esquerda (o caso analógico).
pub(super) fn onze() -> ph2d_input::InputMap {
    let mut map = ph2d_input::InputMap::with_player_defaults();
    for (name, key) in [("fire", Some('J')), ("reload", None), ("laser", Some('H'))] {
        let a = map.create(name);
        if let (Some(k), Some(act)) = (key, map.get_mut(a)) {
            act.bindings.push(Binding::Key(Key(u32::from(k))));
        }
    }
    let ml = map.id("move_left").expect("fábrica");
    if let Some(a) = map.get_mut(ml) {
        a.bindings.push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: false,
        });
        a.set_zone(0.15, 0.5);
    }
    map
}

fn shown_names(m: &Mesa) -> Vec<String> {
    ph2d_app_input_map::view_list::shown(&m.tab().doc)
        .into_iter()
        .filter_map(|a| m.hero.input_map.get(a).map(|x| x.name.clone()))
        .collect()
}

fn painted(m: &Mesa, s: &str) -> bool {
    m.tab().doc.census.texts.iter().any(|t| t.1 == s)
}

#[test]
fn escrever_um_nome_e_carregar_em_add_cria_a_accao() {
    let mut m = lista();
    let field = m.at(ids::NEW_NAME);
    assert!(m.click(field));
    m.type_text("fire");
    let add = m.at(ids::ADD);
    m.click(add);
    assert!(m.hero.input_map.id("fire").is_some(), "o Add criou a acção");
    assert_eq!(m.tab().doc.field_text(ids::NEW_NAME), "", "o campo esvazia");
}

#[test]
fn enter_no_nome_novo_tambem_adiciona() {
    let mut m = lista();
    let field = m.at(ids::NEW_NAME);
    m.click(field);
    m.type_text("dodge");
    assert!(m.key(DocKey::Enter, None));
    assert!(m.hero.input_map.id("dodge").is_some());
}

#[test]
fn nome_repetido_ou_vazio_e_recusado_com_o_porque_a_vista() {
    let mut m = lista();
    let n = m.hero.input_map.len();
    let field = m.at(ids::NEW_NAME);
    m.click(field);
    m.key(DocKey::Enter, None);
    assert_eq!(m.hero.input_map.len(), n, "vazio não cria");
    assert!(
        painted(&m, ph2d_i18n::tr("input_map_list.empty_name")),
        "o porquê está pintado"
    );
    m.type_text("jump");
    m.key(DocKey::Enter, None);
    assert_eq!(m.hero.input_map.len(), n, "repetido não cria");
    assert!(painted(&m, ph2d_i18n::tr("input_map_list.duplicate")));
}

#[test]
fn o_mais_escuta_no_lugar_e_a_tecla_seguinte_vira_ligacao_da_linha() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let reload = id(&m, "reload");
    let plus = m.at(ListHit::Listen(reload).id());
    m.click(plus);
    assert_eq!(
        m.hero.store.input_map_listening(),
        Some(reload),
        "a escuta partilhada armou"
    );
    assert!(
        painted(&m, ph2d_i18n::tr("input_map_list.listen_key")),
        "sem comando: «Press a key.»"
    );
    assert!(painted(&m, "Listening for reload."), "o pé diz quem escuta");
    assert!(m.physical(Key(u32::from('R'))));
    let a = m.hero.input_map.get(reload).expect("reload");
    assert_eq!(a.bindings, vec![Binding::Key(Key(u32::from('R')))]);
    assert!(m.hero.store.input_map_listening().is_none(), "desarmou");
}

#[test]
fn com_comando_ligado_a_escuta_promete_o_comando_e_esc_cancela() {
    let mut m = lista();
    m.gamepad = true;
    let dash = id(&m, "dash");
    let before = m.hero.input_map.get(dash).expect("dash").bindings.clone();
    let plus = m.at(ListHit::Listen(dash).id());
    m.click(plus);
    assert!(painted(&m, ph2d_i18n::tr("input_map_list.listen_pad")));
    m.physical(named("escape"));
    assert!(m.hero.store.input_map_listening().is_none());
    assert_eq!(
        m.hero.input_map.get(dash).expect("dash").bindings,
        before,
        "Esc não liga nada"
    );
}

#[test]
fn o_x_do_chip_tira_a_ligacao_e_o_lixo_tira_a_accao() {
    let mut m = lista();
    let jump = id(&m, "jump");
    let row = m.at(ListHit::Row(jump).id());
    m.hover(row);
    let x = m.at(ListHit::RemoveBinding(jump, 0).id());
    m.click(x);
    let left: Vec<Binding> = m.hero.input_map.get(jump).expect("jump").bindings.clone();
    assert_eq!(
        left,
        vec![Binding::Key(Key(u32::from('Z')))],
        "só o Z ficou"
    );
    m.hover(row);
    let trash = m.at(ListHit::Trash(jump).id());
    m.click(trash);
    assert!(
        m.hero.input_map.id("jump").is_none(),
        "o lixo apagou a acção"
    );
}

#[test]
fn a_busca_por_tecla_deixa_jump_e_move_up_e_o_pe_diz_quem_le() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let search = m.at(ids::SEARCH);
    m.click(search);
    assert!(m.physical(named("up")), "a seta é da busca");
    assert_eq!(shown_names(&m), ["jump", "move_up"]);
    assert!(painted(&m, "Up Arrow is in 2 of 11 actions."));
    assert!(painted(
        &m,
        "jump is read by Platform Player, move_up by Top-Down Player."
    ));
    // Uma letra só também filtra pela tecla com essa letra.
    let clear = m.at(ids::SEARCH_CLEAR);
    m.click(clear);
    m.type_text("h");
    assert!(
        shown_names(&m).contains(&"laser".to_owned()),
        "H é a tecla do laser"
    );
}

#[test]
fn arrastar_a_zona_morta_muda_a_accao_e_e_um_gesto_vivo_ate_soltar() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let ml = id(&m, "move_left");
    let dead = ListHit::Dead(ml).id();
    let r = m.tab().doc.rect_of(dead).expect("o campo pintado");
    m.click((r.x + 1.0, r.y + r.h * 0.5));
    assert!(ph2d_app_input_map::gesture_in_progress(&m.hero));
    m.hover((r.x + 1.0 + r.w * 0.25, r.y + r.h * 0.5));
    let a = m.hero.input_map.get(ml).expect("move_left");
    assert!(
        (a.dead_zone - 0.4).abs() < 0.01,
        "0.15 + um quarto do campo = {}",
        a.dead_zone
    );
    assert!(a.press_point >= a.dead_zone, "a lei do set_zone");
    m.release();
    assert!(!ph2d_app_input_map::gesture_in_progress(&m.hero));
}

#[test]
fn uma_linha_so_de_teclas_nao_tem_dead_nem_press() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let jump = id(&m, "jump");
    assert!(m.tab().doc.rect_of(ListHit::Dead(jump).id()).is_none());
    assert!(m.tab().doc.rect_of(ListHit::Press(jump).id()).is_none());
    let ml = id(&m, "move_left");
    assert!(m.tab().doc.rect_of(ListHit::Dead(ml).id()).is_some());
}

#[test]
fn o_pe_da_busca_nomeia_o_trigger_e_o_script_que_leem_a_accao() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    let row = ph2d_ecs::ActionTriggerRow {
        action: "fire".to_owned(),
        ..Default::default()
    };
    m.sim.world_mut().spawn((
        ph2d_ecs::Name("Door".to_owned()),
        ph2d_ecs::SignalOnAction(vec![row]),
    ));
    let mut lib = ph2d_script::ScriptLibrary::default();
    lib.insert("laser_gun", "if ph2d.action_down(\"laser\") then end");
    m.sim
        .world_mut()
        .insert_resource(ph2d_script::ProjectScripts::new(lib));
    m.frame();
    let search = m.at(ids::SEARCH);
    m.click(search);
    m.type_text("j");
    m.physical(Key(u32::from('J')));
    // «j» escreve (não é tecla de filtro); a busca de uma letra mostra o fire pela tecla J.
    assert!(shown_names(&m).contains(&"fire".to_owned()));
    let readers = &m.tab().doc.snap.readers;
    assert!(readers.get("fire").is_some_and(|r| r.iter().any(
        |x| matches!(x, ph2d_input::readers::Reader::Trigger { object } if object == "Door")
    )));
    assert!(readers.get("laser").is_some_and(|r| r.iter().any(
        |x| matches!(x, ph2d_input::readers::Reader::Script { name } if name == "laser_gun")
    )));
}

#[test]
fn cada_controlo_da_lista_tem_dica_em_ingles_simples() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let ml = id(&m, "move_left");
    let row = m.at(ListHit::Row(ml).id());
    m.hover(row);
    for target in [
        ids::NEW_NAME,
        ids::ADD,
        ids::SEARCH,
        ListHit::Listen(ml).id(),
        ListHit::RemoveBinding(ml, 0).id(),
        ListHit::Trash(ml).id(),
        ListHit::Dead(ml).id(),
        ListHit::Press(ml).id(),
    ] {
        let at = m.at(target);
        m.hover(at);
        assert_eq!(
            m.tab().doc.list.hover_target,
            Some(target),
            "o ponteiro achou o alvo"
        );
        let tip = ph2d_app_input_map::view_list::tip_of(&m.tab().doc, target).expect("dica");
        assert!(
            tip.ends_with('.') && !tip.contains("input_map"),
            "dica em frase: {tip}"
        );
        // Volta à linha para o × e o lixo continuarem pintados.
        m.hover(row);
    }
}
