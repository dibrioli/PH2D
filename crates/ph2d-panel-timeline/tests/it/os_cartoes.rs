//! ⭐ **O cromo da Timeline em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a barra muda de
//! cara, o clique pede o mesmo que `PH2D_CARDS=0`, e a folha não perde altura. Gesto real, rects pintados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
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

/// O que fica À VISTA na barra (doc 15, onda 1): título, clipes e tocar. O resto mora nas janelinhas.
const NA_BARRA: [NodeId; 19] = [
    ids::TIMELINE_TAB_KEYS,
    ids::TIMELINE_TAB_CONTAINERS,
    ids::TIMELINE_TAB_ARRANGE,
    ids::TIMELINE_ANIM_DD,
    ids::TIMELINE_OPTIONS,
    ids::TIMELINE_CLOSE,
    ids::TIMELINE_CLIP_OPT[0],
    ids::TIMELINE_CLIP_OPT[1],
    ids::TIMELINE_CLIP_MENU,
    ids::TIMELINE_ADD_CLIP,
    ids::TIMELINE_GO_START,
    ids::TIMELINE_PREV_FRAME,
    ids::TIMELINE_PLAY,
    ids::TIMELINE_NEXT_FRAME,
    ids::TIMELINE_GO_END,
    ids::TIMELINE_TIME_NUM,
    ids::TIMELINE_FRAME_NUM,
    ids::TIMELINE_AUTOKEY,
    ids::TIMELINE_RECORD,
];

/// Os que se mudaram para «Timeline options» (a engrenagem).
const NAS_OPCOES: [NodeId; 7] = [
    ids::TIMELINE_SNAP,
    ids::TIMELINE_PHYSICS,
    ids::TIMELINE_MOTION_PATH,
    ids::TIMELINE_SPEED,
    ids::TIMELINE_ONION,
    ids::TIMELINE_ONION_MODE,
    ids::TIMELINE_ONION_SETTINGS,
];

/// Os que se mudaram para o menu do clipe (o «⋯» do chip escolhido).
const NO_MENU: [NodeId; 5] = [
    ids::TIMELINE_RENAME_CLIP,
    ids::TIMELINE_DUP_CLIP,
    ids::TIMELINE_REVERSE_CLIP,
    ids::TIMELINE_CLIP_AUTOPLAY,
    ids::TIMELINE_DELETE_CLIP,
];

/// Em cartões, abre a janelinha onde `id` mora agora, e devolve o id a clicar (o Loop e o Ping-Pong
/// viraram células de «When the clip ends»).
fn onde_mora(mesa: &mut Mesa<TimelinePanel>, id: NodeId) -> NodeId {
    let (porta, alvo) = if id == ids::TIMELINE_LOOP {
        (Some(ids::TIMELINE_OPTIONS), ids::TIMELINE_END_LOOP)
    } else if id == ids::TIMELINE_PINGPONG {
        (Some(ids::TIMELINE_OPTIONS), ids::TIMELINE_END_PINGPONG)
    } else if NAS_OPCOES.contains(&id) || id == ids::TIMELINE_LENGTH_NUM {
        (Some(ids::TIMELINE_OPTIONS), id)
    } else if NO_MENU.contains(&id) {
        (Some(ids::TIMELINE_CLIP_MENU), id)
    } else {
        (None, id)
    };
    if let Some(porta) = porta {
        let a = mesa.clica(porta);
        assert!(a.is_empty(), "abrir {porta:?} pediu {a:?}");
        mesa.assenta_em(JANELA);
    }
    alvo
}

/// O que um clique pediu: à shell, ao documento e à vista do painel. Em cartões, o clique é dado onde
/// o controlo mora agora.
fn efeito(mesa: &mut Mesa<TimelinePanel>, id: NodeId) -> String {
    let id = if ferramenta::cards_on() {
        onde_mora(mesa, id)
    } else {
        id
    };
    let acoes = mesa.clica(id);
    format!(
        "{acoes:?} | {:?} | {:?} | {}",
        drain_intents(),
        mesa.st.tab,
        mesa.st.speed_view
    )
}

