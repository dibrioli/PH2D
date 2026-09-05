//! TopBar painter — 5 pill clusters + centered wordmark.

use super::HeroLayout;
use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{ButtonState, IconGlyph, Tooltip, paint_tooltip};
// ⚠️ Do DONO, e não pela boleia de um widget: o `PILL_PADDING_PX` é um token, e viajava por um
// `pub use` do `pill_group` — um widget que este cromo NUNCA chamou (censo de 2026-09-03).
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::PILL_PADDING_PX;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Register every TopBar widget into the [`WidgetStore`]. Called
/// once by `HeroScreen::pre_populate_store`.
pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::TOPBAR_THEME,
        ids::TOPBAR_SAVE,
        ids::TOPBAR_SAVE_AS,
        ids::TOPBAR_OPEN,
        ids::TOPBAR_IMAGE_TOOLS,
        ids::TOPBAR_AUDIO_MIXER,
        ids::TOPBAR_AUDIO_EDITOR,
        // Vector pill MUST be registered here (not only painted/hit-indexed in
        // cluster_painter.rs): a pill absent here has no `InteractiveState`, so
        // pointer-Up never emits `Click` and the tool is dead on click.
        // (Audit 2026-06-02 killer.)
        ids::TOPBAR_VECTOR,
        // Motion Nodes pill — same parity requirement as the Vector pill above
        // (painted + hit-indexed in the fixture → MUST be registered here or the
        // pill is dead on click). Motion Nodes M0.T9.
        ids::TOPBAR_MOTION,
        // Flip pill — same parity requirement (registered here or dead on click).
        // ADR-0114 W2.
        ids::TOPBAR_FLIP,
        ids::TOPBAR_PHYSICS,
        ids::TOPBAR_TOKENS,
        ids::TOPBAR_AUTHORED,
        // Sculpt 3D (ADR-0150) — mesma exigência de paridade dos pills acima: sem registro AQUI
        // ele desenha e nasce morto sob o mouse.
        ids::TOPBAR_SCULPT3D,
        // Modelagem 3D (ADR-0161) — mesma exigência: sem registro AQUI o pill desenha e
        // nasce morto sob o mouse.
        ids::TOPBAR_MODEL3D,
        ids::TOPBAR_WIDGET_GALLERY,
        // Widget Lab — a bancada de desenho. ⚠️ **Registada aqui e SEM pill na fixture**, ao
        // contrário dos vizinhos: os pills são o chrome legado (`F9`) e a porta de produto é a
        // linha do menu *Window*. O que este registo compra é o clique — sem ele o id não fica
        // `active` no Down, o `Click` nunca nasce, e a linha do menu abre um painel que nunca vê.
        ids::TOPBAR_WIDGET_LAB,
        ids::TOPBAR_GRID_SETTINGS,
        ids::TOPBAR_SETTINGS,
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_PAUSE,
        ids::TOPBAR_RESET,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::TOPBAR_RIGHT_ASSETS,
        ids::TOPBAR_RIGHT_SCRIPT,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // **A fila de Image Tools regista-se pela MESMA porta por que é pintada.**
    //
    // ⚠️ Aqui esteve uma lista de dez `ids::IMAGE_ACTION_*` escritos à mão, e foi ela que matou o
    // pill `[SHEET]` (Enio, 2026-08-19: *«botão sheet não funciona»*). O painter deriva a fila do
    // registry — um tool novo aparece na barra sozinho, que é a promessa do drop-crate (ADR-0075)
    // — mas o REGISTO não crescia com ele: sem `InteractiveState` o pill tem
    // `is_focusable() == false`, o Down nunca arma o `active`, o Up nunca emite `Click`. *Ele
    // desenha e nasce morto debaixo do rato*, exatamente como os quatro pills de vetor de
    // `0661862`, e o gate que guarda aquele caso varre `ids::TOPBAR_*` — que uma fila derivada do
    // registry, por construção, não tem.
    //
    // `image_action_pills()` é a mesma função que o painter chama (e traz o seu próprio fallback
    // pré-registry), então painter e registo **não podem** divergir sem uma edição que os separe.
    // Gate do comportamento: `ph2d-tool-registry-init/tests/every_image_tool_pill_dispatches.rs`.
    for pill in image_action_pills() {
        store.register(
            pill.id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Group backdrops — Plain so clicks on empty backdrop space
    // emit `Click(<backdrop_id>)` that `apply_event` prints.
    for id in [
        ids::TOPBAR_LEFT_BACKDROP,
        ids::TOPBAR_RIGHT_BACKDROP,
        ids::TOPBAR_IMAGE_TOOLS_BACKDROP,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // Search input for the project-chip Scene List popover. Lives
    // here so opening the menu doesn't have to allocate state on
    // demand — its buffer is just filtered against the scene list
    // at paint time.
    store.register(
        ids::CTX_SCENE_SEARCH,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    tooltips::seed_tooltips(store);
}

/// Apply a [`WidgetEvent`] against TopBar widgets. Returns false so
/// the chrome dispatch chain can still react (Save → menu, Play →
/// run, etc.); this handler only side-effect-prints the clicked
/// chip's name to stdout (Enio 2026-05-25: "cada um dos componentes
/// deve ao click imprimir seu nome no console").
pub fn apply_event(_store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event
        && let Some(name) = topbar_chip_name(id)
    {
        println!("[topbar] click: {name}");
    }
    false
}

/// Paint a Tooltip floating just above the currently hovered widget.
/// Called by the hero orchestrator after every chrome painter has
/// run so the tooltip lands on top. Tooltip text is read from the
/// store's generic tooltip side-table — populate it via
/// `WidgetStore::set_tooltip(id, text)` from any painter / populate
/// pass.
/// ⚠️ **`viewport` porque a dica tem de CABER, e até 2026-08-28 ela não cabia.** O comentário
/// abaixo prometia *"if that would clip the right edge of the viewport, fall back to
/// right-aligning"* e o código **não tinha esse recuo** — nem o vertical. Enquanto todo
/// consumidor era o top bar (colado ao TOPO, e portanto sempre com espaço por baixo) a ausência
/// não aparecia; os chips do grafo estão no **canto inferior-esquerdo**, onde as duas faltam ao
/// mesmo tempo: a dica caía por baixo da janela e para fora da margem esquerda. *Um comentário
/// que descreve um recuo que o código não tem é pior que nenhum: ele faz o próximo leitor
/// procurar o defeito noutro lugar.*
/// **ONDE a dica cabe** — a lei da colocação, numa função pura porque é a única forma de a
/// gatear: dentro do painter ela não é alcançável de um teste, e uma mutação que a desligasse
/// compilava e passava a suíte inteira.
///
/// Duas regras, e as duas eram **prometidas por comentário e ausentes do código** até
/// 2026-08-28:
///
/// 1. **Horizontal:** centrada no alvo, depois trazida para dentro do viewport. O primeiro chip
///    da barra do grafo fica a `8 px` da margem esquerda, e uma dica de `160 px` centrada nele
///    começaria **fora** da janela.
/// 2. **Vertical:** por BAIXO quando cabe, por CIMA quando não cabe. Os chips estão no canto
///    **inferior**-esquerdo — por baixo deles não há janela nenhuma.
///
/// ⚠️ **A preferência continua a ser por baixo**, e isso é deliberado: é onde o top bar a
/// desenha desde sempre, e uma regra que escolhesse *"o lado com mais espaço"* moveria dicas
/// que já estão certas. *Uma cura que muda o que já funcionava é uma segunda mudança
/// escondida na primeira.*
#[must_use]
pub fn tooltip_rect(target: Rect, pill_w: f32, pill_h: f32, gap: f32, viewport: Rect) -> Rect {
    let x = (target.x + (target.w - pill_w) * 0.5)
        .min(viewport.x + viewport.w - pill_w)
        .max(viewport.x);
    let below = target.y + target.h + gap;
    let y = if below + pill_h <= viewport.y + viewport.h {
        below
    } else {
        (target.y - gap - pill_h).max(viewport.y)
    };
    Rect::new(x, y, pill_w, pill_h)
}

pub fn paint_hover_tooltip(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some(id) = store.hot_id() else {
        return;
    };
    // Suppress the tooltip when the hot widget is an open Dropdown
    // (or any widget whose popover paints directly below the hit
    // rect — they share the exact pill geometry the tooltip wants
    // and would otherwise paint OVER the first option). See
    // `docs/UI_Bugs/README.md` §9.13.
    if matches!(
        store.get(id),
        Some(crate::interaction::InteractiveState::Dropdown { open: true, .. })
            | Some(crate::interaction::InteractiveState::Combobox { open: true, .. })
    ) {
        return;
    }
    let Some(text) = store.tooltip_for(id) else {
        return;
    };
    let Some(target_rect) = hit_index.rect_for(id) else {
        return;
    };
    // Real text measurement instead of `chars × 6.5` — for
    // proportional fonts the approximation is off by 10-30 % and
    // the pill ends up clipped or oversized. See
    // `docs/UI_Bugs/README.md` §3.3.
    let font_size = ph2d_tokens::TypeToken::Sm.px();
    let measured_w = text_system.layout(text, font_size, f32::INFINITY).width();
    let pill_w = (measured_w + Spacing::Xl.px()).max(60.0); // LITERAL-PX-OK: tooltip pill min width (chrome-specific)
    let pill_h = (font_size + 10.0).max(22.0); // LITERAL-PX-OK: tooltip pill height composite + min (chrome-specific)
    let tip_rect = tooltip_rect(target_rect, pill_w, pill_h, Spacing::Sm.px(), viewport);
    let tip = Tooltip::new(NodeId(0), text);
    paint_tooltip(&tip, tip_rect, scene, text_system, theme);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    image_tools_mode: bool,
    motion: &crate::motion::UiMotion,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    // Left half now holds 7 clusters: Theme, Project (Level), Save,
    // Open, Image Tools, Physics, Audio Mixer (Project moved here 2026-05-24;
    // Audio Mixer added 2026-07-05; Physics added 2026-07-27).
    //
    // ⚠️ **O número CRESCEU junto com a lista, e não é detalhe:** o pill novo
    // entra depois do IMG, então deixar o split em 6 empurraria o Audio Mixer
    // para o grupo da DIREITA — um pill mudando de lado da tela por causa de um
    // vizinho, que ninguém pediu.
    let split = 7.min(clusters.len());
    // Single agrupador backdrop spanning ALL left clusters (Enio
    // 2026-05-24: "Os componentes da esquerda devem ter apenas 1
    // fundo"). RailBg + radius Lg, top edge glued to viewport.y so
    // it touches the top of the screen.
    {
        let mut left_w = 0.0_f32;
        for (i, (_, c)) in clusters[..split].iter().enumerate() {
            if i > 0 {
                left_w += gap;
            }
            left_w += cluster_width(c);
        }
        if left_w > 0.0 {
            paint_topbar_group_backdrop(
                ids::TOPBAR_LEFT_BACKDROP,
                scene,
                theme,
                Rect::new(layout.top_bar.x, layout.top_bar.y, left_w, row_h),
                store.rail_button_size().chip_px(),
                layout.viewport.y,
                hit_index,
            );
        }
    }
    // Left half is always painted — the Image Tools mode keeps the
    // identity / Save / Open / ImageTools cluster visible so the user
    // can exit the mode by clicking ImageTools again.
    for (id, cluster) in &clusters[..split] {
        let rect = Rect::new(x, layout.top_bar.y, cluster_width(cluster), row_h);
        // ⭐ O modo ligado é o `active` do chip do Image Tools — pintado pelo próprio chip, pela
        //    matriz do rail. Até 2026-09-05 era um anel de acento traçado AQUI por cima do chip,
        //    reconstruindo o `chip_rect` à mão; num tema moderno (sem moldura) ele sumiria e o
        //    modo ficaria invisível. Ver `paint_topbar_rail_chip`.
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            layout.viewport.y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            image_tools_mode && *id == ids::TOPBAR_IMAGE_TOOLS,
        );
        x = rect.x + rect.w + gap;
    }
    // The wordmark "PH2D · EDITOR" that used to fill the middle gap
    // is intentionally absent now — the engine's identity is carried
    // by the leftmost theme chip (also labelled "PH2D"). Leaving the
    // gap transparent also keeps the topbar's bg fully see-through.
    // (`x` now holds the left group's right edge — used to clamp the right
    // group below so it can't overlap the left clusters when the bar
    // overflows.)

    if image_tools_mode {
        // Mode on — replace the right half with the image-action row.
        paint_image_action_row(layout, scene, text_system, theme, hit_index, store, motion);
        return;
    }

    // Default mode — paint the right clusters (Project / Play / Right /
    // Settings) right-aligned to the bar.
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for (i, (_, c)) in right_clusters.iter().enumerate() {
        if i > 0 {
            right_w += gap;
        }
        right_w += cluster_width(c);
    }
    // Right-align the right clusters — but clamp so the group never starts
    // before the left group ends (`x` = left-group right edge + gap). When
    // the bar overflows (too many clusters for the width), this stops the
    // leftmost right clusters (the vector tool pills) from being painted
    // under Save/Open/IMG — they stay on-screen + clickable; the rightmost
    // clusters clip off the right edge instead (restored by the W2-close
    // topbar UI pass). No-op when everything fits.
    let right_x = (layout.top_bar.x + layout.top_bar.w - right_w).max(x);
    // Single agrupador backdrop spanning ALL right clusters (Enio
    // 2026-05-24: "Os componentes da direita apenas um fundo").
    if right_w > 0.0 {
        paint_topbar_group_backdrop(
            ids::TOPBAR_RIGHT_BACKDROP,
            scene,
            theme,
            Rect::new(right_x, layout.top_bar.y, right_w, row_h),
            store.rail_button_size().chip_px(),
            layout.viewport.y,
            hit_index,
        );
    }
    let mut rx = right_x;
    for (id, cluster) in right_clusters {
        let rect = Rect::new(rx, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            layout.viewport.y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            false,
        );
        rx = rect.x + rect.w + gap;
    }
}

/// Paint the agrupador backdrop behind a topbar cluster group.
/// Height is computed from `chip_px` so the plate is as tight as the
/// side rail's (Enio 2026-05-25: "altura reduzida que caiba apenas
/// os botões e as labels praticamente sem espaços"). `Radius::Md`
/// matches the side rail's backdrop corner. Horizontal `Sm` bleed
/// each side. Hit-registers FIRST so chips painted afterwards win
/// the hit (HitIndex walks back-to-front).
fn paint_topbar_group_backdrop(
    id: NodeId,
    scene: &mut VectorScene,
    theme: Theme,
    group_rect: Rect,
    chip_px: f32,
    viewport_y: f32,
    hit_index: &mut HitIndex,
) {
    let pad_h = Spacing::Sm.px();
    let pad_v = Spacing::Xxs.px();
    // ⚠️ LIDAS do rail, nunca copiadas — ver `LABEL_TO_CHIP_GAP_PX`.
    let label_band_h = crate::widget::LABEL_VISUAL_EXTENT_PX;
    let label_to_chip_gap = crate::widget::LABEL_TO_CHIP_GAP_PX;
    let bg_h = chip_px + label_to_chip_gap + label_band_h + pad_v * 2.0;
    // Colado no topo da viewport (Enio 2026-05-25).
    let bg_y = viewport_y;
    let bg = Rect::new(group_rect.x - pad_h, bg_y, group_rect.w + pad_h * 2.0, bg_h);
    hit_index.register(id, bg);
    fill_rounded_rect(
        scene,
        bg,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
}

/// Cluster-level painter + width helper. Extracted to a sibling
/// file in Wave 2 PR 11.7c so this `mod.rs` stays under the HR-18
/// 600-LOC cap. Same private surface as before; both functions
/// remain `pub(super)`-callable from `paint_top_bar`.
mod chip_name;
mod cluster_painter;
// A fila de Image Tools — a única do topbar DERIVADA do registry. Saiu para irmã quando
// este ficheiro passou o teto de 700 LOC (2026-08-19).
mod image_action_row;
// A tabela de tooltips — saiu para irmã na W4 do 3DModeling (2026-08-22), pelo mesmo teto.
mod tooltips;

use chip_name::topbar_chip_name;
use cluster_painter::{
    TOPBAR_INTER_CHIP_GAP, TOPBAR_RAIL_CHIP_W, cluster_width, paint_top_bar_cluster,
    paint_topbar_rail_chip,
};
pub use image_action_row::image_action_a11y_nodes;
/// ⭐ As ferramentas de imagem como entradas de trilho — a porta que as traz para a fila
/// horizontal, depois de a barra de pills sair de cena.
pub(crate) use image_action_row::image_tool_rail_entries;
use image_action_row::{image_action_pills, paint_image_action_row};

#[cfg(test)]
mod tooltip_placement_tests {
    use super::*;

    const VP: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 800.0,
    };
    const GAP: f32 = 8.0;

    /// **O CASO NORMAL não se mexeu** — um alvo no topo continua a receber a dica POR BAIXO e
    /// CENTRADA. É o controle da wave inteira: o top bar já estava certo, e uma cura que o
    /// movesse seria uma regressão disfarçada.
    #[test]
    fn a_target_with_room_below_keeps_the_tooltip_below_and_centred() {
        let target = Rect::new(600.0, 10.0, 40.0, 28.0);
        let got = tooltip_rect(target, 160.0, 22.0, GAP, VP);
        assert!(
            (got.y - (10.0 + 28.0 + GAP)).abs() < 1e-3,
            "por baixo: {got:?}"
        );
        assert!(
            (got.x - (600.0 + (40.0 - 160.0) * 0.5)).abs() < 1e-3,
            "centrada: {got:?}"
        );
    }

    /// **UM ALVO COLADO AO FUNDO RECEBE A DICA POR CIMA** — os chips da barra do grafo.
    ///
    /// ⚠️ Sem isto a dica é desenhada abaixo da janela: **invisível**, que é o mesmo que não
    /// existir. E era o estado do código enquanto todo consumidor vivia no topo.
    #[test]
    fn a_target_at_the_bottom_gets_the_tooltip_above() {
        // Um chip de 24 px encostado ao fundo, como a barra do grafo o desenha.
        let target = Rect::new(600.0, VP.h - 32.0, 24.0, 24.0);
        let got = tooltip_rect(target, 160.0, 22.0, GAP, VP);
        assert!(
            got.y + got.h <= target.y - GAP + 1e-3,
            "a dica tem de ficar ACIMA do chip: {got:?} contra {target:?}"
        );
        assert!(got.y >= VP.y, "e dentro da janela: {got:?}");
    }

    /// **UM ALVO NA MARGEM ESQUERDA NÃO EMPURRA A DICA PARA FORA** — o primeiro chip da barra.
    ///
    /// ⚠️ Este é o recuo que o comentário do painter **prometia** (*"fall back to
    /// right-aligning"*) e que o código não tinha. Um comentário que descreve um recuo ausente
    /// manda o próximo leitor procurar o defeito noutro lugar.
    #[test]
    fn a_target_at_the_left_edge_keeps_the_tooltip_inside() {
        let target = Rect::new(8.0, 400.0, 24.0, 24.0);
        let got = tooltip_rect(target, 200.0, 22.0, GAP, VP);
        assert!(got.x >= VP.x - 1e-3, "nao pode comecar fora: {got:?}");
        assert!(got.x + got.w <= VP.x + VP.w + 1e-3);
    }

    /// **E na margem DIREITA também** — o par do anterior, e é o par que faz a lei (um clamp
    /// só de um lado passa por metade dos casos).
    #[test]
    fn a_target_at_the_right_edge_keeps_the_tooltip_inside() {
        let target = Rect::new(VP.w - 32.0, 400.0, 24.0, 24.0);
        let got = tooltip_rect(target, 200.0, 22.0, GAP, VP);
        assert!(
            got.x + got.w <= VP.x + VP.w + 1e-3,
            "nao pode acabar fora: {got:?}"
        );
        assert!(got.x >= VP.x - 1e-3);
    }

    /// ⭐⭐ **O MODO LIGADO é o `active` do chip, e VÊ-SE em todas as famílias.**
    ///
    /// Até 2026-09-05 o modo *Image Tools* era um anel de acento traçado pelo `paint_top_bar` POR
    /// CIMA do chip — fora da tabela de estados dele, reconstruindo o `chip_rect` à mão. Num tema
    /// moderno (moldura em repouso = `0`) o anel desapareceria e o modo ficaria invisível. Hoje o
    /// chip sabe que está activo e pinta-se pela matriz do rail (a tinta `AccentSoft`), que é
    /// visível em TODAS as famílias — é isso que este gate mede: o chip com `active = true`
    /// emite tinta DIFERENTE do chip em repouso, no clássico, no moderno e no OLED.
    ///
    /// ⚠️ Compara o `draw_data` (as cores e os pincéis), não a contagem de caminhos: no moderno o
    /// chip activo tem os MESMOS caminhos que o em repouso (fundo + glifo), só a cor muda.
    ///
    /// **Mutação que deve sangrar:** `is_active = state == ButtonState::Pressed` (ignorar o
    /// `active`) em `paint_topbar_rail_chip` — as duas cenas ficam byte a byte iguais.
    #[test]
    fn the_image_tools_chip_shows_the_mode_in_every_family() {
        for theme in [Theme::Forge, Theme::Dark, Theme::Oled] {
            let paint = |active: bool| {
                let mut scene = VectorScene::new();
                let mut text = TextSystem::without_system_fonts();
                let mut hit = HitIndex::new();
                let store = WidgetStore::default();
                let motion = crate::motion::UiMotion::default();
                super::cluster_painter::paint_topbar_rail_chip(
                    ids::TOPBAR_IMAGE_TOOLS,
                    IconGlyph::Builtin(IconId::Image),
                    "IMG",
                    Rect::new(0.0, 0.0, 44.0, 48.0),
                    0.0,
                    &mut scene,
                    &mut text,
                    theme,
                    &mut hit,
                    &store,
                    &motion,
                    active,
                );
                scene.inner().encoding().draw_data.clone()
            };
            let rest = paint(false);
            let on = paint(true);
            assert!(
                !rest.is_empty(),
                "{theme:?}: o chip nao pintou nada — a regua esta' partida"
            );
            assert_ne!(
                rest, on,
                "{theme:?}: o chip do Image Tools com o modo LIGADO pinta igual ao em repouso — o \
                 modo ficou invisivel"
            );
        }
    }
}
