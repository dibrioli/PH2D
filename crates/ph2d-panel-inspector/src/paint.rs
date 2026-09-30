//! Inspector panel paint — ADR-0029 Phase C.1 port of
//! `ph2d_editor_core::screens::hero::inspector::{paint_thunk,
//! paint_inspector}`.
//!
//! `paint` is the typed entry point invoked by the `Panel` trait. It
//! gates on visibility (via [`PanelHostInternal::panel_visible`]),
//! runs the snapshot sync, paints the live Inspector body, and
//! publishes scroll bounds back to the store.

use crate::paint_frame::{PanelFinish, publish_and_finish};
use crate::state::{self, current_inspector_visibility_section};
use crate::state_popovers;
use crate::sync::sync_inspector_from_snapshots;
use crate::{InspectorPanel, sections};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::{HeroLayout, HeroSelection};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

pub(crate) const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: inspector body inset
const SECTION_HEAD_H: f32 = ROW_H_PX;

pub(crate) fn paint(inspector_state: &mut state::InspectorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(InspectorPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::INSP_PANEL);
        return;
    }
    // ⭐⭐⭐ **O RECT PUBLICADO É O DO ENCAIXE** — ver o irmão em `ph2d-panel-hierarchy/src/paint.rs`,
    // que traz o mecanismo inteiro. Este painel e a Hierarquia eram os dois únicos cujo rect era
    // publicado de FORA (`hero::paint`, com `layout.inspector`); os outros 20 publicam `ctx.slot`.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::INSP_PANEL, ctx.slot);
    sync_inspector_from_snapshots(inspector_state, ctx.host);
    let display_unit = ctx.host.project().display_unit;
    let ppm = ctx.host.project().pixels_per_meter;
    let display_angle = ctx.host.project().display_angle;
    state::set_current_display_unit(display_unit, ppm);
    state::set_current_display_angle(display_angle);
    let theme = ctx.host.theme();
    // Splitting the borrows: paint_inspector wants &mut hit_index,
    // &WidgetStore, &mut Scene, &mut TextSystem — all from disjoint
    // refs on the host. Reborrow store via `store()` then `store_mut()`
    // sequentially below for the post-paint scroll publish.
    let pending;
    {
        // Build a transient owning copy of selection to avoid holding an
        // immutable borrow of host while also borrowing host's
        // hit_index mutably; selection is small + Clone. The combined
        // `store_and_hit_index_mut` accessor avoids the dyn-trait
        // aliasing dance for the &WidgetStore + &mut HitIndex pair.
        let selection_clone: Option<HeroSelection> = ctx.host.selection().cloned();
        // ⚠️ **A BANDA DOS POPOVERS, e ela NÃO é o `slot`** — resolvido na integração de 2026-09-10.
        // A `line/UIUX` trocou o 1.º parâmetro desta função de `&HeroLayout` para `slot: Rect`
        // (*«para saber onde ele PRÓPRIO fica, um painel lê o `slot`»*) e esta linha, no mesmo dia,
        // acrescentou um quarto popover diferido no fim dela — que pergunta outra coisa: *até onde
        // um popover pode ESCORREGAR*, que é a banda de chrome e transborda o painel de propósito.
        // As duas edições fundem sem marcador e o `layout` some do escopo.
        // ⛔ Ler o `slot` aqui compila e prende o popover dentro da coluna.
        let popover_layout = ctx.layout;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        pending = paint_inspector(
            ctx.slot,
            popover_layout,
            selection_clone.as_ref(),
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            &mut inspector_state.anchor_selected,
            &mut inspector_state.anim_selected,
            &mut inspector_state.timer_selected,
            &mut inspector_state.action_selected,
            &mut inspector_state.watch_selected,
            &mut inspector_state.trigger_selected,
            &mut inspector_state.emitter_selected,
            &mut inspector_state.tween_selected,
            &mut inspector_state.sm_state_selected,
            &mut inspector_state.sm_trans_selected,
        );
    }
    state::set_current_display_unit(display_unit, ppm); // keep symmetric with legacy
    state::set_current_display_angle(display_angle);
    let store = ctx.host.store_mut();
    // ⭐ **O popover diferido publica o RECT dele aqui, e é o que o faz FECHAR ao clique fora.**
    //
    // ⚠️ A lei já existia no `dispatch::pointer_down` (*«se o usuário clicar fora do dropdown ele
    // deve se fechar»*, Enio 2026-06-24) e lê `store.dropdown_popover()` — que **nenhum dos
    // seletores deste painel publicava**, porque o passe diferido não tem `&mut WidgetStore`. Os
    // outros nove painéis do app publicam-no onde pintam; aqui o passe deposita e este sítio, que
    // é o primeiro com o store mutável depois da pintura, publica.
    if let Some((id, panel, content_h, visible_h)) = state_popovers::take_painted_popover() {
        store.set_dropdown_popover(id, panel);
        // ⚠️ **As duas alturas viajam com o rect**: são elas que dizem à roda e à barra até onde
        // rolar. Um popover clampado mais curto que a lista, publicado sem elas, fica cortado e
        // IMÓVEL.
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        let max_scroll = (content_h - visible_h).max(0.0);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    // As alturas do corpo, o clamp e o dono da barra — a metade da porta que precisa do store
    // mutável (`scroll_area::Pending`).
    pending.publish(store);
}

