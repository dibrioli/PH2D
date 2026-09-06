//! The scrolled body: the collapsible section stack, then the debug rows.
//!
//! Sibling of `paint.rs` because the panel LOC cap is 600 and the two halves
//! grow for different reasons (chrome vs. content).

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, SectionFold, SectionHeader, paint_button, paint_section_header,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::rows;
use crate::state::PhysicsSnapshot;

/// **As secções guiadas por TABELA** — as cinco de [`rows::SECTIONS`], cujo corpo é uma lista de
/// `Row`. Devolve o `y` seguinte.
///
/// Irmã de [`paint_sections`] pelo tecto de 200 LOC por função, e o corte é de RESPONSABILIDADE:
/// aqui o corpo de uma secção é **derivado** da tabela; lá em cima cada corpo é uma coisa
/// diferente (uma grelha de 36 células, dois rádios, uma dica que quebra).
fn table_sections(
    ctx: &mut PaintCtx,
    snapshot: &PhysicsSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = y_in;
    let row_gap = Spacing::Xs.px();
    for section in rows::SECTIONS {
        let (fold, next_y) = header(ctx, section.id, tr(section.title), x, w, y);
        y = next_y;
        if let Some(fold) = fold {
            let mut inner = y;
            // ⭐ **O interruptor mestre da secção Sleep**, no topo dela — o idioma do modificador
            // do Blender: a secção diz o que faz, o 1.º controlo diz se ela está a fazê-lo.
            //
            // ⛔⛔ **Ele é um INTERRUPTOR porque é isso que a `rapier2d` 0.35 lê.** O
            // `sleep_angular_threshold` era um slider `0..10` aqui, e desde a 0.35 o motor lê
            // dele **o sinal**: `>= 0` = os corpos podem dormir · `< 0` = nunca dormem
            // (`ph2d_physics::SLEEP_SPIN_DISABLED` traz o trecho). Arrastar de `0,1` para `2,0`
            // não movia um bit — medido em `ph2d-physics`,
            // `the_magnitude_of_the_spin_threshold_reaches_nobody`.
            //
            // ⚠️ **O rótulo é `panel.physics.sleep_enabled` → *"Enabled"***, e não o da secção nem
            // o do slider morto (*"Spin"*), que era a única opção que MENTIA: o sinal não fala de
            // rotação nenhuma, fala de dormir. (A chave nasceu em 2026-08-30, no mesmo dia que
            // este interruptor; a linha anterior aqui dizia que ela não existia.)
            if section.id == ids::PHYSICS_SEC_SLEEP {
                inner = toggle(
                    ctx,
                    ids::PHYSICS_SLEEP_SPIN,
                    tr("panel.physics.sleep_enabled"),
                    snapshot.settings.sleep_enabled(),
                    x,
                    w,
                    inner,
                ) + row_gap;
            }
            for row in section.rows {
                let value = (row.get)(&snapshot.settings);
                let used = super::paint_row(ctx, row, value, x, w, inner);
                inner += used + row_gap;
            }
            y = end_fold(ctx, fold, inner);
        }
        y += Spacing::Md.px();
    }
    y
}

