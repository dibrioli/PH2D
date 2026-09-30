//! ⭐ **A MOLDURA do corpo do Inspector** — o que existe antes da primeira seção e depois da
//! última: superfície, alças, cabeçalho, clip, e o fecho simétrico.
//!
//! ⚠️ **Ficheiro irmão, e não uma função no `paint_frame`:** aquele já estava a 600 LOC e receber
//! isto levava-o a 636. *Curar um tecto estourando o outro não é curar* — é o mesmo precedente do
//! par `event_precision.rs` de 2026-08-20.
//!
//! ⚠️ **Nada aqui é orquestração de seção**, e é essa a razão de ter saído do `paint_inspector`
//! quando a seção COMPONENT (ADR-0164 / F5) o empurrou acima do tecto: a catraca só desce, e o
//! cluster que sai é o que nunca pertenceu ao orquestrador.

use ph2d_editor_core::ids;

/// ⭐ **A moldura do corpo do Inspector** — o que existe antes de a primeira seção ser pintada.
///
/// ⚠️ **Ela saiu do `paint_inspector` em 2026-08-27 e a razão é a mesma do `paint_head`:** *nada
/// disto é orquestração de seção*. A catraca do `architecture_panel_loc_cap` só desce, e quando a
/// seção COMPONENT (F5) empurrou o orquestrador acima do tecto, o cluster que saiu foi o que nunca
/// pertenceu a ele. ⛔ Levar só a seção nova devolveria o número ao mesmo sítio, *e ficar no mesmo
/// sítio não é encolher*.
///
/// ⚠️ **Ela abre um `push_clip` que o chamador FECHA** (`scene.pop_layer()`), e é por isso que se
/// chama `open_body` e não `layout_body`: o nome diz que há uma metade por fechar.
pub(crate) struct BodyFrame {
    pub rect: ph2d_editor_core::zones::Rect,
    pub content_top: f32,
    pub content_bottom: f32,
    pub inner_x: f32,
    pub inner_w: f32,
    pub body_top_y: f32,
    /// O corpo aberto pela porta — só o [`close_body`] o fecha.
    pub area: ph2d_editor_core::widget::scroll_area::ScrollArea,
}

pub(crate) fn open_body(
    rect: ph2d_editor_core::zones::Rect,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
    store: &ph2d_editor_core::interaction::WidgetStore,
) -> BodyFrame {
    use ph2d_editor_core::widget::panel_chrome::paint_panel_surface;
    use ph2d_editor_core::widget::section_cards::begin_section_cards;
    use ph2d_tokens::Spacing;
    paint_panel_surface(rect, scene, theme);
    // ⛔ **A ALÇA DE ARRASTO E AS DUAS DE RESIZE SAÍRAM** (2026-08-30): esta coluna é ANCORADA,
    // e o rect dela vem do `HeroLayout` sem passar por offset nenhum. Elas saíram **em par** com
    // o `InteractiveState::BlenderHit` do `pre_populate.rs` — uma alça registada no `HitIndex`
    // cujo arrasto não move nada é a forma exacta do controlo morto sob o dedo.

    // O cabeçalho — título, subtítulo, fechar e o divisor. Ver `paint_head`.
    let content_top =
        crate::paint_head::paint_panel_head(rect, scene, text_system, theme, hit_index, store);
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    // ⭐⭐ **A PORTA da rolagem** (spec `04_a_rolagem_unica` D4/D5, 2026-09-29): o corpo volta a
    // ser RECORTADO — o `push_clip` saiu por acidente em `41e6597bf` e o `close_body` ficou com um
    // `pop_layer` sem par — e o `HitIndex` passa a ser recortado pela MESMA banda (o achado A4 da
    // auditoria de 31/08: rolar levava os hit-rects para a faixa do título).
    let area = ph2d_editor_core::widget::scroll_area::open_with(
        scene,
        hit_index,
        store,
        ids::INSP_PANEL,
        ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID,
        ph2d_editor_core::zones::Rect::new(
            rect.x,
            content_top,
            rect.w,
            (content_bottom - content_top).max(0.0),
        ),
    );
    let scroll_y = area.scroll();
    let scrollbar_reserve = ph2d_editor_core::widget::SCROLLBAR_W + Spacing::Sm.px();
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    // ⭐⭐ **O corpo pinta-se DENTRO de cartões** — o fecho é o `close_body`, simétrico deste.
    begin_section_cards(scene, theme, body_top_y);
    BodyFrame {
        rect,
        content_top,
        content_bottom,
        inner_x: rect.x + crate::paint::BODY_PAD,
        inner_w: (rect.w - crate::paint::BODY_PAD * 2.0 - scrollbar_reserve).max(0.0),
        body_top_y,
        area,
    }
}

