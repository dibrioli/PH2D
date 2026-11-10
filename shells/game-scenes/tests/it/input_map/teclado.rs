//! **B · Keyboard** pelas portas reais (plano §3, maquete B): abrir uma acção na lista, clicar numa tecla
//! desenhada, uma barra por acção, o pé da tecla sob o ponteiro, + Add / Remove, o ladrilho que escuta, o
//! comando, o aviso de quem ninguém lê, o arrasto da barra do curso e a cor presa ao `ActionId`.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::doc_surface::Key as DocKey;
use ph2d_input::{ActionId, Binding, GamepadButton, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

fn id(m: &Mesa, name: &str) -> ActionId {
    m.hero.input_map.id(name).expect("a acção existe")
}

fn bindings(m: &Mesa, name: &str) -> Vec<Binding> {
    m.hero
        .input_map
        .get(id(m, name))
        .map(|a| a.bindings.clone())
        .unwrap_or_default()
}

/// A mesa na vista B com o mapa `map`.
fn teclado(map: ph2d_input::InputMap) -> Mesa {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = map;
    m.frame();
    m
}

fn abre(m: &mut Mesa, name: &str) {
    let a = id(m, name);
    m.click(m.at(KbHit::Row(a).id()));
    assert_eq!(m.tab().doc.kb.open, Some(a), "{name} aberta pela lista");
}

fn texts(m: &Mesa) -> Vec<String> {
    m.tab()
        .doc
        .census
        .texts
        .iter()
        .map(|t| t.1.clone())
        .collect()
}

#[test]
fn a_lista_abre_a_acao_e_a_tecla_desenhada_entra_nela_sem_repetir() {
    let mut m = teclado(ph2d_input::InputMap::with_player_defaults());
    abre(&mut m, "jump");
    let x = Key(u32::from('X'));
    m.click(m.at(KbHit::Key(x).id()));
    assert!(
        bindings(&m, "jump").contains(&Binding::Key(x)),
        "X entrou no jump"
    );
    let n = bindings(&m, "jump").len();
    m.click(m.at(KbHit::Key(x).id()));
    assert_eq!(bindings(&m, "jump").len(), n, "clicar de novo não repete");
    assert!(
        texts(&m).iter().any(|t| t == "jump · 3 keys"),
        "o título conta: {:?}",
        texts(&m)
    );
}

#[test]
fn cada_tecla_leva_uma_barra_por_acao_que_a_usa() {
    let m = teclado(ph2d_input::InputMap::with_player_defaults());
    let map = &m.hero.input_map;
    for (hit, who) in &m.tab().doc.kb.bars {
        let KbHit::Key(k) = hit else { continue };
        let want: Vec<ActionId> = map
            .actions()
            .iter()
            .filter(|a| a.bindings.contains(&Binding::Key(*k)))
            .map(|a| a.id)
            .collect();
        assert_eq!(who, &want, "{}", k.label());
    }
    let up = m
        .tab()
        .doc
        .kb
        .bars
        .iter()
        .find(|(h, _)| *h == KbHit::Key(named("up")))
        .map(|(_, w)| w.len());
    assert_eq!(up, Some(2), "a seta para cima leva jump e move_up");
}

#[test]
fn o_ponteiro_numa_tecla_diz_quem_a_usa_e_quem_le() {
    let mut m = teclado(ph2d_input::InputMap::with_player_defaults());
    m.hover(m.at(KbHit::Key(named("up")).id()));
    let t = texts(&m);
    assert!(t.iter().any(|s| s == "Up Arrow:"), "{t:?}");
    assert!(
        t.iter()
            .any(|s| s == "jump (Platform Player) · move_up (Top-Down Player)."),
        "{t:?}"
    );
}

#[test]
fn add_abre_o_nome_no_lugar_cria_e_abre_e_remove_apaga() {
    let mut m = teclado(ph2d_input::InputMap::with_player_defaults());
    m.click(m.at(KbHit::Add.id()));
    assert_eq!(m.tab().doc.focus(), Some(ph2d_app_input_map::ids::NEW_NAME));
    m.type_text("reload");
    m.key(DocKey::Enter, None);
    m.frame();
    let r = id(&m, "reload");
    assert_eq!(m.tab().doc.kb.open, Some(r), "a acção nova abre-se");
    assert!(!m.tab().doc.kb.adding, "o campo fecha");
    m.click(m.at(KbHit::Remove.id()));
    assert!(
        m.hero.input_map.id("reload").is_none(),
        "Remove apaga a aberta"
    );
}

#[test]
fn o_ladrilho_que_escuta_liga_a_tecla_e_quem_ninguem_le_avisa() {
    let mut m = teclado(onze());
    abre(&mut m, "reload");
    let t = texts(&m);
    assert!(
        t.iter().any(|s| s == "Click a key above, or press one"),
        "{t:?}"
    );
    assert!(
        t.iter().any(|s| s
            == "Nobody reads reload yet. A Trigger can turn it into a signal: type reload in its Action."),
        "{t:?}"
    );
    assert!(t.iter().any(|s| s == "Nobody reads it yet"), "{t:?}");
    m.click(m.at(KbHit::Listen.id()));
    assert_eq!(m.tab().doc.snap.listening, Some(id(&m, "reload")));
    let k = Key(u32::from('K'));
    assert!(m.physical(k), "a escuta toma a tecla");
    assert_eq!(bindings(&m, "reload"), vec![Binding::Key(k)]);
}

#[test]
fn no_comando_clicar_no_a_liga_o_botao_de_baixo() {
    let mut m = teclado(ph2d_input::InputMap::with_player_defaults());
    abre(&mut m, "dash");
    m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
    assert_eq!(m.tab().doc.kb.sub, Sub::Gamepad);
    let a = Binding::PadButton(GamepadButton::South);
    m.click(m.at(KbHit::Pad(a).id()));
    assert!(
        bindings(&m, "dash").contains(&a),
        "{:?}",
        bindings(&m, "dash")
    );
    assert!(texts(&m).iter().any(|s| s == "Press a button"));
}

#[test]
fn arrastar_a_marca_da_zona_morta_muda_o_numero_num_gesto() {
    let mut m = teclado(onze());
    abre(&mut m, "move_left");
    let before = m
        .hero
        .input_map
        .get(id(&m, "move_left"))
        .map(|a| a.dead_zone);
    let (x, y) = m.at(KbHit::BarDead.id());
    let w = m
        .tab()
        .doc
        .rect_of(ph2d_app_input_map::ids::ListHit::Dead(id(&m, "move_left")).id())
        .map(|r| r.w)
        .expect("o campo");
    m.click((x, y));
    assert!(m.tab().doc.gesture_in_progress(), "o arrasto é um gesto");
    m.hover((x + w * 0.2, y));
    m.release();
    assert!(!m.tab().doc.gesture_in_progress());
    let after = m
        .hero
        .input_map
        .get(id(&m, "move_left"))
        .map(|a| a.dead_zone);
    assert!(after > before, "{before:?} → {after:?}");
}

#[test]
fn a_cor_de_uma_acao_segue_o_id_depois_de_apagar_outra_antes() {
    let mut m = teclado(onze());
    let fire = id(&m, "fire");
    let j = Key(u32::from('J'));
    let who = |m: &Mesa| {
        m.tab()
            .doc
            .kb
            .bars
            .iter()
            .find(|(h, _)| *h == KbHit::Key(j))
            .map(|(_, w)| w.clone())
    };
    assert_eq!(who(&m), Some(vec![fire]));
    let color = color_of(fire);
    abre(&mut m, "move_left");
    m.click(m.at(KbHit::Remove.id()));
    assert!(m.hero.input_map.id("move_left").is_none());
    assert_eq!(who(&m), Some(vec![fire]), "a barra do J continua a do fire");
    assert_eq!(color_of(fire), color);
}

fn color_of(a: ActionId) -> ph2d_tokens::ColorToken {
    ph2d_tokens::categorical(a.0)
}

#[test]
fn a_acao_sem_tecla_toma_a_tecla_premida_e_o_que_nao_esta_desenhado_aparece_em_also() {
    let mut m = teclado(onze());
    abre(&mut m, "reload");
    let k = Key(u32::from('1'));
    assert!(m.physical(k), "a acção vazia toma a tecla");
    assert_eq!(bindings(&m, "reload"), vec![Binding::Key(k)]);
    let t = texts(&m);
    assert!(
        t.iter().any(|s| s == "Also:") && t.iter().any(|s| s == "1"),
        "{t:?}"
    );
    assert!(
        !m.physical(Key(u32::from('2'))),
        "com tecla, a acção já não toma a premida"
    );
}