/// Paint every section, then the debug rows. Returns the y it ended at.
pub(super) fn paint_sections(
    ctx: &mut PaintCtx,
    snapshot: &PhysicsSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let row_gap = Spacing::Xs.px();
    let mut y = table_sections(ctx, snapshot, x, w, y_in);

    // The Interaction tool (W-Hand). BEFORE the layer matrix and after the world
    // sliders, on purpose: it is the section an artist reaches for while a scene
    // is RUNNING, so it should not sit under a 36-cell grid.
    let (fold, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_INTERACT,
        tr("panel.physics.section.interact"),
        x,
        w,
        y,
    );
    y = next_y;
    if let Some(fold) = fold {
        let mut inner = super::interact::paint_interact(ctx, &snapshot.interaction, x, w, y);
        inner += Spacing::Md.px();
        y = end_fold(ctx, fold, inner);
    }

    // The Joint tool (W-JointTools). Right after Interaction because the two are
    // the same question asked of opposite transport states — what the POINTER
    // does — and a reader who found one should find the other without scrolling
    // past a 36-cell grid.
    let (fold, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_JOINT,
        tr("panel.physics.section.joint"),
        x,
        w,
        y,
    );
    y = next_y;
    if let Some(fold) = fold {
        let mut inner = super::joint::paint_joint(ctx, &snapshot.interaction, x, w, y);
        inner += Spacing::Md.px();
        y = end_fold(ctx, fold, inner);
    }

    // Collision layers. Its own section because the matrix is a different KIND
    // of control from the sliders above — and because it is tall.
    let (fold, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_LAYERS,
        tr("panel.physics.section.layers"),
        x,
        w,
        y,
    );
    y = next_y;
    if let Some(fold) = fold {
        let mut inner = super::matrix::paint(
            ctx,
            ph2d_physics_ecs::LayerMatrix::from_rows(snapshot.settings.layer_matrix),
            x,
            y,
        );
        inner += Spacing::Md.px();
        y = end_fold(ctx, fold, inner);
    }

    let (fold, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_DEBUG,
        tr("panel.physics.section.debug"),
        x,
        w,
        y,
    );
    y = next_y;
    let Some(fold) = fold else {
        return y;
    };

    // "Show Colliders" mirrors the shell's flag — the same one the `B` key
    // owns. The pressed state comes from the SNAPSHOT, never from a local
    // toggle, so the key and this control can never disagree.
    y = toggle(
        ctx,
        ids::PHYSICS_SHOW_COLLIDERS,
        tr("panel.physics.show_colliders"),
        snapshot.show_colliders,
        x,
        w,
        y,
    );
    y += row_gap;

    // Read-only facts, drawn as plain text and hit-indexed by nobody.
    //
    // ⚠️ The world scale is `ProjectSettings::pixels_per_meter`, a PROJECT
    // setting (ADR-0131 D4). It is shown so the metre-valued rows above can be
    // read in pixels — NOT so they can be edited here. A second door onto it
    // would diverge from the one in Project Settings.
    y = readout(
        ctx,
        &format!(
            "{}: {:.0} px/m",
            tr("panel.physics.scale"),
            snapshot.pixels_per_meter
        ),
        x,
        w,
        y,
    );
    // Zero is worth showing: it is the difference between "gravity is wrong"
    // and "nothing in this scene has a body yet".
    y = readout(
        ctx,
        &format!("{}: {}", tr("panel.physics.bodies"), snapshot.body_count),
        x,
        w,
        y,
    );
    y += Spacing::Md.px();

    // ── A CORRIDA GRAVADA (W25) ────────────────────────────────────────────
    //
    // ⚠️ **A segunda VISTA de um fato do documento, nunca uma segunda porta.**
    // A §14 do Inspector mostra o mesmo par de números e emite os mesmos dois
    // verbos, e os dois caminhos caem na MESMA função da shell — o precedente
    // exato do `Show Colliders` acima, que espelha a tecla `B`.
    //
    // ⚠️ **E ela existe porque a §14 é por-ENTIDADE:** ela só nasce sobre um
    // corpo Dynamic selecionado, enquanto a fita é do DOCUMENTO e sobrevive ao
    // player que a gravou. Sem esta vista, apagar o personagem prendia a
    // corrida — no arquivo, ainda a ser o que o Bake replaya, e sem gesto que a
    // alcançasse.
    //
    // ⚠️ **Os dois botões nunca coexistem**, e o ciclo de vida é DERIVADO e não
    // mantido: descartar esvazia a fita viva, então só um dos dois números pode
    // ser não-zero. Gravar de novo esconde o de devolver.
    if snapshot.recorded_run_seconds > 0.0 {
        y = command(
            ctx,
            ids::PHYSICS_CLEAR_RUN,
            &format!(
                "{} ({:.1} s)",
                tr("panel.physics.clear_run"),
                snapshot.recorded_run_seconds
            ),
            x,
            w,
            y,
        );
        y += Spacing::Md.px();
    } else if snapshot.discarded_run_seconds > 0.0 {
        y = command(
            ctx,
            ids::PHYSICS_RESTORE_RUN,
            &format!(
                "{} ({:.1} s)",
                tr("panel.physics.restore_run"),
                snapshot.discarded_run_seconds
            ),
            x,
            w,
            y,
        );
        y += Spacing::Md.px();
    }

    // Reset sits at the BOTTOM, past everything it would undo: a destructive
    // command should not be on the path a hand takes to the first slider.
    y = command(
        ctx,
        ids::PHYSICS_RESET_DEFAULTS,
        tr("panel.physics.reset_defaults"),
        x,
        w,
        y,
    );
    end_fold(ctx, fold, y + row_gap)
}

/// A collapsible section header. Returns `(the_fold, y_after)` — `None` when the section is shut
/// **and still**, the only case in which the body is not painted at all.
///
/// ⚠️ **`Option<SectionFold>` rather than the old `bool`** (F4b): the bool came from
/// `is_collapsed`, which flips on the frame of the click while the fold's `t` is still falling —
/// a body gated on it would vanish at once under a chevron that is still turning.
fn header(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
) -> (Option<SectionFold>, f32) {
    let theme = ctx.host.theme();
    let h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    // ⚠️ O ÚNICO sítio deste painel que desenha um `SectionHeader`, e por isso o único que pode
    // responder «que cabeçalhos existem?» sem uma lista escrita à mão. Ver
    // `state::PAINTED_SECTION_HEADERS`.
    crate::state::note_painted_section_header(id);
    let collapsed = ctx.host.store().is_collapsed(id);
    let rect = Rect::new(x, y, w, h);
    let header = SectionHeader::new(id, title)
        .collapsible(!collapsed)
        .open_t(ctx.host.store().section_open_live(id));
    let body_top = y + h + Spacing::Sm.px();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&header, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    let fold = SectionFold::begin(store, id, x, w, body_top, scene, hit_index);
    (fold, body_top)
}

/// Closes the fold opened by [`header`] and hands back the outgoing `y`.
///
/// ⚠️ Exists because `finish` wants `&WidgetStore`, `&mut VectorScene` and `&mut HitIndex` at
/// once, and in a `PaintCtx` the three come from disjoint fields — the same dance `header` does.
fn end_fold(ctx: &mut PaintCtx, fold: SectionFold, y: f32) -> f32 {
    let scene = &mut *ctx.scene;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    fold.finish(store, scene, hit_index, y)
}

/// A Button used as a toggle.
///
/// **Not a `Checkbox`**: `Checkbox` emits `Toggled`, which this panel's
/// `event.rs` does not forward, so it would be registered and dead on click —
/// the warning `ph2d-panel-painter-layers` carries for the same reason.
fn toggle(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = if on {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        ctx.host.store().button_visual(id)
    };
    let kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).kind(kind).visual(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// A plain action button.
fn command(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, label: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx.host.store().button_visual(id);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).visual(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// A line of text. Hit-indexed by nobody on purpose — it is a fact, not a
/// control, and an affordance it cannot honour would be worse than plain text.
fn readout(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y + ROW_H_PX
}
