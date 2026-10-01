//! The scrolled body: one painter per collapsible section, each taking the THEME it paints in.
//!
//! Sibling of `paint.rs` because the panel LOC cap is 600 and the two halves
//! grow for different reasons (chrome vs. content). ⭐ Since 2026-09-30 the ORDER is not here: the
//! sections are entries of a [`PlanoCtx`](ph2d_editor_core::panel::section_plan_ctx::PlanoCtx),
//! declared in [`super::plano`], and each painter receives the theme the artist chose for its
//! section — never `ctx.host.theme()`, which would paint a re-themed card's widgets in the panel's
//! colours.

use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, section_plan};
use ph2d_editor_core::widget::{Button, SectionFold, paint_button, paint_section_header};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::rows;
use crate::state::PhysicsSnapshot;

/// **Uma secção guiada por TABELA** — uma das cinco de [`rows::SECTIONS`], cujo corpo é uma lista
/// de `Row`. Devolve o `y` seguinte.
///
/// Irmã das secções pintadas à mão ([`interact_section`] … [`debug_section`]), e o corte é de
/// RESPONSABILIDADE: aqui o corpo de uma secção é **derivado** da tabela; lá cada corpo é uma coisa
/// diferente (uma grelha de 36 células, dois rádios, uma dica que quebra).
///
/// ⚠️ **Sem vão no fim** (2026-09-30): o `Spacing::Md` que cada secção somava depois de si era a
/// fronteira entre secções; hoje ela é a borda do CARTÃO, e quem a pinta é o corredor do plano
/// ([`super::plano`]). Os dois juntos punham um vão de secção a mais entre cada par.
pub(super) fn table_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    section: &rows::Section,
    snapshot: &PhysicsSnapshot,
    (x, w, y_in): (f32, f32, f32),
) -> f32 {
    let mut y = y_in;
    // ⚠️ **O vão entre dois controlos é uma PORTA, não um `Spacing` escolhido aqui**
    //    (`ph2d_tokens::control_gap_px`, 3 px). Enio, 2026-09-07: *«entre grupos de botões
    //    temos um espaçamento, entre sliders outro. Para ambos vamos colocar o padrão de 3 px»*.
    //    Este ficheiro é anterior à porta e escrevia o `Spacing::Xs` (4) à mão.
    let row_gap = ph2d_tokens::control_gap_px();
    {
        let (fold, next_y) = header(ctx, theme, section.id, tr(section.title), x, w, y);
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
            if section.id == crate::ids::PHYSICS_SEC_SLEEP {
                inner = check(
                    ctx,
                    theme,
                    crate::ids::PHYSICS_SLEEP_SPIN,
                    tr("panel.physics.sleep_enabled"),
                    snapshot.settings.sleep_enabled(),
                    (x, w, inner),
                    // A coluna das linhas de número desta secção — o *Enabled* alinha com elas.
                    ph2d_editor_core::widget::Seccao::apenas_campos(1),
                );
            }
            for row in section.rows {
                let value = (row.get)(&snapshot.settings);
                let used = super::paint_row(ctx, theme, row, value, x, w, inner);
                inner += used + row_gap;
            }
            y = end_fold(ctx, fold, inner);
        }
    }
    y
}

