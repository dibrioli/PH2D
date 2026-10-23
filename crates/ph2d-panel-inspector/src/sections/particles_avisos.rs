//! **Os AVISOS da secção PARTICLES** — a metade que responde a *«pus o componente e não vejo nada»*.
//!
//! ⚠️ **Irmão por tecto de LOC** (600 por ficheiro de painel): a migração do HR-15 de
//! 2026-09-16 alongou cada rótulo (`tr("chave")` no lugar do literal) e o ficheiro passou
//! o tecto. O corte é por RESPONSABILIDADE, que é o que o tecto pede.

use super::*;

/// **Os AVISOS** — a metade que responde a *«pus o componente e não vejo nada»*.
#[allow(clippy::too_many_arguments)]
pub(super) fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorParticlesInfo,
) -> f32 {
    let mut cur_y = y;
    if !i.clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.particles.the_clock_is_stopped_u_particles_are_born_while_the_clock_plays",
            ),
            ColorToken::Text3,
        );
    } else if i.alive == 0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.particles.nothing_alive_right_now_u_the_emission_ended_or_it_is_switched_off",
            ),
            ColorToken::Text3,
        );
    }
    if !i.emitting {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.particles.it_starts_stopped_u_a_signal_switches_it_on"),
            ColorToken::Warn,
        );
    }
    cur_y
}

/// O corpo da secção — os CONTROLOS, por módulo.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorParticlesInfo,
) -> f32 {
    let mut cur_y = avisos(scene, text_system, theme, x, w, y, i);
    // ── Emissão ──────────────────────────────────────────────────────────────
    cur_y = titulo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.particles.emission"),
    );
    cur_y = check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_PART_EMITTING,
        tr("component.field.fields.1"),
        i.emitting,
    );
    cur_y = check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_PART_ONE_SHOT,
        tr("component.field.fields.2"),
        i.one_shot,
    );
    for n in [
        N::Amount,
        N::Life,
        N::LifeRandom,
        N::Explosiveness,
        N::Prewarm,
        N::TimeScale,
        N::Seed,
    ] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    // ── De onde nascem ───────────────────────────────────────────────────────
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("component.field.fields.10"),
        &crate::ids::INSP_PART_SHAPE,
        &[
            tr("panel.inspector.particles.point"),
            tr("panel.inspector.particles.disc"),
            tr("panel.inspector.particles.ring"),
            tr("panel.inspector.particles.rect"),
        ],
        usize::from(i.shape),
    );
    // ⭐ **As duas medidas só existem fora do ponto** — num ponto elas são inertes, e um controlo
    // vivo que não muda nada é a doença que os gates de param existem para curar.
    if i.shape != 0 {
        cur_y = vec_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            (N::ShapeW, N::ShapeH),
        );
    }
    // ── Para onde vão ────────────────────────────────────────────────────────
    cur_y = titulo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.particles.velocity"),
    );
    for n in [N::Speed, N::SpeedRandom, N::Angle, N::Spread] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    cur_y = titulo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.particles.forces"),
    );
    cur_y = vec_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (N::GravityX, N::GravityY),
    );
    cur_y = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        N::Damping,
    );
    // ── Como são ─────────────────────────────────────────────────────────────
    cur_y = titulo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.particles.look"),
    );
    for n in [N::Size, N::SizeRandom, N::SizeEnd] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    cur_y = super::color_tint::bloco_de_cores(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                crate::ids::INSP_PART_COLOR,
                tr("component.field.fields.21"),
                i.color,
            ),
            (
                crate::ids::INSP_PART_COLOR_END,
                tr("component.field.fields.22"),
                i.color_end,
            ),
        ],
    );
    // ── Onde vivem ───────────────────────────────────────────────────────────
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("component.field.fields.23"),
        &crate::ids::INSP_PART_SPACE,
        &[
            tr("panel.inspector.particles.world"),
            tr("panel.inspector.particles.local"),
        ],
        usize::from(i.space),
    );
    sinais(scene, text_system, theme, hit_index, store, x, w, cur_y)
}

/// Os quatro sinais (acende · apaga · recomeça · grita ao acabar), o NOME ao lado da dica.
#[allow(clippy::too_many_arguments)]
fn sinais(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut cur_y: f32,
) -> f32 {
    // ── Os sinais ────────────────────────────────────────────────────────────
    cur_y = titulo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.particles.signals"),
    );
    for (k, _t) in PARTICLES_TEXTS.into_iter().enumerate() {
        let dica = [
            tr("panel.inspector.particles.switch_on_u"),
            tr("panel.inspector.particles.switch_off_u"),
            tr("panel.inspector.particles.restart_u"),
            tr("panel.inspector.particles.shout_when_done_u"),
        ][k];
        // ⭐ O NOME ao lado da dica, na mesma ordem — report do dono de 2026-09-22.
        let rotulos = [
            tr("component.field.fields.24"),
            tr("component.field.fields.25"),
            tr("component.field.fields.26"),
            tr("component.field.fields.27"),
        ];
        let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &rotulos);
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            rotulos[k],
            crate::ids::INSP_PART_TEXT[k],
            TextInput::new(crate::ids::INSP_PART_TEXT[k], "").placeholder(dica),
            seccao,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_particles_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorParticlesInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PARTICLES_SECTION,
        tr("component.particle_emitter.name"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PARTICLES_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
