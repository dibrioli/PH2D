//! ⭐ **O Inspector em cartões para todo tipo de objecto** (`docs/CartoesUI/10`, censo A2): por gesto
//! real nos rects pintados, com a lista ligada. `PH2D_CARDS=0` continua a pintar o painel de antes.

use crate::seam_anim::anim;
use crate::seam_render_source::atlas_sprite;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    InspectorInstanceInfo, InspectorOrderingInfo, InspectorOrderingMixed, InspectorPropertiesInfo,
    InspectorSamplingInfo, InspectorSamplingMixed, InspectorSliceInfo, InspectorSliceMixed,
    InspectorSpriteInfo, InspectorTransformInfo, InspectorVisibilityInfo, InspectorVisibilityMixed,
    InspectorVisibilitySectionInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{ImageUseEdit, InspectorDraw, InspectorImageUse};
use ph2d_panel_inspector::ids::{
    INSP_ANIM_ROW, INSP_IMAGE_USE, INSP_ORDER_ORDER_IN_LAYER, INSP_ORDER_SHOW_BEHIND,
    INSP_ORDER_SORTING_GROUP, INSP_ORDER_TOP_LEVEL, INSP_ORDER_YSORT_ENABLED, INSP_ORDER_Z_INDEX,
    INSP_ORDER_Z_RELATIVE, INSP_RENDER_TEXTURE_SLOT, INSP_SPRITE_EMISSIVE,
    INSP_SPRITE_EMISSIVE_CHIP, INSP_SPRITE_OPACITY, INSP_SPRITE_OPACITY_CHIP,
    INSP_SPRITE_TINT_SWATCH, INSP_TRANSFORM_POS_X,
};
use ph2d_panel_inspector::{
    CARD_ORDERING, InspectorPanel, InspectorState, set_cards_on, set_current_inspector_anim,
    set_current_inspector_instance, set_current_inspector_ordering,
    set_current_inspector_properties, set_current_inspector_sampling,
    set_current_inspector_selecionados, set_current_inspector_slice, set_current_inspector_sprite,
    set_current_inspector_transform, set_current_inspector_visibility,
    set_current_inspector_visibility_section,
};
use ph2d_tool_vector::ids as vids;
use ph2d_ui_testkit::MockPanelHost;

const BITS: u64 = 0x00AB_CD12;
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 8000.0,
};

fn limpa() {
    set_cards_on(false);
    set_current_inspector_sprite(None);
    set_current_inspector_transform(None);
    set_current_inspector_visibility(None);
    set_current_inspector_visibility_section(None);
    set_current_inspector_ordering(None);
    set_current_inspector_sampling(None);
    set_current_inspector_slice(None);
    set_current_inspector_anim(None);
}

fn objeto() {
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: BITS,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: BITS,
        visible: true,
        mixed: false,
    }));
    set_current_inspector_ordering(Some(InspectorOrderingInfo {
        entity_bits: BITS,
        z_index: None,
        z_as_relative: true,
        show_behind_parent: false,
        sorting_layer: 0,
        order_in_layer: 0,
        y_sort_enabled: false,
        y_sort_point: 0,
        y_sort_axis: [0.0, 0.0],
        sorting_group: false,
        sort_at_root: false,
        top_level: false,
        selected_count: 1,
        mixed: InspectorOrderingMixed::default(),
    }));
}

fn sprite(uso: InspectorImageUse) -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        entity_bits: BITS,
        image_use: uso,
        image_cells: 4,
        hframes: 4,
        vframes: 1,
        frame: 0,
        draw: InspectorDraw::Simple,
        ..atlas_sprite(true)
    }
}

pub(crate) fn pinta(host: &mut MockPanelHost, st: &mut InspectorState) -> Vec<(NodeId, Rect)> {
    pinta_n(host, st, 60)
}

fn pinta_n(host: &mut MockPanelHost, st: &mut InspectorState, n: usize) -> Vec<(NodeId, Rect)> {
    host.settle_section_folds();
    let mut r = Vec::new();
    for _ in 0..n {
        r = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint::<InspectorPanel>(st, JANELA),
        );
    }
    r
}

pub(crate) fn chaves_da_lista(host: &mut MockPanelHost) -> Vec<NodeId> {
    host.store_mut()
        .card_list_mut(core_ids::INSP_PANEL)
        .items
        .iter()
        .map(|it| it.key)
        .collect()
}

