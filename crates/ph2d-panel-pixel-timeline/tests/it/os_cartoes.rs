//! ⭐ **O cromo da linha do tempo do Pixel** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a barra muda de cara, o
//! clique pede o mesmo que `PH2D_CARDS=0`, e a grelha de camadas não perde linhas. Gesto real, rects pintados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::panel::cards::list_menu_of;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelCel, PixelFrameCol, PixelLayerRow, PixelPanelInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_timeline::{
    PixelTimelinePanel, PixelTimelinePanelState, ids, set_current_pixel,
};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

fn camada(id: u32) -> PixelLayerRow {
    PixelLayerRow {
        id,
        name: format!("L{id}"),
        depth: 0,
        group: false,
        collapsed: false,
        visible: true,
        locked: false,
        continuous: false,
        reference: false,
        background: false,
        opacity: 255,
        blend: Default::default(),
    }
}

/// Doze camadas: mais do que a banda mostra.
const CAMADAS: u32 = 12;

fn documento() -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        layers: (1..=CAMADAS).map(camada).collect(),
        active_layer: Some(1),
        frames: (1..=3)
            .map(|id| PixelFrameCol {
                id,
                duration_ms: 100,
            })
            .collect(),
        active_frame: Some(1),
        cels: (1..=CAMADAS)
            .map(|layer| PixelCel {
                layer,
                frame: 1,
                owner: 1,
                opacity: 255,
                z: 0,
                empty: false,
            })
            .collect(),
        ..Default::default()
    }
}

fn mesa(cartoes: bool) -> Mesa<PixelTimelinePanel> {
    force_cards_on_this_thread(Some(cartoes));
    set_current_pixel(documento());
    let mut m0 = Mesa::<PixelTimelinePanel>::nova(PixelTimelinePanelState::default());
    m0.host.set_panel_visible(PixelTimelinePanel::ID, true);
    m0.assenta_em(JANELA);
    m0
}

fn controlos() -> Vec<NodeId> {
    let mut v: Vec<NodeId> = ids::TRANSPORT.iter().map(|t| t.1).collect();
    v.extend(ids::ACTIONS.iter().map(|a| a.1));
    v.extend(ids::TOGGLES.iter().map(|t| t.1));
    v.extend(ids::PLAY_TOGGLES.iter().map(|t| t.0));
    v
}

/// ⭐⭐ Cada controlo da barra pede, ao clique real, EXACTAMENTE o que pedia com `PH2D_CARDS=0`.
/// *Mutação: tirar um grupo de `grupos` ⇒ «não pintado».*
#[test]
fn cada_controlo_pede_o_mesmo_que_o_painel_de_antes() {
    let mut nao_vazios = 0;
    for id in controlos() {
        let antes = format!("{:?}", mesa(false).clica(id));
        let depois = format!("{:?}", mesa(true).clica(id));
        assert_eq!(antes, depois, "{id:?} mudou de efeito");
        nao_vazios += usize::from(antes != "[]");
    }
    assert!(
        nao_vazios >= 15,
        "vácuo: só {nao_vazios} pediram alguma coisa"
    );
}

/// ⭐⭐ As propriedades do activo são campos de cartão: rótulo por cima, pega de arrastar, sem as setas
/// de antes. O par Opacity + Z fica na mesma linha. *Mutação: voltar ao `paint_field_row` ⇒ vermelho.*
#[test]
fn os_numeros_das_propriedades_sao_campos_de_cartao() {
    let novo = mesa(true);
    for id in [ids::FRAME_DURATION, ids::CEL_OPACITY, ids::CEL_Z] {
        assert!(novo.rect(id).is_some(), "{id:?} não pintado");
        assert!(
            novo.host.store().card_field(id).is_some(),
            "{id:?} não é um campo de cartão"
        );
    }
    let (op, z) = (
        novo.rect(ids::CEL_OPACITY).expect("opacidade"),
        novo.rect(ids::CEL_Z).expect("z"),
    );
    assert_eq!(op.y, z.y, "Opacity e Z dividem a linha");
    assert!(op.x + op.w <= z.x, "colunas sem sobreposição");
    let antes = mesa(false);
    assert!(
        antes.host.store().card_field(ids::CEL_Z).is_none(),
        "PH2D_CARDS=0 fica com o corpo de antes"
    );
}

/// Quantas camadas têm célula à vista: a régua desta banda.
fn linhas_a_vista(m0: &Mesa<PixelTimelinePanel>) -> u32 {
    (1..=CAMADAS)
        .filter(|l| m0.rect(ids::cell_id(*l, 1)).is_some())
        .count() as u32
}

