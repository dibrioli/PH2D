//! ⭐⭐ **O painel PIXEL em cartões: nenhum controlo morreu.** Os ids são os das tabelas do painel, e
//! cada um é provado pelo gesto REAL no rect pintado. A paleta e o seletor continuam a responder ao
//! arrasto dentro do cartão; a paleta voa como o cartão do Kanban.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::pixel_panel::{
    PixelBlend, PixelBrushAction, PixelBrushShape, PixelBrushSlotInfo, PixelBrushSlotKind,
    PixelLayerRow, PixelPanelInfo, PixelRequest, PixelSelect, PixelSensor, PixelTool,
    PixelTransformInfo,
};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::mesa::Mesa;
use std::collections::BTreeMap;

const MOLDURA: PixelTransformInfo = PixelTransformInfo {
    x: 3,
    y: 4,
    w: 10,
    h: 12,
    angle: 0,
    skew: 0,
};

fn camada(id: u32, nome: &str) -> PixelLayerRow {
    PixelLayerRow {
        id,
        name: nome.to_owned(),
        depth: 0,
        group: false,
        collapsed: false,
        visible: true,
        locked: false,
        continuous: false,
        reference: false,
        background: false,
        opacity: 255,
        blend: PixelBlend::Normal,
    }
}

fn info(tool: PixelTool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool,
        fg: [200, 30, 40, 255],
        bg: [0, 0, 0, 255],
        palette: (0..32u8)
            .map(|i| [i * 7, 255 - i * 5, i * 3, 255])
            .collect(),
        layers: vec![camada(1, "Back"), camada(2, "Top")],
        active_layer: Some(2),
        has_selection: true,
        has_clip: true,
        can_reselect: true,
        anchor: Some(3),
        remap_pending: true,
        brushes: vec![PixelBrushSlotInfo {
            locked: false,
            kind: PixelBrushSlotKind::Shape {
                shape: PixelBrushShape::Square,
                size: 4,
            },
        }],
        ..PixelPanelInfo::default()
    }
}

fn mesa(i: PixelPanelInfo) -> Mesa<PixelToolsPanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_key_marks(BTreeMap::new());
    set_current_pixel(i);
    let mut m = Mesa::<PixelToolsPanel>::nova(PixelToolsPanelState::default());
    for s in [
        ids::PIXEL_SEC_SELECT,
        ids::PIXEL_SEC_EDIT,
        ids::PIXEL_SEC_SPRITE,
    ] {
        m.host.set_collapsed(s, false);
    }
    m.assenta();
    m.host.drained_actions();
    m
}

fn pede(a: &[EditorAction]) -> Vec<PixelRequest> {
    a.iter()
        .filter_map(|x| match x {
            EditorAction::Pixel(r) => Some(r.clone()),
            _ => None,
        })
        .collect()
}

/// Foto da cena 19 (05/11): «Gradient (Shift+G)» e logo abaixo «Gradient». O nome é do cabeçalho.
#[test]
fn a_tool_card_does_not_repeat_the_name_its_header_already_says() {
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_editor_core::zones::Rect;
    for (tool, nome, vizinha) in [
        (
            PixelTool::Gradient,
            "panel.pixel.gradient",
            "panel.pixel.gradient.shape",
        ),
        (PixelTool::Text, "panel.pixel.text", "panel.pixel.text.size"),
        (
            PixelTool::Bucket,
            "panel.pixel.bucket",
            "panel.pixel.tolerance",
        ),
    ] {
        ferramenta::force_cards_on_this_thread(Some(true));
        set_current_pixel(PixelPanelInfo { tool, ..info(tool) });
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<PixelToolsPanel>();
        let mut st = PixelToolsPanelState::default();
        let vp = Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,  // LITERAL-PX-OK: a largura da doca não muda o que se conta
            h: 4000.0, // LITERAL-PX-OK: alto o bastante para o painel inteiro caber sem rolar
        };
        let (_, medidos) = elisao::medindo(|| host.paint::<PixelToolsPanel>(&mut st, vp));
        let conta = |k: &str| {
            medidos
                .iter()
                .filter(|m| m.texto == ph2d_i18n::tr(k))
                .count()
        };
        assert!(conta(vizinha) > 0, "{tool:?}: o censo não viu «{vizinha}»");
        assert_eq!(conta(nome), 0, "{tool:?}: o nome repete-se no corpo");
    }
}

