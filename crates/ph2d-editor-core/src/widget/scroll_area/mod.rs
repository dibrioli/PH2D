//! ⭐⭐⭐ **A PORTA da rolagem** — todo corpo de painel que rola passa por aqui (spec
//! `docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md`).
//!
//! Até 2026-09-29 cada painel escrevia à mão a mesma sequência — recortar, pintar com `y − scroll`,
//! desfazer o recorte, pintar a barra, registar a barra, publicar as duas alturas, clampar — e o
//! censo desse dia achou os defeitos todos **na cópia**, cada painel com o seu:
//!
//! - 14 painéis registavam o **polegar** e o despachante lia-o como a **trilha** ⇒ arrastar a barra
//!   andava várias vezes mais depressa do que o dedo (com o polegar no mínimo, um pixel saltava ao
//!   fim);
//! - a Hierarquia nunca publicava a altura visível ⇒ a barra não arrastava e o corpo também não;
//! - o painel de ossos pintava com o id da barra do **Vector** ⇒ arrastá-la rolava outro painel;
//! - o Inspector tinha perdido o recorte do corpo e ficara com um `pop_layer` sem par;
//! - só cinco sítios recortavam também o `HitIndex`.
//!
//! ⇒ **um par ABRIR / FECHAR**, e o nome diz que há uma metade por fechar:
//!
//! ```text
//! let area = scroll_area::open(ctx, PANEL, BAR, body);
//! … o corpo pinta a partir de  area.top()  …
//! scroll_area::close(area, ctx, content_h);
//! ```
//!
//! ## O que a porta garante, por construção
//!
//! - **O desenho E o clique são recortados pela MESMA banda** — *uma banda, dois consumidores*. Um
//!   widget rolado para fora não se vê **e** não se clica.
//! - **As duas alturas são sempre publicadas** — é delas que a roda, o arrasto no corpo e a
//!   inércia tiram o fim da lista.
//! - **Regista-se a TRILHA, nunca o polegar**, e o dono dela é **publicado** no store
//!   ([`crate::interaction::WidgetStore::publish_scroll_bar`]) — o despachante lê daí o painel e a
//!   geometria, em vez de uma tabela escrita à mão.
//! - **O alvo é clampado** quando o conteúdo encolhe (uma secção que fecha, um nó apagado), senão
//!   um painel rolado até ao fim abriria em branco.
//!
//! ⚠️ A barra pinta-se **depois** de desfeito o recorte: ela vive no corpo mas não rola com ele.

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::rect_to_vello;
use crate::panel::PaintCtx;
use crate::widget::scrollbar::{SCROLLBAR_THUMB_MIN_H, is_needed, paint_scrollbar, track_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Um corpo rolável ABERTO. Só o [`close`] (ou o [`close_with`]) o consome.
#[must_use = "um corpo aberto tem de ser fechado — senão o recorte fica aberto"]
#[derive(Debug, Clone, Copy)]
pub struct ScrollArea {
    panel: NodeId,
    bar: NodeId,
    body: Rect,
    scroll: f32,
}

impl ScrollArea {
    /// O deslocamento de agora (o VIVO — o que se desenha).
    #[must_use]
    pub fn scroll(&self) -> f32 {
        self.scroll
    }
    /// O `y` onde o conteúdo começa a ser pintado: o topo do corpo menos o deslocamento.
    #[must_use]
    pub fn top(&self) -> f32 {
        self.body.y - self.scroll
    }
    /// A banda visível.
    #[must_use]
    pub fn body(&self) -> Rect {
        self.body
    }
    /// O painel que rola.
    #[must_use]
    pub fn panel(&self) -> NodeId {
        self.panel
    }
}

/// Abre o corpo com as partes soltas — para quem não tem um [`PaintCtx`] à mão.
pub fn open_with(
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    panel: NodeId,
    bar: NodeId,
    body: Rect,
) -> ScrollArea {
    let scroll = store.panel_scroll(panel).max(0.0);
    scene.push_clip(&rect_to_vello(body));
    hit_index.push_clip(body);
    ScrollArea {
        panel,
        bar,
        body,
        scroll,
    }
}

/// Fecha o corpo com as partes soltas. Ver o cabeçalho do módulo para o que isto garante.
pub fn close_with(
    area: ScrollArea,
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &mut WidgetStore,
    content_h: f32,
    theme: Theme,
) {
    close_parts(area, scene, hit_index, store, content_h, theme).publish(store);
}

/// ⭐ **Fechar em DOIS tempos** — para o pintor que só tem o store IMUTÁVEL quando fecha o corpo
/// (o `store_and_hit_index_mut` do host dá `&WidgetStore`).
///
/// Desfaz os recortes, pinta a barra e regista a TRILHA **agora** — tem de ser antes de tudo o que
/// se pinta por cima do corpo (um popover diferido não pode ficar por baixo da barra, nem ser
/// recortado pela banda) — e devolve o que falta: as alturas, o clamp e o dono, que precisam do
/// store mutável. ⚠️ O [`Pending`] é `#[must_use]`: esquecê-lo é a Hierarquia de antes (a barra
/// pinta e nunca arma).
pub fn close_parts(
    area: ScrollArea,
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    content_h: f32,
    theme: Theme,
) -> Pending {
    scene.pop_layer();
    hit_index.pop_clip();
    let visible_h = area.body.h;
    let track = is_needed(content_h, visible_h).then(|| {
        let track = track_rect(area.body);
        let _ = paint_scrollbar(
            area.body,
            area.scroll,
            content_h,
            visible_h,
            store.scrollbar_visual(area.bar),
            scene,
            theme,
        );
        hit_index.register(area.bar, track);
        track
    });
    Pending {
        panel: area.panel,
        bar: area.bar,
        track,
        content_h,
        visible_h,
    }
}

/// A metade do fecho que precisa do store mutável — ver [`close_parts`].
#[must_use = "sem publicar, a barra pinta e nunca arma, e a roda não sabe onde a lista acaba"]
#[derive(Debug, Clone, Copy)]
pub struct Pending {
    panel: NodeId,
    bar: NodeId,
    track: Option<Rect>,
    content_h: f32,
    visible_h: f32,
}

impl Pending {
    /// Publica as duas alturas, clampa o alvo e o dono da barra.
    pub fn publish(self, store: &mut WidgetStore) {
        publish(store, self.panel, self.content_h, self.visible_h);
        if let Some(track) = self.track {
            store.publish_scroll_bar(self.bar, self.panel, track);
        }
    }
}

/// Abre o corpo de um painel — a forma de todo `Panel::paint`.
pub fn open(ctx: &mut PaintCtx, panel: NodeId, bar: NodeId, body: Rect) -> ScrollArea {
    let scroll = ctx.host.store().panel_scroll(panel).max(0.0);
    ctx.scene.push_clip(&rect_to_vello(body));
    ctx.host.hit_index_mut().push_clip(body);
    ScrollArea {
        panel,
        bar,
        body,
        scroll,
    }
}

/// Fecha o corpo de um painel.
pub fn close(area: ScrollArea, ctx: &mut PaintCtx, content_h: f32) {
    let theme = ctx.host.theme();
    ctx.scene.pop_layer();
    ctx.host.hit_index_mut().pop_clip();
    let visible_h = area.body.h;
    publish(ctx.host.store_mut(), area.panel, content_h, visible_h);
    if is_needed(content_h, visible_h) {
        let track = track_rect(area.body);
        let visual = ctx.host.store().scrollbar_visual(area.bar);
        let _ = paint_scrollbar(
            area.body,
            area.scroll,
            content_h,
            visible_h,
            visual,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(area.bar, track);
        ctx.host
            .store_mut()
            .publish_scroll_bar(area.bar, area.panel, track);
    }
}

/// As duas alturas e o clamp do alvo — a metade que NÃO depende de a barra ser precisa.
fn publish(store: &mut WidgetStore, panel: NodeId, content_h: f32, visible_h: f32) {
    store.set_panel_content_h(panel, content_h);
    store.set_panel_visible_h(panel, visible_h);
    let max = (content_h - visible_h).max(0.0);
    if store.panel_scroll_target(panel) > max {
        store.set_panel_scroll(panel, max);
    }
}

/// ⭐ **Carregar na TRILHA fora do polegar faz o polegar SALTAR para debaixo do dedo** — o
/// deslocamento em que o CENTRO do polegar fica no `cursor_y`.
///
/// É o comportamento de omissão do GTK e do Blender, e num tablet o único que faz sentido: «uma
/// página por clique» pede cliques repetidos onde o dedo quer um gesto só. O arrasto continua a
/// partir daqui, com a lei proporcional de sempre.
#[must_use]
pub fn scroll_for_track_press(track: Rect, cursor_y: f32, content_h: f32, visible_h: f32) -> f32 {
    let max = (content_h - visible_h).max(0.0);
    if max <= 0.0 || track.h <= 0.0 {
        return 0.0;
    }
    let thumb_h = (visible_h / content_h * track.h)
        .max(SCROLLBAR_THUMB_MIN_H)
        .min(track.h);
    let range = (track.h - thumb_h).max(1.0);
    let t = ((cursor_y - track.y - thumb_h * 0.5) / range).clamp(0.0, 1.0);
    t * max
}

#[cfg(test)]
mod tests;
