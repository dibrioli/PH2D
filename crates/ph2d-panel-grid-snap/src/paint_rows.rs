//! Number rows + origin/aabb/overlay/opacity/labeled-toggle painters.
//!
//! Ported verbatim from
//! `ph2d_editor_core::grid_snap::panel::paint_rows` during ADR-0029
//! Phase C.4.

use crate::state::{meters_to_display, unit_suffix_paren};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::{TextInputState, Toggle, paint_toggle};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

pub(crate) fn read_number_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

/// ⭐⭐⭐ **A COLUNA DE UMA SECÇÃO DESTE PAINEL — medida uma vez, sobre os nomes que ela pinta.**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com uma seta na linha *Major every (px)*:** *«a caixa recua
/// quando na verdade o nome deveria criar as colunas»*. Medido a `220` de painel: aquele nome pede
/// `92,2 px` e a metade da linha dá `90,0`, logo **só ele** passava da metade e **só a caixa dele**
/// recuava — `x = 100,2` contra os `98,0` das três irmãs. Ver
/// [`ph2d_editor_core::property_row::Seccao`].
///
/// ⚠️ **Toda linha deste painel tem UM campo**, então o que a secção declara é só o nome mais
/// largo — mas a declaração continua a ser dela, e não da linha.
pub(crate) fn seccao(
    text_system: &mut TextSystem,
    nomes: &[&str],
) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(text_system, 1, nomes)
}

/// Os dois nomes que a [`paint_origin_rows`] pinta — **a mesma expressão que ela usa**.
///
/// ⛔ Uma segunda escrita de `"Origin X" + sufixo` seria a segunda resposta à pergunta *«que nome
/// tem esta linha?»*, e a coluna passaria a ser medida sobre um texto que o painel não pinta.
pub(crate) fn origin_labels() -> [String; 2] {
    let suffix = unit_suffix_paren();
    [format!("Origin X{suffix}"), format!("Origin Y{suffix}")]
}

/// Os quatro nomes que a [`paint_aabb_rows`] pinta — ver [`origin_labels`].
pub(crate) fn aabb_labels(label_prefix: &str) -> [String; 4] {
    let suffix = unit_suffix_paren();
    [
        format!("{label_prefix} min X{suffix}"),
        format!("{label_prefix} min Y{suffix}"),
        format!("{label_prefix} max X{suffix}"),
        format!("{label_prefix} max Y{suffix}"),
    ]
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row(
    label: &str,
    id: NodeId,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    paint_number_row_value(
        label,
        id,
        value,
        Some(buffer),
        caret,
        anchor,
        (state, store.hover_live(id)),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        sec,
    )
}

/// Like [`paint_number_row`] but takes an explicit displayed value
/// (caller-supplied — typically read from state, not the store).
/// Used for "universal" origin / spacing rows that mirror the
/// ACTIVE kind's cfg field even though one NodeId is shared.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row_from_state(
    label: &str,
    id: NodeId,
    value: f64,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let (state, _, buffer, caret, anchor) = read_number_input(store, id);
    // Keep the live buffer for in-progress edits; otherwise display
    // the state-supplied value (so switching kinds repaints with the
    // new kind's origin).
    let buffer_arg = if state == TextInputState::Focused {
        Some(buffer)
    } else {
        None
    };
    paint_number_row_value(
        label,
        id,
        value,
        buffer_arg,
        caret,
        anchor,
        (state, store.hover_live(id)),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        sec,
    )
}

/// ⭐⭐⭐ **A linha do painel da Grelha, PELA PORTA do app.**
///
/// ⛔⛔ Ela já lia a coluna do rótulo da porta (`property_label_col_w`) e **re-derivava tudo o
/// resto à mão**: o rect do campo era `w − label_w`, que come os `14 px` da coluna de animação;
/// não havia ponto nenhum; a coluna não **cedia** ao controlo (spec §6-ter); e o nome era pintado
/// em [`TypeToken::Base`] enquanto o resto do app usa `Sm` — *este painel tinha os nomes maiores
/// que todos os outros.*
///
/// ⚠️ **Ela não pinta a partir do store, e é por isso que usa a entrada com VALOR explícito:** um
/// `NodeId` é partilhado por vários tipos de grelha e o número mostrado espelha o campo do tipo
/// ACTIVO. O passe de pintura recebe a loja por `&`, logo não há onde espelhar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row_value(
    label: &str,
    id: NodeId,
    value: f64,
    buffer: Option<&str>,
    caret: usize,
    anchor: Option<usize>,
    // ⚠️ **O PAR na assinatura, e não um `TextInputState` solto** — este helper não tem store,
    // então quem sabe quanto do hover está presente é o chamador; posto no tipo, esquecê-lo
    // deixa de compilar (o precedente do `paint_icon_button`).
    visual: (TextInputState, f32),
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    ph2d_editor_core::property_row::paint_field_row_value(
        scene,
        text_system,
        theme,
        hit_index,
        x,
        w,
        y,
        label,
        id,
        value,
        buffer,
        caret,
        anchor,
        visual,
        None,
        sec,
    )
    .0
}