/// ⭐⭐ **A altura é o recurso deste painel.** As linhas de faixa à vista, antes e depois, a três larguras.
/// O molde do Rive em DUAS fileiras (doc 15, onda 1b, dono 09/11): a folha nunca fica mais baixa que no
/// painel de antes. *Mutações: `area_da_folha` devolver o rect da banda ⇒ a folha sai da moldura; uma
/// 3.ª fileira ⇒ vermelho a 1930.*
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
    let fundo_da_barra = NA_BARRA
        .iter()
        .chain(std::iter::once(&ids::TIMELINE_ADD_MARKER))
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
    let _ = onde_mora(&mut m0, ids::TIMELINE_LENGTH_NUM);
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
    for id in [
        ids::TIMELINE_TIME_NUM,
        ids::TIMELINE_FRAME_NUM,
        ids::TIMELINE_AUTOKEY,
        ids::TIMELINE_RECORD,
        ids::TIMELINE_CLIP_OPT[0],
        ids::TIMELINE_ANIM_DD,
    ] {
        let r = m0.rect(id).expect("pintado");
        assert_eq!(r.h, m::FIELD_H, "{id:?}");
    }
    let r = m0.rect(ids::TIMELINE_TIME_NUM).expect("Time");
    assert_eq!(handle_rect(r).w, m::ICON_CELL);
}

/// ⭐ O chip do dono abre pelo clique real; um clipe escolhe-se clicando o CHIP dele, pelo mesmo id e o
/// mesmo caminho que a opção da lista tinha. *Mutação: chip sem hit ⇒ «não pintado».*
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
    assert!(
        m0.rect(ids::TIMELINE_CLIP_DD).is_none(),
        "a lista dos clipes virou chips: nenhum ▾ à vista"
    );
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
    let mut m1 = mesa_em(true, JANELA, um_clip);
    let _ = onde_mora(&mut m1, ids::TIMELINE_DELETE_CLIP);
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
    assert!(mudos.len() >= 8, "vácuo: {mudos:?}");
    for id in mudos {
        assert!(m0.host.store().tooltip_for(id).is_some(), "{id:?} sem dica");
    }
}

