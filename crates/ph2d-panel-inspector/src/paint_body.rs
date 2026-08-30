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
    pub scroll_y: f32,
    pub inner_x: f32,
    pub inner_w: f32,
    pub body_top_y: f32,
}

pub(crate) fn open_body(
    layout: &ph2d_editor_core::screens::hero::HeroLayout,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
    store: &ph2d_editor_core::interaction::WidgetStore,
) -> BodyFrame {
    use ph2d_editor_core::widget::panel_chrome::paint_panel_surface;
    use ph2d_tokens::Spacing;
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);
    // ⛔ **A ALÇA DE ARRASTO E AS DUAS DE RESIZE SAÍRAM** (2026-08-30): esta coluna é ANCORADA,
    // e o rect dela vem do `HeroLayout` sem passar por offset nenhum. Elas saíram **em par** com
    // o `InteractiveState::BlenderHit` do `pre_populate.rs` — uma alça registada no `HitIndex`
    // cujo arrasto não move nada é a forma exacta do controlo morto sob o dedo.

    // O cabeçalho — título, subtítulo, fechar e o divisor. Ver `paint_head`.
    let content_top =
        crate::paint_head::paint_panel_head(rect, scene, text_system, theme, hit_index, store);
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::INSP_PANEL).max(0.0);
    // ⏳⏳ **ABERTO, MEDIDO E NÃO CURADO: o `HitIndex` deste painel NÃO é recortado**
    // (auditoria de 2026-08-31, achado A4).
    //
    // O `push_clip` acima recorta o DESENHO. O gémeo no `HitIndex` — `hit_index.push_clip(banda)`
    // — não existe, então tudo o que sai do corpo continua **registado** onde ninguém o vê: com
    // `body_top_y = content_top − scroll_y`, rolar leva os hit-rects para a faixa do TÍTULO, e o
    // clique deles passa a valer ali. É a costura que o `ph2d-panel-motion-params` pagou
    // (CLAUDE.md §5.0: *«uma banda, dois consumidores»*).
    //
    // ⛔⛔ **Ela foi IMPLEMENTADA duas vezes e REVERTIDA as duas, com o preço contado:**
    //
    // | tentativa | resultado |
    // |---|---|
    // | recorte simétrico (topo + fundo) | **7** gates vermelhos (`seam_joint`, `seam_physics`) |
    // | só o topo (a cerca no lado perigoso) | **8** vermelhos, outro conjunto |
    //
    // ⚠️ **O que a medição revela é maior que o defeito:** aqueles gates pintam num viewport alto,
    // o painel recebe a altura dele, e as secções de baixo caem **fora** do corpo visível — onde
    // eles as clicam. *Eles provam cliques em widgets que o artista não vê, e são verdes hoje
    // porque o painel tem exactamente este defeito: o defeito e os gates seguram-se um ao outro.*
    //
    // ⇒ curá-lo é reescrever aqueles gates para **rolar antes de clicar**, e isso é uma wave — não
    // um remendo no fim de outra. ⛔ A segunda tentativa (só o topo) reprovou um conjunto
    // DIFERENTE da primeira, o que diz que o mecanismo ainda não está compreendido: shipar
    // qualquer das duas seria trocar um defeito que ninguém reportou por um que não sei nomear.
    let scrollbar_reserve = ph2d_editor_core::widget::SCROLLBAR_W + Spacing::Sm.px();
    ph2d_editor_core::widget::showcase::LAST_BODY_TOP_SCREEN_Y
        .with(|c| c.set(content_top + Spacing::Xs.px()));
    BodyFrame {
        rect,
        content_top,
        content_bottom,
        scroll_y,
        inner_x: rect.x + crate::paint::BODY_PAD,
        inner_w: (rect.w - crate::paint::BODY_PAD * 2.0 - scrollbar_reserve).max(0.0),
        body_top_y: content_top - scroll_y + Spacing::Xs.px(),
    }
}

/// ⭐ **O FECHO do corpo** — o simétrico do [`open_body`], e é o nome que diz que o `push_clip`
/// dele tem aqui a outra metade.
///
/// ⚠️ Saiu do `paint_inspector` com o `open_body`, e pela mesma razão: a seção COMPONENT (F5)
/// empurrou o orquestrador acima do tecto, e o que sai é o que nunca foi orquestração de seção.
pub(crate) fn close_body(
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
    rect: ph2d_editor_core::zones::Rect,
) {
    // **OS TRÊS POPOVERS DIFERIDOS**, pintados por último para ficarem acima de tudo.
    // ⚠️ Saíram do orquestrador em 2026-08-23: os três andam juntos porque partilham UMA lei — o
    // popover pinta-se fora da ordem das seções.
    crate::paint_frame_shared::paint_deferred_popovers(scene, text_system, theme, hit_index);
    scene.pop_layer();
    close_frame_hits(hit_index, rect);
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
            ids::INSP_ADD_COMPONENT,
            ph2d_editor_core::widget::panel_chrome::panel_header_add_button_rect(rect),
        );
    }
}
