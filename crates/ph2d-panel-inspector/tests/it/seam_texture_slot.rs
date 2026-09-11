//! **Varredura de SEAM da RANHURA DA TEXTURA** (plano `docs/Components/07`, wave B3).
//!
//! # ⚠️ O que só se mede aqui
//!
//! A lei da queda (`asset_drop`) diz **o que acontece** quando uma imagem cai na ranhura; ela não
//! sabe se a ranhura chega a ser **pintada**, se está no **hit-index**, nem se o Inspector a
//! **traduz** de volta para a sprite que ela descreve. São três condições independentes, e as três
//! juntas é que fazem o gesto existir para o artista.
//!
//! ⛔⛔ **E a quarta é o CLIQUE.** A ranhura nasceu inerte, com a inércia declarada num doc, e
//! **dois censos independentes recusaram-na** — o da paridade e o do alcance. *A lei deste app é
//! que o que está no índice de acertos é um CONTROLO*, e uma zona que só recebe quedas não é uma
//! coisa que ele tenha. O clique abre a biblioteca, e é ele que torna a queda descoberta.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_sprite, texture_slot_pick,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5107_0001;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
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
    }
}

fn host() -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_sprite(Some(sprite()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

fn clear() {
    set_current_inspector_sprite(None);
}

/// ⭐⭐⭐ **A ranhura é PINTADA e está no hit-index** — sem isto a queda nunca a encontra.
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register` do `paint_texture_slot`.
#[test]
fn the_texture_slot_is_painted_and_hit_indexed() {
    let (mut host, mut state) = host();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_RENDER_TEXTURE_SLOT)
        .map(|(_, r)| *r)
        .expect("a ranhura nunca foi pintada nem registada");
    assert!(rect.w > 0.0 && rect.h > 0.0, "a ranhura tem area zero");
    // ⚠️ E o `hit_at` confirma que ela é **alcançável pelo ponto**, e não só publicada: é essa a
    // pergunta que a queda faz.
    assert_eq!(
        host.hit_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5),
        Some(ids::INSP_RENDER_TEXTURE_SLOT)
    );
    clear();
}

/// ⭐⭐⭐ **O painel TRADUZ o id de volta para a sprite que a ranhura descreve.**
///
/// ⚠️ É a porta que impede a shell de conhecer a tabela de ids de um painel — o molde do
/// `catalog_row_pick` do navegador.
///
/// **Mutação que deve sangrar:** `texture_slot_pick` devolver `None` sempre.
#[test]
fn the_panel_translates_the_slot_id_back_to_its_sprite() {
    let (_host, _state) = host();
    assert_eq!(
        texture_slot_pick(ids::INSP_RENDER_TEXTURE_SLOT),
        Some(ENTITY)
    );
    // ⛔ E um id qualquer não é a ranhura.
    assert_eq!(texture_slot_pick(ids::INSP_ENTITY_NAME), None);
    clear();
}

/// ⛔⛔ **Sem sprite não há ranhura** — a §3 não existe, e traduzir o id devolveria uma entidade
/// que ninguém está a ver.
///
/// ⚠️ **A metade de AUSÊNCIA precisa da irmã de PRESENÇA** (o gate acima): sozinha, ela fica verde
/// sobre um painel que nunca pinta a ranhura.
#[test]
fn with_no_sprite_there_is_no_slot() {
    clear();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_RENDER_TEXTURE_SLOT),
        "a ranhura foi pintada sem sprite nenhuma"
    );
    assert_eq!(texture_slot_pick(ids::INSP_RENDER_TEXTURE_SLOT), None);
}

/// ⭐⭐⭐ **Carregar na ranhura ABRE A BIBLIOTECA** — *«o que é que eu posso pôr aqui?»*.
///
/// ⛔⛔ **A 1.ª versão deste gate afirmava o CONTRÁRIO** (*«carregar não publica nada»*), e a
/// inércia estava declarada num doc como se isso a tornasse legítima. **Dois censos recusaram-na**
/// — o da paridade e o do alcance —, e a lição fica escrita: *a lei deste app é que o que está no
/// índice de acertos é um CONTROLO*. Uma inércia declarada num comentário não deixa de ser um
/// controlo morto.
///
/// **Mutação que deve sangrar:** tirar o `texture_slot_click` da tabela `SINGLE_ID_CLICKS`.
#[test]
fn clicking_the_texture_slot_opens_the_library() {
    let (mut host, mut state) = host();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_RENDER_TEXTURE_SLOT)
        .map(|(_, r)| *r)
        .expect("a ranhura nunca foi pintada");
    for ev in host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let sent = host.drained_actions();
    assert!(
        sent.iter().any(|a| matches!(
            a,
            ph2d_editor_core::action_bus::EditorAction::OpenAssetBrowser
        )),
        "carregar na ranhura nao abriu a biblioteca: {sent:?}"
    );
    clear();
}