fn y_de(rects: &[(NodeId, Rect)], id: NodeId) -> f32 {
    rects
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| r.y)
        .unwrap_or_else(|| panic!("{id:?} não pintado"))
}

fn tem(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects.iter().any(|(i, _)| *i == id)
}

/// A imagem pinta-se em cartões, com a ordem que o dono decidiu: Image e Use logo abaixo do nome e
/// antes do Transform; a tinta e o resto depois. `PH2D_CARDS=0` pinta o painel de antes.
#[test]
fn a_imagem_pinta_em_cartoes_na_ordem_do_dono() {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    host.store_mut()
        .set_collapsed(core_ids::INSP_LIVE_COLOR_SECTION, false);
    let r = pinta(&mut host, &mut st);
    let (nome, slot, usa, pos, tinta) = (
        y_de(&r, core_ids::INSP_ENTITY_NAME),
        y_de(&r, INSP_RENDER_TEXTURE_SLOT),
        y_de(&r, INSP_IMAGE_USE[0]),
        y_de(&r, INSP_TRANSFORM_POS_X),
        y_de(&r, INSP_SPRITE_TINT_SWATCH),
    );
    assert!(
        nome < slot && slot < usa && usa < pos && pos < tinta,
        "nome {nome} · image {slot} · use {usa} · transform {pos} · tint {tinta}"
    );
    let chaves = chaves_da_lista(&mut host);
    for s in [
        ph2d_panel_inspector::CARD_IMAGE,
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
    ] {
        assert!(chaves.contains(&s), "sem o cartão {s:?}");
    }
    limpa();
    objeto();
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let r = pinta(&mut host, &mut st);
    assert!(
        !chaves_da_lista(&mut host).contains(&ph2d_panel_inspector::CARD_IMAGE),
        "cartões desligados não pintam o cartão Image"
    );
    assert!(
        tem(&r, INSP_RENDER_TEXTURE_SLOT),
        "o painel de antes tem a linha"
    );
    limpa();
}

/// O clique no seletor Use, no rect do cartão, pede o mesmo que no painel de antes.
#[test]
fn o_seletor_use_no_cartao_pede_o_uso() {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let r = pinta(&mut host, &mut st);
    let alvo = r
        .iter()
        .find(|(i, _)| *i == INSP_IMAGE_USE[1])
        .map(|(_, r)| *r)
        .expect("Repeated pintado");
    let evs = host.click_at(alvo.x + alvo.w * 0.5, alvo.y + alvo.h * 0.5);
    for ev in evs {
        host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let pedidos = host.drained_actions();
    limpa();
    assert_eq!(
        pedidos,
        vec![EditorAction::ImageUseEdit {
            entity_bits: BITS,
            edit: ImageUseEdit::Use(InspectorImageUse::Repeated),
        }]
    );
}

/// A Animation da sprite: um cartão logo abaixo da Image e só no uso Animated.
#[test]
fn a_animation_da_imagem_e_o_cartao_seguinte_e_so_no_uso_animated() {
    limpa();
    set_cards_on(true);
    objeto();
    let mut a = anim(true, true);
    a.entity_bits = BITS;
    set_current_inspector_anim(Some(a));
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Animated)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let r = pinta(&mut host, &mut st);
    let (usa, linha, pos) = (
        y_de(&r, INSP_IMAGE_USE[0]),
        y_de(&r, INSP_ANIM_ROW[0]),
        y_de(&r, INSP_TRANSFORM_POS_X),
    );
    assert!(usa < linha && linha < pos, "{usa} · {linha} · {pos}");
    assert!(chaves_da_lista(&mut host).contains(&core_ids::INSP_LIVE_ANIM_SECTION));
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let r = pinta(&mut host, &mut st);
    limpa();
    assert!(!tem(&r, INSP_ANIM_ROW[0]), "parada não mostra Animation");
}

/// Nenhuma linha do Ordering morre: as do painel de antes, para qualquer objecto.
#[test]
fn o_ordering_tem_todas_as_linhas_em_qualquer_objecto() {
    limpa();
    set_cards_on(true);
    objeto();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    host.store_mut().set_collapsed(CARD_ORDERING, false);
    let r = pinta(&mut host, &mut st);
    limpa();
    for id in [
        INSP_ORDER_Z_INDEX,
        INSP_ORDER_Z_RELATIVE,
        INSP_ORDER_SHOW_BEHIND,
        INSP_ORDER_ORDER_IN_LAYER,
        INSP_ORDER_YSORT_ENABLED,
        INSP_ORDER_SORTING_GROUP,
        INSP_ORDER_TOP_LEVEL,
    ] {
        assert!(tem(&r, id), "Ordering sem {id:?}");
    }
}

