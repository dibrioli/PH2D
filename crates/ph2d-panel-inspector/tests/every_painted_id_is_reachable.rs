//! **TODO ID QUE O INSPECTOR REGISTA NO ÍNDICE DE ACERTO TEM DE SER ALCANÇÁVEL PELO PONTEIRO.**
//!
//! # O defeito, e por que ele não faz barulho nenhum
//!
//! Registar um rectângulo no `HitIndex` é dizer *"o rato que cair aqui é meu"*. Mas quem decide se
//! aquele acerto vira um `Click` é o [`is_focusable`] do despachante
//! ([`interaction/dispatch/focus.rs`](../../ph2d-editor-core/src/interaction/dispatch/focus.rs)),
//! e ele tem **dois** caminhos para dizer sim — `is_collapsible_section(id)`, ou uma entrada no
//! `WidgetStore`. Um id que falha os dois cai no ramo `None => false`: não fica `active` no Down e
//! **nunca emite `Click`**.
//!
//! ⚠️ **Nada nessa cadeia falha alto.** Sem erro, sem log, sem warning. O controlo pinta, o cursor
//! muda, o artista clica — e não acontece coisa nenhuma.
//!
//! # Como isto foi encontrado (auditoria de 7 lentes, 2026-08-21)
//!
//! Três cabeçalhos de seção — **Ordering**, **Sampling** e **Material & Blend** — pintavam o
//! chevron de dobrar e não dobravam, porque estavam fora do laço `mark_collapsible_section`. E
//! cinco pontos de cor de seção estavam pintados sem registo nenhum.
//!
//! ⚠️ **A mesma família já tinha sido curada ao lado.** O comentário em
//! [`pre_populate.rs`](../../ph2d-editor-core/src/screens/hero/pre_populate.rs) descreve o
//! mecanismo palavra por palavra — *«ambos pintam um chevron `.collapsible(...)` e leem
//! `is_collapsed`, e ambos faltavam aqui — então o chevron prometia uma dobra que não podia
//! acontecer»* — e a cura foi escrita para a §11/§12 **sem cobrir as vizinhas**. *Uma cura escrita
//! ao lado do buraco não o cobre; um gate derivado cobre.*
//!
//! # Por que a fonte é o que o PAINEL PINTA, e não uma lista
//!
//! ⛔ Uma lista de ids dentro deste ficheiro teria de ser atualizada para apanhar o controlo novo —
//! e um gate que precisa de ser atualizado para apanhar o caso novo **não apanha caso novo nenhum**
//! (foi assim que o `node_id_collisions.rs` ficou cego a 78 dos 90 ids desta superfície). Aqui a
//! lista é a **saída do `paint`**: o `MockPanelHost::paint` devolve exatamente os pares
//! `(id, rect)` que as seções registaram. Um controlo novo entra nesta varredura no dia em que é
//! pintado, sem ninguém se lembrar dele.
//!
//! ⚠️ **A barra é a do `is_focusable`, não uma inventada aqui.** Não se exige `Button`: as linhas
//! de menu e os pontos de cor registam-se como `Plain`, que o `is_focusable` aceita. *Um gate que
//! acusa o legítimo é desligado na primeira semana* (a primeira versão do gate irmão
//! `every_menu_row_is_registered` exigiu `Button` e acusou as dezassete linhas, incluindo as que
//! funcionavam há meses).
//!
//! [`is_focusable`]: ph2d_editor_core::interaction