/// The Interaction tool (W-Hand). Declared BEFORE the layer matrix and after the world sliders,
/// on purpose: it is the section an artist reaches for while a scene is RUNNING, so it should not
/// sit under a 36-cell grid. (Since 2026-09-30 that is the NATURAL order — the artist may drag it.)
pub(super) fn interact_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    snapshot: &PhysicsSnapshot,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let (fold, y) = header(
        ctx,
        theme,
        crate::ids::PHYSICS_SEC_INTERACT,
        tr("panel.physics.section.interact"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let inner = super::interact::paint_interact(ctx, theme, &snapshot.interaction, x, w, y);
    end_fold(ctx, fold, inner + Spacing::Md.px())
}

/// The Joint tool (W-JointTools). Right after Interaction in the natural order because the two
/// are the same question asked of opposite transport states — what the POINTER does — and a
/// reader who found one should find the other without scrolling past a 36-cell grid.
pub(super) fn joint_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    snapshot: &PhysicsSnapshot,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let (fold, y) = header(
        ctx,
        theme,
        crate::ids::PHYSICS_SEC_JOINT,
        tr("panel.physics.section.joint"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let inner = super::joint::paint_joint(ctx, theme, &snapshot.interaction, x, w, y);
    end_fold(ctx, fold, inner + Spacing::Md.px())
}

/// Collision layers. Its own section because the matrix is a different KIND of control from the
/// sliders — and because it is tall.
pub(super) fn layers_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    snapshot: &PhysicsSnapshot,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let (fold, y) = header(
        ctx,
        theme,
        crate::ids::PHYSICS_SEC_LAYERS,
        tr("panel.physics.section.layers"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let inner = super::matrix::paint(
        ctx,
        theme,
        ph2d_physics_ecs::LayerMatrix::from_rows(snapshot.settings.layer_matrix),
        x,
        y,
    );
    end_fold(ctx, fold, inner + Spacing::Md.px())
}

/// Debug: the collider overlay, the two read-only facts, the recorded-run verbs and Reset.
///
/// ⚠️ **Ela arrasta-se como as outras** (2026-09-30). O *Reset to Defaults* mora no fundo DELA
/// (*«past everything it would undo»*), e na ordem natural ela é a última do painel; se o artista
/// a puser em cima, é a escolha dele — prendê-la como FIXA pô-la-ia no TOPO (o plano pinta as
/// fixas primeiro), que é o contrário do que aquela regra pede.
pub(super) fn debug_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    snapshot: &PhysicsSnapshot,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let row_gap = ph2d_tokens::control_gap_px(); // a mesma porta da secção de tabela
    let (fold, mut y) = header(
        ctx,
        theme,
        crate::ids::PHYSICS_SEC_DEBUG,
        tr("panel.physics.section.debug"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };

    // "Show Colliders" mirrors the shell's flag — the same one the `B` key
    // owns. The pressed state comes from the SNAPSHOT, never from a local
    // toggle, so the key and this control can never disagree.
    //
    // ⚠️ A coluna do nome é MEDIDA sobre o próprio rótulo (`Seccao::medida`) e não a de omissão:
    //    ele é a ÚNICA linha da secção *Debug*, logo não há vizinho com quem alinhar, e a caixa
    //    precisa de pouco — com a coluna de omissão (metade da linha) o `Show Colliders` era
    //    CORTADO no degrau estreito da escada (medido 2026-09-24 pelas duas catracas de elisão).
    let colliders = tr("panel.physics.show_colliders");
    let seccao = ph2d_editor_core::widget::Seccao::medida(ctx.text_system, 1, &[colliders]);
    y = check(
        ctx,
        theme,
        crate::ids::PHYSICS_SHOW_COLLIDERS,
        colliders,
        snapshot.show_colliders,
        (x, w, y),
        seccao,
    );

    // Read-only facts, drawn as plain text and hit-indexed by nobody.
    //
    // ⚠️ The world scale is `ProjectSettings::pixels_per_meter`, a PROJECT
    // setting (ADR-0131 D4). It is shown so the metre-valued rows above can be
    // read in pixels — NOT so they can be edited here. A second door onto it
    // would diverge from the one in Project Settings.
    y = readout(
        ctx,
        theme,
        &ph2d_i18n::tr_with(
            "panel.physics.scale_readout",
            &[
                ("label", &tr("panel.physics.scale")),
                ("value", &format!("{:.0}", snapshot.pixels_per_meter)),
            ],
        ),
        x,
        w,
        y,
    );
    // Zero is worth showing: it is the difference between "gravity is wrong"
    // and "nothing in this scene has a body yet".
    y = readout(
        ctx,
        theme,
        // ⚠️ **A forma da frase vem da tabela, não do `format!`.** Antes só a PALAVRA era
        // traduzida e o `": "` ficava no código — e há línguas em que o dois-pontos leva espaço
        // antes. *Uma frase composta é um MODELO; traduzir só as peças dela deixa a gramática no
        // fonte.*
        &tr_with(
            "panel.physics.bodies_count",
            &[("n", &snapshot.body_count.to_string())],
        ),
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
            theme,
            crate::ids::PHYSICS_CLEAR_RUN,
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
            theme,
            crate::ids::PHYSICS_RESTORE_RUN,
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
        theme,
        crate::ids::PHYSICS_RESET_DEFAULTS,
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
    theme: Theme,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
) -> (Option<SectionFold>, f32) {
    let h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    // ⚠️ O ÚNICO sítio deste painel que desenha um `SectionHeader`, e por isso o único que pode
    // responder «que cabeçalhos existem?» sem uma lista escrita à mão. Ver
    // `state::PAINTED_SECTION_HEADERS`.
    crate::state::note_painted_section_header(id);
    let rect = Rect::new(x, y, w, h);
    // ⭐⭐ **O cabeçalho é o do PLANO** (2026-09-30, *«siga com os outros painéis»*): a pega de dez
    //    pontos à direita — todas as secções deste painel se arrastam ([`super::plano`]) —, e o
    //    registo escreve a secção no livro do quadro, que é o que o botão direito no título lê para
    //    abrir o menu de tema e o arrasto da pega lê para resolver a queda.
    let header = section_plan::cabecalho(ctx.host.store(), id, title);
    let body_top = y + h + Spacing::Sm.px();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&header, rect, scene, text_system, theme);
    section_plan::regista_cabecalho(hit_index, id, rect);
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

/// ⭐⭐ **Uma linha de MARCAR, pela porta da casa** ([`ph2d_editor_core::property_row::paint_check_row`])
/// — o nome na coluna que o CHAMADOR declara (a das linhas de número da secção, quando há; medida
/// sobre o próprio rótulo, quando a caixa está sozinha) e a caixa na do valor. Ordem do dono (2026-09-24): *«Caixas de marcar na
/// Física»* — eram botões acesos. ⚠️ O valor é o `bool` do MODELO (o *snapshot*), e ela avança a
/// linha e o vão de toda linha (`row_pitch_px`), que é o que o chamador somava à mão.
#[allow(clippy::too_many_arguments)]
fn check(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    (x, w, y): (f32, f32, f32),
    seccao: ph2d_editor_core::widget::Seccao,
) -> f32 {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (id, label, on),
        seccao,
    )
}

/// A plain action button.
fn command(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let rect = ph2d_editor_core::property_row::caixa_do_botao(ctx.text_system, x, w, y, label);
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
fn readout(ctx: &mut PaintCtx, theme: Theme, text: &str, x: f32, w: f32, y: f32) -> f32 {
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
