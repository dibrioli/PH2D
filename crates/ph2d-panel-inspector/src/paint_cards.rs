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
/// ⭐⭐⭐ **A FRASE DA SELECÇÃO — o TERCEIRO cartão, e o primeiro a ser lido.**
///
/// ⛔⛔⛔ Ela vivia em **vinte e uma** secções, escrita por **quatro** pintores diferentes, com
/// quatro redacções. Medido pela porta do produto em 2026-09-22: **cinco** cópias idênticas no
/// mesmo quadro do Inspector, mais uma sexta a dizer o mesmo por outras palavras — que é o report
/// do dono (*«vários componentes cheios de mensagens»*) à letra.
///
/// ⚠️ **Ela é um facto da SELECÇÃO e não do componente**, logo é do painel: o mesmo número, no
/// mesmo instante, para todas as secções. *Um facto do painel escrito por secção multiplica-se
/// pelo número de componentes que o objecto tem.*
///
/// ⚠️ **Ela vem ANTES dos outros dois cartões**, e é uma decisão de ordem: *estás a editar uma de
/// N* muda o significado de tudo o que vem abaixo, incluindo o cartão da cópia.
fn paint_selection_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let n = crate::state::current_inspector_selecionados();
    if n <= 1 {
        return y;
    }
    crate::sections::rows::aviso(
        scene,
        text_system,
        theme,
        inner_x,
        inner_w,
        y,
        &ph2d_i18n::tr_with("panel.inspector.selection.primary_only", &[("n", &n)]),
        ph2d_tokens::ColorToken::Warn,
    )
}

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
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let mut y = paint_selection_card(scene, text_system, theme, inner_x, inner_w, y);
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
            inner_x,
            inner_w,
            y,
        );
    }
    y
}