/// ⭐⭐ **A altura é o recurso.** A grelha mostra pelo menos as camadas de antes.
/// *Mutação: a barra devolver `corpo.y` ⇒ a grelha desce e perde uma linha.*
#[test]
fn a_grelha_nao_perde_linhas_para_o_cromo() {
    let (antes, novo) = (mesa(false), mesa(true));
    let (la, ln) = (linhas_a_vista(&antes), linhas_a_vista(&novo));
    let topo = |m0: &Mesa<PixelTimelinePanel>| {
        let banda = m0
            .host
            .store()
            .panel_rect(core_ids::PIXEL_TIMELINE_PANEL)
            .expect("rect");
        m0.rect(ids::cell_id(1, 1)).expect("1.ª célula").y - banda.y
    };
    eprintln!(
        "linhas à vista: antes {la}, cartões {ln}; 1.ª linha a {} → {} px do topo",
        topo(&antes),
        topo(&novo)
    );
    assert!(la > 0, "vácuo");
    assert!(ln >= la, "a grelha perdeu linhas: {la} → {ln}");
}

/// ⭐ Sem «…» (não há cartão a repor), e o «×» é o de sempre, dentro da moldura.
#[test]
fn nao_ha_menu_morto_e_o_fecho_e_o_de_sempre() {
    let m0 = mesa(true);
    assert!(
        m0.rect(list_menu_of(core_ids::PIXEL_TIMELINE_PANEL))
            .is_none()
    );
    assert_eq!(
        m0.host
            .store()
            .card_list_close(core_ids::PIXEL_TIMELINE_PANEL),
        Some(ids::CLOSE)
    );
    let r = m0.rect(ids::CLOSE).expect("«×»");
    assert_eq!(r.w, m::HEAD_ICON_BOX);
}

/// ⭐ Os ícones e os segmentos têm a altura do campo do cartão; sem cartões, a linha de sempre.
#[test]
fn a_barra_tem_a_altura_do_cartao() {
    let id = ids::TRANSPORT[0].1;
    assert_eq!(mesa(true).rect(id).expect("pintado").h, m::FIELD_H);
    assert_eq!(
        mesa(false).rect(id).expect("pintado").h,
        ph2d_tokens::ROW_H_PX
    );
}

/// ⭐ O chip da velocidade abre pelo clique real, e a opção escolhe.
/// *Mutação: não guardar `speed_open` ⇒ a opção não se pinta.*
#[test]
fn a_velocidade_abre_e_a_opcao_escolhe() {
    let mut m0 = mesa(true);
    let _ = m0.clica(ids::PLAY_SPEED);
    assert!(
        matches!(
            m0.host.store().get(ids::PLAY_SPEED),
            Some(InteractiveState::Dropdown { open: true, .. })
        ),
        "não abriu"
    );
    m0.assenta_em(JANELA);
    let acoes = m0.clica(ids::speed_option_id(1));
    assert!(!acoes.is_empty(), "a opção não pediu nada");
}

/// ⭐ Janela estreita: a barra quebra em fileiras e a grelha começa depois delas.
#[test]
fn numa_janela_estreita_a_barra_quebra_e_nada_cobre_a_grelha() {
    force_cards_on_this_thread(Some(true));
    set_current_pixel(documento());
    let mut m0 = Mesa::<PixelTimelinePanel>::nova(PixelTimelinePanelState::default());
    m0.host.set_panel_visible(PixelTimelinePanel::ID, true);
    m0.assenta_em(Rect {
        w: 1000.0,
        ..JANELA
    });
    let banda = m0
        .host
        .store()
        .panel_rect(core_ids::PIXEL_TIMELINE_PANEL)
        .expect("rect");
    let celula = m0.rect(ids::cell_id(1, 1)).expect("célula");
    let mut pintados = 0;
    for id in controlos() {
        if let Some(r) = m0.rect(id) {
            pintados += 1;
            assert!(r.x + r.w <= banda.x + banda.w + 0.5, "{id:?} sai: {r:?}");
            assert!(
                r.y + r.h <= celula.y + 0.5,
                "{id:?} por cima da grelha: {r:?}"
            );
        }
    }
    assert!(pintados >= 15, "vácuo: {pintados}");
}

/// As marcas de key de slice: só nos quadros pedidos, e sem nenhuma, a grelha não pinta nenhuma (P11).
#[test]
fn as_marcas_de_slice_pintam_so_nos_quadros_com_key() {
    let mut m = mesa(true);
    let antes = m.st.marcas_de_slice_pintadas();
    m.assenta_em(JANELA);
    assert_eq!(m.st.marcas_de_slice_pintadas(), antes, "sem seleção, nada");
    let quadros = |m: &Mesa<PixelTimelinePanel>| m.st.marcas_de_slice_pintadas();
    set_current_pixel(PixelPanelInfo {
        slice_keys: vec![0, 2],
        ..documento()
    });
    m.assenta_em(JANELA);
    let pintadas = quadros(&m) - antes;
    assert!(
        pintadas >= 2 && pintadas.is_multiple_of(2),
        "duas por quadro: {pintadas}"
    );
}
