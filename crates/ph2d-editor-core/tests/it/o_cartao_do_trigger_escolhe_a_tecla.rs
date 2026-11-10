//! ⭐ **A tecla escolhida no cartão do Trigger chega ao Input Map** (onda 4 do F3, dono 09/11) —
//! pelas MESMAS portas do produto: o pedido do quadro, o `dispatch_key` e o `apply_event`.
//!
//! As leis: o pedido cria a acção que falta e arma a escuta · a tecla TROCA as de teclado (os
//! botões de comando ficam) · `Esc` cancela sem tocar em nada · a escuta da janela do mapa
//! continua a ACRESCENTAR · a linha Key lê a tecla e quem mais a usa.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_trigger_edits::{
    InspectorActionTriggerInfo, InspectorTriggerRow, NoMapa, TeclaDaAccao, le_teclas,
};
use ph2d_editor_core::interaction::dispatch::dispatch_key;
use ph2d_editor_core::screens::hero::{
    HeroScreen,
    chrome::{PedidoDoMapa, aplica_pedidos_do_mapa},
};
use ph2d_host::{KeyEvent, KeyKind, Modifiers};
use ph2d_input::{Binding, GamepadButton, Key};

const KEY_A: u32 = 0x41;
const KEY_Z: u32 = 0x5A;
const KEY_ESCAPE: u32 = 0x1B;
/// Um nome que o mapa de fábrica não traz.
const NOME: &str = "fire_test";

fn carrega(h: &mut HeroScreen, code: u32) {
    let arena = Bump::new();
    let ev = KeyEvent {
        keycode: code,
        kind: KeyKind::Down,
        modifiers: Modifiers::default(),
        timestamp_ns: 0,
    };
    let evts: Vec<_> = dispatch_key(&mut h.store, ev, &arena).to_vec();
    for e in evts {
        h.apply_event(e);
    }
}

fn ligacoes(h: &HeroScreen) -> Vec<Binding> {
    let id = h.input_map.id(NOME).expect("a acção existe");
    h.input_map.get(id).expect("existe").bindings.clone()
}

/// O artista já tinha `A` e o botão de baixo do comando; escolhe `Z` no cartão.
fn com_a_e_comando() -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    let id = h.input_map.create(NOME);
    let a = h.input_map.get_mut(id).expect("existe");
    a.bindings.push(Binding::Key(Key(KEY_A)));
    a.bindings.push(Binding::PadButton(GamepadButton::South));
    h
}

#[test]
fn o_pedido_cria_a_accao_que_falta_e_espera_a_tecla() {
    let mut h = HeroScreen::new(NodeId(1));
    assert!(h.input_map.id(NOME).is_none());
    aplica_pedidos_do_mapa(
        &mut h,
        vec![PedidoDoMapa::CriarEEscutar(format!("  {NOME} "))],
    );
    let id = h
        .input_map
        .id(NOME)
        .expect("a acção nasceu, com o nome aparado");
    assert_eq!(
        h.store.input_map_listening(),
        Some(id),
        "e a escuta ficou armada nela"
    );
    carrega(&mut h, KEY_Z);
    assert_eq!(ligacoes(&h), vec![Binding::Key(Key(KEY_Z))]);
    assert!(
        h.store.input_map_listening().is_none(),
        "e desarmou sozinha"
    );
}

#[test]
fn so_criar_nao_arma_a_escuta() {
    let mut h = HeroScreen::new(NodeId(1));
    aplica_pedidos_do_mapa(&mut h, vec![PedidoDoMapa::Criar(NOME.into())]);
    assert!(h.input_map.id(NOME).is_some());
    assert!(h.store.input_map_listening().is_none());
}

#[test]
fn a_tecla_do_cartao_troca_as_do_teclado_e_guarda_o_comando() {
    let mut h = com_a_e_comando();
    aplica_pedidos_do_mapa(&mut h, vec![PedidoDoMapa::CriarEEscutar(NOME.into())]);
    carrega(&mut h, KEY_Z);
    assert_eq!(
        ligacoes(&h),
        vec![
            Binding::PadButton(GamepadButton::South),
            Binding::Key(Key(KEY_Z))
        ],
        "o `A` saiu, o comando ficou, o `Z` entrou"
    );
}

#[test]
fn esc_no_cartao_cancela_sem_tocar_nas_teclas() {
    let mut h = com_a_e_comando();
    let antes = ligacoes(&h);
    aplica_pedidos_do_mapa(&mut h, vec![PedidoDoMapa::CriarEEscutar(NOME.into())]);
    carrega(&mut h, KEY_ESCAPE);
    assert_eq!(ligacoes(&h), antes);
    assert!(h.store.input_map_listening().is_none());
    // ⚠️ E a escuta seguinte da JANELA não herda a «troca» do cartão cancelado.
    let id = h.input_map.id(NOME).expect("existe");
    h.store.listen_for_binding(id);
    carrega(&mut h, KEY_Z);
    assert_eq!(ligacoes(&h).len(), 3, "a janela acrescenta");
}

#[test]
fn a_escuta_da_janela_continua_a_acrescentar() {
    let mut h = com_a_e_comando();
    let id = h.input_map.id(NOME).expect("existe");
    h.store.listen_for_binding(id);
    carrega(&mut h, KEY_Z);
    assert_eq!(
        ligacoes(&h),
        vec![
            Binding::Key(Key(KEY_A)),
            Binding::PadButton(GamepadButton::South),
            Binding::Key(Key(KEY_Z))
        ]
    );
}

fn fileira(action: &str) -> InspectorTriggerRow {
    InspectorTriggerRow {
        action: action.into(),
        edge: 0,
        signal: "s".into(),
        no_mapa: NoMapa::Ligada,
        item: 0,
        tecla: TeclaDaAccao::default(),
    }
}

/// A linha Key diz as teclas de TECLADO e a primeira que outra acção também usa.
#[test]
fn a_linha_key_le_a_tecla_e_quem_mais_a_usa() {
    // ⚠️ Mapa LIMPO: o de fábrica já liga `A` a `move_left`.
    let mut mapa = ph2d_input::InputMap::default();
    let id = mapa.create(NOME);
    let a = mapa.get_mut(id).expect("existe");
    a.bindings.push(Binding::Key(Key(KEY_A)));
    a.bindings.push(Binding::PadButton(GamepadButton::South));
    let outra = mapa.create("other_test");
    mapa.get_mut(outra)
        .expect("existe")
        .bindings
        .push(Binding::Key(Key(KEY_A)));
    let mut info = InspectorActionTriggerInfo {
        entity_bits: 1,
        rows: vec![fileira(NOME), fileira("nao_existe")],
        clock_playing: true,
        selected_count: 1,
    };
    le_teclas(&mut info, &mapa);
    let t = &info.rows[0].tecla;
    assert_eq!(t.id, Some(id));
    assert_eq!(
        t.teclas,
        vec![Key(KEY_A).label()],
        "o comando não entra na lista"
    );
    assert_eq!(
        t.tambem,
        Some((Key(KEY_A).label(), "other_test".to_string()))
    );
    assert_eq!(
        info.rows[1].tecla,
        TeclaDaAccao::default(),
        "acção desconhecida: nada"
    );
}