/// ⭐ Sem «…»: as linhas dele repõem cartões, e a moldura da Timeline não tem nenhum. O «×» fica na ponta
/// e é o `TIMELINE_CLOSE` de sempre (o clique está na tabela acima).
/// *Mutação: `pinta_moldura` chamar `menu::pinta` ⇒ há rect.*
#[test]
fn nao_ha_menu_morto_e_o_fecho_e_o_de_sempre() {
    let m0 = mesa(true);
    assert!(m0.rect(list_menu_of(PAINEL)).is_none(), "«…» morto pintado");
    assert_eq!(
        m0.host.store().card_list_close(PAINEL),
        Some(ids::TIMELINE_CLOSE)
    );
    let banda = m0.host.store().panel_rect(PAINEL).expect("rect");
    let lista = ph2d_editor_core::panel::cards::banda::geometria(banda).lista;
    let x = m0.rect(ids::TIMELINE_CLOSE).expect("«×» pintado");
    assert!(x.x + x.w <= lista.x + lista.w, "«×» fora da lista: {x:?}");
    assert!(
        x.x + x.w + m::LIST_PAD_X + 0.5 >= lista.x + lista.w,
        "«×» longe da ponta: {x:?}"
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
    for id in NA_BARRA.iter() {
        let r = m0.rect(*id).unwrap_or_else(|| panic!("{id:?} não pintado"));
        assert!(r.x + r.w <= lista.x + lista.w + 0.5, "{id:?} sai: {r:?}");
        assert!(r.y + r.h <= add.y + 0.5, "{id:?} em cima da folha: {r:?}");
    }
}

/// ⭐ `PH2D_CARDS=0` pinta o painel de antes: os controlos à altura de sempre.
/// *Mutação: ramo `cartoes` sempre ligado ⇒ vermelho.*
#[test]
fn sem_cartoes_e_o_painel_de_antes() {
    let antes = mesa(false);
    assert_eq!(
        antes.rect(ids::TIMELINE_PLAY).expect("Play").h,
        ph2d_tokens::ROW_H_PX
    );
    let novo = mesa(true);
    assert_eq!(novo.rect(ids::TIMELINE_PLAY).expect("Play").h, m::FIELD_H);
    let _ = PointerKind::Move;
}

/// ⭐⭐ **Duas fileiras** (doc 15, onda 1b, dono 09/11 depois do smoke da onda 1): na do título, abas ·
/// tocar · Time · Frame · fps … dono · engrenagem; na 2.ª, Auto Key · Record · Marker ANTES do 1.º chip,
/// depois os chips e o «+». *Mutações: `f.quebra()` antes do tocar ⇒ o Play desce; os chips antes do
/// gravar ⇒ o Auto Key passa à direita do chip.*
#[test]
fn a_barra_tem_duas_fileiras_no_molde_do_rive() {
    let m0 = mesa(true);
    let r = |id: NodeId| m0.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado"));
    // Pelo CENTRO: a célula de uma aba é o segmento, recolhido dentro da caixa.
    let y = |id: NodeId| {
        let r = r(id);
        r.y + r.h * 0.5
    };
    let titulo = y(ids::TIMELINE_TAB_KEYS);
    for id in [
        ids::TIMELINE_GO_START,
        ids::TIMELINE_PLAY,
        ids::TIMELINE_GO_END,
        ids::TIMELINE_TIME_NUM,
        ids::TIMELINE_FRAME_NUM,
        ids::TIMELINE_ANIM_DD,
        ids::TIMELINE_OPTIONS,
    ] {
        assert!(
            (y(id) - titulo).abs() < 0.5,
            "{id:?} fora da fileira do título"
        );
    }
    // A ordem da 1.ª: abas, tocar, Time, Frame e, encostados à direita, o dono e a engrenagem.
    let ordem = [
        ids::TIMELINE_TAB_ARRANGE,
        ids::TIMELINE_GO_START,
        ids::TIMELINE_GO_END,
        ids::TIMELINE_TIME_NUM,
        ids::TIMELINE_FRAME_NUM,
        ids::TIMELINE_ANIM_DD,
        ids::TIMELINE_OPTIONS,
    ];
    for par in ordem.windows(2) {
        assert!(
            r(par[0]).x + r(par[0]).w <= r(par[1]).x + 0.5,
            "{:?} não vem antes de {:?}",
            par[0],
            par[1]
        );
    }
    let segunda = y(ids::TIMELINE_AUTOKEY);
    assert!(
        segunda > titulo + m::FIELD_H * 0.5,
        "o gravar sobe ao título"
    );
    for id in [
        ids::TIMELINE_RECORD,
        ids::TIMELINE_ADD_MARKER,
        ids::TIMELINE_CLIP_OPT[0],
        ids::TIMELINE_CLIP_OPT[1],
        ids::TIMELINE_ADD_CLIP,
        ids::TIMELINE_CLIP_MENU,
    ] {
        assert!((y(id) - segunda).abs() < 0.5, "{id:?} fora da 2.ª fileira");
    }
    let ordem = [
        ids::TIMELINE_AUTOKEY,
        ids::TIMELINE_RECORD,
        ids::TIMELINE_ADD_MARKER,
        ids::TIMELINE_CLIP_OPT[0],
    ];
    for par in ordem.windows(2) {
        assert!(
            r(par[0]).x + r(par[0]).w <= r(par[1]).x + 0.5,
            "{:?} não vem antes de {:?}",
            par[0],
            par[1]
        );
    }
    // Os separadores (onda 1c): um fio com a folga do fluxo dos DOIS lados, entre Arrange e o |◁ e
    // entre Marker e «Clips». Sem eles a folga é UM vão (o rótulo «Clips» medido à parte).
    let sep = ph2d_editor_core::panel::cards::banda::SEPARADOR_W + m::FIELD_GAP * 2.0;
    let abas = r(ids::TIMELINE_TAB_ARRANGE);
    let folga = r(ids::TIMELINE_GO_START).x - (abas.x + abas.w);
    // O alvo da aba (onda 2i) cresceu meio vão para cada lado dentro da antiga caixa das abas.
    assert!(
        folga >= sep + m::SEG_PAD * 0.5 - 0.5,
        "sem separador entre Arrange e o |◁: folga {folga}"
    );
    let marker = r(ids::TIMELINE_ADD_MARKER);
    let clips_w = ph2d_text::TextSystem::without_system_fonts()
        .prefix_width(ph2d_i18n::tr("panel.timeline.card.clips"), m::LABEL_PX)
        .ceil();
    let folga = r(ids::TIMELINE_CLIP_OPT[0]).x - (marker.x + marker.w) - clips_w - m::FIELD_GAP;
    assert!(
        (folga - sep).abs() < 0.5,
        "sem separador entre Marker e Clips: folga {folga}, esperada {sep}"
    );
    // Nada abaixo da 2.ª fileira: a barra tem DUAS.
    let add = r(ids::TIMELINE_ADD_TRACK);
    assert!(
        add.y < segunda + m::FIELD_H * 1.5,
        "há uma 3.ª fileira: a folha começa a {}",
        add.y
    );
    // O que saiu da barra não está à vista enquanto a janelinha dele está fechada.
    for id in NAS_OPCOES.iter().chain(NO_MENU.iter()).chain([
        &ids::TIMELINE_LOOP,
        &ids::TIMELINE_PINGPONG,
        &ids::TIMELINE_LENGTH_NUM,
    ]) {
        assert!(m0.rect(*id).is_none(), "{id:?} ainda pintado na barra");
    }
    // O ⋯ mora no chip ESCOLHIDO (o clipe 0 do documento).
    let chip = m0.rect(ids::TIMELINE_CLIP_OPT[0]).expect("chip");
    let menu = m0.rect(ids::TIMELINE_CLIP_MENU).expect("⋯");
    assert!(
        (menu.x - (chip.x + chip.w)).abs() < 0.5,
        "o ⋯ não encosta ao chip escolhido"
    );
}

/// ⭐⭐ «Stop» pede exatamente o que desmarcar a caixa acesa pedia, e a célula já escolhida não pede
/// nada. *Mutação: Stop pedir sempre o Loop ⇒ vermelho no Ping-Pong.*
#[test]
fn stop_pede_o_que_desmarcar_a_caixa_pedia() {
    for ping_pong in [false, true] {
        let snap = TimelineViewSnapshot {
            loop_range: Some((0.0, 4.0)),
            loop_ping_pong: ping_pong,
            ..documento()
        };
        let caixa = if ping_pong {
            ids::TIMELINE_PINGPONG
        } else {
            ids::TIMELINE_LOOP
        };
        let antes = {
            let mut m = mesa_em(false, JANELA, snap.clone());
            let a = m.clica(caixa);
            format!("{a:?} | {:?}", drain_intents())
        };
        let mut m = mesa_em(true, JANELA, snap.clone());
        let _ = onde_mora(&mut m, ids::TIMELINE_LOOP);
        let a = m.clica(ids::TIMELINE_END_STOP);
        assert_eq!(
            antes,
            format!("{a:?} | {:?}", drain_intents()),
            "ping-pong {ping_pong}"
        );
        let escolhida = if ping_pong {
            ids::TIMELINE_END_PINGPONG
        } else {
            ids::TIMELINE_END_LOOP
        };
        m.assenta_em(JANELA);
        let a = m.clica(escolhida);
        assert!(a.is_empty(), "a célula escolhida pediu {a:?}");
    }
}

/// ⭐ As janelinhas abrem pela porta, ficam abertas enquanto se mexe nelas e fecham pelo clique de fora;
/// uma linha do menu do clipe fecha o menu. *Mutação: tirar `fecha_pelo_clique_de_fora` ⇒ fica aberta.*
#[test]
fn as_janelinhas_abrem_e_fecham_pelo_clique_de_fora() {
    let mut m0 = mesa(true);
    let _ = onde_mora(&mut m0, ids::TIMELINE_SNAP);
    assert!(m0.st.opcoes_abertas, "a engrenagem não abriu");
    let _ = m0.clica(ids::TIMELINE_SNAP);
    m0.assenta_em(JANELA);
    assert!(m0.st.opcoes_abertas, "mexer numa opção fechou a janelinha");
    let fora = m0.rect(ids::TIMELINE_CLIP_OPT[0]).expect("chip");
    let ev = m0.host.click_at(fora.x - 4.0, fora.y + fora.h * 0.5);
    let _ = m0.aplica(ev);
    m0.assenta_em(JANELA);
    assert!(!m0.st.opcoes_abertas, "o clique de fora não fechou");
    assert!(
        m0.rect(ids::TIMELINE_SNAP).is_none(),
        "fechada e ainda pintada"
    );

    let _ = onde_mora(&mut m0, ids::TIMELINE_DUP_CLIP);
    assert!(m0.st.menu_do_clip, "o ⋯ não abriu");
    let _ = m0.clica(ids::TIMELINE_DUP_CLIP);
    assert!(!m0.st.menu_do_clip, "a linha do menu não o fechou");
    assert!(
        drain_intents()
            .iter()
            .any(|i| matches!(i, TimelineIntent::DuplicateClip { .. })),
        "Duplicate não pediu nada"
    );

    // Esconder o painel com uma janelinha aberta: ela não volta aberta.
    let _ = onde_mora(&mut m0, ids::TIMELINE_SNAP);
    assert!(m0.st.opcoes_abertas, "a engrenagem não reabriu");
    m0.host.paint_hidden::<TimelinePanel>(&mut m0.st, JANELA);
    assert!(
        !m0.st.opcoes_abertas && !m0.st.menu_do_clip,
        "o painel escondido guardou a janelinha aberta"
    );
}

/// ⭐⭐ **Os chips ficam numa fileira só**: com dezasseis clipes, o que não cabe vai para o «+N ▾», que abre
/// a lista de sempre; o tocar continua logo abaixo. *Mutação: tirar o `cortou` ⇒ os chips descem em
/// fileiras e o tocar desce com eles.*
#[test]
fn com_muitos_clipes_os_chips_ficam_numa_fileira_e_o_resto_vai_para_a_lista() {
    let clips: Vec<String> = (0..16).map(|i| format!("clip numero {i}")).collect();
    let snap = TimelineViewSnapshot {
        clip_index: (0..16).collect(),
        clips,
        ..documento()
    };
    let janela = Rect {
        w: 1100.0,
        ..JANELA
    };
    let mut m0 = mesa_em(true, janela, snap);
    let primeiro = m0.rect(ids::TIMELINE_CLIP_OPT[0]).expect("o 1.º chip");
    let pintados: Vec<Rect> = ids::TIMELINE_CLIP_OPT
        .iter()
        .filter_map(|id| m0.rect(*id))
        .collect();
    assert!(
        pintados.len() < 16,
        "vácuo: os dezasseis couberam a 1100 px"
    );
    for r in &pintados {
        assert!(
            (r.y - primeiro.y).abs() < 0.5,
            "um chip desceu de fileira: {r:?}"
        );
    }
    let mais = m0.rect(ids::TIMELINE_CLIP_DD).expect("o «+N»");
    assert!((mais.y - primeiro.y).abs() < 0.5, "o «+N» fora da fileira");
    let play = m0.rect(ids::TIMELINE_PLAY).expect("Play");
    assert!(
        play.y < primeiro.y + (m::FIELD_H + m::FIELD_ROW_GAP) * 1.5,
        "o tocar desceu mais que uma fileira"
    );
    let _ = m0.clica(ids::TIMELINE_CLIP_DD);
    m0.assenta_em(janela);
    let ultimo = ids::TIMELINE_CLIP_OPT[15];
    assert!(
        m0.rect(ultimo).is_some(),
        "a lista não mostrou o último clipe"
    );
}

/// ⭐ Doc 15 onda 2i (dono 09/11): as abas passaram a pílula sem caixa e Auto Key · Record · Marker a
/// discretos — SÓ a cara. Os alvos não encolheram: cada um é pelo menos o medido antes da mudança
/// (abas 36,8 / 69,9 / 55,2 × 24; Auto Key 108 × 28; Record 176 × 28; Marker 75 × 28), e o clique na
/// aba ainda troca a aba.
#[test]
fn os_alvos_da_barra_nao_encolheram_com_a_cara_nova() {
    let mut mesa = mesa(true);
    for (id, w, h) in [
        (ids::TIMELINE_TAB_KEYS, 36.8, 24.0),
        (ids::TIMELINE_TAB_CONTAINERS, 69.9, 24.0),
        (ids::TIMELINE_TAB_ARRANGE, 55.2, 24.0),
        (ids::TIMELINE_AUTOKEY, 108.0, 28.0),
        (ids::TIMELINE_RECORD, 176.0, 28.0),
        (ids::TIMELINE_ADD_MARKER, 75.0, 28.0),
    ] {
        let r = mesa.rect(id).expect("o controlo está pintado");
        assert!(
            r.w >= w && r.h >= h,
            "{id:?}: alvo {}×{} < {w}×{h}",
            r.w,
            r.h
        );
    }
    let _ = mesa.clica(ids::TIMELINE_TAB_ARRANGE);
    mesa.assenta_em(JANELA);
    assert_eq!(
        mesa.st.tab,
        ph2d_panel_timeline::tab::Tab::Arrange,
        "a aba ainda troca"
    );
    ferramenta::force_cards_on_this_thread(None);
}
