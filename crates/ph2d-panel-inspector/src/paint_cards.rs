//! ⭐⭐ **OS CARTÕES DO TOPO** — o que se lê **antes** das seções.
//!
//! # ⚠️ Por que são um ficheiro irmão, e não um bloco no orquestrador
//!
//! O teto de 200 LOC por função (`architecture_panel_loc_cap`) — e a lei que ele carrega: *as
//! tolerâncias encolhem, nunca crescem, e uma feature nova paga-as com um CORTE*. O cartão de
//! propriedades levou o `paint_inspector` de 278 a 295, e os dois cartões saem juntos porque
//! partilham uma **lei**, não uma vizinhança: eles vêm antes de toda seção, não têm cabeçalho, não
//! recolhem e não ancoram nota. *Nada disto é orquestração de seção* — que é exactamente a razão
//! pela qual o cabeçalho saiu para o `paint_head` e a moldura para o `paint_body`.
//!
//! ⛔ **E não podiam ir para o `paint_frame.rs`**, que está a 562 de um teto de 600: *curar um teto
//! estourando o outro não é curar*.

use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::screens::hero::{InspectorInstanceInfo, InspectorPropertiesInfo};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Pinta os dois cartões, em ordem, e devolve o `y` de baixo.
///
/// ⚠️ **A ORDEM é a da leitura, e é uma decisão:** primeiro *de quem sou cópia* (o vínculo com a
/// biblioteca — *um artista que só descobre no fim do painel que está a editar uma cópia já
/// editou*), depois *o que eu sou* (as propriedades declaradas).
///
/// ⚠️ **Cada um existe SEM o outro:** um objecto solto declara propriedades e não é cópia de nada
/// (o report do Enio de 2026-08-31); uma cópia de um mestre sem chaves é o contrário.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_top_cards(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    instance_info: Option<&InspectorInstanceInfo>,
    properties_info: Option<&InspectorPropertiesInfo>,
    // ⭐ Qual eixo do cartão de propriedades está a ser reescrito, se algum — ver
    // `ids::INSP_INSTANCE_VALUE_EDIT`.
    editing_value: Option<usize>,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let mut y = y;
    if let Some(info) = instance_info {
        y = crate::sections::instance::paint_instance_card(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            info,
            inner_x,
            inner_w,
            y,
        );
    }
    if let Some(info) = properties_info {
        y = crate::sections::properties::paint_properties_card(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            info,
            editing_value,
            inner_x,
            inner_w,
            y,
        );
    }
    y
}