/// Paint Origin X + Origin Y rows reading current values from
/// `state.active_origin()`. Returns Y after the second row.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_origin_rows(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let origin = state.active_origin();
    let suffix = unit_suffix_paren();
    let y = paint_number_row_from_state(
        &format!("Origin X{suffix}"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_X,
        meters_to_display(origin[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    paint_number_row_from_state(
        &format!("Origin Y{suffix}"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_Y,
        meters_to_display(origin[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    )
}

/// Paint a labeled AABB (min X / min Y / max X / max Y) as 4
/// stacked NumberInput rows reading current values from state.
/// Used by Quadtree and Voronoi for their `bounds` field.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_aabb_rows(
    label_prefix: &str,
    min_x_id: NodeId,
    min_y_id: NodeId,
    max_x_id: NodeId,
    max_y_id: NodeId,
    min: ph2d_grid::Vec2,
    max: ph2d_grid::Vec2,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let suffix = unit_suffix_paren();
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min X{suffix}"),
        min_x_id,
        meters_to_display(min[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min Y{suffix}"),
        min_y_id,
        meters_to_display(min[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} max X{suffix}"),
        max_x_id,
        meters_to_display(max[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    paint_number_row_from_state(
        &format!("{label_prefix} max Y{suffix}"),
        max_y_id,
        meters_to_display(max[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    )
}

pub(crate) fn paint_show_overlay_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    let sec = seccao(text_system, &["Show grid"]);
    paint_labeled_toggle(
        "Show grid",
        ph2d_editor_core::grid_snap::ids::GS_SHOW_OVERLAY,
        state.show_overlay,
        row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_opacity_slider_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    // Canonical label + track + chip — the same painter the Widget
    // Gallery slider uses, so this matches every other slider in the
    // app. (Was a one-off label + bare `paint_slider`.)
    let value = store
        .slider(ph2d_editor_core::grid_snap::ids::GS_OPACITY_SLIDER)
        .map(|(_, v)| v)
        .unwrap_or(state.opacity);
    ph2d_editor_core::widget::paint_slider_with_chip(
        row,
        "Opacity",
        value,
        ph2d_editor_core::grid_snap::ids::GS_OPACITY_SLIDER,
        ph2d_editor_core::NodeId(0),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_toggle(
    label: &str,
    id: NodeId,
    on: bool,
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    sec: ph2d_editor_core::property_row::Seccao,
) {
    // ⭐⭐⭐ **O NOME VAI À COLUNA DO NOME, como o das linhas de número desta mesma secção**
    // (2026-09-15). ⛔⛔ Ele era pintado encostado à ESQUERDA, num orçamento escrito à mão
    // (`row.w − 60`), e em [`LABEL_FONT_SIZE`] — que é o corpo `Base`, **um px maior** que o `Sm`
    // das linhas de número ao lado. *Duas colunas de nome e dois corpos de letra na mesma
    // secção* — a queixa do dono de 2026-09-14 (*«as labels alinhadas todas à direita»*) a
    // sobreviver na única linha que não era um campo.
    let fonte = ph2d_tokens::TypeToken::Sm.px();
    let colunas = ph2d_editor_core::widget::colunas_da_linha(row.x, row.w, row.y, row.h, sec);
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label,
        colunas.label.x,
        colunas.label.y + (colunas.label.h - fonte) * 0.5,
        fonte,
        colunas.label.w,
        resolve(ColorToken::Text1, theme),
    );

    let toggle_w = 40.0; // LITERAL-PX-OK: canonical compact toggle width (matches Inspector toggles)
    let toggle_h = 20.0; // LITERAL-PX-OK: canonical compact toggle height (matches Inspector toggles)
    let toggle_rect = Rect::new(
        row.x + row.w - toggle_w - Spacing::Xs.px(),
        row.y + (row.h - toggle_h) * 0.5,
        toggle_w,
        toggle_h,
    );
    // ⚠️ **Era um literal de struct com uma derivação PRIVADA do estado** (`toggle_state`, a sexta
    //    cópia da mesma pergunta no app) — e um literal não tem como ganhar um campo novo sem que
    //    o compilador o exija, que foi exactamente o que aconteceu aqui. O construtor + a porta
    //    única resolvem os dois: o `on` vem do modelo, o estado e o `t` do store.
    let toggle = Toggle::new(id, String::new())
        .on(on)
        .visual(store.toggle_visual(id));
    paint_toggle(&toggle, toggle_rect, scene, theme);
    hit_index.register(id, toggle_rect);
}

// ⛔ **A re-exportação do `button_state` MORREU em 2026-09-15.** Ela existia *«para o
// `paint_kinds.rs` o usar sem atravessar módulos»*, e o único chamador dele mudou-se para o
// `paint_kinds_bounded.rs` (corte por responsabilidade), que o importa do dono —
// `crate::paint_helpers`. *Um atalho que sobrevive ao chamador que o pediu é um segundo caminho
// para o mesmo símbolo.*
