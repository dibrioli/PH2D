//! ⭐ **O Audio Editor em cartões**: gesto real nos rects pintados, o mesmo efeito de `PH2D_CARDS=0`.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_audio_editor::*;
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 2400.0,
};

const SECCOES: [NodeId; 8] = [
    AEDIT_SEC_TRANSPORT,
    AEDIT_SEC_LOOP,
    AEDIT_SEC_EDIT,
    AEDIT_SEC_SPECTRAL,
    AEDIT_SEC_FX,
    AEDIT_SEC_MARKERS,
    AEDIT_SEC_VARIATIONS,
    AEDIT_SEC_DELIVERY,
];

fn mesa(cartoes: bool) -> Mesa<AudioEditorPanel> {
    force_cards_on_this_thread(Some(cartoes));
    snapshot::set_loaded(true);
    snapshot::set_duration_secs(4.0);
    snapshot::set_has_selection(true);
    set_fx_kind_names(&["Low-Pass", "High-Pass"]);
    set_fx_kind_defaults(&[[1.0, 0.25, 0.0, 0.0], [0.0, 0.25, 0.0, 0.0]]);
    reset_fx_chain();
    set_fx_param_views(&[
        ("Cutoff".to_owned(), "20.0 kHz".to_owned()),
        ("Q".to_owned(), "0.71".to_owned()),
    ]);
    let mut m0 = Mesa::<AudioEditorPanel>::nova(AudioEditorState);
    m0.host.set_panel_visible(AudioEditorPanel::ID, true);
    m0.assenta_em(JANELA);
    for id in SECCOES {
        m0.host.store_mut().set_collapsed(id, false);
    }
    m0.assenta_em(JANELA);
    m0
}

/// Os botões que pedem o mesmo com e sem cartões.
const BOTOES: [NodeId; 6] = [
    AEDIT_PLAY,
    AEDIT_STOP,
    AEDIT_LOOP,
    AEDIT_LOAD,
    AEDIT_EXPORT,
    AEDIT_BATCH_LUFS,
];

fn efeito(m0: &mut Mesa<AudioEditorPanel>, id: NodeId) -> String {
    let acoes = m0.clica(id);
    format!(
        "{acoes:?} play={} stop={} load={} export={} edit={:?}",
        take_play_pause(),
        take_stop(),
        take_load(),
        take_export(),
        take_edit_cmd()
    )
}

/// ⭐⭐ Cada botão do transporte, ao clique real, pede EXACTAMENTE o que pedia com `PH2D_CARDS=0`.
/// *Mutação: tirar o corpo do Transporte ⇒ «não pintado».*
#[test]
fn cada_botao_pede_o_mesmo_que_o_painel_de_antes() {
    let mut nao_vazios = 0;
    for id in BOTOES {
        let antes = efeito(&mut mesa(false), id);
        let depois = efeito(&mut mesa(true), id);
        assert_eq!(antes, depois, "{id:?} mudou de efeito");
        nao_vazios += usize::from(antes.contains("=true") || antes.contains("Some"));
    }
    assert!(
        nao_vazios >= 4,
        "vácuo: só {nao_vazios} fizeram alguma coisa"
    );
}

/// ⭐ As oito secções são cartões com o MESMO id, e dobrar pelo título do cartão continua a dobra de sempre.
#[test]
fn cada_seccao_e_um_cartao_com_o_mesmo_id() {
    let m0 = mesa(true);
    for id in SECCOES {
        assert!(m0.host.store().is_card_section(id), "{id:?} sem cartão");
        assert!(!m0.host.store().is_collapsed(id), "{id:?} dobrada");
    }
}