/// Sampling, 9-Slice e Visibility: quem tem o componente ganha o cartão, seja que objecto for.
#[test]
fn os_componentes_partilhados_tem_cartao_fora_da_imagem() {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_sampling(Some(InspectorSamplingInfo {
        entity_bits: BITS,
        filter_tag: 0,
        repeat_tag: 0,
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSamplingMixed::default(),
    }));
    set_current_inspector_slice(Some(InspectorSliceInfo {
        entity_bits: BITS,
        present: true,
        draw_mode_tag: 1,
        borders: [8.0; 4],
        size: [0.0, 0.0],
        tile_modes: [0; 8],
        centre_tile_mode: 0,
        tile_mode_tag: 1,
        fill_center: true,
        selected_count: 1,
        mixed: InspectorSliceMixed::default(),
    }));
    set_current_inspector_visibility_section(Some(InspectorVisibilitySectionInfo {
        entity_bits: BITS,
        layer_mask: 1,
        clip_mode: 0,
        mask_mode: 1,
        alpha_cutoff: 0.5,
        mask_source: false,
        on_screen: false,
        rect: [0.0, 0.0, 0.0, 0.0],
        selected_count: 1,
        mixed: InspectorVisibilityMixed::default(),
    }));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let _ = pinta(&mut host, &mut st);
    let chaves = chaves_da_lista(&mut host);
    limpa();
    for s in [
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        ph2d_panel_inspector::CARD_VISIBILITY,
    ] {
        assert!(chaves.contains(&s), "sem o cartão {s:?}");
    }
    assert!(!chaves.contains(&ph2d_panel_inspector::CARD_IMAGE));
}

/// Os giros são da forma vetorial: no resto seriam um clique sem efeito.
#[test]
fn os_giros_nao_aparecem_fora_do_vetor() {
    limpa();
    set_cards_on(true);
    objeto();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let r = pinta(&mut host, &mut st);
    limpa();
    assert!(tem(&r, INSP_TRANSFORM_POS_X));
    for id in [
        vids::VECTOR_ARRANGE_ROTATE_CW,
        vids::VECTOR_ARRANGE_FLIP_H,
        vids::VECTOR_ARRANGE_FLIP_V,
    ] {
        assert!(!tem(&r, id), "{id:?} morto no Transform");
    }
}

/// Sem objecto, a lista diz o que fazer em vez de ficar muda.
#[test]
fn sem_objecto_a_lista_diz_o_que_fazer() {
    limpa();
    set_cards_on(true);
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let _ = pinta(&mut host, &mut st);
    let chaves = chaves_da_lista(&mut host);
    limpa();
    assert_eq!(chaves, vec![ph2d_panel_inspector::CARD_EMPTY]);
}

/// Alfinete (plano 09, F4): Render e Color têm-no porque a escrita deles vai ao preso, gesto a gesto
/// (`ph2d-panel-registry-init`, `o_alfinete_da_sprite::todo_controlo_do_*_preso_escreve_no_preso`).
/// A Imagem continua sem ele: o corpo dela escreve pela seleção. Quem tinha, fica.
#[test]
fn so_tem_alfinete_o_cartao_de_sprite_que_escreve_no_preso() {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let _ = pinta(&mut host, &mut st);
    let store = host.store();
    let (render, cor, image, transform) = (
        store.is_card_pinnable(core_ids::INSP_LIVE_RENDER_SECTION),
        store.is_card_pinnable(core_ids::INSP_LIVE_COLOR_SECTION),
        store.is_card_pinnable(ph2d_panel_inspector::CARD_IMAGE),
        store.is_card_pinnable(ph2d_panel_inspector::CARD_TRANSFORM),
    );
    limpa();
    assert!(render && cor, "render {render}, cor {cor}");
    assert!(!image, "a Imagem escreve pela seleção: alfinete sem efeito");
    assert!(transform, "o Transform mantém o alfinete");
}