fn opcoes(a: &[EditorAction]) -> ph2d_editor_core::pixel_panel::PixelOptions {
    pede(a)
        .into_iter()
        .rev()
        .find_map(|r| match r {
            PixelRequest::Options(o) => Some(*o),
            _ => None,
        })
        .unwrap_or_else(|| panic!("nenhuma opção pedida: {a:?}"))
}

/// Só `abre` aberto: a lista cabe na coluna do teste, e o que passa da janela não regista alvo.
fn so_aberto(me: &mut Mesa<PixelToolsPanel>, abre: NodeId) {
    for s in [
        ids::PIXEL_SEC_OPTIONS,
        ids::PIXEL_SEC_BRUSHES,
        ids::PIXEL_SEC_TRANSFORM,
        ids::PIXEL_SEC_COLORS,
        ids::PIXEL_SEC_LAYERS,
        ids::PIXEL_SEC_PALETTE,
        ids::PIXEL_SEC_PICKER,
        ids::PIXEL_SEC_SYMMETRY,
        ids::PIXEL_SEC_SELECT,
        ids::PIXEL_SEC_EDIT,
        ids::PIXEL_SEC_SPRITE,
    ] {
        me.host.set_collapsed(s, s != abre);
    }
    me.assenta();
}

/// ⭐ Os verbos dos três menus, nos cartões: todos pintam e o pedido da tabela chega. `fora` = os que
/// moram noutro cartão. *Mutação: tirar um `botoes` do `menu_edit` ⇒ «não pintado».*
#[test]
fn os_verbos_dos_menus_nenhum_morre() {
    let i = info(PixelTool::Pencil);
    let moldura = [ids::EDIT_ACTIONS[15].0, ids::EDIT_ACTIONS[16].0];
    type Tabela<'a> = (&'a [(NodeId, PixelRequest, &'a str)], NodeId);
    let tabelas: [Tabela<'_>; 3] = [
        (&ids::SELECT_ACTIONS, ids::PIXEL_SEC_SELECT),
        (&ids::EDIT_ACTIONS, ids::PIXEL_SEC_EDIT),
        (&ids::SPRITE_ACTIONS, ids::PIXEL_SEC_SPRITE),
    ];
    for (tabela, secao) in tabelas {
        for (id, pedido, chave) in tabela {
            if moldura.contains(id) {
                continue;
            }
            let mut me = mesa(i.clone());
            so_aberto(&mut me, secao);
            assert!(me.rect(*id).is_some(), "«{chave}» não pintado");
            let a = me.clica(*id);
            assert!(pede(&a).contains(pedido), "«{chave}»: {a:?}");
        }
    }
}

/// A ponta, a tinta, a simetria e o ladrilho do Pencil escrevem nas opções.
#[test]
fn as_opcoes_do_lapis_chegam() {
    let mut me = mesa(info(PixelTool::Pencil));
    for (forma, id) in ids::SHAPES {
        let pedido = PixelRequest::Brush(PixelBrushAction::Shape(forma));
        assert!(pede(&me.clica(id)).contains(&pedido), "{forma:?}");
    }
    for (tinta, id) in ids::INKS {
        assert_eq!(opcoes(&me.clica(id)).ink, tinta);
    }
    assert!(opcoes(&me.clica(ids::PIXEL_PERFECT)).pixel_perfect);
    assert!(opcoes(&me.clica(ids::SYM_H)).sym_h);
    assert!(opcoes(&me.clica(ids::SYM_V)).sym_v);
    assert!(opcoes(&me.clica(ids::SYM_D1)).sym_d1);
    assert!(opcoes(&me.clica(ids::SYM_D2)).sym_d2);
    assert!(opcoes(&me.clica(ids::TILE_X)).tile_x);
    assert!(opcoes(&me.clica(ids::TILE_Y)).tile_y);
}

/// ⭐ O número: a pega arrasta o `SIZE`; o Esc devolve. *Mutação: tirar o `campos_no_cartao` ⇒ sem campo.*
#[test]
fn o_tamanho_do_pincel_responde_a_pega_e_ao_esc() {
    let mut me = mesa(info(PixelTool::Pencil));
    let antes = me.host.store().number_value(ids::SIZE).expect("semeado");
    let ev = me.arrasta_a_pega(ids::SIZE, 40.0);
    let o = opcoes(&me.aplica(ev));
    assert!(f64::from(o.size) > antes, "{} de {antes}", o.size);
    let ev = me.esc();
    let o = opcoes(&me.aplica(ev));
    assert_eq!(f64::from(o.size), antes, "o Esc não devolveu");
}

/// ⭐ O tamanho da paleta divide a linha com a cor da âncora, e continua vivo: a pega pede o mesmo id.
/// *Mutação: tirar o campo de `ancora_no_cartao` ⇒ sem rect.*
#[test]
fn o_tamanho_da_paleta_vive_ao_lado_da_ancora() {
    let mut me = mesa(info(PixelTool::Pencil));
    so_aberto(&mut me, ids::PIXEL_SEC_PALETTE);
    let (cor, tamanho) = (
        me.rect(ids::PIXEL_PAL_EDIT).expect("âncora"),
        me.rect(ids::PALETTE_SIZE)
            .expect("o tamanho da paleta morreu"),
    );
    assert!(cor.x + cor.w <= tamanho.x, "sem sobreposição");
    assert_eq!(cor.y, tamanho.y, "a mesma linha");
    let antes = me
        .host
        .store()
        .number_value(ids::PALETTE_SIZE)
        .expect("semeado");
    let ev = me.arrasta_a_pega(ids::PALETTE_SIZE, 40.0);
    assert!(!ev.is_empty(), "a pega não respondeu");
    assert_ne!(
        me.host.store().number_value(ids::PALETTE_SIZE),
        Some(antes),
        "o número não mudou"
    );
}

/// Ajuste de ferramenta não é canal (D17): nem o campo nem a escolha pintam a marca de animação.
#[test]
fn os_ajustes_da_ferramenta_nao_levam_marca() {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_current_pixel(info(PixelTool::Pencil));
    set_key_marks(BTreeMap::from([
        (ids::SIZE, KeyMark::Chave),
        (ids::SHAPES[0].1, KeyMark::Chave),
        (ids::PIXEL_PERFECT, KeyMark::Chave),
    ]));
    let me = Mesa::<PixelToolsPanel>::nova(PixelToolsPanelState::default());
    for id in [ids::SIZE, ids::SHAPES[0].1, ids::PIXEL_PERFECT] {
        assert!(me.rect(key_mark_id(id)).is_none(), "{id:?} ganhou marca");
    }
    set_key_marks(BTreeMap::new());
}

/// As opções de cada ferramenta aparecem com ela e só com ela.
#[test]
fn cada_ferramenta_mostra_as_suas_opcoes() {
    let tem = |t, id| mesa(info(t)).rect(id).is_some();
    assert!(tem(PixelTool::Bucket, ids::TOLERANCE));
    assert!(!tem(PixelTool::Pencil, ids::TOLERANCE));
    assert!(tem(PixelTool::Spray, ids::SPRAY_WIDTH));
    assert!(tem(PixelTool::MagicWand, ids::WAND_TOLERANCE));
    assert!(tem(PixelTool::RectMarquee, ids::SELECT_MODES[3].1));
    assert!(
        !tem(PixelTool::Eyedropper, ids::SYM_H),
        "o conta-gotas não pinta"
    );
    assert!(
        !tem(PixelTool::Bucket, ids::SAVE_HERE),
        "o balde não leva pincel"
    );
    assert!(tem(PixelTool::Transform, ids::AUTO_SELECT_LAYER));
}

/// ⭐ A paleta dentro do cartão: esquerdo = frente, direito = fundo, a amostra acende.
#[test]
fn a_paleta_no_cartao_e_a_frente_e_o_fundo() {
    let mut me = mesa(info(PixelTool::Pencil));
    let a = me.clica(ids::palette_id(5));
    assert!(
        pede(&a).contains(&PixelRequest::Swatch {
            index: 5,
            how: PixelSelect::Only,
        }),
        "{a:?}"
    );
    let r = me.rect(ids::palette_id(9)).expect("pintada");
    let ev = me.host.right_click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    let a = me.aplica(ev);
    let cor = info(PixelTool::Pencil).palette[9];
    assert!(pede(&a).contains(&PixelRequest::Bg(cor)), "{a:?}");
}

fn ponteiro(me: &mut Mesa<PixelToolsPanel>, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
    me.host.dispatch_pointer_event(PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 0,
    })
}

/// ⭐⭐ Arrastar uma amostra dentro do cartão voa, abre o vão onde ela cai e não pede nada até largar.
/// *Mutação: tirar o `voo::voa` de `pinta` não muda a vista, mas tirar `paleta_vista` do corpo ⇒ sem voo.*
#[test]
fn arrastar_uma_amostra_no_cartao_voa_e_pede_ao_largar() {
    let mut me = mesa(info(PixelTool::Pencil));
    let a = me.rect(ids::palette_id(4)).expect("a");
    let b = me.rect(ids::palette_id(8)).expect("b");
    let (x0, y0, x1, y1) = (
        a.x + a.w * 0.5,
        a.y + a.h * 0.5,
        b.x + b.w * 0.5,
        b.y + b.h * 0.5,
    );
    let mut ev = ponteiro(&mut me, PointerKind::Down, x0, y0);
    ev.extend(ponteiro(&mut me, PointerKind::Move, x1, y1));
    assert!(
        pede(&me.aplica(ev)).is_empty(),
        "a meio do arrasto não se pede nada"
    );
    me.assenta();
    assert!(me.st.palette_flight().is_some(), "o bloco não voa");
    assert!(
        me.st.palette_painted().iter().any(Option::is_none),
        "o vão não abriu: {:?}",
        me.st.palette_painted()
    );
    let ev = ponteiro(&mut me, PointerKind::Up, x1, y1);
    let a = me.aplica(ev);
    assert!(
        pede(&a)
            .iter()
            .any(|r| matches!(r, PixelRequest::DragSwatch { .. })),
        "largar não pediu a troca: {a:?}"
    );
}

/// O seletor segue o ponteiro no cartão, antes de largar: os rects saem do corpo do cartão.
#[test]
fn o_seletor_no_cartao_segue_o_ponteiro() {
    let mut me = mesa(info(PixelTool::Pencil));
    let (area, _) = ids::SELECTOR_AREAS[0];
    let r = me.rect(area).expect("o quadro pinta-se");
    let (x, y) = (r.x + r.w * 0.3, r.y + r.h * 0.3);
    let ev = ponteiro(&mut me, PointerKind::Down, x, y);
    let _ = me.aplica(ev);
    me.assenta();
    let primeira = me.host.drained_actions();
    let _ = ponteiro(&mut me, PointerKind::Move, x + 30.0, y + 30.0);
    me.assenta();
    let depois = me.host.drained_actions();
    let cores = |a: &[EditorAction]| {
        pede(a)
            .into_iter()
            .filter(|r| matches!(r, PixelRequest::Fg(_)))
            .count()
    };
    assert!(
        cores(&depois) > 0,
        "o arrasto não pediu cor: {primeira:?} · {depois:?}"
    );
    let _ = ponteiro(&mut me, PointerKind::Up, x + 30.0, y + 30.0);
}

/// As camadas no cartão: olho, seleção e a mistura que abre pelo despacho de sempre.
#[test]
fn as_camadas_no_cartao_respondem() {
    use ph2d_editor_core::pixel_layer_row::PixelLayerWidget as L;
    let mut me = mesa(info(PixelTool::Pencil));
    for w in [L::Eye, L::Lock, L::Solo, L::Row] {
        let id = ids::layer_widget_id(1, w);
        assert!(!pede(&me.clica(id)).is_empty(), "{w:?}");
    }
    let caixa = ids::LAYER_BLEND;
    let r = me.rect(caixa).expect("a mistura");
    let _ = me.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    me.assenta();
    let opcao = ids::blend_option_id(2);
    let a = me.clica(opcao);
    assert!(!pede(&a).is_empty(), "a opção não chegou: {a:?}");
    for (id, _, chave) in &ids::LAYER_ACTIONS {
        // O Convert To troca com o tipo da camada activa.
        let mut i = info(PixelTool::Pencil);
        i.active_tilemap = *chave == "panel.pixel.layer.to_image";
        let mut m2 = mesa(i);
        assert!(!pede(&m2.clica(*id)).is_empty(), "«{chave}»");
    }
}

/// Os pincéis guardados: recuperar, trancar, apagar; a Dynamics abre e os sensores escrevem.
#[test]
fn os_pincéis_guardados_e_a_dinamica_respondem() {
    let mut me = mesa(info(PixelTool::Pencil));
    for id in [
        ids::brush_slot_id(0),
        ids::brush_lock_id(0),
        ids::brush_delete_id(0),
        ids::SAVE_HERE,
    ] {
        assert!(!pede(&me.clica(id)).is_empty(), "{id:?}");
    }
    assert!(!me.st.dynamics_open());
    let _ = me.clica(ids::DYNAMICS);
    me.assenta();
    assert!(me.st.dynamics_open(), "a Dynamics não abriu");
    let id = ids::sensor_id(ids::DynRow::Size, 1);
    let o = opcoes(&me.clica(id));
    assert_ne!(o.dynamics.size.sensor, PixelSensor::Off);
}

/// A barra da paleta: ≡ e o menu das predefinições abrem como antes e as linhas respondem.
#[test]
fn os_menus_da_paleta_abrem_no_cartao() {
    let mut me = mesa(info(PixelTool::Pencil));
    let _ = me.clica(ids::PALETTE_MENU);
    me.assenta();
    let (id, pedido, _) = &ids::ACTIONS[0];
    let a = me.clica(*id);
    assert!(pede(&a).contains(pedido), "{a:?}");
}

/// Sem Pixel Sprite em Draw, um cartão com a dica e nenhum controlo.
#[test]
fn fora_do_sprite_so_ha_a_dica() {
    let me = mesa(PixelPanelInfo::default());
    assert!(me.rect(ids::PIXEL_SEC_OPTIONS).is_some());
    assert!(me.rect(ids::PIXEL_SEC_PALETTE).is_none());
}

/// O «×» fecha; os três menus nascem dobrados, e a escolha do artista ganha à semente.
#[test]
fn os_menus_nascem_dobrados_e_o_x_fecha() {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_current_pixel(info(PixelTool::Pencil));
    let mut me = Mesa::<PixelToolsPanel>::nova(PixelToolsPanelState::default());
    assert!(me.host.store().is_collapsed(ids::PIXEL_SEC_EDIT));
    assert!(
        me.rect(ids::EDIT_ACTIONS[17].0).is_none(),
        "dobrado, sem alvo"
    );
    let r = me.rect(ids::PIXEL_SEC_EDIT).expect("cabeçalho");
    let _ = me.host.click_at(r.x + r.w * 0.3, r.y + 13.0);
    me.assenta();
    assert!(me.rect(ids::EDIT_ACTIONS[17].0).is_some(), "o clique abriu");
    let _ = me.clica(ids::PIXEL_CLOSE);
    assert!(
        !me.host
            .panel_visible(ph2d_editor_core::screens::hero::PANEL_PIXEL)
    );
}

/// Com células levantadas, o cartão da moldura: os campos e o Apply | Cancel.
#[test]
fn a_moldura_tem_o_seu_cartao() {
    let mut i = info(PixelTool::Transform);
    i.transform = Some(MOLDURA);
    let mut me = mesa(i);
    so_aberto(&mut me, ids::PIXEL_SEC_TRANSFORM);
    for (_, id, _) in ids::XFORM_FIELDS {
        assert!(me.rect(id).is_some(), "{id:?}");
    }
    for k in [15, 16] {
        let (id, pedido, chave) = &ids::EDIT_ACTIONS[k];
        let a = me.clica(*id);
        assert!(pede(&a).contains(pedido), "«{chave}»: {a:?}");
    }
    assert!(
        mesa(info(PixelTool::Transform))
            .rect(ids::PIXEL_SEC_TRANSFORM)
            .is_none()
    );
}

/// `PH2D_CARDS=0`: o corpo de antes.
#[test]
fn cards_off_pinta_o_corpo_de_antes() {
    ferramenta::force_cards_on_this_thread(Some(false));
    set_current_pixel(info(PixelTool::Pencil));
    let me = Mesa::<PixelToolsPanel>::nova(PixelToolsPanelState::default());
    assert!(me.rect(ids::PIXEL_SEC_OPTIONS).is_none());
    assert!(me.rect(ids::SIZE).is_some());
    assert!(me.host.store().card_field(ids::SIZE).is_none());
}