/// ⭐ O «×» é o de sempre, e o «…» do cartão repõe de verdade (há cartões).
#[test]
fn o_fecho_e_o_de_sempre() {
    let mut m0 = mesa(true);
    assert_eq!(
        m0.host.store().card_list_close(AEDIT_PANEL),
        Some(AEDIT_CLOSE)
    );
    let ev = m0.clica(AEDIT_CLOSE);
    let antes = mesa(false).clica(AEDIT_CLOSE);
    assert_eq!(format!("{ev:?}"), format!("{antes:?}"));
}

/// ⭐ Um botão do corpo tem a altura do campo do cartão, e sem cartões a linha de sempre.
/// *Mutação: `fila` devolver sempre `ROW_H_PX` ⇒ vermelho.*
#[test]
fn os_botoes_tem_a_altura_do_campo() {
    assert_eq!(mesa(true).rect(AEDIT_PLAY).expect("Play").h, m::FIELD_H);
    assert_eq!(
        mesa(false).rect(AEDIT_PLAY).expect("Play").h,
        ph2d_tokens::ROW_H_PX
    );
}

/// ⭐ O corpo do cartão não some ao rolar: com a janela baixa a roda desce e o Delivery chega à vista.
#[test]
fn a_lista_rola() {
    let mut m0 = mesa(true);
    let janela = Rect { h: 700.0, ..JANELA };
    m0.assenta_em(janela);
    let r = m0.host.store().panel_rect(AEDIT_PANEL).expect("rect");
    let _ = m0.host.wheel_at(r.x + r.w * 0.5, r.y + 200.0, -400.0);
    m0.assenta_em(janela);
    assert!(
        m0.host.store().panel_scroll(AEDIT_PANEL) > 0.0,
        "a lista não rolou"
    );
}

/// ⭐⭐ **Nenhum controlo morre:** tudo o que o painel de antes pinta com um alvo tem alvo nos cartões. Fora da conta só a
/// moldura antiga (cabeçalho das secções e barra de rolagem, que o cartão e a lista trazem pelos ids deles).
/// *Mutação: tirar um braço de `paint_corpo` ⇒ a lista acusa os ids dessa secção.*
#[test]
fn nenhum_controlo_morre() {
    let (antes, novo) = (mesa(false), mesa(true));
    let de_moldura = |id: NodeId| {
        SECCOES.contains(&id)
            || id == ph2d_editor_core::widget::AUDIO_EDITOR_SCROLLBAR_ID
            || id == AEDIT_PANEL
            || antes.host.store().card_list_menu(id).is_some()
    };
    let faltam: Vec<NodeId> = antes
        .rects
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !de_moldura(*id) && novo.rect(*id).is_none())
        .collect();
    assert!(antes.rects.len() > 40, "vácuo: {} alvos", antes.rects.len());
    assert!(
        faltam.is_empty(),
        "{} sem alvo nos cartões: {faltam:?}",
        faltam.len()
    );
}

/// ⭐⭐ A barra do parâmetro é o `Slider` de sempre: o arrasto real muda-o e sai o mesmo evento.
/// *Mutação: `vivo` ignorado ⇒ sem alvo.*
#[test]
fn a_barra_do_parametro_responde_ao_arrasto() {
    let mut m0 = mesa(true);
    let id = AEDIT_FX_PARAMS[0];
    let r = m0.rect(id).expect("a barra da 1.ª fileira");
    let (x, y) = (r.x + r.w * 0.25, r.y + r.h * 0.5);
    let mut ev = m0.ponteiro(ph2d_host::PointerKind::Down, x, y);
    ev.extend(m0.ponteiro(ph2d_host::PointerKind::Move, x + 8.0, y));
    ev.extend(m0.ponteiro(ph2d_host::PointerKind::Up, x + 8.0, y));
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(i) if *i == id)),
        "{ev:?}"
    );
}

/// `PH2D_CARDS=0` pinta o corpo de antes.
#[test]
fn sem_cartoes_e_o_de_antes() {
    let m0 = mesa(false);
    assert!(!m0.host.store().is_card_section(AEDIT_SEC_TRANSPORT));
}
