//! ⭐ **O cromo da Timeline em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a barra muda de
//! cara, o clique pede o mesmo que `PH2D_CARDS=0`, e a folha não perde altura. Gesto real, rects pintados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{dicas, ferramenta, list_menu_of};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{
    TimelinePanelState, drain_intents, last_visible_h, set_current_timeline,
};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::mesa::Mesa;

const PAINEL: NodeId = ph2d_editor_core::ids::TIMELINE_PANEL;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

fn documento() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        time_seconds: 2.0,
        frame: 120,
        view_length_seconds: 4.0,
        view_length_explicit: true,
        clips: vec!["Main".to_owned(), "Walk".to_owned()],
        clip_index: vec![0, 1],
        ..TimelineViewSnapshot::default()
    }
}

fn mesa_em(cartoes: bool, janela: Rect, snap: TimelineViewSnapshot) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(snap));
    let _ = drain_intents();
    let mut mesa = Mesa::nova(TimelinePanelState::default());
    mesa.assenta_em(janela);
    let _ = drain_intents();
    let _ = mesa.host.drained_actions();
    mesa
}

fn mesa(cartoes: bool) -> Mesa<TimelinePanel> {
    mesa_em(cartoes, JANELA, documento())
}

const BOTOES: [NodeId; 15] = [
    ids::TIMELINE_GO_START,
    ids::TIMELINE_PREV_FRAME,
    ids::TIMELINE_PLAY,
    ids::TIMELINE_NEXT_FRAME,
    ids::TIMELINE_GO_END,
    ids::TIMELINE_ADD_CLIP,
    ids::TIMELINE_DUP_CLIP,
    ids::TIMELINE_RENAME_CLIP,
    ids::TIMELINE_DELETE_CLIP,
    ids::TIMELINE_REVERSE_CLIP,
    ids::TIMELINE_ADD_MARKER,
    ids::TIMELINE_ONION_SETTINGS,
    ids::TIMELINE_ADD_TRACK,
    ids::TIMELINE_TAB_ARRANGE,
    ids::TIMELINE_CLOSE,
];

const CAIXAS: [NodeId; 11] = [
    ids::TIMELINE_LOOP,
    ids::TIMELINE_PINGPONG,
    ids::TIMELINE_PHYSICS,
    ids::TIMELINE_AUTOKEY,
    ids::TIMELINE_RECORD,
    ids::TIMELINE_MOTION_PATH,
    ids::TIMELINE_SNAP,
    ids::TIMELINE_SPEED,
    ids::TIMELINE_ONION,
    ids::TIMELINE_ONION_MODE,
    ids::TIMELINE_CLIP_AUTOPLAY,
];

const NUMEROS: [NodeId; 3] = [
    ids::TIMELINE_TIME_NUM,
    ids::TIMELINE_FRAME_NUM,
    ids::TIMELINE_LENGTH_NUM,
];

/// O que um clique pediu: à shell, ao documento e à vista do painel.
fn efeito(mesa: &mut Mesa<TimelinePanel>, id: NodeId) -> String {
    let acoes = mesa.clica(id);
    format!(
        "{acoes:?} | {:?} | {:?} | {}",
        drain_intents(),
        mesa.st.tab,
        mesa.st.speed_view
    )
}

/// ⭐⭐ **A altura é o recurso deste painel.** As linhas de faixa à vista, antes e depois, a três larguras.
/// *Mutação: `area_da_folha` devolver o rect da banda ⇒ a folha sai da moldura.*
#[test]
fn a_folha_nao_perde_linhas_para_o_cromo() {
    let mut medidas = Vec::new();
    for w in [1930.0, 1600.0, 1100.0] {
        let janela = Rect { w, ..JANELA };
        let _ = mesa_em(false, janela, documento());
        let antes = last_visible_h();
        let m1 = mesa_em(true, janela, documento());
        let depois = last_visible_h();
        let largura = m1.host.store().panel_rect(PAINEL).map_or(0.0, |r| r.w);
        medidas.push((w, largura, antes, depois));
    }
    eprintln!("altura das faixas (janela, banda, antes, cartões): {medidas:?}");
    assert!(medidas[0].2 > 0.0, "vácuo: nada medido");
    for (w, _, antes, depois) in medidas {
        assert!(
            depois >= antes,
            "a folha encolheu a {w}: {antes} px antes, {depois} px em cartões"
        );
    }
}