/// ⭐ **O FECHO do corpo** — os popovers diferidos, o `pop_layer` do clip que o [`open_body`]
/// abriu, os cantos, o re-registo dos hits e os CARTÕES. É o simétrico dele, e o nome diz que o
/// `push_clip`
/// dele tem aqui a outra metade.
///
/// ⚠️ Saiu do `paint_inspector` com o `open_body`, e pela mesma razão: a seção COMPONENT (F5)
/// empurrou o orquestrador acima do tecto, e o que sai é o que nunca foi orquestração de seção.
#[allow(clippy::too_many_arguments)]
pub(crate) fn close_body(
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
    rect: ph2d_editor_core::zones::Rect,
    store: &ph2d_editor_core::interaction::WidgetStore,
    layout: &ph2d_editor_core::screens::HeroLayout,
    area: ph2d_editor_core::widget::scroll_area::ScrollArea,
    content_h: f32,
) -> ph2d_editor_core::widget::scroll_area::Pending {
    // ⭐ Os cartões vão para BAIXO do corpo, antes de tudo o que se pinta por cima.
    ph2d_editor_core::widget::section_cards::end_section_cards(scene);
    // ⭐ A porta FECHA aqui: depois dos cartões (que rolam com o corpo) e ANTES dos popovers, que
    //    escorregam para fora do painel de propósito — nem o desenho nem o clique deles podem ser
    //    recortados pela banda do corpo, e a barra fica por baixo deles.
    let pending = ph2d_editor_core::widget::scroll_area::close_parts(
        area, scene, hit_index, store, content_h, theme,
    );
    // **OS QUATRO POPOVERS DIFERIDOS**, pintados por último para ficarem acima de tudo.
    // ⚠️ Saíram do orquestrador em 2026-08-23: os quatro andam juntos porque partilham UMA lei — o
    // popover pinta-se fora da ordem das seções.
    crate::popovers::paint_deferred_popovers(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        layout.popover_region(),
    );
    close_frame_hits(hit_index, rect);
    pending
}

/// ⭐ **O que se re-regista no FIM do quadro, e porquê** — as alças, o X e o `+`.
///
/// ⚠️ **Estes hits têm de vir DEPOIS de tudo**, e a razão é uma só: o `HitIndex` resolve
/// back-to-front, então quem regista por último ganha. A alça de arrasto cobre a banda do título e
/// um widget do corpo que rolou para debaixo dela também — sem este bloco, o que o dedo alcança no
/// cabeçalho é o que rolou para lá, e não o botão que o olho vê.
///
/// A nota original é de **2026-05-24** (o padrão foi reportado na Widget Gallery) e cobria só o X.
/// ⚠️ **O `+` da F3 (ADR-0166) nasceu sem ele e ficou MORTO SOB O DEDO** — pintado, a acender no
/// hover, e clicável a nada. Foi o 1.º smoke do Enio que o apanhou; o gate que o defende agora
/// pergunta *quem ganha o clique*, e não *o id foi registado* (que era `true` o tempo todo).
///
/// ⚠️ **Saiu do [`paint_inspector`] pela catraca**, que levou o orquestrador a 304 contra uma
/// tolerância de 292 que **só desce**: o cluster inteiro sai, e não só a linha nova — *ficar no
/// mesmo sítio não é encolher*.
fn close_frame_hits(
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
    rect: ph2d_editor_core::zones::Rect,
) {
    // ⛔ **As três alças saíram (2026-08-30)** — esta coluna é ANCORADA. Elas eram
    // re-registadas aqui, no fim do quadro, para ganharem o z-order ao corpo; sem braço que as
    // consuma, re-registá-las seria pintar chrome morto sob o dedo.
    hit_index.register(
        ids::INSP_CLOSE,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
    // ⚠️ **Só quando ele é PINTADO** — sem objeto sob o Inspector o botão não existe, e um hit
    // registado sobre um botão que ninguém desenhou é a metade oposta do mesmo defeito.
    if crate::state::current_inspector_transform().is_some() {
        hit_index.register(
            crate::ids::INSP_ADD_COMPONENT,
            ph2d_editor_core::widget::panel_chrome::panel_header_add_button_rect(rect),
        );
    }
}