use ph2d_editor_core::screens::hero::{
    InspectorAnchorInfo, InspectorAnchorRow, InspectorBlendInfo, InspectorBlendMixed,
    InspectorNameInfo, InspectorOrderingInfo, InspectorOrderingMixed, InspectorSamplingInfo,
    InspectorSamplingMixed, InspectorSliceInfo, InspectorSliceMixed, InspectorSpriteInfo,
    InspectorSpriteMixed, InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    InspectorVisibilityMixed, InspectorVisibilitySectionInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_anchor, set_current_inspector_blend,
    set_current_inspector_name, set_current_inspector_ordering, set_current_inspector_sampling,
    set_current_inspector_slice, set_current_inspector_sprite, set_current_inspector_transform,
    set_current_inspector_visibility, set_current_inspector_visibility_section,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_1234;

/// **A cena completa do lado SPRITE**, que é o alcance desta varredura.
///
/// ⚠️ As seções de física/joint/roldana/player ficam a `None` de propósito: elas têm os seus
/// próprios seams (`seam_physics`/`seam_joint`/`seam_wheel`/`seam_player`) e varrê-las aqui faria
/// este gate falhar por trabalho de outra linha — *um gate que acusa o vizinho é desligado tão
/// depressa como um que acusa o legítimo*.
fn publish_the_whole_sprite_scene() {
    set_current_inspector_sprite(Some(InspectorSpriteInfo {
        entity_bits: ENTITY,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        emissive: 0.0,
        sheet_label: None,
        source_pixels: Some((256, 256)),
        can_reimport: true,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        // ⚠️ **A grelha é 4×2 e a região está LIGADA de propósito.** Os dois fixtures que já
        // existiam punham `1×1` e `region_enabled: false`, e `render_source.rs` só pinta as rows
        // de sub-rect quando a checkbox está marcada — por isso `INSP_REGION_X/Y/W/H` nunca eram
        // pintados em teste nenhum, num ficheiro que dizia cobrir «a costura da seção Render
        // Source». *Um fixture só prova o que contém.*
        hframes: 4,
        vframes: 2,
        frame: 3,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: true,
        region_rect: [8.0, 8.0, 64.0, 64.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }));
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "hero_idle".into(),
    }));
    set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: ENTITY,
        visible: true,
        mixed: false,
    }));
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: ENTITY,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_current_inspector_ordering(Some(InspectorOrderingInfo {
        entity_bits: ENTITY,
        z_index: None,
        z_as_relative: true,
        show_behind_parent: false,
        sorting_layer: 2,
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
    set_current_inspector_sampling(Some(InspectorSamplingInfo {
        entity_bits: ENTITY,
        filter_tag: 0,
        repeat_tag: 0,
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSamplingMixed::default(),
    }));
    // §5 9-Slice. ⚠️ `Tiled` de propósito: é o estado em que TODOS os controlos da seção são
    // pintados. Um fixture em `Simple` deixaria onze ids fora da amostra — em `Simple` a seção
    // está DESLIGADA e mostra só a dica e o «Remove» (o modo é a seção desligada). *Um fixture
    // que não contém o fenómeno que o gate diz medir.*
    set_current_inspector_slice(Some(InspectorSliceInfo {
        entity_bits: ENTITY,
        present: true,
        draw_mode_tag: 2,
        borders: [8.0; 4],
        size: [0.0, 0.0],
        tile_modes: [0; 8],
        centre_tile_mode: 0,
        tile_mode_tag: 1,
        fill_center: true,
        selected_count: 1,
        mixed: InspectorSliceMixed::default(),
    }));
    // §12 Sockets / Named Anchors. ⚠️ **Uma âncora do tipo `Region`** (bounds E center), porque
    // é o único estado em que TODOS os campos do editor são pintados — um Socket esconde oito
    // NumberInputs, e um fixture assim deixaria-os fora da amostra.
    set_current_inspector_anchor(Some(InspectorAnchorInfo {
        entity_bits: ENTITY,
        rows: vec![InspectorAnchorRow {
            name: "muzzle".into(),
            pos: [28.0, -4.0],
            rot_deg: 12.0,
            bounds: Some([8.0, 4.0, 24.0, 24.0]),
            center: Some([2.0, 2.0, 8.0, 8.0]),
        }],
        present: true,
        selected_count: 1,
        mixed: false,
    }));
    set_current_inspector_blend(Some(InspectorBlendInfo {
        entity_bits: ENTITY,
        blend_tag: 0,
        selected_count: 1,
        mixed: InspectorBlendMixed::default(),
    }));
    set_current_inspector_visibility_section(Some(InspectorVisibilitySectionInfo {
        entity_bits: ENTITY,
        layer_mask: 1,
        clip_mode: 0,
        // ⚠️ Não-zero de propósito: `visibility.rs:205` só pinta a row `alpha_cutoff` quando o
        // sprite obedece a uma máscara.
        mask_mode: 1,
        alpha_cutoff: 0.5,
        mask_source: false,
        on_screen: false,
        rect: [0.0, 0.0, 0.0, 0.0],
        selected_count: 1,
        mixed: InspectorVisibilityMixed::default(),
    }));
}

fn clear_the_scene() {
    set_current_inspector_sprite(None);
    set_current_inspector_name(None);
    set_current_inspector_visibility(None);
    set_current_inspector_transform(None);
    set_current_inspector_ordering(None);
    set_current_inspector_sampling(None);
    set_current_inspector_blend(None);
    set_current_inspector_visibility_section(None);
}

/// **A varredura.** Pinta o Inspector inteiro com um sprite selecionado e exige que cada id que ele
/// registou passe na mesma pergunta que o despachante faz.
#[test]
fn every_id_the_inspector_paints_can_actually_be_clicked() {
    // ⚠️ **`with_panel_and_shared_chrome`, nunca `with_panel`.** O app popula o store a partir de
    // DUAS fontes (chrome partilhado + `Panel::populate`), e um harness que corre só a segunda
    // responde «não registado» a ids que estão registados — a primeira versão desta varredura
    // acusou 22 ids e **14 eram falso-positivo desta lacuna**. *Um gate que acusa o legítimo é
    // desligado na primeira semana.*
    let mut host = MockPanelHost::with_panel_and_shared_chrome::<InspectorPanel>();
    let mut state = InspectorState::default();
    publish_the_whole_sprite_scene();
    // Abre as dobras: uma seção colapsada não pinta o corpo, e um gate que mede metade do painel
    // não mede o painel.
    host.settle_section_folds();
    // ⚠️ Alto de propósito (o painel real rola). Um viewport curto cortaria as seções de baixo —
    // que são precisamente as três que esta auditoria encontrou partidas.
    let painted = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 8000.0));

    assert!(
        painted.len() > 40,
        "a cena nao pintou o painel inteiro (so' {} ids) — sem isto a varredura mede metade do \
         painel e passa por acidente",
        painted.len()
    );

    let mut dead: Vec<ph2d_a11y::NodeId> = Vec::new();
    for (id, _rect) in &painted {
        // ⚠️ **Esta é a regra do `is_focusable`, copiada dele e não inventada aqui:** um cabeçalho
        // de seção passa por `is_collapsible_section`; todo o resto precisa de uma entrada no
        // store. Falhar os dois é o ramo `None => false` — o clique nunca nasce.
        let reachable = host.store().is_collapsible_section(*id) || host.store().get(*id).is_some();
        if !reachable {
            dead.push(*id);
        }
    }
    dead.sort_by_key(|n| n.0);
    dead.dedup();

    clear_the_scene();

    assert!(
        dead.is_empty(),
        "estes {} ids sao PINTADOS e registados no indice de acerto, mas o despachante recusa-os \
         (`is_focusable` -> `None => false`): o clique nunca vira `Click` e o artista ve' um \
         controlo morto.\n  {:?}\n\n\
         Cure em `screens/hero/pre_populate.rs`: um cabecalho de secao entra no laco \
         `mark_collapsible_section`; qualquer outro controlo entra no laco `register(id, \
         InteractiveState::Plain)` (ou com o estado que lhe convier).\n\n\
         ⚠️ Um controlo vive em TRES sitios — o que se PINTA, o que se REGISTA (isto) e o que se \
         DESPACHA. Faltar o do meio nao da' erro nenhum: da' um controlo que nao faz nada.",
        dead.len(),
        dead
    );
}