/// Tudo aberto, para o inventário: o que está dobrado não pinta.
pub(crate) fn abre_tudo(host: &mut MockPanelHost) {
    for id in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        core_ids::INSP_LIVE_ORDERING_SECTION,
        core_ids::INSP_LIVE_BLEND_SECTION,
        core_ids::INSP_LIVE_SCRIPT_SECTION,
        core_ids::INSP_VIS_LAYER_HEADER,
        ph2d_panel_inspector::CARD_MATERIAL,
        CARD_ORDERING,
    ] {
        host.store_mut().set_collapsed(id, false);
    }
}

fn ids_pintados(cartoes: bool) -> Vec<(NodeId, Rect)> {
    limpa();
    set_cards_on(cartoes);
    crate::every_painted_id_is_reachable::publish_the_whole_sprite_scene();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut host);
    // A primeira passada semeia a dobra de cada cartão; abre-se de novo por cima da semente.
    let _ = pinta(&mut host, &mut st);
    abre_tudo(&mut host);
    let r = pinta(&mut host, &mut st);
    crate::every_painted_id_is_reachable::clear_the_scene();
    limpa();
    r
}

/// ⭐ **Inventário: nenhum controlo da sprite morre.** Cada id do corpo que o painel de antes pinta
/// aparece na lista de cartões. Só ficam de fora a moldura do painel de antes (cabeçalhos, pegas,
/// fechar) e as duas barras: a receita (11) põe o chip no cartão, e a barra continua registada.
#[test]
fn todo_controlo_do_painel_de_antes_existe_nos_cartoes() {
    let antes = ids_pintados(false);
    let depois = ids_pintados(true);
    let faltam: Vec<NodeId> = antes
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !tem(&depois, *id) && !da_moldura(*id))
        .collect();
    assert!(faltam.is_empty(), "sem lugar nos cartões: {faltam:?}");
}

/// O que só existe na moldura do painel de antes: cabeçalhos e pegas de secção, fechar e o «+».
fn da_moldura(id: NodeId) -> bool {
    let cabecalhos = [
        core_ids::INSP_LIVE_NAME_SECTION,
        core_ids::INSP_LIVE_VISIBILITY_SECTION,
        core_ids::INSP_LIVE_TRANSFORM_SECTION,
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        core_ids::INSP_LIVE_ORDERING_SECTION,
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        core_ids::INSP_LIVE_BLEND_SECTION,
        core_ids::INSP_LIVE_ANCHOR_SECTION,
        core_ids::INSP_LIVE_ANIM_SECTION,
        core_ids::INSP_CLOSE,
        core_ids::INSP_PANEL,
        INSP_SPRITE_OPACITY,
        INSP_SPRITE_EMISSIVE,
        ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID,
    ];
    cabecalhos.contains(&id) || cabecalhos.iter().any(|c| core_ids::grip_of(*c) == Some(id))
}

/// O que um clique no rect pintado de `id` pede, com a lista ligada ou não.
fn clique(cartoes: bool, id: NodeId) -> Option<(Vec<String>, Option<NodeId>)> {
    limpa();
    set_cards_on(cartoes);
    crate::every_painted_id_is_reachable::publish_the_whole_sprite_scene();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut host);
    let _ = pinta_n(&mut host, &mut st, 2);
    abre_tudo(&mut host);
    let r = pinta_n(&mut host, &mut st, 12);
    let alvo = r
        .iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0 && q.h > 0.0)
        .map(|(_, q)| *q);
    let fora = alvo.map(|q| {
        let evs = host.click_at(q.x + q.w * 0.5, q.y + q.h * 0.5);
        for ev in evs {
            host.apply_panel_event::<InspectorPanel>(&mut st, ev);
        }
        let _ = host.paint::<InspectorPanel>(&mut st, JANELA);
        let pedidos = host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect::<Vec<_>>();
        (pedidos, host.store().picker_target())
    });
    crate::every_painted_id_is_reachable::clear_the_scene();
    limpa();
    fora
}

/// ⭐ **Por gesto, id a id:** clicar no rect de cada controlo pede o mesmo nos cartões e em
/// `PH2D_CARDS=0`. Os campos de número só ganham foco (lista vazia nos dois lados).
#[test]
fn cada_clique_pede_o_mesmo_que_no_painel_de_antes() {
    let antes = ids_pintados(false);
    let depois = ids_pintados(true);
    let mut comuns: Vec<NodeId> = antes
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| tem(&depois, *id) && !da_moldura(*id))
        .collect();
    comuns.sort_by_key(|n| n.0);
    comuns.dedup();
    let mut com_efeito = 0;
    let mut diferentes = Vec::new();
    for id in &comuns {
        let (a, d) = (clique(false, *id), clique(true, *id));
        if a != d {
            diferentes.push((*id, a.clone(), d.clone()));
        }
        if a.as_ref()
            .is_some_and(|(p, c)| !p.is_empty() || c.is_some())
        {
            com_efeito += 1;
        }
    }
    assert!(diferentes.is_empty(), "{diferentes:#?}");
    assert!(
        com_efeito >= 20 && comuns.len() >= 40,
        "amostra magra: {com_efeito} com efeito em {}",
        comuns.len()
    );
}