/// ⭐ A folha mora dentro da moldura, abaixo do último controlo, e nada dela sai da lista.
#[test]
fn a_folha_cabe_na_moldura_e_comeca_depois_da_barra() {
    let m0 = mesa(true);
    let banda = m0.host.store().panel_rect(PAINEL).expect("rect");
    let fundo_da_barra = CAIXAS
        .iter()
        .chain(BOTOES.iter().filter(|i| **i != ids::TIMELINE_ADD_TRACK))
        .map(|i| {
            let r = m0.rect(*i).unwrap_or_else(|| panic!("{i:?} não pintado"));
            r.y + r.h
        })
        .fold(0.0_f32, f32::max);
    let add = m0.rect(ids::TIMELINE_ADD_TRACK).expect("+ Track");
    assert!(
        add.y >= fundo_da_barra,
        "a folha começa a {} e a barra acaba a {fundo_da_barra}",
        add.y
    );
    let geo = ph2d_editor_core::panel::cards::banda::geometria(banda);
    assert!(add.x >= geo.corpo.x, "a coluna de nomes sai da moldura");
    assert!(
        add.y + last_visible_h() + add.h <= geo.lista.y + geo.lista.h,
        "as faixas passam da moldura"
    );
}

/// ⭐⭐ Cada botão e cada caixa pedem, ao clique real, EXACTAMENTE o que pediam com `PH2D_CARDS=0`.
/// *Mutação: tirar um item da fila ⇒ «não pintado».*
#[test]
fn cada_controlo_pede_o_mesmo_que_o_painel_de_antes() {
    let mut nao_vazios = 0;
    for id in BOTOES.iter().chain(CAIXAS.iter()) {
        let antes = efeito(&mut mesa(false), *id);
        let depois = efeito(&mut mesa(true), *id);
        assert_eq!(antes, depois, "{id:?} mudou de efeito");
        nao_vazios += usize::from(antes != efeito_vazio());
    }
    assert!(
        nao_vazios >= 20,
        "vácuo: só {nao_vazios} controlos fizeram alguma coisa"
    );
}

fn efeito_vazio() -> String {
    format!("[] | [] | {:?} | false", TimelinePanelState::default().tab)
}

fn valor_pedido(acoes: &[EditorAction], alvo: NodeId) -> Option<f64> {
    acoes.iter().rev().find_map(|a| match a {
        EditorAction::TimelinePanelEvent(PanelEvent::SetValue(id, v)) if *id == alvo => Some(*v),
        _ => None,
    })
}

/// ⭐⭐ O scrub na PEGA do Time e do Frame pede o valor, só desse campo, e o Esc devolve-o.
/// *Mutação: tirar o `mark_card_field` ⇒ o Down na pega é foco de texto e nada se pede.*
#[test]
fn o_scrub_na_pega_pede_o_valor_e_o_esc_devolve() {
    for (id, valor, passo) in [
        (ids::TIMELINE_TIME_NUM, 2.0, 1.0 / 60.0),
        (ids::TIMELINE_FRAME_NUM, 120.0, 1.0),
    ] {
        let mut m0 = mesa(true);
        let ev = m0.arrasta_a_pega(id, 30.0);
        let acoes = m0.aplica(ev);
        let arrastado = valor_pedido(&acoes, id).unwrap_or_else(|| panic!("{id:?}: {acoes:?}"));
        assert!(
            arrastado > valor + passo * 5.0 && arrastado <= valor + passo * 30.0,
            "{id:?}: arrastou para {arrastado}"
        );
        for outro in NUMEROS.iter().filter(|o| **o != id) {
            assert_eq!(valor_pedido(&acoes, *outro), None, "escreveu noutro campo");
        }
        // Com o documento ainda no valor velho, o quadro seguinte não pode deitar o arrasto fora.
        m0.assenta_em(JANELA);
        assert!(
            (m0.host.store().number_value(id).unwrap_or(f64::NAN) - arrastado).abs() < 1e-6,
            "{id:?}: o espelho do documento comeu o arrasto"
        );
        let ev = m0.esc();
        let acoes = m0.aplica(ev);
        let voltou = valor_pedido(&acoes, id).expect("o Esc avisa o painel");
        assert!((voltou - valor).abs() < 1e-6, "{id:?}: voltou a {voltou}");
    }
}