#[allow(clippy::too_many_arguments)]
/// # Os TRÊS que são estado do PAINEL, e não da cena
///
/// ⚠️ A prosa deles mudou-se do meio da lista de parâmetros para aqui em 2026-08-31, quando o
/// terceiro empurrou a função sobre o teto de 200 LOC: comentários dentro do corpo contam, e o
/// idioma do Rust para documentar um parâmetro é o doc-comment da função. *A tolerância desceu
/// junto — ela só anda nessa direcção.*
///
/// - `anchor_selected` — §12: qual linha da lista de âncoras está aberta. Saturado dentro de
///   `paint_anchor_section` contra o tamanho da lista: apagar a última âncora não o pode deixar a
///   apontar para o vazio.
/// - `anim_selected` — §11: qual animação está aberta no editor. Mesmo contrato.
/// - `timer_selected` — TIMERS: qual timer está aberto no editor. Mesmo contrato.
/// - `action_selected` — SIGNAL ACTIONS: qual acção está aberta. Mesmo contrato.
/// - `sm_state_selected` / `sm_trans_selected` — STATE MACHINE: **duas** listas independentes, e
///   partilhá-las faria abrir um estado fechar a seta que o artista estava a editar.
/// - `editing_value` — qual eixo do cartão de propriedades está a ser **reescrito**; ver
///
/// # A moldura e o fecho
///
/// A abertura ([`crate::paint_body::open_body`]) traz superfície, alças, cabeçalho, clip e a
/// caixa interior; o fecho ([`crate::paint_body::close_body`]) traz os popovers diferidos, o
/// `pop_layer` daquele clip, os cantos, o re-registo dos hits e os cartões. ⚠️ **Nada disso é
/// orquestração de seção** — é a mesma razão pela qual o cabeçalho saiu para o `paint_head` em
/// 2026-08-23, e a razão pela qual esta prosa vive AQUI: *comentário dentro do corpo conta para
/// o teto de 200 LOC; doc-comment não.*
fn paint_inspector(
    slot: Rect,
    // ⚠️ Ao LADO do `slot`, nunca no lugar dele: o `slot` diz onde este painel FICA, o `layout` diz
    // até onde um popover diferido pode ESCORREGAR (`HeroLayout::popover_region`, que é o que os
    // outros dez sítios da casa lhe perguntam).
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    anchor_selected: &mut usize,
    anim_selected: &mut usize,
    timer_selected: &mut usize,
    action_selected: &mut usize,
    watch_selected: &mut usize,
    trigger_selected: &mut usize,
    emitter_selected: &mut usize,
    tween_selected: &mut usize,
    sm_state_selected: &mut usize,
    sm_trans_selected: &mut usize,
) -> ph2d_editor_core::widget::scroll_area::Pending {
    let crate::paint_body::BodyFrame {
        rect,
        content_top,
        content_bottom,
        inner_x,
        inner_w,
        body_top_y,
        area,
    } = crate::paint_body::open_body(slot, scene, text_system, theme, hit_index, store);
    // Os snapshots e o `any_section`, numa pergunta só. Ver `paint_frame::LiveSnapshots`.
    //
    // ⚠️ **Ele NÃO é destruturado, e a diferença é o teto de LOC**: a lista de nomes era vinte e
    // duas linhas de puro re-vínculo, e cada secção nova acrescentava mais uma — este orquestrador
    // rebentou o cap de `200` no dia em que a CAMERA chegou, e a lista era metade do que ele fazia.
    // *Um `let` que só renomeia um campo não é código: é uma segunda lista da primeira.*
    let snaps = crate::paint_frame::LiveSnapshots::fetch();
    // ⭐⭐ Os dois CARTÕES do topo — o porquê vive no cabeçalho de [`crate::paint_cards`].
    // ⚠️ O livro dos cartões começa ONDE O CONTEÚDO começa — sem isto os `Xs` de folga do topo
    //    fechavam como um cartão vazio no quadro em que não há cartão de instância (2026-09-29).
    ph2d_editor_core::widget::section_cards::skip_section_header(body_top_y + Spacing::Xs.px());
    let mut y = crate::paint_cards::paint_top_cards(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        snaps.instance_info.as_ref(),
        snaps.properties_info.as_ref(),
        inner_x,
        inner_w,
        body_top_y + Spacing::Xs.px(),
    );
    // ⛔ **O macro `live_section!` MORREU em 2026-09-09, e a morte dele é o ganho.**
    //
    // Ele existia para as três seções que TODO objecto tem — as únicas que ainda se pintavam aqui
    // dentro — e era ele que as prendia a este corpo: um macro captura os locais em volta, então
    // extraí-las obrigava a transformar essa captura em argumentos. Foi o que a
    // [`crate::paint_frame_shared::paint_core_sections`] fez, e o macro ficou sem um único
    // consumidor. *Uma abstracção que sobrevive ao último chamador é uma cerca sobre um campo
    // vazio.*
    // ⭐⭐ **As TRÊS que TODO objecto tem** — §1 Name, §8 Visibility, §2 Transform. Ver o cabeçalho
    // de [`crate::paint_frame_shared::paint_core_sections`]: elas saíram daqui quando a secção
    // SIGNAL ACTIONS empurrou este orquestrador contra a catraca dele.
    // ⭐⭐⭐ **O PLANO** (2026-09-29): os grupos EMPURRAM as secções deles e o plano pinta-as pela
    //    ordem que o artista escolheu com a pega — ver [`crate::plano`]. A ordem em que os grupos
    //    empurram é a NATURAL (a da paleta), e é ela que uma secção nunca movida segue.
    let mut plano = crate::plano::Plano::new();
    crate::paint_frame_shared::push_core_sections(
        &mut plano,
        store,
        inner_x,
        inner_w,
        ROW_H_PX,
        SECTION_HEAD_H,
        snaps.name_present,
        snaps.visibility_info.is_some(),
        snaps.transform_info.is_some(),
    );
    // **As três da SPRITE** — só existem se houver sprite.
    crate::paint_frame_shared::push_sprite_sections(
        &mut plano,
        store,
        inner_x,
        inner_w,
        SECTION_HEAD_H,
        snaps.sprite_info.as_ref(),
    );
    // **As quatro COMPARTILHADAS** — qualquer entidade com `Transform`.
    crate::paint_frame_shared::push_shared_sections(
        &mut plano,
        store,
        inner_x,
        inner_w,
        SECTION_HEAD_H,
        snaps.slice_info.as_ref(),
        snaps.ordering_info.as_ref(),
        snaps.sampling_info.as_ref(),
        snaps.blend_info.as_ref(),
    );
    crate::paint_frame::push_physics_sections(
        &mut plano,
        store,
        inner_x,
        inner_w,
        SECTION_HEAD_H,
        snaps.physics_info.as_ref(),
        snaps.joint_info.as_ref(),
        snaps.wheel_info.as_ref(),
        snaps.player_info.as_ref(),
    );
    // **As OPCIONAIS** — pela ordem da paleta (ver `paint_familias`).
    crate::paint_optional::push_optional_sections(
        &mut plano,
        store,
        inner_x,
        inner_w,
        SECTION_HEAD_H,
        anim_selected,
        anchor_selected,
        timer_selected,
        action_selected,
        watch_selected,
        trigger_selected,
        emitter_selected,
        tween_selected,
        sm_state_selected,
        sm_trans_selected,
        &snaps,
    );
    let mut tela = crate::plano::Tela {
        scene: &mut *scene,
        text: &mut *text_system,
        hit: &mut *hit_index,
    };
    let (fim, fantasma) = plano.run(&mut tela, store, theme, inner_x, inner_w, SECTION_HEAD_H, y);
    y = fim;
    if snaps.any_section {
        crate::paint_frame::paint_trailing_notes(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut y,
        );
    }
    // ⭐ Os FANTASMAS — a secção ou a nota que a pega arrasta, por cima de tudo o que o corpo
    //    pintou (2026-09-30). Nunca os dois: só há um arrasto de cada vez.
    if let Some(f) = fantasma {
        f.pinta(scene, store);
    }
    ph2d_editor_core::widget::showcase::paint_note_drag_ghost(
        scene,
        text_system,
        hit_index,
        store,
        ids::INSP_PANEL,
        theme,
    );
    publish_and_finish(
        scene,
        text_system,
        theme,
        PanelFinish {
            any_section: snaps.any_section,
            has_selection: selection.is_some(),
            inner_x,
            inner_w,
            content_top,
            content_bottom,
            body_top_y,
            y,
        },
    );
    crate::paint_body::close_body(
        scene,
        text_system,
        theme,
        hit_index,
        rect,
        store,
        layout,
        area,
        crate::state::last_inspector_content_h(),
    )
}

/// **O corpo da §8 Visibility** — a caixa `Visible` mais os controlos do componente opcional.
///
/// ⚠️ **Função IRMÃ, e não um ficheiro novo:** o `paint.rs` está com folga larga no cap de
/// FICHEIRO, e o que estourou foi o cap de FUNÇÃO do orquestrador — os dois medem grandezas
/// diferentes, e extrair para aqui cura o que estourou sem tocar no outro.
///
/// ⚠️ **As duas metades andam juntas por uma LEI:** os controlos de camada/clip/máscara/on-screen
/// só fazem sentido *debaixo* da caixa que diz se o objecto se vê, e é essa adjacência que a §8
/// promete. Separá-las poria a pergunta e a qualificação dela em sítios diferentes do painel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn visibility_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let mut yy = sections::paint_visibility_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
    );
    // W3 §8: the optional-component controls (layer mask / clip / mask / on-screen) sit directly
    // below the Visible toggle.
    if let Some(vis) = current_inspector_visibility_section() {
        yy = sections::paint_visibility_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            yy,
            &vis,
        );
    }
    yy
}