/// Barra + chip no cartão: o campo chega à barra, e a barra é o que escreve no objecto.
#[test]
fn escrever_na_opacidade_e_na_emissao_do_cartao_chega_ao_objecto() {
    for (chip, barra, quer) in [
        (INSP_SPRITE_OPACITY_CHIP, INSP_SPRITE_OPACITY, "Opacity"),
        (INSP_SPRITE_EMISSIVE_CHIP, INSP_SPRITE_EMISSIVE, "Emissive"),
    ] {
        limpa();
        set_cards_on(true);
        crate::every_painted_id_is_reachable::publish_the_whole_sprite_scene();
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        abre_tudo(&mut host);
        let _ = pinta(&mut host, &mut st);
        abre_tudo(&mut host);
        let r = pinta(&mut host, &mut st);
        assert!(!tem(&r, barra), "a barra não se pinta no cartão");
        let q = r
            .iter()
            .rev()
            .find(|(i, _)| *i == chip)
            .map(|(_, q)| *q)
            .expect("o chip no cartão");
        let (a, b) = (q.x + q.w * 0.5, q.y + q.h * 0.5);
        let mut evs = host.drag_at(a, b, a - 30.0, b);
        evs.extend(host.drag_at(a, b, a - 60.0, b));
        for ev in evs {
            host.apply_panel_event::<InspectorPanel>(&mut st, ev);
        }
        let _ = host.paint::<InspectorPanel>(&mut st, JANELA);
        let pedidos: Vec<String> = host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect();
        crate::every_painted_id_is_reachable::clear_the_scene();
        limpa();
        assert!(
            pedidos.iter().any(|p| p.contains(quer)),
            "o arrasto em {quer} não pediu nada: {pedidos:?}"
        );
    }
}

/// Os três facto do topo: a frase da selecção, a cópia e as propriedades têm lugar na lista.
#[test]
fn a_frase_a_copia_e_as_propriedades_tem_lugar_na_lista() {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Still)));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let um = y_de(&pinta(&mut host, &mut st), INSP_RENDER_TEXTURE_SLOT);
    set_current_inspector_selecionados(3);
    set_current_inspector_instance(Some(InspectorInstanceInfo {
        entity_bits: BITS,
        master_name: "Car".into(),
        overridden: Vec::new(),
        orphan_rows: Vec::new(),
        root_bits: BITS,
        is_variant: false,
        apply_levels: Vec::new(),
        apply_levels_beyond: 0,
        removed_rows: Vec::new(),
        added_rows: Vec::new(),
    }));
    set_current_inspector_properties(Some(InspectorPropertiesInfo {
        entity_bits: BITS,
        root_bits: BITS,
        rows: Vec::new(),
        beyond: 0,
        source_name: Some("Casa".into()),
    }));
    let tres = y_de(&pinta(&mut host, &mut st), INSP_RENDER_TEXTURE_SLOT);
    let chaves = chaves_da_lista(&mut host);
    set_current_inspector_selecionados(0);
    set_current_inspector_instance(None);
    set_current_inspector_properties(None);
    limpa();
    assert!(
        tres > um,
        "a frase e os cartões descem a Image: {um} → {tres}"
    );
    for s in [
        ph2d_panel_inspector::CARD_INSTANCE,
        ph2d_panel_inspector::CARD_PROPERTIES,
    ] {
        assert!(chaves.contains(&s), "sem o cartão {s:?}");
    }
    let (obj, copia, img) = (
        chaves
            .iter()
            .position(|k| *k == ph2d_panel_inspector::CARD_OBJECT),
        chaves
            .iter()
            .position(|k| *k == ph2d_panel_inspector::CARD_INSTANCE),
        chaves
            .iter()
            .position(|k| *k == ph2d_panel_inspector::CARD_IMAGE),
    );
    assert!(obj < copia && copia < img, "{obj:?} {copia:?} {img:?}");
}