/// ⭐ O scrub da Dur vai ao documento pelo mesmo intent de sempre, 0,2 s por pixel.
#[test]
fn o_scrub_da_duracao_escreve_a_duracao() {
    let mut m0 = mesa(true);
    let ev = m0.arrasta_a_pega(ids::TIMELINE_LENGTH_NUM, 10.0);
    let _ = m0.aplica(ev);
    let intents = drain_intents();
    let pedida = intents.iter().rev().find_map(|i| match i {
        TimelineIntent::SetSceneLength { len } => *len,
        _ => None,
    });
    let pedida = pedida.unwrap_or_else(|| panic!("a Dur não escreveu: {intents:?}"));
    assert!(pedida > 4.0 && pedida <= 6.0, "pedida {pedida}");
}

/// ⭐ Escrever o número continua a pedir o valor: clique no VALOR, nunca na pega.
#[test]
fn escrever_o_numero_pede_o_valor() {
    let mut m0 = mesa(true);
    let ev = m0.host.type_into_number(ids::TIMELINE_TIME_NUM, "3.5");
    let acoes = m0.aplica(ev);
    assert_eq!(valor_pedido(&acoes, ids::TIMELINE_TIME_NUM), Some(3.5));
}

/// A célula da pega é a do campo: o rect pintado tem a altura do campo do cartão.
#[test]
fn os_numeros_e_as_caixas_tem_a_altura_do_cartao() {
    let m0 = mesa(true);
    for id in NUMEROS.iter().chain(CAIXAS.iter()) {
        let r = m0.rect(*id).expect("pintado");
        assert_eq!(r.h, m::FIELD_H, "{id:?}");
    }
    let r = m0.rect(ids::TIMELINE_TIME_NUM).expect("Time");
    assert_eq!(handle_rect(r).w, m::ICON_CELL);
}

/// ⭐ Os dois chips abrem pelo clique real e a opção escolhe, pelo mesmo caminho de sempre.
/// *Mutação: chip sem hit ⇒ «não pintado».*
#[test]
fn os_chips_abrem_e_a_opcao_escolhe() {
    let mut m0 = mesa(true);
    let aberto = |m: &Mesa<TimelinePanel>, id| {
        matches!(
            m.host.store().get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    };
    let _ = m0.clica(ids::TIMELINE_ANIM_DD);
    assert!(aberto(&m0, ids::TIMELINE_ANIM_DD), "Animation não abriu");
    let _ = m0.clica(ids::TIMELINE_ANIM_DD);
    let _ = m0.clica(ids::TIMELINE_CLIP_DD);
    assert!(aberto(&m0, ids::TIMELINE_CLIP_DD), "o clip não abriu");
    m0.assenta_em(JANELA);
    let _ = m0.clica(ids::TIMELINE_CLIP_OPT[1]);
    let intents = drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, TimelineIntent::SetActiveClip { index: 1 })),
        "a opção não escolheu: {intents:?}"
    );
}

/// ⭐ Na lista de Containers não há Play: apagado, e sem alvo. O lixo só existe acima de um clip.
/// *Mutações: Play vivo ⇒ há rect; lixo sempre ⇒ há rect com um clip.*
#[test]
fn o_que_nao_pode_nao_tem_alvo() {
    let mut m0 = mesa(true);
    assert!(m0.rect(ids::TIMELINE_PLAY).is_some(), "controlo: há Play");
    let _ = m0.clica(ids::TIMELINE_TAB_CONTAINERS);
    m0.assenta_em(JANELA);
    assert!(m0.rect(ids::TIMELINE_PLAY).is_none(), "Play com alvo");

    let um_clip = TimelineViewSnapshot {
        clips: vec!["Main".to_owned()],
        clip_index: vec![0],
        ..documento()
    };
    let m1 = mesa_em(true, JANELA, um_clip);
    assert!(
        m1.rect(ids::TIMELINE_DELETE_CLIP).is_none(),
        "lixo com alvo"
    );
    assert!(m1.rect(ids::TIMELINE_DUP_CLIP).is_some(), "controlo");
}

/// ⭐ Todo ícone mudo tem dica. *Mutação: tirar uma linha de `DICAS` ⇒ vermelho nesse id.*
#[test]
fn todo_icone_mudo_tem_dica() {
    let m0 = mesa(true);
    let mudos = dicas::icon_only_ids(PAINEL);
    assert!(mudos.len() >= 11, "vácuo: {mudos:?}");
    for id in mudos {
        assert!(m0.host.store().tooltip_for(id).is_some(), "{id:?} sem dica");
    }
}

/// ⭐ O «…» abre o menu da lista e o «×» é o `TIMELINE_CLOSE` de sempre (o clique está na tabela acima).
#[test]
fn o_menu_abre_e_o_fecho_e_o_de_sempre() {
    let mut m0 = mesa(true);
    let menu = list_menu_of(PAINEL);
    let r = m0.rect(menu).expect("«…» pintado");
    let ev = m0.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(i) if *i == menu)),
        "{ev:?}"
    );
    assert!(ph2d_editor_core::panel::cards::apply_list_menu_click(
        m0.host.store_mut(),
        WidgetEvent::Click(menu),
    ));
    assert!(m0.host.store().context_menu().is_some(), "o menu não abriu");
    assert_eq!(
        m0.host.store().card_list_close(PAINEL),
        Some(ids::TIMELINE_CLOSE)
    );
}

/// ⭐ A pega do topo da banda continua a ser o topo; é ela que dá altura à folha.
#[test]
fn a_costura_do_topo_continua_na_borda_da_banda() {
    let m0 = mesa(true);
    let banda = m0.host.store().panel_rect(PAINEL).expect("rect");
    assert_eq!(
        m0.host.hit_at(banda.x + banda.w * 0.5, banda.y + 1.0),
        Some(ids::TIMELINE_RESIZE_T)
    );
}

/// ⭐ A roda sobre a folha rola as linhas: a superfície da timeline ganha à lista.
#[test]
fn a_roda_sobre_a_folha_continua_a_rolar_as_linhas() {
    let tracks: Vec<_> = (0..40)
        .map(|i| ph2d_timeline::TrackView {
            target: ph2d_timeline::AnimTarget::new(i as u64),
            prop: ph2d_timeline::PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            expr: None,
            keys: Vec::new(),
        })
        .collect();
    let snap = TimelineViewSnapshot {
        tracks,
        ..documento()
    };
    let mut m0 = mesa_em(true, JANELA, snap);
    let add = m0.rect(ids::TIMELINE_ADD_TRACK).expect("+ Track");
    let _ = m0.host.wheel_at(add.x + 300.0, add.y + 60.0, -40.0);
    m0.assenta_em(JANELA);
    assert!(m0.st.scroll_y > 0.0, "a roda não rolou as linhas");
}

/// ⭐ Janela estreita: a barra quebra em fileiras e nada sai da lista nem fica por cima da folha.
#[test]
fn numa_janela_estreita_a_barra_quebra_e_nada_sai() {
    let janela = Rect { w: 900.0, ..JANELA };
    let m0 = mesa_em(true, janela, documento());
    let banda = m0.host.store().panel_rect(PAINEL).expect("rect");
    let lista = ph2d_editor_core::panel::cards::banda::geometria(banda).lista;
    let add = m0.rect(ids::TIMELINE_ADD_TRACK).expect("+ Track");
    for id in NUMEROS.iter().chain(CAIXAS.iter()) {
        let r = m0.rect(*id).unwrap_or_else(|| panic!("{id:?} não pintado"));
        assert!(r.x + r.w <= lista.x + lista.w + 0.5, "{id:?} sai: {r:?}");
        assert!(r.y + r.h <= add.y + 0.5, "{id:?} em cima da folha: {r:?}");
    }
}

/// ⭐ `PH2D_CARDS=0` pinta o painel de antes: sem «…», com os controlos à altura de sempre.
/// *Mutação: ramo `cartoes` sempre ligado ⇒ vermelho.*
#[test]
fn sem_cartoes_e_o_painel_de_antes() {
    let antes = mesa(false);
    assert!(
        antes.rect(list_menu_of(PAINEL)).is_none(),
        "«…» no de antes"
    );
    assert_eq!(
        antes.rect(ids::TIMELINE_PLAY).expect("Play").h,
        ph2d_tokens::ROW_H_PX
    );
    let novo = mesa(true);
    assert!(novo.rect(list_menu_of(PAINEL)).is_some());
    assert_eq!(novo.rect(ids::TIMELINE_PLAY).expect("Play").h, m::FIELD_H);
    let _ = PointerKind::Move;
}
